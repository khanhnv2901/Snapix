use gtk4::cairo;
use snapix_core::canvas::{Background, BackgroundStyleId};

use super::{paint_background, rounded_rect};

pub(crate) fn signature_shadow_profile(background: &Background) -> (f64, f64) {
    match background {
        Background::Style {
            id: BackgroundStyleId::Blueprint,
            intensity,
        } => (
            1.0 + *intensity as f64 * 0.08,
            1.0 + *intensity as f64 * 0.16,
        ),
        Background::Style {
            id: BackgroundStyleId::MidnightPanel,
            intensity,
        } => (
            1.02 + *intensity as f64 * 0.20,
            1.0 + *intensity as f64 * 0.22,
        ),
        Background::Style {
            id: BackgroundStyleId::CutPaper,
            intensity,
        } => (
            0.92 - *intensity as f64 * 0.10,
            0.98 - *intensity as f64 * 0.10,
        ),
        Background::Style {
            id: BackgroundStyleId::TerminalGlow,
            intensity,
        } => (
            1.04 + *intensity as f64 * 0.28,
            1.0 + *intensity as f64 * 0.12,
        ),
        Background::Style {
            id: BackgroundStyleId::Redacted,
            intensity,
        } => (
            0.92 + *intensity as f64 * 0.08,
            1.0 + *intensity as f64 * 0.08,
        ),
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

    let (fill_r, fill_g, fill_b, fill_a, stroke_r, stroke_g, stroke_b, stroke_a) = match background
    {
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

pub(crate) fn paint_signature_background(
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
        BackgroundStyleId::CutPaper => {
            paint_cut_paper_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::TerminalGlow => {
            paint_terminal_glow_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::Redacted => {
            paint_redacted_background(cr, x, y, width, height, intensity)
        }
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
    cr.rectangle(
        x + width * 0.68,
        y + height * 0.08,
        width * 0.24,
        height * 0.24,
    );
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
    rounded_rect(
        cr,
        x + width * 0.05,
        y + height * 0.08,
        width * 0.90,
        height * 0.84,
        22.0,
    );
    cr.stroke().ok();

    cr.set_source_rgba(0.44, 0.68, 1.0, 0.03 + 0.08 * intensity);
    rounded_rect(
        cr,
        x + width * 0.08,
        y + height * 0.12,
        width * 0.84,
        height * 0.76,
        18.0,
    );
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
    cr.rectangle(
        x + width * 0.68,
        y + height * 0.10,
        width * 0.22,
        height * 0.18,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.14, 0.16, 0.22, 0.05 + 0.16 * intensity);
    cr.move_to(x + width * 0.62, y + height);
    cr.line_to(x + width, y + height * 0.62);
    cr.line_to(x + width, y + height);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.16 + 0.32 * intensity);
    cr.rectangle(
        x + width * 0.08,
        y + height * 0.10,
        width * 0.20,
        height * 0.07,
    );
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
    cr.rectangle(
        x + width * 0.08,
        y + height * 0.12,
        width * 0.22,
        height * 0.10,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.98, 0.77, 0.26, 0.10 + 0.25 * intensity);
    cr.rectangle(
        x + width * 0.76,
        y + height * 0.76,
        width * 0.12,
        height * 0.07,
    );
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
        cr.rectangle(
            x + width * 0.10,
            top,
            width * (0.58 - i as f64 * 0.05),
            height * 0.07,
        );
        cr.fill().ok();
    }

    cr.set_source_rgba(0.88, 0.28, 0.30, 0.08 + 0.28 * intensity);
    cr.rectangle(
        x + width * 0.72,
        y + height * 0.12,
        width * 0.16,
        height * 0.12,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.93, 0.95, 0.98, 0.04 + 0.14 * intensity);
    cr.set_line_width(2.0);
    rounded_rect(
        cr,
        x + width * 0.06,
        y + height * 0.10,
        width * 0.88,
        height * 0.80,
        18.0,
    );
    cr.stroke().ok();
}
