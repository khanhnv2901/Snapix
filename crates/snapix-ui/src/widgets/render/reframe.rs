use gtk4::cairo;

use crate::widgets::geometry::rounded_rect;
use crate::widgets::CanvasLayout;

pub(crate) fn draw_reframe_overlay(
    cr: &cairo::Context,
    layout: CanvasLayout,
    opacity: f64,
    zoom: f32,
) {
    if opacity <= 0.01 {
        return;
    }

    let x = layout.viewport_x;
    let y = layout.viewport_y;
    let width = layout.viewport_width;
    let height = layout.viewport_height;
    let radius = 18.0;

    cr.save().ok();
    rounded_rect(cr, x, y, width, height, radius);
    cr.clip();
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.05 * opacity);
    cr.paint().ok();

    cr.set_line_width(1.0);
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.22 * opacity);
    for fraction in [1.0 / 3.0, 2.0 / 3.0] {
        let grid_x = x + width * fraction;
        let grid_y = y + height * fraction;
        cr.move_to(grid_x, y);
        cr.line_to(grid_x, y + height);
        cr.move_to(x, grid_y);
        cr.line_to(x + width, grid_y);
    }
    cr.stroke().ok();
    cr.restore().ok();

    cr.set_source_rgba(0.55, 0.80, 1.0, 0.95 * opacity);
    cr.set_line_width(2.0);
    cr.set_dash(&[8.0, 6.0], 0.0);
    rounded_rect(cr, x, y, width, height, radius);
    cr.stroke().ok();
    cr.set_dash(&[], 0.0);

    cr.set_source_rgba(0.04, 0.07, 0.11, 0.84 * opacity);
    rounded_rect(cr, x + 14.0, y + 14.0, 250.0, 30.0, 14.0);
    cr.fill().ok();

    cr.set_source_rgba(0.94, 0.98, 1.0, 0.98 * opacity);
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(13.0);
    cr.move_to(x + 24.0, y + 34.0);
    let _ = cr.show_text("Reframe: drag/pinch to adjust");

    let zoom_text = format!("{}%", (zoom.max(1.0) * 100.0).round() as u32);
    let zoom_width = cr
        .text_extents(&zoom_text)
        .map(|extents| extents.width() + 30.0)
        .unwrap_or(68.0)
        .max(68.0);
    let zoom_x = x + width - zoom_width - 14.0;

    cr.set_source_rgba(0.04, 0.07, 0.11, 0.84 * opacity);
    rounded_rect(cr, zoom_x, y + 14.0, zoom_width, 30.0, 14.0);
    cr.fill().ok();

    cr.set_source_rgba(0.55, 0.80, 1.0, 0.98 * opacity);
    cr.move_to(zoom_x + 14.0, y + 34.0);
    let _ = cr.show_text(&zoom_text);

    cr.set_source_rgba(0.90, 0.95, 1.0, 0.82 * opacity);
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(12.0);
    cr.move_to(x + 24.0, y + height - 18.0);
    let _ = cr.show_text("Double-click reset  •  Enter finish  •  Esc exit");
}
