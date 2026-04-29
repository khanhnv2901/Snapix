use gtk4::cairo;

pub(crate) fn paint_terminal_glow_background(
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
