use gtk4::cairo;
use snapix_core::canvas::{Document, ImageScaleMode};

use crate::editor::{CropDrag, CropSelection, EditorState, ToolKind};

use super::super::{CanvasLayout, CropInteractionMode};
use super::layout::{canvas_layout, composition_frame_bounds};
use super::paint::{paint_empty_state, paint_image, rounded_rect, workspace_palette};

pub(crate) fn draw_crop_mode_canvas(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    document: &Document,
) {
    let palette = workspace_palette();
    let (r, g, b) = palette.crop_backdrop_rgb;
    cr.set_source_rgb(r, g, b);
    cr.paint().ok();

    let Some(image) = document.base_image.as_ref() else {
        let bounds = composition_frame_bounds(document, width, height);
        paint_empty_state(cr, bounds, 16.0);
        return;
    };

    let Some(layout) = canvas_layout(document, width, height) else {
        return;
    };

    cr.set_source_rgba(0.0, 0.0, 0.0, 0.22);
    rounded_rect(
        cr,
        layout.image_x + 12.0,
        layout.image_y + 18.0,
        layout.image_width,
        layout.image_height,
        18.0,
    );
    cr.fill().ok();

    paint_image(
        cr,
        (
            layout.image_x,
            layout.image_y,
            layout.image_width,
            layout.image_height,
        ),
        image,
        18.0,
        ImageScaleMode::Fit,
        snapix_core::canvas::ImageAnchor::Center,
    );
}

pub(crate) fn draw_crop_overlay(cr: &cairo::Context, state: &EditorState, width: i32, height: i32) {
    if state.active_tool() != ToolKind::Crop {
        return;
    }

    let Some(layout) = canvas_layout(state.document(), width, height) else {
        return;
    };
    let overlay = state
        .crop_drag()
        .and_then(|crop_drag| crop_drag_widget_bounds(layout, crop_drag))
        .or_else(|| {
            state
                .crop_selection()
                .and_then(|selection| crop_selection_widget_bounds(layout, selection))
        });
    let Some((x, y, overlay_width, overlay_height)) = overlay else {
        return;
    };
    let radius = 18.0;

    cr.save().ok();
    rounded_rect(
        cr,
        layout.image_x,
        layout.image_y,
        layout.image_width,
        layout.image_height,
        radius,
    );
    cr.clip();
    cr.set_source_rgba(0.02, 0.03, 0.04, 0.48);
    cr.set_fill_rule(cairo::FillRule::EvenOdd);
    cr.rectangle(
        layout.image_x,
        layout.image_y,
        layout.image_width,
        layout.image_height,
    );
    cr.rectangle(x, y, overlay_width, overlay_height);
    cr.fill().ok();
    cr.restore().ok();

    cr.save().ok();
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.34);
    cr.set_line_width(8.0);
    cr.rectangle(x, y, overlay_width, overlay_height);
    cr.stroke().ok();
    cr.restore().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.92);
    cr.set_line_width(2.0);
    cr.rectangle(x, y, overlay_width, overlay_height);
    cr.stroke().ok();

    draw_crop_grid(cr, x, y, overlay_width, overlay_height);

    draw_crop_handle(cr, x, y);
    draw_crop_handle(cr, x + overlay_width / 2.0, y);
    draw_crop_handle(cr, x + overlay_width, y);
    draw_crop_handle(cr, x, y + overlay_height / 2.0);
    draw_crop_handle(cr, x + overlay_width, y + overlay_height / 2.0);
    draw_crop_handle(cr, x, y + overlay_height);
    draw_crop_handle(cr, x + overlay_width / 2.0, y + overlay_height);
    draw_crop_handle(cr, x + overlay_width, y + overlay_height);
}

pub(crate) fn hit_crop_interaction(
    bounds: (f64, f64, f64, f64),
    pointer_x: f64,
    pointer_y: f64,
) -> Option<CropInteractionMode> {
    let (x, y, width, height) = bounds;
    let center_x = x + width / 2.0;
    let center_y = y + height / 2.0;
    let right = x + width;
    let bottom = y + height;

    if near_handle(pointer_x, pointer_y, x, y) {
        return Some(CropInteractionMode::ResizeTopLeft);
    }
    if near_handle(pointer_x, pointer_y, center_x, y) {
        return Some(CropInteractionMode::ResizeTop);
    }
    if near_handle(pointer_x, pointer_y, right, y) {
        return Some(CropInteractionMode::ResizeTopRight);
    }
    if near_handle(pointer_x, pointer_y, x, center_y) {
        return Some(CropInteractionMode::ResizeLeft);
    }
    if near_handle(pointer_x, pointer_y, right, center_y) {
        return Some(CropInteractionMode::ResizeRight);
    }
    if near_handle(pointer_x, pointer_y, x, bottom) {
        return Some(CropInteractionMode::ResizeBottomLeft);
    }
    if near_handle(pointer_x, pointer_y, center_x, bottom) {
        return Some(CropInteractionMode::ResizeBottom);
    }
    if near_handle(pointer_x, pointer_y, right, bottom) {
        return Some(CropInteractionMode::ResizeBottomRight);
    }
    if pointer_x >= x && pointer_x <= right && pointer_y >= y && pointer_y <= bottom {
        return Some(CropInteractionMode::Move);
    }
    None
}

pub(crate) fn adjusted_crop_bounds(
    layout: CanvasLayout,
    initial_bounds: (f64, f64, f64, f64),
    mode: CropInteractionMode,
    offset_x: f64,
    offset_y: f64,
) -> Option<(f64, f64, f64, f64)> {
    const MIN_SIZE: f64 = 2.0;

    let (mut x, mut y, mut width, mut height) = initial_bounds;
    let left_limit = layout.image_x;
    let top_limit = layout.image_y;
    let right_limit = layout.image_x + layout.image_width;
    let bottom_limit = layout.image_y + layout.image_height;

    match mode {
        CropInteractionMode::Move => {
            x = (x + offset_x).clamp(left_limit, right_limit - width);
            y = (y + offset_y).clamp(top_limit, bottom_limit - height);
        }
        CropInteractionMode::ResizeTopLeft => {
            let new_left = (x + offset_x).clamp(left_limit, x + width - MIN_SIZE);
            let new_top = (y + offset_y).clamp(top_limit, y + height - MIN_SIZE);
            width += x - new_left;
            height += y - new_top;
            x = new_left;
            y = new_top;
        }
        CropInteractionMode::ResizeTop => {
            let new_top = (y + offset_y).clamp(top_limit, y + height - MIN_SIZE);
            height += y - new_top;
            y = new_top;
        }
        CropInteractionMode::ResizeTopRight => {
            let new_right = (x + width + offset_x).clamp(x + MIN_SIZE, right_limit);
            let new_top = (y + offset_y).clamp(top_limit, y + height - MIN_SIZE);
            width = new_right - x;
            height += y - new_top;
            y = new_top;
        }
        CropInteractionMode::ResizeLeft => {
            let new_left = (x + offset_x).clamp(left_limit, x + width - MIN_SIZE);
            width += x - new_left;
            x = new_left;
        }
        CropInteractionMode::ResizeRight => {
            let new_right = (x + width + offset_x).clamp(x + MIN_SIZE, right_limit);
            width = new_right - x;
        }
        CropInteractionMode::ResizeBottomLeft => {
            let new_left = (x + offset_x).clamp(left_limit, x + width - MIN_SIZE);
            let new_bottom = (y + height + offset_y).clamp(y + MIN_SIZE, bottom_limit);
            width += x - new_left;
            height = new_bottom - y;
            x = new_left;
        }
        CropInteractionMode::ResizeBottom => {
            let new_bottom = (y + height + offset_y).clamp(y + MIN_SIZE, bottom_limit);
            height = new_bottom - y;
        }
        CropInteractionMode::ResizeBottomRight => {
            let new_right = (x + width + offset_x).clamp(x + MIN_SIZE, right_limit);
            let new_bottom = (y + height + offset_y).clamp(y + MIN_SIZE, bottom_limit);
            width = new_right - x;
            height = new_bottom - y;
        }
    }

    if width < MIN_SIZE || height < MIN_SIZE {
        return None;
    }

    Some((x, y, width, height))
}

pub(crate) fn crop_drag_widget_bounds(
    layout: CanvasLayout,
    crop_drag: &CropDrag,
) -> Option<(f64, f64, f64, f64)> {
    let start_x = crop_drag
        .start_x()
        .clamp(layout.image_x, layout.image_x + layout.image_width);
    let start_y = crop_drag
        .start_y()
        .clamp(layout.image_y, layout.image_y + layout.image_height);
    let end_x = crop_drag
        .current_x()
        .clamp(layout.image_x, layout.image_x + layout.image_width);
    let end_y = crop_drag
        .current_y()
        .clamp(layout.image_y, layout.image_y + layout.image_height);

    let x = start_x.min(end_x);
    let y = start_y.min(end_y);
    let width = (start_x.max(end_x) - x).max(0.0);
    let height = (start_y.max(end_y) - y).max(0.0);

    if width < 2.0 || height < 2.0 {
        return None;
    }

    Some((x, y, width, height))
}

pub(crate) fn crop_selection_widget_bounds(
    layout: CanvasLayout,
    selection: CropSelection,
) -> Option<(f64, f64, f64, f64)> {
    let x = layout.image_x + selection.x() as f64 * layout.image_scale;
    let y = layout.image_y + selection.y() as f64 * layout.image_scale;
    let width = selection.width() as f64 * layout.image_scale;
    let height = selection.height() as f64 * layout.image_scale;

    if width < 2.0 || height < 2.0 {
        return None;
    }

    Some((x, y, width, height))
}

pub(crate) fn crop_rect_to_image_pixels(
    document: &Document,
    layout: CanvasLayout,
    crop_drag: &CropDrag,
) -> Option<(u32, u32, u32, u32)> {
    let (x, y, width, height) = crop_drag_widget_bounds(layout, crop_drag)?;
    widget_rect_to_image_pixels(document, layout, x, y, width, height)
}

pub(crate) fn widget_rect_to_image_pixels(
    document: &Document,
    layout: CanvasLayout,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<(u32, u32, u32, u32)> {
    let image = document.base_image.as_ref()?;
    let left = ((x - layout.image_x) / layout.image_scale).floor().max(0.0) as u32;
    let top = ((y - layout.image_y) / layout.image_scale).floor().max(0.0) as u32;
    let right = ((x + width - layout.image_x) / layout.image_scale)
        .ceil()
        .min(image.width as f64) as u32;
    let bottom = ((y + height - layout.image_y) / layout.image_scale)
        .ceil()
        .min(image.height as f64) as u32;

    if right <= left || bottom <= top {
        return None;
    }

    Some((left, top, right - left, bottom - top))
}

fn draw_crop_grid(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64) {
    cr.save().ok();
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.18);
    cr.set_line_width(1.0);

    let x_third = width / 3.0;
    let y_third = height / 3.0;

    for multiplier in [1.0, 2.0] {
        let vertical = x + x_third * multiplier;
        cr.move_to(vertical, y);
        cr.line_to(vertical, y + height);

        let horizontal = y + y_third * multiplier;
        cr.move_to(x, horizontal);
        cr.line_to(x + width, horizontal);
    }

    cr.stroke().ok();
    cr.restore().ok();
}

fn draw_crop_handle(cr: &cairo::Context, center_x: f64, center_y: f64) {
    const HANDLE_SIZE: f64 = 10.0;
    const HANDLE_RADIUS: f64 = 3.0;
    let half = HANDLE_SIZE / 2.0;

    cr.save().ok();
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.28);
    rounded_rect(
        cr,
        center_x - half,
        center_y - half,
        HANDLE_SIZE,
        HANDLE_SIZE,
        HANDLE_RADIUS,
    );
    cr.fill().ok();

    cr.set_source_rgb(1.0, 1.0, 1.0);
    rounded_rect(
        cr,
        center_x - half + 1.0,
        center_y - half + 1.0,
        HANDLE_SIZE - 2.0,
        HANDLE_SIZE - 2.0,
        HANDLE_RADIUS,
    );
    cr.fill().ok();
    cr.restore().ok();
}

fn near_handle(pointer_x: f64, pointer_y: f64, handle_x: f64, handle_y: f64) -> bool {
    const HIT_RADIUS: f64 = 18.0;
    (pointer_x - handle_x).abs() <= HIT_RADIUS && (pointer_y - handle_y).abs() <= HIT_RADIUS
}
