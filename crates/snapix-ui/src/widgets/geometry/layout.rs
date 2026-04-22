use snapix_core::canvas::{Document, Image, ImageAnchor, ImageScaleMode};

use super::super::CanvasLayout;

pub(crate) fn canvas_layout(document: &Document, width: i32, height: i32) -> Option<CanvasLayout> {
    preview_canvas_layout(document, width, height)
}

pub(crate) fn preview_canvas_layout(
    document: &Document,
    width: i32,
    height: i32,
) -> Option<CanvasLayout> {
    let image = document.base_image.as_ref()?;
    let (frame_x, frame_y, frame_w, frame_h) = composition_frame_bounds(document, width, height);
    let composition_scale = composition_scale(document, width, height);
    let image_bounds = inset_frame(
        frame_x,
        frame_y,
        frame_w,
        frame_h,
        document.frame.padding as f64 * composition_scale,
    );

    layout_for_bounds_with_mode(
        image,
        image_bounds,
        document.image_scale_mode,
        document.image_anchor,
    )
}

pub(crate) fn composition_size(document: &Document) -> (f64, f64) {
    const OUTER_MARGIN: f64 = 3.0;
    match document.base_image.as_ref() {
        Some(image) => {
            let padding = document.frame.padding.max(0.0) as f64;
            let content_width = image.width as f64 + padding * 2.0;
            let content_height = image.height as f64 + padding * 2.0;
            let (ratio_width, ratio_height) =
                expand_to_output_ratio(content_width, content_height, document);
            (
                ratio_width + OUTER_MARGIN * 2.0,
                ratio_height + OUTER_MARGIN * 2.0,
            )
        }
        None => (480.0, 320.0),
    }
}

fn expand_to_output_ratio(width: f64, height: f64, document: &Document) -> (f64, f64) {
    let Some((ratio_width, ratio_height)) = document.output_ratio.dimensions() else {
        return (width, height);
    };

    let current_ratio = width / height.max(1.0);
    let target_ratio = ratio_width / ratio_height.max(1.0);
    if current_ratio < target_ratio {
        (height * target_ratio, height)
    } else {
        (width, width / target_ratio)
    }
}

pub(crate) fn composition_frame_bounds(
    document: &Document,
    width: i32,
    height: i32,
) -> (f64, f64, f64, f64) {
    let (natural_width, natural_height) = composition_size(document);
    let available_width = (width as f64).max(160.0);
    let available_height = (height as f64).max(160.0);
    let scale = composition_scale(document, width, height);
    let frame_width = (natural_width * scale).max(160.0);
    let frame_height = (natural_height * scale).max(160.0);
    let frame_x = (available_width - frame_width) / 2.0;
    let frame_y = (available_height - frame_height) / 2.0;
    (frame_x, frame_y, frame_width, frame_height)
}

pub(crate) fn composition_scale(document: &Document, width: i32, height: i32) -> f64 {
    let (natural_width, natural_height) = composition_size(document);
    let available_width = (width as f64).max(160.0);
    let available_height = (height as f64).max(160.0);
    f64::min(
        available_width / natural_width,
        available_height / natural_height,
    )
}

pub(crate) fn directional_shadow_padding(offset: f64, positive_side: bool, padding: f64) -> f64 {
    if padding <= 0.0 {
        return 0.0;
    }

    if offset > 0.0 {
        if positive_side {
            padding
        } else {
            0.0
        }
    } else if offset < 0.0 {
        if positive_side {
            0.0
        } else {
            padding
        }
    } else {
        padding * 0.5
    }
}

pub(crate) fn layout_for_bounds_with_mode(
    image: &Image,
    bounds: (f64, f64, f64, f64),
    mode: ImageScaleMode,
    anchor: ImageAnchor,
) -> Option<CanvasLayout> {
    let (x, y, max_width, max_height) = bounds;
    let image_w = image.width as f64;
    let image_h = image.height as f64;
    if image_w <= 0.0 || image_h <= 0.0 {
        return None;
    }
    let fit_scale = f64::min(max_width / image_w, max_height / image_h);
    let fill_scale = f64::max(max_width / image_w, max_height / image_h);
    let scale = match mode {
        ImageScaleMode::Fit => fit_scale,
        ImageScaleMode::Fill => fill_scale,
    };
    let draw_w = image_w * scale;
    let draw_h = image_h * scale;
    let (align_x, align_y) = match mode {
        ImageScaleMode::Fit => (0.5, 0.5),
        ImageScaleMode::Fill => anchor.alignment(),
    };
    let draw_x = x + (max_width - draw_w) * align_x;
    let draw_y = y + (max_height - draw_h) * align_y;

    Some(CanvasLayout {
        image_x: draw_x,
        image_y: draw_y,
        image_width: draw_w,
        image_height: draw_h,
        image_scale: scale,
    })
}

pub(crate) fn point_in_layout(x: f64, y: f64, layout: CanvasLayout) -> bool {
    x >= layout.image_x
        && y >= layout.image_y
        && x <= layout.image_x + layout.image_width
        && y <= layout.image_y + layout.image_height
}

pub(crate) fn point_in_bounds(x: f64, y: f64, bounds: (f64, f64, f64, f64)) -> bool {
    let (left, top, width, height) = bounds;
    x >= left && y >= top && x <= left + width && y <= top + height
}

pub(crate) fn expand_bounds(bounds: (f64, f64, f64, f64), padding: f64) -> (f64, f64, f64, f64) {
    (
        bounds.0 - padding,
        bounds.1 - padding,
        bounds.2 + padding * 2.0,
        bounds.3 + padding * 2.0,
    )
}

pub(crate) fn annotation_rect_to_widget_bounds(
    layout: CanvasLayout,
    bounds: &snapix_core::canvas::Rect,
) -> (f64, f64, f64, f64) {
    (
        layout.image_x + bounds.x as f64 * layout.image_scale,
        layout.image_y + bounds.y as f64 * layout.image_scale,
        bounds.width as f64 * layout.image_scale,
        bounds.height as f64 * layout.image_scale,
    )
}

pub(crate) fn widget_point_to_image_pixel(
    document: &Document,
    layout: CanvasLayout,
    x: f64,
    y: f64,
) -> Option<(u32, u32)> {
    let image = document.base_image.as_ref()?;
    if !point_in_layout(x, y, layout) {
        return None;
    }

    let image_x = ((x - layout.image_x) / layout.image_scale)
        .round()
        .clamp(0.0, image.width.saturating_sub(1) as f64) as u32;
    let image_y = ((y - layout.image_y) / layout.image_scale)
        .round()
        .clamp(0.0, image.height.saturating_sub(1) as f64) as u32;
    Some((image_x, image_y))
}

pub(crate) fn inset_frame(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    padding: f64,
) -> (f64, f64, f64, f64) {
    let padded_x = x + padding;
    let padded_y = y + padding;
    let padded_w = (width - padding * 2.0).max(80.0);
    let padded_h = (height - padding * 2.0).max(80.0);
    (padded_x, padded_y, padded_w, padded_h)
}

#[cfg(test)]
mod tests {
    use snapix_core::canvas::{ImageAnchor, ImageScaleMode, OutputRatio};

    use super::{
        composition_size, layout_for_bounds_with_mode, preview_canvas_layout,
        widget_point_to_image_pixel,
    };
    use crate::widgets::test_support::sample_document;

    #[test]
    fn widget_point_maps_to_expected_image_pixel() {
        let document = sample_document(100, 50);
        let layout = preview_canvas_layout(&document, 300, 200).expect("layout");
        let target_x = 10.0;
        let target_y = 5.0;

        let point = widget_point_to_image_pixel(
            &document,
            layout,
            layout.image_x + layout.image_scale * target_x,
            layout.image_y + layout.image_scale * target_y,
        )
        .expect("point in image");

        assert_eq!(point, (10, 5));
    }

    #[test]
    fn widget_point_outside_layout_returns_none() {
        let document = sample_document(100, 50);
        let layout = preview_canvas_layout(&document, 300, 200).expect("layout");

        assert_eq!(
            widget_point_to_image_pixel(&document, layout, layout.image_x - 1.0, layout.image_y),
            None
        );
    }

    #[test]
    fn composition_size_expands_to_selected_output_ratio() {
        let mut document = sample_document(100, 50);
        document.frame.padding = 0.0;
        document.output_ratio = OutputRatio::Square;

        let (width, height) = composition_size(&document);

        assert_eq!(width, 106.0);
        assert_eq!(height, 106.0);
    }

    #[test]
    fn fill_mode_expands_image_to_cover_bounds() {
        let image = sample_document(100, 50).base_image.expect("image");
        let layout = layout_for_bounds_with_mode(
            &image,
            (0.0, 0.0, 120.0, 120.0),
            ImageScaleMode::Fill,
            ImageAnchor::Center,
        )
        .expect("layout");

        assert_eq!(layout.image_width, 240.0);
        assert_eq!(layout.image_height, 120.0);
        assert_eq!(layout.image_x, -60.0);
        assert_eq!(layout.image_y, 0.0);
    }

    #[test]
    fn fill_mode_respects_top_left_anchor() {
        let image = sample_document(100, 50).base_image.expect("image");
        let layout = layout_for_bounds_with_mode(
            &image,
            (0.0, 0.0, 120.0, 120.0),
            ImageScaleMode::Fill,
            ImageAnchor::TopLeft,
        )
        .expect("layout");

        assert_eq!(layout.image_x, 0.0);
        assert_eq!(layout.image_y, 0.0);
    }
}
