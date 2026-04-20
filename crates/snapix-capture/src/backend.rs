use anyhow::Result;
use async_trait::async_trait;
use snapix_core::canvas::{Image, Rect};

#[async_trait]
pub trait CaptureBackend: Send + Sync {
    async fn capture_full(&self) -> Result<Image>;
    async fn capture_region(&self, region: Rect) -> Result<Image>;
    async fn capture_window(&self) -> Result<Image>;
    fn supports_interactive(&self) -> bool;
    fn name(&self) -> &'static str;
}

/// Detect the best available backend for the running session.
#[cfg(unix)]
pub fn detect_backend() -> Box<dyn CaptureBackend> {
    let wayland_display = std::env::var("WAYLAND_DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let xdg_session = std::env::var("XDG_SESSION_TYPE")
        .map(|v| v.to_lowercase() == "wayland")
        .unwrap_or(false);
    let is_wayland = wayland_display || xdg_session;

    #[cfg(feature = "wayland")]
    if is_wayland {
        tracing::info!("Detected Wayland session — using XDG portal backend");
        return Box::new(crate::wayland::WaylandBackend::new());
    }

    #[cfg(feature = "x11")]
    {
        tracing::info!("Detected X11 session — using x11rb backend");
        return Box::new(crate::x11::X11Backend::new());
    }

    #[allow(unreachable_code)]
    {
        panic!("No capture backend available. Enable the 'x11' or 'wayland' feature.");
    }
}
