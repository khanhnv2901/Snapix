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
    base.add_color_stop_rgb(0.0, 0.97, 0.78, 0.14);
    base.add_color_stop_rgb(0.48, 0.90, 0.68, 0.10);
    base.add_color_stop_rgb(1.0, 0.98, 0.86, 0.26);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    let banner_y = y + height * 0.14;
    let banner_h = height * (0.24 + 0.02 * intensity);
    cr.set_source_rgba(0.10, 0.11, 0.13, 0.10 + 0.10 * intensity);
    cr.rectangle(x, banner_y + banner_h * 0.12, width, banner_h);
    cr.fill().ok();

    cr.set_source_rgba(0.98, 0.82, 0.12, 0.96);
    cr.rectangle(x, banner_y, width, banner_h);
    cr.fill().ok();

    let stripe_span = (banner_h / 1.45).clamp(22.0, 40.0);
    let stripe_width = stripe_span * (0.56 + 0.08 * intensity);
    let travel = width + banner_h + stripe_span * 4.0;
    let start = x - banner_h - stripe_span * 2.0;

    let mut offset = 0.0;
    while offset < travel {
        cr.save().ok();
        cr.translate(start + offset, banner_y);
        cr.move_to(0.0, 0.0);
        cr.line_to(stripe_width, 0.0);
        cr.line_to(stripe_width + banner_h, banner_h);
        cr.line_to(banner_h, banner_h);
        cr.close_path();
        cr.set_source_rgba(0.06, 0.06, 0.07, 0.92);
        cr.fill().ok();
        cr.restore().ok();
        offset += stripe_span;
    }

    cr.set_source_rgba(0.08, 0.08, 0.10, 0.12 + 0.10 * intensity);
    cr.rectangle(x, y + height * 0.76, width, height * 0.06);
    cr.fill().ok();

    cr.set_source_rgba(1.0, 0.97, 0.74, 0.12 + 0.10 * intensity);
    cr.set_line_width(2.0);
    cr.move_to(x + width * 0.08, y + height * 0.76);
    cr.line_to(x + width * 0.28, y + height * 0.76);
    cr.move_to(x + width * 0.72, y + height * 0.82);
    cr.line_to(x + width * 0.92, y + height * 0.82);
    cr.stroke().ok();

    let label_w = width * 0.14;
    let label_h = banner_h * 0.42;
    cr.set_source_rgba(0.12, 0.12, 0.14, 0.94);
    cr.rectangle(
        x + width * 0.08,
        banner_y + banner_h * 0.30,
        label_w,
        label_h,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.98, 0.88, 0.24, 0.95);
    cr.set_line_width(1.2);
    cr.rectangle(
        x + width * 0.08,
        banner_y + banner_h * 0.30,
        label_w,
        label_h,
    );
    cr.stroke().ok();

    cr.set_source_rgba(0.10, 0.10, 0.12, 0.18 + 0.10 * intensity);
    cr.rectangle(
        x + width * 0.78,
        y + height * 0.12,
        width * 0.12,
        height * 0.10,
    );
    cr.fill().ok();
}
