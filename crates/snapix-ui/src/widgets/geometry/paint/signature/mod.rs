pub(crate) mod blueprint;
pub(crate) mod cut_paper;
pub(crate) mod midnight_panel;
pub(crate) mod redacted;
pub(crate) mod style;
pub(crate) mod terminal_glow;
pub(crate) mod warning_tape;

use gtk4::cairo;
use snapix_core::canvas::{Background, BackgroundStyleId};

use super::{paint_background, rounded_rect};
use style::{preview_palette, shadow_profile};

pub(crate) fn signature_shadow_profile(background: &Background) -> (f64, f64) {
    match background {
        Background::Style { id, intensity } => {
            let profile = shadow_profile(*id, *intensity as f64);
            (profile.blur_scale, profile.strength_scale)
        }
        _ => (1.0, 1.0),
    }
}

pub(crate) fn paint_signature_preview_thumbnail(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    background: &Background,
) {
    paint_background(cr, 0.0, 0.0, width, height, background, 28.0);

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

    let palette = match background {
        Background::Style { id, .. } => preview_palette(*id),
        _ => preview_palette(BackgroundStyleId::Blueprint),
    };
    let (fill_r, fill_g, fill_b, fill_a) = palette.fill_rgba;
    let (stroke_r, stroke_g, stroke_b, stroke_a) = palette.stroke_rgba;

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
    radius: f64,
    id: BackgroundStyleId,
    intensity: f64,
) {
    if radius > 0.0 {
        rounded_rect(cr, x, y, width, height, radius);
        cr.clip();
    }

    match id {
        BackgroundStyleId::Blueprint => {
            blueprint::paint_blueprint_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::MidnightPanel => {
            midnight_panel::paint_midnight_panel_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::CutPaper => {
            cut_paper::paint_cut_paper_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::TerminalGlow => {
            terminal_glow::paint_terminal_glow_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::Redacted => {
            redacted::paint_redacted_background(cr, x, y, width, height, intensity)
        }
        BackgroundStyleId::WarningTape => {
            warning_tape::paint_warning_tape_background(cr, x, y, width, height, intensity)
        }
    }

    // Add subtle grain/texture for "Editorial Tech" look
    let grain_opacity = match id {
        BackgroundStyleId::CutPaper => 0.04,
        BackgroundStyleId::TerminalGlow => 0.03,
        BackgroundStyleId::Blueprint => 0.02,
        BackgroundStyleId::WarningTape => 0.025,
        _ => 0.015,
    };
    paint_grain(
        cr,
        x,
        y,
        width,
        height,
        grain_opacity * (0.5 + 0.5 * intensity),
    );

    if radius > 0.0 {
        cr.reset_clip();
    }
}

fn paint_grain(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, opacity: f64) {
    if opacity <= 0.0 {
        return;
    }

    thread_local! {
        static GRAIN_PATTERN: cairo::SurfacePattern = {
            let size = 64;
            let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, size, size)
                .expect("failed to create noise surface");

            if let Ok(mut data) = surface.data() {
                let mut state = 42u32;
                for i in 0..(size * size) as usize {
                    state = state.wrapping_mul(1103515245).wrapping_add(12345);
                    let val = ((state >> 16) & 0xFF) as u8;
                    // Generate opaque grain; opacity will be applied during paint
                    data[i * 4] = val; // B
                    data[i * 4 + 1] = val; // G
                    data[i * 4 + 2] = val; // R
                    data[i * 4 + 3] = 255; // A
                }
            }
            surface.mark_dirty();

            let pattern = cairo::SurfacePattern::create(&surface);
            pattern.set_extend(cairo::Extend::Repeat);
            pattern.set_filter(cairo::Filter::Nearest);
            pattern
        };
    }

    GRAIN_PATTERN.with(|pattern| {
        cr.save().ok();
        cr.set_source(pattern).ok();
        cr.rectangle(x, y, width, height);
        cr.clip();
        cr.paint_with_alpha(opacity).ok();
        cr.restore().ok();
    });
}

#[cfg(test)]
mod tests {
    use snapix_core::canvas::{Background, BackgroundStyleId};

    use super::signature_shadow_profile;

    #[test]
    fn signature_shadow_profile_uses_style_specific_scales() {
        let blueprint = Background::Style {
            id: BackgroundStyleId::Blueprint,
            intensity: 0.65,
        };
        let cut_paper = Background::Style {
            id: BackgroundStyleId::CutPaper,
            intensity: 0.65,
        };

        let blueprint_profile = signature_shadow_profile(&blueprint);
        let cut_paper_profile = signature_shadow_profile(&cut_paper);

        assert!(blueprint_profile.0 > 1.0);
        assert!(blueprint_profile.1 > 1.0);
        assert!(cut_paper_profile.0 < 1.0);
        assert!(cut_paper_profile.1 < 1.0);
    }

    #[test]
    fn non_signature_background_uses_neutral_shadow_profile() {
        let background = Background::BlurredScreenshot { radius: 24.0 };
        assert_eq!(signature_shadow_profile(&background), (1.0, 1.0));
    }
}
