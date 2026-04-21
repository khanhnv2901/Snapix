use std::collections::{HashMap, HashSet};

use gtk4::cairo;
use snapix_core::canvas::{Annotation, Color, Document, TextStyle};
use snapix_core::canvas::Image;

use crate::editor::CropDrag;
use crate::widgets::geometry::{
    blurred_region_image, crop_drag_widget_bounds, make_surface, set_color,
};
use crate::widgets::CanvasLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BlurImageIdentity {
    width: u32,
    height: u32,
    len: usize,
    data_ptr: usize,
}

impl BlurImageIdentity {
    fn from_image(image: &Image) -> Self {
        Self {
            width: image.width,
            height: image.height,
            len: image.data.len(),
            data_ptr: image.data.as_ptr() as usize,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BlurCacheKey {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius_bits: u32,
}

impl BlurCacheKey {
    fn new(x: u32, y: u32, width: u32, height: u32, radius: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            radius_bits: radius.to_bits(),
        }
    }
}

#[derive(Default)]
pub(crate) struct BlurSurfaceCache {
    image_identity: Option<BlurImageIdentity>,
    surfaces: HashMap<BlurCacheKey, cairo::ImageSurface>,
}

impl BlurSurfaceCache {
    pub(crate) fn prepare_for_document(&mut self, document: &Document) {
        let Some(image) = document.base_image.as_ref() else {
            self.clear();
            return;
        };

        let identity = BlurImageIdentity::from_image(image);
        if self.image_identity != Some(identity) {
            self.image_identity = Some(identity);
            self.surfaces.clear();
        }
    }

    pub(crate) fn retain_for_document(&mut self, document: &Document) {
        let Some(image) = document.base_image.as_ref() else {
            self.clear();
            return;
        };

        let keys: HashSet<_> = document
            .annotations
            .iter()
            .filter_map(|annotation| {
                let Annotation::Blur { bounds, radius } = annotation else {
                    return None;
                };
                blur_cache_key(image, bounds, *radius)
            })
            .collect();

        self.surfaces.retain(|key, _| keys.contains(key));
    }

    fn surface_for(
        &mut self,
        image: &Image,
        key: BlurCacheKey,
    ) -> Option<&cairo::ImageSurface> {
        if !self.surfaces.contains_key(&key) {
            let region =
                blurred_region_image(image, key.x, key.y, key.width, key.height, f32::from_bits(key.radius_bits))?;
            let surface = make_surface(&region)?;
            self.surfaces.insert(key, surface);
        }
        self.surfaces.get(&key)
    }

    pub(crate) fn clear(&mut self) {
        self.image_identity = None;
        self.surfaces.clear();
    }
}

pub(super) fn draw_annotations(
    cr: &cairo::Context,
    document: &Document,
    layout: CanvasLayout,
    blur_cache: &mut BlurSurfaceCache,
) {
    blur_cache.prepare_for_document(document);

    for annotation in &document.annotations {
        match annotation {
            Annotation::Arrow {
                from,
                to,
                color,
                width,
            } => draw_arrow(cr, layout, from.x, from.y, to.x, to.y, color, *width),
            Annotation::Text {
                pos,
                content,
                style,
            } => draw_text_annotation(cr, layout, pos.x, pos.y, content, style),
            Annotation::Rect {
                bounds,
                stroke,
                fill,
            } => draw_rect_annotation(
                cr,
                layout,
                bounds,
                &stroke.color,
                stroke.width,
                fill.as_ref(),
            ),
            Annotation::Ellipse {
                bounds,
                stroke,
                fill,
            } => draw_ellipse_annotation(
                cr,
                layout,
                bounds,
                &stroke.color,
                stroke.width,
                fill.as_ref(),
            ),
            Annotation::Blur { bounds, radius } => {
                draw_blur_annotation(cr, document, layout, bounds, *radius, blur_cache)
            }
            _ => {}
        }
    }

    blur_cache.retain_for_document(document);
}

pub(super) fn draw_blur_preview(cr: &cairo::Context, layout: CanvasLayout, blur_drag: &CropDrag) {
    let Some((x, y, width, height)) = crop_drag_widget_bounds(layout, blur_drag) else {
        return;
    };

    cr.save().ok();
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.10);
    cr.rectangle(x, y, width, height);
    cr.fill().ok();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.set_line_width(2.0);
    cr.rectangle(x, y, width, height);
    cr.stroke().ok();
    cr.restore().ok();
}

pub(super) fn draw_rect_preview(
    cr: &cairo::Context,
    layout: CanvasLayout,
    rect_drag: &CropDrag,
    color: &Color,
    width: f32,
) {
    let Some((x, y, rect_width, rect_height)) = crop_drag_widget_bounds(layout, rect_drag) else {
        return;
    };
    draw_rect_shape(cr, x, y, rect_width, rect_height, color, width, None);
}

fn draw_rect_annotation(
    cr: &cairo::Context,
    layout: CanvasLayout,
    bounds: &snapix_core::canvas::Rect,
    color: &Color,
    width: f32,
    fill: Option<&Color>,
) {
    let x = layout.image_x + bounds.x as f64 * layout.image_scale;
    let y = layout.image_y + bounds.y as f64 * layout.image_scale;
    let rect_width = bounds.width as f64 * layout.image_scale;
    let rect_height = bounds.height as f64 * layout.image_scale;
    draw_rect_shape(cr, x, y, rect_width, rect_height, color, width, fill);
}

pub(super) fn draw_ellipse_preview(
    cr: &cairo::Context,
    layout: CanvasLayout,
    ellipse_drag: &CropDrag,
    color: &Color,
    width: f32,
) {
    let Some((x, y, shape_width, shape_height)) = crop_drag_widget_bounds(layout, ellipse_drag)
    else {
        return;
    };
    draw_ellipse_shape(cr, x, y, shape_width, shape_height, color, width, None);
}

fn draw_ellipse_annotation(
    cr: &cairo::Context,
    layout: CanvasLayout,
    bounds: &snapix_core::canvas::Rect,
    color: &Color,
    width: f32,
    fill: Option<&Color>,
) {
    let x = layout.image_x + bounds.x as f64 * layout.image_scale;
    let y = layout.image_y + bounds.y as f64 * layout.image_scale;
    let shape_width = bounds.width as f64 * layout.image_scale;
    let shape_height = bounds.height as f64 * layout.image_scale;
    draw_ellipse_shape(cr, x, y, shape_width, shape_height, color, width, fill);
}

fn draw_rect_shape(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: &Color,
    stroke_width: f32,
    fill: Option<&Color>,
) {
    cr.save().ok();
    if let Some(fill) = fill {
        set_color(cr, fill);
        cr.rectangle(x, y, width, height);
        cr.fill_preserve().ok();
    } else {
        cr.rectangle(x, y, width, height);
    }
    set_color(cr, color);
    cr.set_line_width(stroke_width as f64);
    cr.stroke().ok();
    cr.restore().ok();
}

pub(super) fn draw_ellipse_shape(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: &Color,
    stroke_width: f32,
    fill: Option<&Color>,
) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let rx = width / 2.0;
    let ry = height / 2.0;
    let cx = x + rx;
    let cy = y + ry;

    cr.save().ok();
    cr.translate(cx, cy);
    cr.scale(rx.max(1.0), ry.max(1.0));
    cr.new_sub_path();
    cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
    cr.restore().ok();

    if let Some(fill) = fill {
        cr.save().ok();
        cr.translate(cx, cy);
        cr.scale(rx.max(1.0), ry.max(1.0));
        cr.new_sub_path();
        cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
        set_color(cr, fill);
        cr.fill_preserve().ok();
        cr.restore().ok();
    }

    cr.save().ok();
    cr.translate(cx, cy);
    cr.scale(rx.max(1.0), ry.max(1.0));
    cr.new_sub_path();
    cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
    set_color(cr, color);
    cr.set_line_width((stroke_width as f64 / rx.min(ry).max(1.0)).max(0.08));
    cr.stroke().ok();
    cr.restore().ok();
}

fn draw_blur_annotation(
    cr: &cairo::Context,
    document: &Document,
    layout: CanvasLayout,
    bounds: &snapix_core::canvas::Rect,
    radius: f32,
    blur_cache: &mut BlurSurfaceCache,
) {
    let Some(image) = document.base_image.as_ref() else {
        return;
    };

    let Some(key) = blur_cache_key(image, bounds, radius) else {
        return;
    };
    let Some(surface) = blur_cache.surface_for(image, key) else {
        return;
    };

    let draw_x = layout.image_x + key.x as f64 * layout.image_scale;
    let draw_y = layout.image_y + key.y as f64 * layout.image_scale;
    let draw_w = key.width as f64 * layout.image_scale;
    let draw_h = key.height as f64 * layout.image_scale;

    cr.save().ok();
    cr.rectangle(draw_x, draw_y, draw_w, draw_h);
    cr.clip();
    cr.translate(draw_x, draw_y);
    cr.scale(layout.image_scale, layout.image_scale);
    cr.set_source_surface(&surface, 0.0, 0.0).ok();
    cr.paint().ok();
    cr.restore().ok();
}

fn blur_cache_key(
    image: &Image,
    bounds: &snapix_core::canvas::Rect,
    radius: f32,
) -> Option<BlurCacheKey> {
    let x = bounds.x.max(0.0).floor() as u32;
    let y = bounds.y.max(0.0).floor() as u32;
    let width = bounds.width.ceil().max(0.0) as u32;
    let height = bounds.height.ceil().max(0.0) as u32;
    if width < 2 || height < 2 || x >= image.width || y >= image.height {
        return None;
    }

    let clamped_width = width.min(image.width - x);
    let clamped_height = height.min(image.height - y);
    Some(BlurCacheKey::new(
        x,
        y,
        clamped_width,
        clamped_height,
        radius,
    ))
}

fn draw_text_annotation(
    cr: &cairo::Context,
    layout: CanvasLayout,
    x: f32,
    y: f32,
    content: &str,
    style: &TextStyle,
) {
    let draw_x = layout.image_x + x as f64 * layout.image_scale;
    let draw_y = layout.image_y + y as f64 * layout.image_scale;
    let font_size = (style.font_size as f64 * layout.image_scale).max(14.0);

    cr.save().ok();
    cr.select_font_face(
        &style.font_family,
        cairo::FontSlant::Normal,
        if style.bold {
            cairo::FontWeight::Bold
        } else {
            cairo::FontWeight::Normal
        },
    );
    cr.set_font_size(font_size);

    cr.move_to(draw_x + 2.0, draw_y + 2.0);
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
    cr.show_text(content).ok();

    cr.move_to(draw_x, draw_y);
    set_color(cr, &style.color);
    cr.show_text(content).ok();
    cr.restore().ok();
}

pub(super) fn draw_arrow(
    cr: &cairo::Context,
    layout: CanvasLayout,
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    color: &Color,
    width: f32,
) {
    let start_x = layout.image_x + from_x as f64 * layout.image_scale;
    let start_y = layout.image_y + from_y as f64 * layout.image_scale;
    let end_x = layout.image_x + to_x as f64 * layout.image_scale;
    let end_y = layout.image_y + to_y as f64 * layout.image_scale;

    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 1.0 {
        return;
    }

    let angle = dy.atan2(dx);
    let head_length = (width as f64 * 3.4).max(14.0);
    let head_angle = 28.0_f64.to_radians();
    let stroke_width = width as f64;

    cr.save().ok();
    set_color(cr, color);
    cr.set_line_width(stroke_width);
    cr.set_line_cap(cairo::LineCap::Round);

    let shaft_end_x = end_x - head_length * angle.cos();
    let shaft_end_y = end_y - head_length * angle.sin();
    cr.move_to(start_x, start_y);
    cr.line_to(shaft_end_x, shaft_end_y);
    cr.stroke().ok();

    cr.move_to(end_x, end_y);
    cr.line_to(
        end_x - head_length * (angle - head_angle).cos(),
        end_y - head_length * (angle - head_angle).sin(),
    );
    cr.line_to(
        end_x - head_length * (angle + head_angle).cos(),
        end_y - head_length * (angle + head_angle).sin(),
    );
    cr.close_path();
    cr.fill().ok();
    cr.restore().ok();
}

#[cfg(test)]
mod tests {
    use gtk4::cairo::{Antialias, Context, Format, ImageSurface};
    use snapix_core::canvas::{Annotation, Color, Document, Image, Rect};

    use super::{blur_cache_key, BlurSurfaceCache, draw_ellipse_shape};

    fn alpha_at(surface: &mut ImageSurface, x: i32, y: i32) -> u8 {
        let stride = surface.stride() as usize;
        let data = surface.data().expect("surface data");
        let offset = y as usize * stride + x as usize * 4;
        data[offset + 3]
    }

    #[test]
    fn ellipse_shape_does_not_connect_to_existing_path_point() {
        let mut surface = ImageSurface::create(Format::ARgb32, 160, 80).expect("surface");
        {
            let cr = Context::new(&surface).expect("context");
            cr.set_antialias(Antialias::None);

            cr.move_to(110.0, 18.0);
            draw_ellipse_shape(
                &cr,
                20.0,
                20.0,
                60.0,
                24.0,
                &Color {
                    r: 255,
                    g: 99,
                    b: 71,
                    a: 255,
                },
                4.0,
                None,
            );
        }

        surface.flush();

        assert_eq!(alpha_at(&mut surface, 85, 32), 0);
    }

    #[test]
    fn blur_surface_cache_resets_when_base_image_changes() {
        let image_a = Image::new(24, 24, vec![120; 24 * 24 * 4]);
        let image_b = Image::new(24, 24, vec![180; 24 * 24 * 4]);
        let mut cache = BlurSurfaceCache::default();

        let mut document = Document::new(image_a);
        document.annotations.push(Annotation::Blur {
            bounds: Rect {
                x: 2.0,
                y: 3.0,
                width: 10.0,
                height: 8.0,
            },
            radius: 12.0,
        });

        cache.prepare_for_document(&document);
        let image = document.base_image.as_ref().expect("image");
        let key = blur_cache_key(
            image,
            &Rect {
                x: 2.0,
                y: 3.0,
                width: 10.0,
                height: 8.0,
            },
            12.0,
        )
        .expect("blur key");
        assert!(cache.surface_for(image, key).is_some());
        assert_eq!(cache.surfaces.len(), 1);

        document.base_image = Some(image_b);
        cache.prepare_for_document(&document);

        assert!(cache.surfaces.is_empty());
    }
}
