use super::rounded_rect;
use gtk4::cairo;

pub(crate) fn paint_redacted_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);
    let base = cairo::LinearGradient::new(x, y, x + width, y + height);
    base.add_color_stop_rgb(0.0, 0.12, 0.14, 0.18);
    base.add_color_stop_rgb(1.0, 0.18, 0.21, 0.27);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.04, 0.05, 0.07, 0.12 + 0.36 * intensity);
    for i in 0..4 {
        let top = y + height * (0.16 + i as f64 * 0.15);
        cr.rectangle(
            x + width * 0.10,
            top,
            width * (0.58 - i as f64 * 0.05),
            height * 0.07,
        );
        cr.fill().ok();
    }

    cr.set_source_rgba(0.88, 0.28, 0.30, 0.08 + 0.28 * intensity);
    cr.rectangle(
        x + width * 0.72,
        y + height * 0.12,
        width * 0.16,
        height * 0.12,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.93, 0.95, 0.98, 0.04 + 0.14 * intensity);
    cr.set_line_width(2.0);
    rounded_rect(
        cr,
        x + width * 0.06,
        y + height * 0.10,
        width * 0.88,
        height * 0.80,
        18.0,
    );
    cr.stroke().ok();
}
