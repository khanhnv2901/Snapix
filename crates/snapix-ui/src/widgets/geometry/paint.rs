use gtk4::cairo;
use libadwaita::StyleManager;
use snapix_core::canvas::{
    Annotation, Background, BackgroundStyleId, Color, Document, Image, ImageAnchor,
    ImageScaleMode,
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
        Background::Style { id, intensity } => {
            paint_signature_background(cr, x, y, width, height, *id, *intensity as f64);
            return;
        }
    }

    rounded_rect(cr, x, y, width, height, 28.0);
    cr.fill().ok();
}

pub(crate) fn signature_shadow_profile(background: &Background) -> (f64, f64) {
    match background {
        Background::Style {
            id: BackgroundStyleId::Blueprint,
            intensity,
        } => (1.0 + *intensity as f64 * 0.08, 1.0 + *intensity as f64 * 0.16),
        Background::Style {
            id: BackgroundStyleId::MidnightPanel,
            intensity,
        } => (1.02 + *intensity as f64 * 0.20, 1.0 + *intensity as f64 * 0.22),
        Background::Style {
            id: BackgroundStyleId::CutPaper,
            intensity,
        } => (0.92 - *intensity as f64 * 0.10, 0.98 - *intensity as f64 * 0.10),
        Background::Style {
            id: BackgroundStyleId::TerminalGlow,
            intensity,
        } => (1.04 + *intensity as f64 * 0.28, 1.0 + *intensity as f64 * 0.12),
        Background::Style {
            id: BackgroundStyleId::Redacted,
            intensity,
        } => (0.92 + *intensity as f64 * 0.08, 1.0 + *intensity as f64 * 0.08),
        _ => (1.0, 1.0),
    }
}

pub(crate) fn paint_signature_preview_thumbnail(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    background: &Background,
) {
    paint_background(cr, 0.0, 0.0, width, height, background);

    let inset_x = width * 0.17;
    let inset_y = height * 0.19;
    let inset_w = width * 0.66;
    let inset_h = height * 0.50;
    let radius = 9.0;

    let (shadow_blur_scale, shadow_strength_scale) = signature_shadow_profile(background);
    let shadow_steps = 10;
    let base_blur = 6.0 * shadow_blur_scale;
    for i in 1..=shadow_steps {
        let t = i as f64 / shadow_steps as f64;
        let expand = t * base_blur;
        let alpha = (0.09 * shadow_strength_scale) * (1.0 - t).powf(1.7);
        cr.set_source_rgba(0.0, 0.0, 0.0, alpha);
        rounded_rect(
            cr,
            inset_x - expand + 1.5 * t,
            inset_y - expand + 2.0 * t,
            inset_w + expand * 2.0,
            inset_h + expand * 2.0,
            radius + expand * 0.35,
        );
        cr.fill().ok();
    }

    let (fill_r, fill_g, fill_b, fill_a, stroke_r, stroke_g, stroke_b, stroke_a) =
        match background {
            Background::Style {
                id: BackgroundStyleId::CutPaper,
                ..
            } => (0.99, 0.98, 0.95, 0.92, 0.20, 0.18, 0.16, 0.16),
            Background::Style {
                id: BackgroundStyleId::TerminalGlow,
                ..
            } => (0.08, 0.14, 0.14, 0.90, 0.28, 0.96, 0.76, 0.16),
            Background::Style {
                id: BackgroundStyleId::Redacted,
                ..
            } => (0.12, 0.13, 0.16, 0.90, 0.92, 0.94, 0.98, 0.10),
            _ => (0.96, 0.97, 1.0, 0.90, 0.82, 0.88, 1.0, 0.12),
        };

    cr.set_source_rgba(fill_r, fill_g, fill_b, fill_a);
    rounded_rect(cr, inset_x, inset_y, inset_w, inset_h, radius);
    cr.fill_preserve().ok();
    cr.set_source_rgba(stroke_r, stroke_g, stroke_b, stroke_a);
    cr.set_line_width(1.2);
    cr.stroke().ok();

    let header_h = inset_h * 0.16;
    cr.set_source_rgba(stroke_r, stroke_g, stroke_b, stroke_a * 1.2);
    cr.rectangle(inset_x, inset_y, inset_w, header_h);
    cr.fill().ok();

    cr.set_source_rgba(stroke_r, stroke_g, stroke_b, stroke_a * 1.6);
    cr.rectangle(
        inset_x + inset_w * 0.08,
        inset_y + header_h + inset_h * 0.16,
        inset_w * 0.56,
        2.0,
    );
    cr.rectangle(
        inset_x + inset_w * 0.08,
        inset_y + header_h + inset_h * 0.34,
        inset_w * 0.42,
        2.0,
    );
    cr.fill().ok();
}

fn paint_signature_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    id: BackgroundStyleId,
    intensity: f64,
) {
    rounded_rect(cr, x, y, width, height, 28.0);
    cr.clip();

    match id {
        BackgroundStyleId::Blueprint => {
            paint_blueprint_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::MidnightPanel => {
            paint_midnight_panel_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::CutPaper => paint_cut_paper_background(cr, x, y, width, height, intensity),
        BackgroundStyleId::TerminalGlow => {
            paint_terminal_glow_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::Redacted => paint_redacted_background(cr, x, y, width, height, intensity),
    }

    cr.reset_clip();
}

fn paint_blueprint_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);
    let base = cairo::LinearGradient::new(x, y, x + width, y + height);
    base.add_color_stop_rgb(0.0, 0.07, 0.11, 0.20);
    base.add_color_stop_rgb(1.0, 0.05, 0.08, 0.16);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.35, 0.84, 0.98, 0.06 + 0.12 * intensity);
    let grid = (width.min(height) / 12.0).clamp(18.0, 42.0);
    let mut gx = x;
    while gx <= x + width {
        cr.move_to(gx, y);
        cr.line_to(gx, y + height);
        gx += grid;
    }
    let mut gy = y;
    while gy <= y + height {
        cr.move_to(x, gy);
        cr.line_to(x + width, gy);
        gy += grid;
    }
    cr.set_line_width(1.0);
    cr.stroke().ok();

    cr.set_source_rgba(0.25, 0.88, 1.0, 0.08 + 0.18 * intensity);
    cr.rectangle(x + width * 0.68, y + height * 0.08, width * 0.24, height * 0.24);
    cr.fill().ok();

    cr.set_source_rgba(0.42, 0.93, 1.0, 0.10 + 0.25 * intensity);
    cr.set_line_width(2.0);
    cr.move_to(x + width * 0.10, y + height * 0.78);
    cr.line_to(x + width * 0.44, y + height * 0.78);
    cr.move_to(x + width * 0.10, y + height * 0.84);
    cr.line_to(x + width * 0.28, y + height * 0.84);
    cr.stroke().ok();
}

fn paint_midnight_panel_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);
    let base = cairo::RadialGradient::new(
        x + width * 0.5,
        y + height * 0.35,
        width * 0.10,
        x + width * 0.5,
        y + height * 0.35,
        width.max(height) * 0.85,
    );
    base.add_color_stop_rgb(0.0, 0.15, 0.20, 0.33);
    base.add_color_stop_rgb(0.65, 0.09, 0.12, 0.20);
    base.add_color_stop_rgb(1.0, 0.05, 0.07, 0.12);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.72, 0.82, 1.0, 0.04 + 0.10 * intensity);
    cr.set_line_width(2.0);
    rounded_rect(cr, x + width * 0.05, y + height * 0.08, width * 0.90, height * 0.84, 22.0);
    cr.stroke().ok();

    cr.set_source_rgba(0.44, 0.68, 1.0, 0.03 + 0.08 * intensity);
    rounded_rect(cr, x + width * 0.08, y + height * 0.12, width * 0.84, height * 0.76, 18.0);
    cr.stroke().ok();

    let glow = cairo::LinearGradient::new(x, y + height * 0.1, x + width, y + height * 0.55);
    glow.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.0);
    glow.add_color_stop_rgba(0.55, 0.35, 0.54, 1.0, 0.0);
    glow.add_color_stop_rgba(1.0, 0.26, 0.48, 1.0, 0.04 + 0.16 * intensity);
    cr.set_source(&glow).ok();
    cr.rectangle(x + width * 0.58, y, width * 0.42, height);
    cr.fill().ok();
}

fn paint_cut_paper_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);
    cr.set_source_rgb(0.93, 0.89, 0.82);
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.18, 0.22, 0.28, 0.05 + 0.12 * intensity);
    cr.move_to(x, y + height * 0.18);
    cr.line_to(x + width * 0.42, y);
    cr.line_to(x + width * 0.68, y);
    cr.line_to(x + width * 0.22, y + height * 0.52);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(0.84, 0.44, 0.36, 0.06 + 0.20 * intensity);
    cr.rectangle(x + width * 0.68, y + height * 0.10, width * 0.22, height * 0.18);
    cr.fill().ok();

    cr.set_source_rgba(0.14, 0.16, 0.22, 0.05 + 0.16 * intensity);
    cr.move_to(x + width * 0.62, y + height);
    cr.line_to(x + width, y + height * 0.62);
    cr.line_to(x + width, y + height);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.16 + 0.32 * intensity);
    cr.rectangle(x + width * 0.08, y + height * 0.10, width * 0.20, height * 0.07);
    cr.fill().ok();
}

fn paint_terminal_glow_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);
    let base = cairo::LinearGradient::new(x, y, x, y + height);
    base.add_color_stop_rgb(0.0, 0.03, 0.08, 0.07);
    base.add_color_stop_rgb(0.55, 0.04, 0.10, 0.09);
    base.add_color_stop_rgb(1.0, 0.02, 0.05, 0.05);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    let glow = cairo::RadialGradient::new(
        x + width * 0.72,
        y + height * 0.30,
        width * 0.04,
        x + width * 0.72,
        y + height * 0.30,
        width.max(height) * 0.55,
    );
    glow.add_color_stop_rgba(0.0, 0.24, 0.97, 0.73, 0.08 + 0.24 * intensity);
    glow.add_color_stop_rgba(0.55, 0.09, 0.64, 0.46, 0.03 + 0.10 * intensity);
    glow.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.0);
    cr.set_source(&glow).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.27, 0.98, 0.75, 0.03 + 0.08 * intensity);
    let line_gap = (height / 24.0).clamp(8.0, 16.0);
    let mut gy = y + line_gap * 0.5;
    while gy <= y + height {
        cr.move_to(x, gy);
        cr.line_to(x + width, gy);
        gy += line_gap;
    }
    cr.set_line_width(1.0);
    cr.stroke().ok();

    cr.set_source_rgba(0.27, 0.98, 0.75, 0.06 + 0.18 * intensity);
    cr.rectangle(x + width * 0.08, y + height * 0.12, width * 0.22, height * 0.10);
    cr.fill().ok();

    cr.set_source_rgba(0.98, 0.77, 0.26, 0.10 + 0.25 * intensity);
    cr.rectangle(x + width * 0.76, y + height * 0.76, width * 0.12, height * 0.07);
    cr.fill().ok();
}

fn paint_redacted_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);
    let base = cairo::LinearGradient::new(x, y, x + width, y + height);
    base.add_color_stop_rgb(0.0, 0.12, 0.14, 0.18);
    base.add_color_stop_rgb(1.0, 0.18, 0.21, 0.27);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.04, 0.05, 0.07, 0.12 + 0.36 * intensity);
    for i in 0..4 {
        let top = y + height * (0.16 + i as f64 * 0.15);
        cr.rectangle(x + width * 0.10, top, width * (0.58 - i as f64 * 0.05), height * 0.07);
        cr.fill().ok();
    }

    cr.set_source_rgba(0.88, 0.28, 0.30, 0.08 + 0.28 * intensity);
    cr.rectangle(x + width * 0.72, y + height * 0.12, width * 0.16, height * 0.12);
    cr.fill().ok();

    cr.set_source_rgba(0.93, 0.95, 0.98, 0.04 + 0.14 * intensity);
    cr.set_line_width(2.0);
    rounded_rect(cr, x + width * 0.06, y + height * 0.10, width * 0.88, height * 0.80, 18.0);
    cr.stroke().ok();
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

#[allow(clippy::too_many_arguments)]
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
