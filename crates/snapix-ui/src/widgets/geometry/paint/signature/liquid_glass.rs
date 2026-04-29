use gtk4::cairo;

use super::rounded_rect;

pub(crate) fn paint_liquid_glass_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    let base = cairo::LinearGradient::new(x, y, x + width, y + height);
    base.add_color_stop_rgb(0.0, 0.93, 0.98, 1.0);
    base.add_color_stop_rgb(0.45, 0.98, 0.96, 1.0);
    base.add_color_stop_rgb(1.0, 0.92, 0.96, 1.0);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    paint_glow(
        cr,
        x + width * 0.18,
        y + height * 0.24,
        width.max(height) * 0.46,
        (0.18, 0.72, 1.0),
        0.24 * intensity,
    );
    paint_glow(
        cr,
        x + width * 0.78,
        y + height * 0.30,
        width.max(height) * 0.42,
        (0.86, 0.36, 1.0),
        0.20 * intensity,
    );
    paint_glow(
        cr,
        x + width * 0.52,
        y + height * 0.86,
        width.max(height) * 0.50,
        (0.34, 0.92, 0.80),
        0.22 * intensity,
    );

    paint_glass_ribbon(
        cr,
        x + width * 0.08,
        y + height * 0.18,
        width * 0.84,
        height * 0.26,
        0.18 + 0.20 * intensity,
    );
    paint_glass_ribbon(
        cr,
        x + width * 0.18,
        y + height * 0.58,
        width * 0.68,
        height * 0.22,
        0.14 + 0.16 * intensity,
    );

    let sheen = cairo::LinearGradient::new(x + width * 0.18, y, x + width * 0.82, y + height);
    sheen.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 0.0);
    sheen.add_color_stop_rgba(0.42, 1.0, 1.0, 1.0, 0.18 * intensity);
    sheen.add_color_stop_rgba(0.50, 1.0, 1.0, 1.0, 0.04 * intensity);
    sheen.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0);
    cr.set_source(&sheen).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();
}

fn paint_glow(
    cr: &cairo::Context,
    center_x: f64,
    center_y: f64,
    radius: f64,
    color: (f64, f64, f64),
    alpha: f64,
) {
    let glow = cairo::RadialGradient::new(center_x, center_y, 0.0, center_x, center_y, radius);
    glow.add_color_stop_rgba(0.0, color.0, color.1, color.2, alpha);
    glow.add_color_stop_rgba(0.62, color.0, color.1, color.2, alpha * 0.24);
    glow.add_color_stop_rgba(1.0, color.0, color.1, color.2, 0.0);
    cr.set_source(&glow).ok();
    cr.arc(center_x, center_y, radius, 0.0, std::f64::consts::TAU);
    cr.fill().ok();
}

fn paint_glass_ribbon(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, alpha: f64) {
    cr.save().ok();
    rounded_rect(cr, x, y, width, height, height * 0.48);
    cr.clip();

    let fill = cairo::LinearGradient::new(x, y, x + width, y + height);
    fill.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, alpha);
    fill.add_color_stop_rgba(0.52, 1.0, 1.0, 1.0, alpha * 0.38);
    fill.add_color_stop_rgba(1.0, 0.75, 0.92, 1.0, alpha * 0.42);
    cr.set_source(&fill).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, alpha * 0.70);
    cr.set_line_width(1.4);
    rounded_rect(
        cr,
        x + width * 0.03,
        y + height * 0.12,
        width * 0.94,
        height * 0.32,
        height * 0.16,
    );
    cr.stroke().ok();
    cr.restore().ok();
}
