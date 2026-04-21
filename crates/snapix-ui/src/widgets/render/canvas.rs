use gtk4::cairo;
use snapix_core::canvas::{Annotation, Document, ImageScaleMode};

use crate::editor::{EditorState, ToolKind};
use crate::widgets::geometry::{
    composition_frame_bounds, composition_scale, directional_shadow_padding,
    draw_arrow_resize_handles, draw_resize_handles, inset_frame, layout_for_bounds_with_mode,
    paint_background, paint_empty_state, paint_image, preview_canvas_layout,
    resizable_annotation_widget_bounds, rounded_rect, selection_annotation_widget_bounds,
};
use crate::widgets::CanvasLayout;

use super::annotations::{
    draw_annotations, draw_arrow, draw_blur_preview, draw_ellipse_preview, draw_rect_preview,
    BlurSurfaceCache,
};

pub(super) fn draw_canvas(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    document: &Document,
    blur_cache: &mut BlurSurfaceCache,
) {
    cr.set_source_rgb(0.09, 0.10, 0.13);
    cr.paint().ok();

    let (frame_x, frame_y, frame_w, frame_h) = composition_frame_bounds(document, width, height);
    let composition_scale = composition_scale(document, width, height);

    paint_background(cr, frame_x, frame_y, frame_w, frame_h, &document.background);

    let image_bounds = inset_frame(
        frame_x,
        frame_y,
        frame_w,
        frame_h,
        document.frame.padding as f64 * composition_scale,
    );

    let shadow_target = match document.base_image.as_ref() {
        Some(img) => {
            if document.image_scale_mode == ImageScaleMode::Fill {
                image_bounds
            } else {
                layout_for_bounds_with_mode(
                    img,
                    image_bounds,
                    document.image_scale_mode,
                    document.image_anchor,
                )
                    .map(|layout| {
                        (
                            layout.image_x,
                            layout.image_y,
                            layout.image_width,
                            layout.image_height,
                        )
                    })
                    .unwrap_or(image_bounds)
            }
        }
        None => image_bounds,
    };

    if document.frame.shadow {
        let blur = document.frame.shadow_blur.max(0.0) as f64 * composition_scale;
        let offset_x = document.frame.shadow_offset_x as f64 * composition_scale;
        let offset_y = document.frame.shadow_offset_y as f64 * composition_scale;
        let shadow_padding = document.frame.shadow_padding.max(0.0) as f64 * composition_scale;
        let strength = document.frame.shadow_strength.clamp(0.0, 1.0) as f64;
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
        paint_image(
            cr,
            image_bounds,
            image,
            document.frame.corner_radius as f64 * composition_scale,
            document.image_scale_mode,
            document.image_anchor,
        );
        if let Some(layout) = preview_canvas_layout(document, width, height) {
            draw_annotations(cr, document, layout, blur_cache);
        }
    } else {
        paint_empty_state(cr, image_bounds, document.frame.corner_radius as f64);
    }
}

pub(crate) fn draw_editor_canvas(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    state: &EditorState,
    blur_cache: &mut BlurSurfaceCache,
) {
    draw_canvas(cr, width, height, state.document(), blur_cache);
    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
        if let Some(index) = state.selected_annotation() {
            draw_selected_annotation(cr, state.document(), layout, index);
        }
    }
    if state.active_tool() == ToolKind::Arrow {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(arrow_drag) = state.arrow_drag() {
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
            }
        }
    } else if state.active_tool() == ToolKind::Rectangle {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(rect_drag) = state.rect_drag() {
                draw_rect_preview(
                    cr,
                    layout,
                    rect_drag,
                    &state.active_color(),
                    state.active_width(),
                );
            }
        }
    } else if state.active_tool() == ToolKind::Ellipse {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(ellipse_drag) = state.ellipse_drag() {
                draw_ellipse_preview(
                    cr,
                    layout,
                    ellipse_drag,
                    &state.active_color(),
                    state.active_width(),
                );
            }
        }
    } else if state.active_tool() == ToolKind::Blur {
        if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
            if let Some(blur_drag) = state.blur_drag() {
                draw_blur_preview(cr, layout, blur_drag);
            }
        }
    }
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
        if let Annotation::Arrow { .. } = annotation {
            draw_arrow_resize_handles(cr, layout, annotation);
        }
    }
    if resizable_annotation_widget_bounds(document, layout, index).is_some() {
        draw_resize_handles(cr, bounds);
    }
    cr.restore().ok();
}
