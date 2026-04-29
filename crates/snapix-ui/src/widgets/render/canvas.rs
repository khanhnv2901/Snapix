use gtk4::cairo;
use snapix_core::canvas::{Annotation, Background, Document, ImageAnchor, ImageScaleMode};

use crate::editor::{EditorState, ToolKind};
use crate::widgets::geometry::{
    composition_frame_bounds, composition_scale, directional_shadow_padding,
    draw_arrow_resize_handles, draw_resize_handles, image_bounds_for_document, paint_background,
    paint_empty_state, paint_image_for_document, paint_surface, preview_canvas_layout,
    resizable_annotation_widget_bounds, rounded_rect, selection_annotation_widget_bounds,
    signature_shadow_profile, workspace_palette,
};
use crate::widgets::CanvasLayout;

use super::annotations::{
    draw_annotations, draw_arrow, draw_blur_preview, draw_ellipse_preview, draw_line,
    draw_rect_preview, BlurSurfaceCache,
};
use super::reframe::draw_reframe_overlay;

pub(super) fn draw_canvas(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    document: &Document,
    blur_cache: &mut BlurSurfaceCache,
) {
    draw_canvas_with_background_radius(cr, width, height, document, blur_cache, 28.0);
}

pub(super) fn draw_canvas_with_background_radius(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    document: &Document,
    blur_cache: &mut BlurSurfaceCache,
    background_radius: f64,
) {
    let palette = workspace_palette();
    let (canvas_r, canvas_g, canvas_b) = palette.canvas_rgb;
    cr.set_source_rgb(canvas_r, canvas_g, canvas_b);
    cr.paint().ok();

    let (frame_x, frame_y, frame_w, frame_h) = composition_frame_bounds(document, width, height);
    let composition_scale = composition_scale(document, width, height);

    let painted_special_background = match &document.background {
        Background::BlurredScreenshot { radius } => {
            if let Some(image) = document.base_image.as_ref() {
                blur_cache.prepare_for_document(document);
                if let Some(surface) = blur_cache.background_surface_for(image, *radius) {
                    paint_surface(
                        cr,
                        (frame_x, frame_y, frame_w, frame_h),
                        image.width,
                        image.height,
                        surface,
                        background_radius,
                        ImageScaleMode::Fill,
                        ImageAnchor::Center,
                    );
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        Background::Image { path } => {
            if let Some(surface) = blur_cache.custom_image_surface_for(path) {
                paint_surface(
                    cr,
                    (frame_x, frame_y, frame_w, frame_h),
                    surface.width() as u32,
                    surface.height() as u32,
                    surface,
                    background_radius,
                    ImageScaleMode::Fill,
                    ImageAnchor::Center,
                );
                true
            } else {
                false
            }
        }
        _ => false,
    };

    if !painted_special_background {
        paint_background(
            cr,
            frame_x,
            frame_y,
            frame_w,
            frame_h,
            &document.background,
            background_radius,
        );
    }

    cr.save().ok();
    clip_to_composition_frame(cr, document, width, height, background_radius);

    let image_bounds = image_bounds_for_document(
        document,
        (frame_x, frame_y, frame_w, frame_h),
        composition_scale,
    );

    let shadow_target = match document.base_image.as_ref() {
        Some(img) => crate::widgets::geometry::layout_for_document(img, image_bounds, document)
            .map(|layout| {
                (
                    layout.image_x,
                    layout.image_y,
                    layout.image_width,
                    layout.image_height,
                )
            })
            .unwrap_or(image_bounds),
        None => image_bounds,
    };

    if document.frame.shadow {
        let (shadow_blur_scale_factor, shadow_strength_scale_factor) =
            signature_shadow_profile(&document.background);
        let blur = document.frame.shadow_blur.max(0.0) as f64
            * composition_scale
            * shadow_blur_scale_factor;
        let offset_x = document.frame.shadow_offset_x as f64 * composition_scale;
        let offset_y = document.frame.shadow_offset_y as f64 * composition_scale;
        let shadow_padding = document.frame.shadow_padding.max(0.0) as f64 * composition_scale;
        let strength = (document.frame.shadow_strength.clamp(0.0, 1.0) as f64
            * shadow_strength_scale_factor)
            .clamp(0.0, 1.0);
        let shadow_steps = ((blur / 2.5).round() as i32).clamp(8, 24);
        let max_expand = blur.max(6.0);
        let left_pad = directional_shadow_padding(offset_x, false, shadow_padding);
        let right_pad = directional_shadow_padding(offset_x, true, shadow_padding);
        let top_pad = directional_shadow_padding(offset_y, false, shadow_padding);
        let bottom_pad = directional_shadow_padding(offset_y, true, shadow_padding);
        let shadow_x = shadow_target.0 - left_pad;
        let shadow_y = shadow_target.1 - top_pad;
        let shadow_w = shadow_target.2 + left_pad + right_pad;
        let shadow_h = shadow_target.3 + top_pad + bottom_pad;
        let shadow_radius = document.frame.corner_radius as f64 * composition_scale
            + shadow_padding.max(left_pad.max(right_pad).max(top_pad).max(bottom_pad)) * 0.35;

        for i in 1..=shadow_steps {
            let t = i as f64 / shadow_steps as f64;
            let expand = t * max_expand;
            let alpha = (0.10 * strength) * (1.0 - t).powf(1.7);
            let drift = t.powf(1.2);
            cr.set_source_rgba(0.0, 0.0, 0.0, alpha);
            rounded_rect(
                cr,
                shadow_x - expand + offset_x * drift,
                shadow_y - expand + offset_y * drift,
                shadow_w + expand * 2.0,
                shadow_h + expand * 2.0,
                shadow_radius + expand * 0.45,
            );
            cr.fill().ok();
        }

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.08 * strength);
        rounded_rect(cr, shadow_x, shadow_y, shadow_w, shadow_h, shadow_radius);
        cr.fill().ok();
    }

    if let Some(image) = document.base_image.as_ref() {
        paint_image_for_document(
            cr,
            image_bounds,
            image,
            document.frame.corner_radius as f64 * composition_scale,
            document,
        );
        if let Some(layout) = preview_canvas_layout(document, width, height) {
            cr.save().ok();
            cr.rectangle(
                layout.viewport_x,
                layout.viewport_y,
                layout.viewport_width,
                layout.viewport_height,
            );
            cr.clip();
            draw_annotations(cr, document, layout, blur_cache);
            cr.restore().ok();
        }
    } else {
        paint_empty_state(cr, image_bounds, document.frame.corner_radius as f64);
    }

    cr.restore().ok();
}

pub(crate) fn draw_editor_canvas(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    state: &EditorState,
    overlay_opacity: f64,
    blur_cache: &mut BlurSurfaceCache,
) {
    draw_canvas(cr, width, height, state.document(), blur_cache);
    if overlay_opacity > 0.01 {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            cr.save().ok();
            clip_to_composition_frame(cr, state.document(), width, height, 28.0);
            draw_reframe_overlay(cr, layout, overlay_opacity, state.document().image_zoom);
            cr.restore().ok();
        }
    }
    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
        if let Some(index) = state.selected_annotation() {
            cr.save().ok();
            clip_to_composition_frame(cr, state.document(), width, height, 28.0);
            draw_selected_annotation(cr, state.document(), layout, index);
            cr.restore().ok();
        }
    }
    if state.active_tool() == ToolKind::Arrow {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(arrow_drag) = state.arrow_drag() {
                cr.save().ok();
                clip_to_composition_frame(cr, state.document(), width, height, 28.0);
                draw_arrow(
                    cr,
                    layout,
                    arrow_drag.start_x(),
                    arrow_drag.start_y(),
                    arrow_drag.current_x(),
                    arrow_drag.current_y(),
                    &state.active_color(),
                    state.active_width(),
                );
                cr.restore().ok();
            }
        }
    } else if state.active_tool() == ToolKind::Line {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(arrow_drag) = state.arrow_drag() {
                cr.save().ok();
                clip_to_composition_frame(cr, state.document(), width, height, 28.0);
                draw_line(
                    cr,
                    layout,
                    arrow_drag.start_x(),
                    arrow_drag.start_y(),
                    arrow_drag.current_x(),
                    arrow_drag.current_y(),
                    &state.active_color(),
                    state.active_width(),
                );
                cr.restore().ok();
            }
        }
    } else if state.active_tool() == ToolKind::Rectangle {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(rect_drag) = state.rect_drag() {
                cr.save().ok();
                clip_to_composition_frame(cr, state.document(), width, height, 28.0);
                draw_rect_preview(
                    cr,
                    layout,
                    rect_drag,
                    &state.active_color(),
                    state.active_width(),
                );
                cr.restore().ok();
            }
        }
    } else if state.active_tool() == ToolKind::Ellipse {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(ellipse_drag) = state.ellipse_drag() {
                cr.save().ok();
                clip_to_composition_frame(cr, state.document(), width, height, 28.0);
                draw_ellipse_preview(
                    cr,
                    layout,
                    ellipse_drag,
                    &state.active_color(),
                    state.active_width(),
                );
                cr.restore().ok();
            }
        }
    } else if state.active_tool() == ToolKind::Blur {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(blur_drag) = state.blur_drag() {
                cr.save().ok();
                clip_to_composition_frame(cr, state.document(), width, height, 28.0);
                draw_blur_preview(cr, layout, blur_drag);
                cr.restore().ok();
            }
        }
    }
}

fn clip_to_composition_frame(
    cr: &cairo::Context,
    document: &Document,
    width: i32,
    height: i32,
    radius: f64,
) {
    let (frame_x, frame_y, frame_w, frame_h) = composition_frame_bounds(document, width, height);
    rounded_rect(cr, frame_x, frame_y, frame_w, frame_h, radius);
    cr.clip();
}

fn draw_selected_annotation(
    cr: &cairo::Context,
    document: &Document,
    layout: CanvasLayout,
    index: usize,
) {
    let Some(bounds) = selection_annotation_widget_bounds(document, layout, index) else {
        return;
    };
    let (x, y, width, height) = bounds;
    cr.save().ok();
    cr.set_source_rgba(0.55, 0.78, 1.0, 0.95);
    cr.set_line_width(2.0);
    cr.set_dash(&[6.0, 4.0], 0.0);
    rounded_rect(cr, x, y, width, height, 10.0);
    cr.stroke().ok();
    cr.set_dash(&[], 0.0);
    if let Some(annotation) = document.annotations.get(index) {
        if matches!(
            annotation,
            Annotation::Arrow { .. } | Annotation::Line { .. }
        ) {
            draw_arrow_resize_handles(cr, layout, annotation);
        }
    }
    if resizable_annotation_widget_bounds(document, layout, index).is_some() {
        draw_resize_handles(cr, bounds);
    }
    cr.restore().ok();
}
