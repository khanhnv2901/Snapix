use anyhow::{Context, Result};
use gtk4::cairo;
use snapix_core::canvas::Document;

use crate::widgets::geometry::composition_size;
use crate::widgets::RenderedDocument;

use super::{canvas::draw_canvas_with_background_radius, BlurSurfaceCache};

pub(crate) fn render_document_rgba(document: &Document) -> Result<RenderedDocument> {
    let (width, height) = export_size(document);
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
        .context("Failed to create export surface")?;
    {
        let cr = cairo::Context::new(&surface).context("Failed to create cairo context")?;
        let mut blur_cache = BlurSurfaceCache::default();
        draw_canvas_with_background_radius(&cr, width, height, document, &mut blur_cache, 0.0);
    }
    surface.flush();
    let stride = surface.stride() as usize;
    let data = surface
        .data()
        .context("Failed to read export surface data")?;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for y in 0..height as usize {
        for x in 0..width as usize {
            let src = y * stride + x * 4;
            let dst = (y * width as usize + x) * 4;

            let b = data[src];
            let g = data[src + 1];
            let r = data[src + 2];
            let a = data[src + 3];

            if a == 0 {
                rgba[dst] = 0;
                rgba[dst + 1] = 0;
                rgba[dst + 2] = 0;
            } else {
                rgba[dst] = ((r as u16 * 255) / a as u16).min(255) as u8;
                rgba[dst + 1] = ((g as u16 * 255) / a as u16).min(255) as u8;
                rgba[dst + 2] = ((b as u16 * 255) / a as u16).min(255) as u8;
            }
            rgba[dst + 3] = a;
        }
    }

    Ok(RenderedDocument {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

fn export_size(document: &Document) -> (i32, i32) {
    const MIN_EXPORT_WIDTH: i32 = 1200;
    const MIN_EXPORT_HEIGHT: i32 = 800;
    let (natural_width, natural_height): (f64, f64) = composition_size(document);
    let scale: f64 = f64::max(
        f64::max(
            MIN_EXPORT_WIDTH as f64 / natural_width,
            MIN_EXPORT_HEIGHT as f64 / natural_height,
        ),
        1.0,
    );

    (
        (natural_width * scale).round() as i32,
        (natural_height * scale).round() as i32,
    )
}
