use anyhow::{Context, Result};
use async_trait::async_trait;
use snapix_core::canvas::{Image, Rect};

use crate::backend::CaptureBackend;

pub struct X11Backend;

impl X11Backend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for X11Backend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CaptureBackend for X11Backend {
    fn name(&self) -> &'static str {
        "x11rb"
    }

    fn supports_interactive(&self) -> bool {
        true
    }

    async fn capture_full(&self) -> Result<Image> {
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::*;
        use x11rb::rust_connection::RustConnection;

        let (conn, screen_num) =
            RustConnection::connect(None).context("Failed to connect to X11 display")?;

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let width = screen.width_in_pixels;
        let height = screen.height_in_pixels;

        let image = conn
            .get_image(ImageFormat::Z_PIXMAP, root, 0, 0, width, height, !0u32)
            .context("get_image request failed")?
            .reply()
            .context("get_image reply failed")?;

        let raw = image.data;
        let depth = image.depth;

        let rgba = bgr_to_rgba(&raw, width as u32, height as u32, depth);
        Ok(Image::new(width as u32, height as u32, rgba))
    }

    async fn capture_region(&self, region: Rect) -> Result<Image> {
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::*;
        use x11rb::rust_connection::RustConnection;

        let (conn, screen_num) =
            RustConnection::connect(None).context("Failed to connect to X11 display")?;

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        let image = conn
            .get_image(
                ImageFormat::Z_PIXMAP,
                root,
                region.x as i16,
                region.y as i16,
                region.width as u16,
                region.height as u16,
                !0u32,
            )
            .context("get_image request failed")?
            .reply()
            .context("get_image reply failed")?;

        let raw = image.data;
        let depth = image.depth;
        let w = region.width as u32;
        let h = region.height as u32;

        let rgba = bgr_to_rgba(&raw, w, h, depth);
        Ok(Image::new(w, h, rgba))
    }

    async fn capture_window(&self) -> Result<Image> {
        // For M0: capture the focused window via EWMH _NET_ACTIVE_WINDOW.
        // Fall back to full-screen capture until interactive overlay is built.
        tracing::warn!("capture_window: falling back to full-screen capture (M0 stub)");
        self.capture_full().await
    }
}

/// Convert X11 ZPixmap (BGR / BGRX / BGRA) to RGBA8.
fn bgr_to_rgba(raw: &[u8], width: u32, height: u32, depth: u8) -> Vec<u8> {
    let bytes_per_pixel = if depth >= 24 { 4 } else { 3 };
    let mut out = Vec::with_capacity((width * height * 4) as usize);

    for chunk in raw.chunks(bytes_per_pixel) {
        let b = chunk.first().copied().unwrap_or(0);
        let g = chunk.get(1).copied().unwrap_or(0);
        let r = chunk.get(2).copied().unwrap_or(0);
        out.push(r);
        out.push(g);
        out.push(b);
        out.push(255);
    }

    out
}
