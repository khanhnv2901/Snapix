use gtk4::cairo;

pub(crate) fn paint_ink_wash_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    let base = cairo::LinearGradient::new(x, y, x + width * 0.25, y + height);
    base.add_color_stop_rgb(0.0, 0.99, 0.97, 0.93);
    base.add_color_stop_rgb(0.48, 0.94, 0.98, 0.99);
    base.add_color_stop_rgb(1.0, 0.97, 0.93, 0.99);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    paint_bloom(
        cr,
        x + width * 0.16,
        y + height * 0.22,
        width.max(height) * 0.46,
        (0.27, 0.84, 0.95),
        0.34 * intensity,
    );
    paint_bloom(
        cr,
        x + width * 0.78,
        y + height * 0.20,
        width.max(height) * 0.42,
        (0.98, 0.40, 0.62),
        0.30 * intensity,
    );
    paint_bloom(
        cr,
        x + width * 0.64,
        y + height * 0.82,
        width.max(height) * 0.52,
        (0.50, 0.38, 0.96),
        0.28 * intensity,
    );
    paint_bloom(
        cr,
        x + width * 0.18,
        y + height * 0.86,
        width.max(height) * 0.42,
        (0.99, 0.72, 0.25),
        0.22 * intensity,
    );
    paint_bloom(
        cr,
        x + width * 0.50,
        y + height * 0.48,
        width.max(height) * 0.48,
        (0.20, 0.86, 0.66),
        0.18 * intensity,
    );

    let wash = cairo::LinearGradient::new(x + width * 0.10, y, x + width * 0.92, y + height);
    wash.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 0.08);
    wash.add_color_stop_rgba(0.36, 0.98, 0.74, 0.90, 0.07 + 0.10 * intensity);
    wash.add_color_stop_rgba(0.66, 0.38, 0.72, 1.0, 0.04 + 0.08 * intensity);
    wash.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.04);
    cr.set_source(&wash).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    paint_soft_stain(
        cr,
        x + width * 0.10,
        y + height * 0.55,
        width * 0.42,
        height * 0.22,
        0.10 * intensity,
    );
    paint_soft_stain(
        cr,
        x + width * 0.56,
        y + height * 0.34,
        width * 0.36,
        height * 0.18,
        0.08 * intensity,
    );

    cr.set_source_rgba(0.10, 0.14, 0.20, 0.045 + 0.035 * intensity);
    cr.set_line_width(1.2);
    cr.move_to(x + width * 0.06, y + height * 0.72);
    cr.curve_to(
        x + width * 0.22,
        y + height * 0.62,
        x + width * 0.42,
        y + height * 0.80,
        x + width * 0.64,
        y + height * 0.66,
    );
    cr.curve_to(
        x + width * 0.76,
        y + height * 0.58,
        x + width * 0.86,
        y + height * 0.60,
        x + width * 0.96,
        y + height * 0.48,
    );
    cr.stroke().ok();
}

fn paint_bloom(
    cr: &cairo::Context,
    center_x: f64,
    center_y: f64,
    radius: f64,
    color: (f64, f64, f64),
    alpha: f64,
) {
    let bloom = cairo::RadialGradient::new(
        center_x,
        center_y,
        radius * 0.04,
        center_x,
        center_y,
        radius,
    );
    bloom.add_color_stop_rgba(0.0, color.0, color.1, color.2, alpha);
    bloom.add_color_stop_rgba(0.42, color.0, color.1, color.2, alpha * 0.44);
    bloom.add_color_stop_rgba(0.78, color.0, color.1, color.2, alpha * 0.12);
    bloom.add_color_stop_rgba(1.0, color.0, color.1, color.2, 0.0);
    cr.set_source(&bloom).ok();
    cr.arc(center_x, center_y, radius, 0.0, std::f64::consts::TAU);
    cr.fill().ok();
}

fn paint_soft_stain(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, alpha: f64) {
    cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
    cr.move_to(x, y + height * 0.55);
    cr.curve_to(
        x + width * 0.16,
        y + height * 0.05,
        x + width * 0.50,
        y - height * 0.10,
        x + width * 0.88,
        y + height * 0.20,
    );
    cr.curve_to(
        x + width * 1.05,
        y + height * 0.62,
        x + width * 0.50,
        y + height * 1.12,
        x + width * 0.08,
        y + height * 0.90,
    );
    cr.close_path();
    cr.fill().ok();
}
