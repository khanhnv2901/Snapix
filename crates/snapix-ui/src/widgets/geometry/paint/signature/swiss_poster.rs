use gtk4::cairo;

pub(crate) fn paint_swiss_poster_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    cr.set_source_rgb(0.95, 0.95, 0.92);
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.86, 0.10, 0.14, 0.92);
    cr.rectangle(
        x + width * 0.08,
        y + height * 0.10,
        width * 0.12,
        height * 0.72,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.10, 0.12, 0.16, 0.10 + 0.08 * intensity);
    cr.set_line_width(1.0);
    let columns = 6;
    for i in 1..columns {
        let t = i as f64 / columns as f64;
        let px = x + width * (0.18 + 0.74 * t);
        cr.move_to(px.floor() + 0.5, y + height * 0.08);
        cr.line_to(px.floor() + 0.5, y + height * 0.92);
    }
    cr.stroke().ok();

    cr.set_source_rgba(0.08, 0.08, 0.10, 0.92);
    cr.rectangle(
        x + width * 0.28,
        y + height * 0.18,
        width * 0.28,
        height * 0.02,
    );
    cr.rectangle(
        x + width * 0.28,
        y + height * 0.24,
        width * 0.42,
        height * 0.014,
    );
    cr.rectangle(
        x + width * 0.28,
        y + height * 0.30,
        width * 0.22,
        height * 0.014,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.08, 0.08, 0.10, 0.08 + 0.10 * intensity);
    cr.rectangle(
        x + width * 0.60,
        y + height * 0.56,
        width * 0.18,
        height * 0.18,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.86, 0.10, 0.14, 0.16 + 0.20 * intensity);
    cr.rectangle(
        x + width * 0.28,
        y + height * 0.62,
        width * 0.44,
        height * 0.05,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.10, 0.12, 0.16, 0.72);
    cr.set_line_width(1.4);
    cr.move_to(x + width * 0.28, y + height * 0.84);
    cr.line_to(x + width * 0.86, y + height * 0.84);
    cr.stroke().ok();
}
