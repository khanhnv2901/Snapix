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
    base.add_color_stop_rgb(0.0, 0.06, 0.10, 0.18);
    base.add_color_stop_rgb(0.55, 0.04, 0.08, 0.15);
    base.add_color_stop_rgb(1.0, 0.03, 0.06, 0.12);
    cr.set_source(&base).ok();
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(0.34, 0.82, 0.98, 0.05 + 0.10 * intensity);
    let major_grid = (width.min(height) / 9.5).clamp(24.0, 58.0);
    let minor_grid = major_grid / 2.0;
    let mut gx = 0.0_f64;
    while gx <= width {
        let lx = x + gx.floor() + 0.5;
        cr.move_to(lx, y);
        cr.line_to(lx, y + height);
        gx += minor_grid;
    }
    let mut gy = 0.0_f64;
    while gy <= height {
        let ly = y + gy.floor() + 0.5;
        cr.move_to(x, ly);
        cr.line_to(x + width, ly);
        gy += minor_grid;
    }
    cr.set_line_width(1.0);
    cr.stroke().ok();

    cr.set_source_rgba(0.52, 0.92, 1.0, 0.08 + 0.18 * intensity);
    let mut major_x = 0.0_f64;
    while major_x <= width {
        let lx = x + major_x.floor() + 0.5;
        cr.move_to(lx, y);
        cr.line_to(lx, y + height);
        major_x += major_grid;
    }
    let mut major_y = 0.0_f64;
    while major_y <= height {
        let ly = y + major_y.floor() + 0.5;
        cr.move_to(x, ly);
        cr.line_to(x + width, ly);
        major_y += major_grid;
    }
    cr.set_line_width(1.4);
    cr.stroke().ok();

    cr.set_source_rgba(0.24, 0.90, 1.0, 0.10 + 0.22 * intensity);
    cr.rectangle(
        x + width * 0.70,
        y + height * 0.08,
        width * 0.18,
        height * 0.19,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.22, 0.88, 1.0, 0.05 + 0.14 * intensity);
    cr.rectangle(
        x + width * 0.08,
        y + height * 0.12,
        width * 0.22,
        height * 0.08,
    );
    cr.fill().ok();

    cr.set_source_rgba(0.62, 0.95, 1.0, 0.18 + 0.22 * intensity);
    cr.set_line_width(2.0);
    cr.rectangle(
        x + width * 0.09,
        y + height * 0.14,
        width * 0.08,
        height * 0.12,
    );
    cr.stroke().ok();

    cr.set_source_rgba(0.68, 0.96, 1.0, 0.14 + 0.26 * intensity);
    cr.set_line_width(2.2);
    cr.move_to(x + width * 0.08, y + height * 0.76);
    cr.line_to(x + width * 0.46, y + height * 0.76);
    cr.move_to(x + width * 0.08, y + height * 0.82);
    cr.line_to(x + width * 0.30, y + height * 0.82);
    cr.move_to(x + width * 0.08, y + height * 0.88);
    cr.line_to(x + width * 0.22, y + height * 0.88);
    cr.stroke().ok();

    cr.set_source_rgba(0.70, 0.96, 1.0, 0.12 + 0.20 * intensity);
    cr.set_line_width(1.4);
    let cx = x + width * 0.66;
    let cy = y + height * 0.74;
    let r = width.min(height) * 0.09;
    cr.arc(
        cx,
        cy,
        r,
        std::f64::consts::PI,
        std::f64::consts::TAU * 0.92,
    );
    cr.stroke().ok();

    cr.set_source_rgba(0.78, 0.98, 1.0, 0.16 + 0.18 * intensity);
    cr.arc(cx, cy, 2.6, 0.0, std::f64::consts::TAU);
    cr.fill().ok();
}
