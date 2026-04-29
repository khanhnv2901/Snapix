use gtk4::cairo;

pub(crate) fn paint_memphis_grid_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    intensity: f64,
) {
    let intensity = intensity.clamp(0.2, 1.0);

    cr.set_source_rgb(0.98, 0.97, 0.92);
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    let step = (width.min(height) / 8.5).clamp(28.0, 52.0);
    cr.set_source_rgba(0.10, 0.12, 0.18, 0.04 + 0.06 * intensity);
    cr.set_line_width(1.0);
    let mut gx = 0.0_f64;
    while gx <= width {
        let px = x + gx.floor() + 0.5;
        cr.move_to(px, y);
        cr.line_to(px, y + height);
        gx += step;
    }
    let mut gy = 0.0_f64;
    while gy <= height {
        let py = y + gy.floor() + 0.5;
        cr.move_to(x, py);
        cr.line_to(x + width, py);
        gy += step;
    }
    cr.stroke().ok();

    cr.set_source_rgba(0.96, 0.34, 0.58, 0.88);
    cr.rectangle(
        x + width * 0.09,
        y + height * 0.16,
        width * 0.18,
        height * 0.16,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.14, 0.70, 0.98, 0.90);
    cr.arc(
        x + width * 0.78,
        y + height * 0.24,
        width.min(height) * 0.085,
        0.0,
        std::f64::consts::TAU,
    );
    cr.fill().ok();

    cr.set_source_rgba(1.0, 0.84, 0.12, 0.94);
    cr.move_to(x + width * 0.58, y + height * 0.64);
    cr.line_to(x + width * 0.88, y + height * 0.56);
    cr.line_to(x + width * 0.82, y + height * 0.82);
    cr.line_to(x + width * 0.52, y + height * 0.88);
    cr.close_path();
    cr.fill().ok();

    cr.set_source_rgba(0.08, 0.08, 0.10, 0.84);
    cr.set_line_width(3.0);
    cr.move_to(x + width * 0.16, y + height * 0.60);
    cr.line_to(x + width * 0.32, y + height * 0.74);
    cr.line_to(x + width * 0.22, y + height * 0.92);
    cr.stroke().ok();

    cr.set_line_width(2.0);
    let dot_count = (6.0 + intensity * 4.0).round() as usize;
    for i in 0..dot_count {
        let t = i as f64 / dot_count.max(1) as f64;
        cr.arc(
            x + width * (0.42 + 0.18 * t),
            y + height * 0.22,
            2.2,
            0.0,
            std::f64::consts::TAU,
        );
        cr.stroke().ok();
    }
}
