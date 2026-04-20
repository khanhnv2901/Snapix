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
        Some(Command::Capture { mode, output }) => {
            async_std::task::block_on(run_capture(mode, output))
        }
        // No subcommand → launch the GTK4 GUI.
        None => {
            info!("Starting Snapix GUI");
            let code = snapix_ui::SnapixApp::run();
            std::process::exit(code.value());
        }
    }
}

async fn run_capture(mode: CaptureMode, output: PathBuf) -> Result<()> {
    use snapix_capture::detect_backend;

    let backend = detect_backend();
    info!("Using capture backend: {}", backend.name());

    let image = match mode {
        CaptureMode::Full => backend.capture_full().await?,
        CaptureMode::Region => {
            // Interactive region selection requires the GUI; fall back to full for now.
            tracing::warn!(
                "Interactive region capture not yet implemented — capturing full screen"
            );
            backend.capture_full().await?
        }
        CaptureMode::Window => backend.capture_window().await?,
    };

    let dyn_img = image.to_dynamic();
    dyn_img.save(&output)?;
    info!("Saved screenshot to {}", output.display());

    Ok(())
}
