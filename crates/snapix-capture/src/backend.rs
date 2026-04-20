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

/// Session type detected from environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Unknown,
}

/// Detect the current session type using multiple methods.
pub fn detect_session() -> SessionType {
    // Method 1: Check WAYLAND_DISPLAY (most reliable for Wayland)
    if std::env::var("WAYLAND_DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        return SessionType::Wayland;
    }

    // Method 2: Check XDG_SESSION_TYPE
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => return SessionType::Wayland,
            "x11" => return SessionType::X11,
            _ => {}
        }
    }

    // Method 3: Check DISPLAY for X11
    if std::env::var("DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        return SessionType::X11;
    }

    // Method 4: Check GDK_BACKEND hint
    if let Ok(gdk_backend) = std::env::var("GDK_BACKEND") {
        match gdk_backend.to_lowercase().as_str() {
            "wayland" => return SessionType::Wayland,
            "x11" => return SessionType::X11,
            _ => {}
        }
    }

    SessionType::Unknown
}

/// Detect the best available backend for the running session.
#[cfg(unix)]
pub fn detect_backend() -> Box<dyn CaptureBackend> {
    let session = detect_session();
    tracing::debug!("Detected session type: {:?}", session);

    #[cfg(feature = "wayland")]
    if session == SessionType::Wayland {
        tracing::info!("Using XDG portal backend (Wayland)");
        return Box::new(crate::wayland::WaylandBackend::new());
    }

    #[cfg(feature = "x11")]
    if session == SessionType::X11 || session == SessionType::Unknown {
        tracing::info!("Using x11rb backend (X11)");
        return Box::new(crate::x11::X11Backend::new());
    }

    #[cfg(all(feature = "wayland", not(feature = "x11")))]
    {
        tracing::warn!("Unknown session type, falling back to Wayland portal");
        return Box::new(crate::wayland::WaylandBackend::new());
    }

    #[allow(unreachable_code)]
    {
        panic!("No capture backend available. Enable the 'x11' or 'wayland' feature.");
    }
}
