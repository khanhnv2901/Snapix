use gtk4::prelude::*;
use libadwaita::Application;
use snapix_core::canvas::Document;

use crate::editor::EditorWindow;

pub const APP_ID: &str = "io.github.snapix.Snapix";

#[derive(Debug, Clone)]
pub struct LaunchBanner {
    pub kind: LaunchBannerKind,
    pub text: String,
}

impl LaunchBanner {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            kind: LaunchBannerKind::Info,
            text: text.into(),
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            kind: LaunchBannerKind::Warning,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchBannerKind {
    Info,
    Warning,
}

#[derive(Debug, Clone, Default)]
pub struct LaunchContext {
    pub document: Document,
    pub banner: Option<LaunchBanner>,
}

pub struct SnapixApp;

impl SnapixApp {
    pub fn run(context: LaunchContext) -> glib::ExitCode {
        let app = Application::builder().application_id(APP_ID).build();

        app.connect_activate(move |app| build_ui(app, context.clone()));
        app.run()
    }
}

fn build_ui(app: &Application, context: LaunchContext) {
    install_editor_css();
    let editor = EditorWindow::new(app, context);
    editor.present();
}

fn install_editor_css() {
    const EDITOR_CSS: &str = r#"
/* ── Shell ─────────────────────────────────────────────────────── */
.snapix-shell {
    background: #0c1017;
}

/* ── Capture row ────────────────────────────────────────────────── */
.capture-row {
    padding: 8px 16px 8px 16px;
    border-bottom: 1px solid alpha(#ffffff, 0.05);
}

.capture-pill {
    padding: 8px 14px;
    color: #f5f7ff;
    border: 1px solid alpha(#ffffff, 0.08);
    border-radius: 10px;
}

.capture-pill.fullscreen { background: linear-gradient(135deg, #844dff, #6e3ce8); }
.capture-pill.region     { background: linear-gradient(135deg, #3fb9c8, #2a9f9f); }
.capture-pill.window     { background: linear-gradient(135deg, #e54f8a, #c43c6d); }
.capture-pill.import     { background: linear-gradient(135deg, #f0a73f, #d4791f); }
.capture-pill.utility    { background: #1a2230; }

/* ── Tool row ───────────────────────────────────────────────────── */
.tool-row {
    padding: 6px 16px 6px 16px;
    border-bottom: 1px solid alpha(#ffffff, 0.05);
}

.tool-row-card {
    background: #111722;
    border: 1px solid alpha(#ffffff, 0.06);
    border-radius: 14px;
    padding: 8px 12px;
}

.tool-pill {
    padding: 7px 12px;
    background: transparent;
    color: alpha(#f5f7ff, 0.72);
    border: 1px solid transparent;
    border-radius: 10px;
}

.tool-pill:checked {
    background: linear-gradient(135deg, #8d5bff, #643bda);
    color: #ffffff;
    border-color: alpha(#ffffff, 0.16);
}

/* Color swatch dots */
.color-swatch-btn {
    padding: 4px;
    background: transparent;
    border: 2px solid transparent;
    border-radius: 50%;
    min-width: 28px;
    min-height: 28px;
}

.color-swatch-btn.active {
    border-color: white;
}

.color-dot {
    border-radius: 50%;
    min-width: 18px;
    min-height: 18px;
}

.color-dot-0 { background: #ff6236; }
.color-dot-1 { background: #e53935; }
.color-dot-2 { background: #e91e8c; }
.color-dot-3 { background: #7c4dff; }
.color-dot-4 { background: #2196f3; }
.color-dot-5 { background: #009688; }
.color-dot-6 { background: #4caf50; }
.color-dot-7 { background: #ffd600; }
.color-dot-8 { background: #f0f0f0; }
.color-dot-9 { background: #1e1e2e; border: 1px solid alpha(white, 0.25); }

/* Width selector dots */
.width-btn {
    padding: 6px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    min-width: 32px;
    min-height: 32px;
}

.width-btn.active {
    border-color: alpha(white, 0.4);
}

.width-dot-inner {
    border-radius: 50%;
    background: alpha(white, 0.40);
}

.width-dot-inner.active {
    background: white;
}

.wd-sm { min-width: 6px;  min-height: 6px;  }
.wd-md { min-width: 10px; min-height: 10px; }
.wd-lg { min-width: 14px; min-height: 14px; }
.wd-xl { min-width: 18px; min-height: 18px; }

/* Delete button */
.tool-delete-btn {
    padding: 7px 10px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 10px;
    color: alpha(#f5f7ff, 0.55);
}

.tool-delete-btn:hover {
    background: alpha(#e53935, 0.18);
    color: #e57373;
}

/* ── Canvas ──────────────────────────────────────────────────────── */
.canvas-card {
    background: #111722;
    border: 1px solid alpha(#ffffff, 0.06);
    border-radius: 18px;
}

.canvas-wrap {
    padding: 14px;
}

/* ── Inspector ───────────────────────────────────────────────────── */
.inspector-card {
    background: #111722;
    border: 1px solid alpha(#ffffff, 0.06);
    border-radius: 18px;
    padding: 16px;
}

.section-title {
    font-weight: 700;
    color: #f3f5fb;
}

.dim-copy {
    color: alpha(#f5f7ff, 0.55);
    font-size: smaller;
}

/* Output ratio buttons */
.ratio-btn {
    padding: 5px 4px;
    background: alpha(#ffffff, 0.05);
    border: 1px solid alpha(#ffffff, 0.08);
    border-radius: 8px;
    color: alpha(#f5f7ff, 0.65);
    font-size: smaller;
}

.ratio-btn.selected {
    background: alpha(#8d5bff, 0.35);
    border-color: alpha(#8d5bff, 0.6);
    color: #d0c0ff;
}

/* Background swatches */
.background-swatch {
    min-height: 38px;
    min-width: 38px;
    border: 2px solid transparent;
    border-radius: 10px;
}

.background-swatch.selected {
    border-color: #d9ddff;
}

.swatch-cornflower { background: linear-gradient(135deg, #6ea2ff, #8263f5); }
.swatch-sunset     { background: linear-gradient(135deg, #ffb46c, #e85d44); }
.swatch-ocean      { background: linear-gradient(135deg, #38bdf8, #0f766e); }
.swatch-forest     { background: linear-gradient(135deg, #4ade80, #15803d); }
.swatch-rose       { background: linear-gradient(135deg, #f9a8d4, #be185d); }
.swatch-midnight   { background: linear-gradient(135deg, #6366f1, #1e1b4b); }
.swatch-golden     { background: linear-gradient(135deg, #fbbf24, #b45309); }
.swatch-lavender   { background: linear-gradient(135deg, #c4b5fd, #7c3aed); }
.swatch-mint       { background: linear-gradient(135deg, #6ee7b7, #0d9488); }
.swatch-slate      { background: #1f242d; }
.swatch-charcoal   { background: #2d3748; }
.swatch-deepspace  { background: linear-gradient(135deg, #1a1a2e, #16213e); }

/* ── Bottom bar ──────────────────────────────────────────────────── */
.bottom-bar {
    background: #0d1219;
    border-top: 1px solid alpha(#ffffff, 0.07);
    padding: 8px 0;
    min-height: 48px;
}

.format-pill {
    padding: 5px 12px;
    background: alpha(#ffffff, 0.06);
    border: 1px solid alpha(#ffffff, 0.10);
    border-radius: 8px;
    color: alpha(#f5f7ff, 0.65);
    font-size: smaller;
}

.format-pill:checked {
    background: alpha(#8d5bff, 0.30);
    border-color: alpha(#8d5bff, 0.55);
    color: #d0c0ff;
}

.bottom-action-btn {
    padding: 6px 16px;
    border-radius: 8px;
}
"#;

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(EDITOR_CSS);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
