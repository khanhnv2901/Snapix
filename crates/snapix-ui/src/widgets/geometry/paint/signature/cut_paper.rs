use gtk4::cairo;

pub(crate) fn paint_cut_paper_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    let base = cairo::LinearGradient::new(x, y, x, y + height);
    base.add_color_stop_rgb(0.0, 0.97, 0.94, 0.88);
    base.add_color_stop_rgb(1.0, 0.90, 0.84, 0.76);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.10, 0.12, 0.16, 0.05 + 0.08 * intensity);
    cr.move_to(x - width * 0.06, y + height * 0.20);
    cr.line_to(x + width * 0.36, y - height * 0.02);
    cr.line_to(x + width * 0.72, y + height * 0.10);
    cr.line_to(x + width * 0.28, y + height * 0.54);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(0.95, 0.92, 0.86, 0.84);
    cr.move_to(x + width * 0.56, y - height * 0.02);
    cr.line_to(x + width * 0.98, y + height * 0.18);
    cr.line_to(x + width * 0.82, y + height * 0.50);
    cr.line_to(x + width * 0.44, y + height * 0.28);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(0.90, 0.58, 0.46, 0.12 + 0.22 * intensity);
    cr.move_to(x + width * 0.68, y + height * 0.12);
    cr.line_to(x + width * 0.90, y + height * 0.08);
    cr.line_to(x + width * 0.88, y + height * 0.28);
    cr.line_to(x + width * 0.64, y + height * 0.24);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(0.18, 0.20, 0.26, 0.05 + 0.12 * intensity);
    cr.move_to(x + width * 0.52, y + height);
    cr.line_to(x + width, y + height * 0.62);
    cr.line_to(x + width, y + height);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.22 + 0.26 * intensity);
    cr.rectangle(
        x + width * 0.08,
        y + height * 0.10,
        width * 0.24,
        height * 0.06,
    );
    cr.rectangle(
        x + width * 0.11,
        y + height * 0.17,
        width * 0.16,
        height * 0.04,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.10, 0.12, 0.16, 0.06 + 0.08 * intensity);
    cr.set_line_width(1.2);
    cr.move_to(x + width * 0.48, y + height * 0.32);
    cr.line_to(x + width * 0.84, y + height * 0.48);
    cr.stroke().ok();
}
