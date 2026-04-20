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
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::*;
        use x11rb::rust_connection::RustConnection;

        let (conn, screen_num) =
            RustConnection::connect(None).context("Failed to connect to X11 display")?;

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        // Get _NET_ACTIVE_WINDOW atom
        let active_atom = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .context("intern_atom request failed")?
            .reply()
            .context("intern_atom reply failed")?
            .atom;

        // Query the active window from root
        let reply = conn
            .get_property(false, root, active_atom, AtomEnum::WINDOW, 0, 1)
            .context("get_property request failed")?
            .reply()
            .context("get_property reply failed")?;

        let active_window = if reply.format == 32 && !reply.value.is_empty() {
            let bytes: [u8; 4] = reply.value[0..4]
                .try_into()
                .context("Invalid window ID format")?;
            u32::from_ne_bytes(bytes)
        } else {
            tracing::warn!("No active window found, falling back to full-screen capture");
            return self.capture_full().await;
        };

        if active_window == 0 || active_window == root {
            tracing::warn!("Active window is root, falling back to full-screen capture");
            return self.capture_full().await;
        }

        tracing::debug!("Capturing active window: 0x{:x}", active_window);

        // Get window geometry
        let geom = conn
            .get_geometry(active_window)
            .context("get_geometry request failed")?
            .reply()
            .context("get_geometry reply failed")?;

        // Translate coordinates to root window (for nested windows)
        let coords = conn
            .translate_coordinates(active_window, root, 0, 0)
            .context("translate_coordinates request failed")?
            .reply()
            .context("translate_coordinates reply failed")?;

        let x = coords.dst_x;
        let y = coords.dst_y;
        let width = geom.width;
        let height = geom.height;

        tracing::debug!(
            "Window geometry: {}x{} at ({}, {})",
            width,
            height,
            x,
            y
        );

        // Capture the region from root window (includes decorations rendered by compositor)
        let image = conn
            .get_image(ImageFormat::Z_PIXMAP, root, x, y, width, height, !0u32)
            .context("get_image request failed")?
            .reply()
            .context("get_image reply failed")?;

        let raw = image.data;
        let depth = image.depth;

        let rgba = bgr_to_rgba(&raw, width as u32, height as u32, depth);
        Ok(Image::new(width as u32, height as u32, rgba))
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
