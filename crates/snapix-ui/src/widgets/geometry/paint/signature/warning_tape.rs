use gtk4::cairo;

pub(crate) fn paint_warning_tape_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    let base = cairo::LinearGradient::new(x, y, x + width, y + height);
    base.add_color_stop_rgb(0.0, 0.96, 0.78, 0.10);
    base.add_color_stop_rgb(0.52, 0.90, 0.67, 0.07);
    base.add_color_stop_rgb(1.0, 0.98, 0.83, 0.24);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    let stripe_span = (width.min(height) / 5.8).clamp(28.0, 68.0);
    let stripe_width = stripe_span * (0.48 + 0.10 * intensity);
    let travel = width + height + stripe_span * 4.0;
    let start = x - height - stripe_span * 2.0;

    let mut offset = 0.0;
    while offset < travel {
        cr.save().ok();
        cr.translate(start + offset, y);
        cr.move_to(0.0, 0.0);
        cr.line_to(stripe_width, 0.0);
        cr.line_to(stripe_width + height, height);
        cr.line_to(height, height);
        cr.close_path();
        cr.set_source_rgba(0.07, 0.08, 0.10, 0.88);
        cr.fill().ok();
        cr.restore().ok();
        offset += stripe_span;
    }

    let band_height = height * (0.14 + 0.02 * intensity);
    for ratio in [0.18_f64, 0.76] {
        cr.set_source_rgba(0.99, 0.90, 0.36, 0.16 + 0.14 * intensity);
        cr.rectangle(x, y + height * ratio, width, band_height);
        cr.fill().ok();
    }

    cr.set_source_rgba(1.0, 0.97, 0.74, 0.10 + 0.10 * intensity);
    cr.set_line_width(2.0);
    cr.move_to(x + width * 0.10, y + height * 0.16);
    cr.line_to(x + width * 0.36, y + height * 0.16);
    cr.move_to(x + width * 0.70, y + height * 0.84);
    cr.line_to(x + width * 0.90, y + height * 0.84);
    cr.stroke().ok();
}
