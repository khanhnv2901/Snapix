use gtk4::cairo;

pub(crate) fn paint_blueprint_background(
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
