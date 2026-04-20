use anyhow::Result;
use async_trait::async_trait;
use snapix_core::canvas::{Image, Rect};

use crate::backend::CaptureBackend;

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
        use ashpd::desktop::screenshot::ScreenshotRequest;

        let response = ScreenshotRequest::default()
            .interactive(false)
            .send()
            .await?
            .response()?;

        load_uri_to_image(response.uri().as_str()).await
    }

    async fn capture_region(&self, _region: Rect) -> Result<Image> {
        // XDG portal doesn't support pre-selected regions directly;
        // use interactive mode and crop in the editor instead.
        use ashpd::desktop::screenshot::ScreenshotRequest;

        let response = ScreenshotRequest::default()
            .interactive(true)
            .send()
            .await?
            .response()?;

        load_uri_to_image(response.uri().as_str()).await
    }

    async fn capture_window(&self) -> Result<Image> {
        self.capture_full().await
    }
}

async fn load_uri_to_image(uri: &str) -> Result<Image> {
    // Portal returns a file:// URI pointing to a temp PNG.
    let path = uri
        .strip_prefix("file://")
        .ok_or_else(|| anyhow::anyhow!("Unexpected URI scheme: {uri}"))?;

    let dyn_img = image::open(path)?;
    Ok(Image::from_dynamic(dyn_img))
}
