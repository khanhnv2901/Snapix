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
