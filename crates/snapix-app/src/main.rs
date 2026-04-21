use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(
    name    = "snapix",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Snapix — Linux screenshot beautifier",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Capture a screenshot from the command line (no GUI).
    Capture {
        /// What to capture: full, region, window
        #[arg(long, default_value = "full")]
        mode: CaptureMode,

        /// Region X coordinate (required for X11 region capture from CLI)
        #[arg(long)]
        x: Option<u32>,

        /// Region Y coordinate (required for X11 region capture from CLI)
        #[arg(long)]
        y: Option<u32>,

        /// Region width (required for X11 region capture from CLI)
        #[arg(long)]
        width: Option<u32>,

        /// Region height (required for X11 region capture from CLI)
        #[arg(long)]
        height: Option<u32>,

        /// Output file path (PNG or JPG based on extension)
        #[arg(short, long, default_value = "screenshot.png")]
        output: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
enum CaptureMode {
    Full,
    Region,
    Window,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("snapix=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Capture {
            mode,
            x,
            y,
            width,
            height,
            output,
        }) => async_std::task::block_on(run_capture(mode, x, y, width, height, output)),
        // No subcommand → launch the GTK4 GUI.
        None => {
            info!("Starting Snapix GUI");
            let launch_context = async_std::task::block_on(prepare_editor_launch());
            let code = snapix_ui::SnapixApp::run(launch_context);
            std::process::exit(code.value());
        }
    }
}

async fn prepare_editor_launch() -> snapix_ui::LaunchContext {
    use snapix_capture::SessionType;
    use snapix_core::canvas::Document;
    use snapix_ui::{LaunchBanner, LaunchContext};

    let backend = snapix_capture::detect_backend();
    let session = snapix_capture::detect_session();
    info!(
        "Capturing startup screenshot for editor via {}",
        backend.name()
    );

    match backend.capture_full().await {
        Ok(image) => LaunchContext {
            document: Document::new(image),
            banner: Some(LaunchBanner::info(format!(
                "Screenshot captured via {} and loaded into the editor.",
                backend.name()
            ))),
        },
        Err(error) => {
            tracing::warn!("Startup full-screen capture failed: {error:#}");

            if session == SessionType::Wayland {
                tracing::info!(
                    "Retrying startup capture with interactive window mode on Wayland portal"
                );

                match backend.capture_window().await {
                    Ok(image) => {
                        return LaunchContext {
                            document: Document::new(image),
                            banner: Some(LaunchBanner::info(format!(
                                "Full-screen startup capture failed on {}, so Snapix fell back to interactive window capture and loaded the result into the editor.",
                                backend.name()
                            ))),
                        };
                    }
                    Err(fallback_error) => {
                        tracing::warn!(
                            "Startup fallback window capture also failed: {fallback_error:#}"
                        );

                        return LaunchContext {
                            document: Document::default(),
                            banner: Some(LaunchBanner::warning(format!(
                                "Startup capture failed: full-screen error: {error}; interactive fallback error: {fallback_error}. The editor opened with an empty document."
                            ))),
                        };
                    }
                }
            }

            LaunchContext {
                document: Document::default(),
                banner: Some(LaunchBanner::warning(format!(
                    "Startup capture failed: {error}. The editor opened with an empty document."
                ))),
            }
        }
    }
}

async fn run_capture(
    mode: CaptureMode,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    output: PathBuf,
) -> Result<()> {
    use snapix_capture::{detect_backend, detect_session, SessionType};
    use snapix_core::canvas::Rect;

    let backend = detect_backend();
    let session = detect_session();
    info!("Using capture backend: {}", backend.name());

    let image = match mode {
        CaptureMode::Full => backend.capture_full().await?,
        CaptureMode::Region => match resolve_region_rect(session, x, y, width, height)? {
            Some(region) => backend.capture_region(region).await?,
            None => {
                tracing::info!("Using interactive region capture via {}", backend.name());
                backend
                    .capture_region(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    })
                    .await?
            }
        },
        CaptureMode::Window => {
            if session == SessionType::Wayland {
                anyhow::bail!(
                    "Window capture is not reliably available from the Wayland CLI flow. Use `snapix --mode region` for portal selection or launch the GUI."
                );
            }
            backend.capture_window().await?
        }
    };

    let dyn_img = image.to_dynamic();
    dyn_img.save(&output)?;
    info!("Saved screenshot to {}", output.display());

    Ok(())
}

fn resolve_region_rect(
    session: snapix_capture::SessionType,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Option<snapix_core::canvas::Rect>> {
    match (x, y, width, height) {
        (Some(x), Some(y), Some(width), Some(height)) => {
            if width == 0 || height == 0 {
                anyhow::bail!("Region width and height must be greater than zero");
            }
            Ok(Some(snapix_core::canvas::Rect {
                x: x as f32,
                y: y as f32,
                width: width as f32,
                height: height as f32,
            }))
        }
        (None, None, None, None) => {
            if session == snapix_capture::SessionType::X11 {
                anyhow::bail!(
                    "CLI region capture on X11 requires --x, --y, --width, and --height. For interactive selection, launch the GUI."
                );
            }
            Ok(None)
        }
        _ => anyhow::bail!(
            "Region capture requires either all of --x, --y, --width, --height or none of them"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_region_rect;
    use snapix_capture::SessionType;

    #[test]
    fn resolve_region_rect_accepts_explicit_bounds() {
        let region =
            resolve_region_rect(SessionType::X11, Some(10), Some(20), Some(300), Some(200))
                .unwrap()
                .expect("expected explicit region");

        assert_eq!(region.x, 10.0);
        assert_eq!(region.y, 20.0);
        assert_eq!(region.width, 300.0);
        assert_eq!(region.height, 200.0);
    }

    #[test]
    fn resolve_region_rect_rejects_partial_bounds() {
        let error = resolve_region_rect(SessionType::X11, Some(10), None, Some(300), Some(200))
            .expect_err("expected partial bounds to fail");

        assert!(error
            .to_string()
            .contains("either all of --x, --y, --width, --height or none of them"));
    }

    #[test]
    fn resolve_region_rect_rejects_zero_sized_bounds() {
        let error = resolve_region_rect(SessionType::X11, Some(10), Some(20), Some(0), Some(200))
            .expect_err("expected zero-sized bounds to fail");

        assert!(error
            .to_string()
            .contains("Region width and height must be greater than zero"));
    }

    #[test]
    fn resolve_region_rect_requires_explicit_bounds_on_x11() {
        let error = resolve_region_rect(SessionType::X11, None, None, None, None)
            .expect_err("expected X11 region capture without bounds to fail");

        assert!(error
            .to_string()
            .contains("CLI region capture on X11 requires --x, --y, --width, and --height"));
    }

    #[test]
    fn resolve_region_rect_allows_interactive_wayland_flow() {
        let region = resolve_region_rect(SessionType::Wayland, None, None, None, None).unwrap();

        assert!(region.is_none());
    }
}
