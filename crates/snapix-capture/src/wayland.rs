use anyhow::{Context, Result};
use async_trait::async_trait;
use snapix_core::canvas::{Image, Rect};

use crate::backend::CaptureBackend;

/// Error types specific to Wayland portal capture.
#[derive(Debug, thiserror::Error)]
pub enum WaylandCaptureError {
    #[error("Screenshot portal request was cancelled by user")]
    Cancelled,
    #[error("Screenshot portal is not available on this system")]
    PortalUnavailable,
    #[error("Permission denied by portal")]
    PermissionDenied,
    #[error("Portal error: {0}")]
    PortalError(String),
}

pub struct WaylandBackend;

impl WaylandBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WaylandBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CaptureBackend for WaylandBackend {
    fn name(&self) -> &'static str {
        "ashpd-portal"
    }

    fn supports_interactive(&self) -> bool {
        // XDG portal handles the region selector in a portal dialog.
        false
    }

    async fn capture_full(&self) -> Result<Image> {
        capture_via_portal(false).await
    }

    async fn capture_region(&self, _region: Rect) -> Result<Image> {
        // XDG portal doesn't support pre-selected regions directly;
        // use interactive mode and crop in the editor instead.
        capture_via_portal(true).await
    }

    async fn capture_window(&self) -> Result<Image> {
        // Portal interactive mode allows window selection
        capture_via_portal(true).await
    }
}

async fn capture_via_portal(interactive: bool) -> Result<Image> {
    use ashpd::desktop::screenshot::ScreenshotRequest;

    tracing::debug!(
        "Requesting screenshot via XDG portal (interactive={})",
        interactive
    );

    let request = ScreenshotRequest::default().interactive(interactive);

    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("cancelled") || err_str.contains("Cancelled") {
                return Err(WaylandCaptureError::Cancelled.into());
            }
            if err_str.contains("not found") || err_str.contains("No such") {
                return Err(WaylandCaptureError::PortalUnavailable.into());
            }
            return Err(anyhow::anyhow!("Portal request failed: {}", e));
        }
    };

    let screenshot = response
        .response()
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("cancelled") || err_str.contains("Cancelled") {
                WaylandCaptureError::Cancelled
            } else if err_str.contains("denied") || err_str.contains("permission") {
                WaylandCaptureError::PermissionDenied
            } else {
                WaylandCaptureError::PortalError(err_str)
            }
        })
        .context("Failed to get screenshot response")?;

    let uri = screenshot.uri().as_str();
    tracing::debug!("Screenshot saved to: {}", uri);

    load_uri_to_image(uri).await
}

async fn load_uri_to_image(uri: &str) -> Result<Image> {
    // Portal returns a file:// URI pointing to a temp PNG.
    let path = uri
        .strip_prefix("file://")
        .ok_or_else(|| anyhow::anyhow!("Unexpected URI scheme: {uri}"))?;

    // URL decode the path (handles spaces and special chars)
    let decoded_path = urlencoding_decode(path);

    let dyn_img = image::open(&decoded_path)
        .with_context(|| format!("Failed to open screenshot image: {}", decoded_path))?;

    tracing::info!(
        "Loaded screenshot: {}x{}",
        dyn_img.width(),
        dyn_img.height()
    );

    Ok(Image::from_dynamic(dyn_img))
}

/// Simple URL decoding for file paths.
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else {
            result.push(c);
        }
    }

    result
}
