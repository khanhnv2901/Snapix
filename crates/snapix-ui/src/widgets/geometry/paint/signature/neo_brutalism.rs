use gtk4::cairo;

pub(crate) fn paint_neo_brutalism_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    cr.set_source_rgb(0.98, 0.95, 0.88);
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.16, 0.20, 0.94, 0.92);
    cr.rectangle(
        x + width * 0.62,
        y + height * 0.08,
        width * 0.26,
        height * 0.20,
    );
    cr.fill().ok();

    cr.set_source_rgba(1.0, 0.40, 0.16, 0.96);
    cr.rectangle(
        x + width * 0.10,
        y + height * 0.66,
        width * 0.28,
        height * 0.18,
    );
    cr.fill().ok();

    cr.set_source_rgba(1.0, 0.88, 0.12, 0.96);
    cr.arc(
        x + width * 0.22,
        y + height * 0.24,
        width.min(height) * 0.11,
        0.0,
        std::f64::consts::TAU,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.08, 0.08, 0.10, 0.12 + 0.14 * intensity);
    cr.rectangle(
        x + width * 0.52,
        y + height * 0.52,
        width * 0.32,
        height * 0.16,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.08, 0.08, 0.10, 0.92);
    cr.set_line_width((2.5 + 1.5 * intensity).clamp(2.0, 4.0));
    cr.rectangle(
        x + width * 0.07,
        y + height * 0.10,
        width * 0.86,
        height * 0.78,
    );
    cr.stroke().ok();

    cr.set_source_rgba(0.08, 0.08, 0.10, 0.88);
    cr.rectangle(
        x + width * 0.09,
        y + height * 0.14,
        width * 0.20,
        height * 0.028,
    );
    cr.rectangle(
        x + width * 0.09,
        y + height * 0.19,
        width * 0.13,
        height * 0.018,
    );
    cr.fill().ok();
}
