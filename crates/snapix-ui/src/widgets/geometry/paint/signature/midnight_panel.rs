use super::rounded_rect;
use gtk4::cairo;

pub(crate) fn paint_midnight_panel_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    let base = cairo::RadialGradient::new(
        x + width * 0.56,
        y + height * 0.24,
        width * 0.06,
        x + width * 0.56,
        y + height * 0.28,
        width.max(height) * 0.92,
    );
    base.add_color_stop_rgb(0.0, 0.16, 0.21, 0.34);
    base.add_color_stop_rgb(0.50, 0.08, 0.11, 0.19);
    base.add_color_stop_rgb(1.0, 0.03, 0.05, 0.10);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    let lift = cairo::LinearGradient::new(x, y, x, y + height);
    lift.add_color_stop_rgba(0.0, 0.84, 0.91, 1.0, 0.03 + 0.05 * intensity);
    lift.add_color_stop_rgba(0.55, 0.18, 0.24, 0.36, 0.0);
    lift.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.0);
    cr.set_source(&lift).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.78, 0.86, 1.0, 0.05 + 0.10 * intensity);
    cr.set_line_width(2.2);
    rounded_rect(
        cr,
        x + width * 0.04,
        y + height * 0.07,
        width * 0.92,
        height * 0.86,
        24.0,
    );
    cr.stroke().ok();

    cr.set_source_rgba(0.42, 0.60, 0.96, 0.04 + 0.10 * intensity);
    cr.set_line_width(1.3);
    rounded_rect(
        cr,
        x + width * 0.08,
        y + height * 0.12,
        width * 0.84,
        height * 0.74,
        20.0,
    );
    cr.stroke().ok();

    cr.set_source_rgba(0.86, 0.92, 1.0, 0.06 + 0.06 * intensity);
    cr.rectangle(
        x + width * 0.11,
        y + height * 0.17,
        width * 0.24,
        height * 0.015,
    );
    cr.rectangle(
        x + width * 0.11,
        y + height * 0.22,
        width * 0.16,
        height * 0.012,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.22, 0.30, 0.46, 0.30 + 0.22 * intensity);
    rounded_rect(
        cr,
        x + width * 0.70,
        y + height * 0.12,
        width * 0.15,
        height * 0.11,
        12.0,
    );
    cr.fill().ok();

    let glow = cairo::LinearGradient::new(x + width * 0.56, y, x + width, y + height * 0.70);
    glow.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.0);
    glow.add_color_stop_rgba(0.42, 0.14, 0.32, 0.68, 0.0);
    glow.add_color_stop_rgba(0.78, 0.20, 0.52, 1.0, 0.04 + 0.10 * intensity);
    glow.add_color_stop_rgba(1.0, 0.08, 0.42, 0.96, 0.08 + 0.16 * intensity);
    cr.set_source(&glow).ok();
    cr.rectangle(x + width * 0.58, y, width * 0.42, height);
    cr.fill().ok();

    cr.set_source_rgba(0.42, 0.74, 1.0, 0.08 + 0.14 * intensity);
    cr.rectangle(
        x + width * 0.94,
        y + height * 0.12,
        width * 0.01,
        height * 0.64,
    );
    cr.fill().ok();
}
