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
