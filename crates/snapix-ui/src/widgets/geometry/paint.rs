use gtk4::cairo;
use libadwaita::StyleManager;
use snapix_core::canvas::{
    Annotation, Background, Color, Document, Image, ImageAnchor, ImageScaleMode,
};

use super::super::CanvasLayout;
use super::{layout_for_bounds_with_mode, layout_for_document};

#[derive(Clone, Copy)]
pub(crate) struct WorkspacePalette {
    pub(crate) canvas_rgb: (f64, f64, f64),
    pub(crate) blurred_fill_rgb: (f64, f64, f64),
    pub(crate) crop_backdrop_rgb: (f64, f64, f64),
}

pub(crate) fn workspace_palette() -> WorkspacePalette {
    if StyleManager::default().is_dark() {
        WorkspacePalette {
            canvas_rgb: (0.09, 0.10, 0.13),
            blurred_fill_rgb: (0.15, 0.18, 0.23),
            crop_backdrop_rgb: (0.07, 0.08, 0.10),
        }
    } else {
        WorkspacePalette {
            canvas_rgb: (0.94, 0.96, 0.99),
            blurred_fill_rgb: (0.88, 0.91, 0.95),
            crop_backdrop_rgb: (0.92, 0.95, 0.98),
        }
    }
}

pub(crate) fn paint_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    background: &Background,
) {
    let palette = workspace_palette();
    match background {
        Background::Solid { color } => {
            set_color(cr, color);
        }
        Background::Gradient {
            from,
            to,
            angle_deg,
        } => {
            let angle = (*angle_deg as f64).to_radians();
            let center_x = x + width / 2.0;
            let center_y = y + height / 2.0;
            let half_length = ((width * width + height * height).sqrt()) / 2.0;
            let dx = angle.cos() * half_length;
            let dy = angle.sin() * half_length;
            let gradient = cairo::LinearGradient::new(
                center_x - dx,
                center_y - dy,
                center_x + dx,
                center_y + dy,
            );
            gradient.add_color_stop_rgba(0.0, to_f64(from.r), to_f64(from.g), to_f64(from.b), 1.0);
            gradient.add_color_stop_rgba(1.0, to_f64(to.r), to_f64(to.g), to_f64(to.b), 1.0);
            cr.set_source(&gradient).ok();
        }
        Background::Image { .. } | Background::BlurredScreenshot { .. } => {
            let (r, g, b) = palette.blurred_fill_rgb;
            cr.set_source_rgb(r, g, b);
        }
    }

    rounded_rect(cr, x, y, width, height, 28.0);
    cr.fill().ok();
}

pub(crate) fn paint_empty_state(cr: &cairo::Context, bounds: (f64, f64, f64, f64), radius: f64) {
    let (x, y, width, height) = bounds;

    cr.set_source_rgb(0.96, 0.97, 0.99);
    rounded_rect(cr, x, y, width, height, radius);
    cr.fill().ok();

    cr.set_source_rgb(0.82, 0.85, 0.90);
    cr.set_line_width(2.0);
    rounded_rect(cr, x, y, width, height, radius);
    cr.stroke().ok();

    cr.set_source_rgb(0.73, 0.77, 0.83);
    cr.set_line_width(3.0);
    cr.move_to(x + width * 0.28, y + height * 0.32);
    cr.line_to(x + width * 0.72, y + height * 0.68);
    cr.move_to(x + width * 0.72, y + height * 0.32);
    cr.line_to(x + width * 0.28, y + height * 0.68);
    cr.stroke().ok();

    let title_y = y + height * 0.76;
    let subtitle_y = title_y + 28.0;
    let hint_y = subtitle_y + 24.0;

    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size((width.min(height) * 0.05).clamp(18.0, 28.0));
    cr.set_source_rgb(0.20, 0.24, 0.30);
    center_text(cr, x + width / 2.0, title_y, "No image loaded");

    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size((width.min(height) * 0.028).clamp(12.0, 18.0));
    cr.set_source_rgb(0.38, 0.42, 0.48);
    center_text(
        cr,
        x + width / 2.0,
        subtitle_y,
        "Use Fullscreen, Region, or Import to start editing.",
    );
    cr.set_source_rgb(0.46, 0.49, 0.55);
    center_text(
        cr,
        x + width / 2.0,
        hint_y,
        "Then annotate, copy, or export from the bottom bar.",
    );
}

pub(crate) fn paint_image(
    cr: &cairo::Context,
    bounds: (f64, f64, f64, f64),
    image: &Image,
    radius: f64,
    mode: ImageScaleMode,
    anchor: ImageAnchor,
) {
    if let Some(surface) = make_surface(image) {
        paint_surface(
            cr,
            bounds,
            image.width,
            image.height,
            &surface,
            radius,
            mode,
            anchor,
        );
    }
}

pub(crate) fn paint_image_for_document(
    cr: &cairo::Context,
    bounds: (f64, f64, f64, f64),
    image: &Image,
    radius: f64,
    document: &Document,
) {
    if let Some(surface) = make_surface(image) {
        paint_surface_for_document(
            cr,
            bounds,
            image.width,
            image.height,
            &surface,
            radius,
            document,
        );
    }
}

pub(crate) fn paint_surface(
    cr: &cairo::Context,
    bounds: (f64, f64, f64, f64),
    source_width: u32,
    source_height: u32,
    surface: &cairo::ImageSurface,
    radius: f64,
    mode: ImageScaleMode,
    anchor: ImageAnchor,
) {
    let (x, y, max_width, max_height) = bounds;
    let source = Image {
        width: source_width,
        height: source_height,
        data: Vec::new(),
    };
    let Some(layout) = layout_for_bounds_with_mode(&source, bounds, mode, anchor) else {
        return;
    };

    rounded_rect(cr, x, y, max_width, max_height, radius);
    cr.clip();
    rounded_rect(
        cr,
        layout.image_x,
        layout.image_y,
        layout.image_width,
        layout.image_height,
        radius,
    );
    cr.clip();

    cr.save().ok();
    cr.translate(layout.image_x, layout.image_y);
    cr.scale(layout.image_scale, layout.image_scale);
    cr.set_source_surface(surface, 0.0, 0.0).ok();
    cr.paint().ok();
    cr.restore().ok();

    cr.reset_clip();
}

pub(crate) fn paint_surface_for_document(
    cr: &cairo::Context,
    bounds: (f64, f64, f64, f64),
    source_width: u32,
    source_height: u32,
    surface: &cairo::ImageSurface,
    radius: f64,
    document: &Document,
) {
    let (x, y, max_width, max_height) = bounds;
    let source = Image {
        width: source_width,
        height: source_height,
        data: Vec::new(),
    };
    let Some(layout) = layout_for_document(&source, bounds, document) else {
        return;
    };

    rounded_rect(cr, x, y, max_width, max_height, radius);
    cr.clip();
    rounded_rect(
        cr,
        layout.image_x,
        layout.image_y,
        layout.image_width,
        layout.image_height,
        radius,
    );
    cr.clip();

    cr.save().ok();
    cr.translate(layout.image_x, layout.image_y);
    cr.scale(layout.image_scale, layout.image_scale);
    cr.set_source_surface(surface, 0.0, 0.0).ok();
    cr.paint().ok();
    cr.restore().ok();

    cr.reset_clip();
}

pub(crate) fn make_surface(image: &Image) -> Option<cairo::ImageSurface> {
    let mut surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        image.width as i32,
        image.height as i32,
    )
    .ok()?;

    {
        let stride = surface.stride() as usize;
        let mut data = surface.data().ok()?;

        for y in 0..image.height as usize {
            for x in 0..image.width as usize {
                let src = (y * image.width as usize + x) * 4;
                let dst = y * stride + x * 4;

                let r = image.data[src];
                let g = image.data[src + 1];
                let b = image.data[src + 2];
                let a = image.data[src + 3];

                data[dst] = ((b as u16 * a as u16) / 255) as u8;
                data[dst + 1] = ((g as u16 * a as u16) / 255) as u8;
                data[dst + 2] = ((r as u16 * a as u16) / 255) as u8;
                data[dst + 3] = a;
            }
        }
    }

    surface.mark_dirty();
    Some(surface)
}

pub(crate) fn center_text(cr: &cairo::Context, center_x: f64, baseline_y: f64, text: &str) {
    let Ok(extents) = cr.text_extents(text) else {
        return;
    };
    cr.move_to(
        center_x - extents.width() / 2.0 - extents.x_bearing(),
        baseline_y,
    );
    let _ = cr.show_text(text);
}

pub(crate) fn blurred_region_image(
    image: &Image,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: f32,
) -> Option<Image> {
    if width == 0 || height == 0 {
        return None;
    }

    let mut sub = Vec::with_capacity((width * height * 4) as usize);
    for row in y..(y + height) {
        let start = ((row * image.width + x) * 4) as usize;
        let end = start + (width * 4) as usize;
        sub.extend_from_slice(&image.data[start..end]);
    }

    let rgba = image::RgbaImage::from_raw(width, height, sub)?;
    let blurred = image::imageops::blur(&rgba, radius.max(2.0));
    Some(Image::from_dynamic(image::DynamicImage::ImageRgba8(
        blurred,
    )))
}

pub(crate) fn draw_resize_handles(cr: &cairo::Context, bounds: (f64, f64, f64, f64)) {
    let (x, y, width, height) = bounds;
    let right = x + width;
    let bottom = y + height;
    for (hx, hy) in [(x, y), (right, y), (x, bottom), (right, bottom)] {
        cr.set_source_rgba(0.07, 0.10, 0.14, 0.96);
        cr.new_sub_path();
        cr.arc(hx, hy, 6.5, 0.0, std::f64::consts::TAU);
        cr.fill_preserve().ok();
        cr.set_source_rgba(0.95, 0.98, 1.0, 1.0);
        cr.set_line_width(2.0);
        cr.stroke().ok();
    }
}

pub(crate) fn draw_arrow_resize_handles(
    cr: &cairo::Context,
    layout: CanvasLayout,
    annotation: &Annotation,
) {
    let Annotation::Arrow { from, to, .. } = annotation else {
        return;
    };
    let handles = [
        (
            layout.image_x + from.x as f64 * layout.image_scale,
            layout.image_y + from.y as f64 * layout.image_scale,
        ),
        (
            layout.image_x + to.x as f64 * layout.image_scale,
            layout.image_y + to.y as f64 * layout.image_scale,
        ),
    ];
    for (hx, hy) in handles {
        cr.set_source_rgba(0.07, 0.10, 0.14, 0.96);
        cr.new_sub_path();
        cr.arc(hx, hy, 6.5, 0.0, std::f64::consts::TAU);
        cr.fill_preserve().ok();
        cr.set_source_rgba(0.95, 0.98, 1.0, 1.0);
        cr.set_line_width(2.0);
        cr.stroke().ok();
    }
}

pub(crate) fn rounded_rect(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    let radius = radius.min(width / 2.0).min(height / 2.0);
    let degrees = std::f64::consts::PI / 180.0;

    cr.new_sub_path();
    cr.arc(
        x + width - radius,
        y + radius,
        radius,
        -90.0 * degrees,
        0.0 * degrees,
    );
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0 * degrees,
        90.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + height - radius,
        radius,
        90.0 * degrees,
        180.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        180.0 * degrees,
        270.0 * degrees,
    );
    cr.close_path();
}

pub(crate) fn set_color(cr: &cairo::Context, color: &Color) {
    cr.set_source_rgba(
        to_f64(color.r),
        to_f64(color.g),
        to_f64(color.b),
        to_f64(color.a),
    );
}

pub(crate) fn to_f64(value: u8) -> f64 {
    f64::from(value) / 255.0
}
