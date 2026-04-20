use std::cell::RefCell;
use std::rc::Rc;

use gtk4::cairo;
use gtk4::prelude::*;
use snapix_core::canvas::{Background, Color, Document, Image};

use crate::editor::EditorState;

#[derive(Clone)]
pub struct DocumentCanvas {
    drawing_area: gtk4::DrawingArea,
}

impl DocumentCanvas {
    pub fn new(state: Rc<RefCell<EditorState>>) -> Self {
        let drawing_area = gtk4::DrawingArea::builder()
            .content_width(1100)
            .content_height(760)
            .hexpand(true)
            .vexpand(true)
            .build();

        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let state = state.borrow();
            draw_canvas(cr, width, height, state.document());
        });

        Self { drawing_area }
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }

    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
    }
}

fn draw_canvas(cr: &cairo::Context, width: i32, height: i32, document: &Document) {
    cr.set_source_rgb(0.09, 0.10, 0.13);
    cr.paint().ok();

    let margin = 40.0;
    let frame_x = margin;
    let frame_y = margin;
    let frame_w = (width as f64 - margin * 2.0).max(160.0);
    let frame_h = (height as f64 - margin * 2.0).max(160.0);

    paint_background(cr, frame_x, frame_y, frame_w, frame_h, &document.background);

    let image_bounds = inset_frame(
        frame_x,
        frame_y,
        frame_w,
        frame_h,
        document.frame.padding as f64,
    );

    if document.frame.shadow {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
        rounded_rect(
            cr,
            image_bounds.0 + 10.0,
            image_bounds.1 + 16.0,
            image_bounds.2,
            image_bounds.3,
            document.frame.corner_radius as f64,
        );
        cr.fill().ok();
    }

    if let Some(image) = document.base_image.as_ref() {
        paint_image(cr, image_bounds, image, document.frame.corner_radius as f64);
    } else {
        paint_empty_state(cr, image_bounds, document.frame.corner_radius as f64);
    }
}

fn paint_background(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    background: &Background,
) {
    match background {
        Background::Solid { color } => {
            set_color(cr, color);
        }
        Background::Gradient { from, to, .. } => {
            let gradient = cairo::LinearGradient::new(x, y, x + width, y + height);
            gradient.add_color_stop_rgba(0.0, to_f64(from.r), to_f64(from.g), to_f64(from.b), 1.0);
            gradient.add_color_stop_rgba(1.0, to_f64(to.r), to_f64(to.g), to_f64(to.b), 1.0);
            cr.set_source(&gradient).ok();
        }
        Background::Image { .. } | Background::BlurredScreenshot { .. } => {
            cr.set_source_rgb(0.15, 0.18, 0.23);
        }
    }

    rounded_rect(cr, x, y, width, height, 28.0);
    cr.fill().ok();
}

fn paint_empty_state(cr: &cairo::Context, bounds: (f64, f64, f64, f64), radius: f64) {
    let (x, y, width, height) = bounds;

    cr.set_source_rgb(0.96, 0.97, 0.99);
    rounded_rect(cr, x, y, width, height, radius);
    cr.fill().ok();

    cr.set_source_rgb(0.82, 0.85, 0.90);
    cr.set_line_width(2.0);
    rounded_rect(cr, x, y, width, height, radius);
    cr.stroke().ok();

    cr.set_source_rgb(0.73, 0.77, 0.83);
    cr.set_line_width(3.0);
    cr.move_to(x + width * 0.28, y + height * 0.32);
    cr.line_to(x + width * 0.72, y + height * 0.68);
    cr.move_to(x + width * 0.72, y + height * 0.32);
    cr.line_to(x + width * 0.28, y + height * 0.68);
    cr.stroke().ok();
}

fn paint_image(cr: &cairo::Context, bounds: (f64, f64, f64, f64), image: &Image, radius: f64) {
    let (x, y, max_width, max_height) = bounds;
    let image_w = image.width as f64;
    let image_h = image.height as f64;
    let scale = f64::min(max_width / image_w, max_height / image_h);
    let draw_w = image_w * scale;
    let draw_h = image_h * scale;
    let draw_x = x + (max_width - draw_w) / 2.0;
    let draw_y = y + (max_height - draw_h) / 2.0;

    rounded_rect(cr, draw_x, draw_y, draw_w, draw_h, radius);
    cr.clip();

    if let Some(surface) = make_surface(image) {
        cr.save().ok();
        cr.translate(draw_x, draw_y);
        cr.scale(scale, scale);
        cr.set_source_surface(&surface, 0.0, 0.0).ok();
        cr.paint().ok();
        cr.restore().ok();
    }

    cr.reset_clip();
}

fn make_surface(image: &Image) -> Option<cairo::ImageSurface> {
    let mut surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        image.width as i32,
        image.height as i32,
    )
    .ok()?;

    {
        let stride = surface.stride() as usize;
        let mut data = surface.data().ok()?;

        for y in 0..image.height as usize {
            for x in 0..image.width as usize {
                let src = (y * image.width as usize + x) * 4;
                let dst = y * stride + x * 4;

                let r = image.data[src];
                let g = image.data[src + 1];
                let b = image.data[src + 2];
                let a = image.data[src + 3];

                data[dst] = ((b as u16 * a as u16) / 255) as u8;
                data[dst + 1] = ((g as u16 * a as u16) / 255) as u8;
                data[dst + 2] = ((r as u16 * a as u16) / 255) as u8;
                data[dst + 3] = a;
            }
        }
    }

    surface.mark_dirty();
    Some(surface)
}

fn inset_frame(x: f64, y: f64, width: f64, height: f64, padding: f64) -> (f64, f64, f64, f64) {
    let padded_x = x + padding;
    let padded_y = y + padding;
    let padded_w = (width - padding * 2.0).max(80.0);
    let padded_h = (height - padding * 2.0).max(80.0);
    (padded_x, padded_y, padded_w, padded_h)
}

fn rounded_rect(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
    let radius = radius.min(width / 2.0).min(height / 2.0);
    let degrees = std::f64::consts::PI / 180.0;

    cr.new_sub_path();
    cr.arc(
        x + width - radius,
        y + radius,
        radius,
        -90.0 * degrees,
        0.0 * degrees,
    );
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0 * degrees,
        90.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + height - radius,
        radius,
        90.0 * degrees,
        180.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        180.0 * degrees,
        270.0 * degrees,
    );
    cr.close_path();
}

fn set_color(cr: &cairo::Context, color: &Color) {
    cr.set_source_rgba(
        to_f64(color.r),
        to_f64(color.g),
        to_f64(color.b),
        to_f64(color.a),
    );
}

fn to_f64(value: u8) -> f64 {
    f64::from(value) / 255.0
}
