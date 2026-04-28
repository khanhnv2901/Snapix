use gtk4::prelude::*;
use libadwaita::Application;
use snapix_core::canvas::Document;

use crate::editor::{apply_style_preferences, load_preferences, EditorWindow};

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
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(false);
    }
    if let Ok(preferences) = load_preferences() {
        apply_style_preferences(&preferences);
    }
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
    padding: 7px 16px 7px 16px;
    border-bottom: 1px solid alpha(#ffffff, 0.05);
}

.capture-cluster,
.capture-export-row {
    background: alpha(#ffffff, 0.03);
    border: 1px solid alpha(#ffffff, 0.06);
    border-radius: 14px;
    padding: 5px 8px;
    box-shadow: inset 0 1px 0 alpha(#ffffff, 0.03);
}

.cluster-title {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: alpha(#f5f7ff, 0.42);
    margin-left: 6px;
    margin-bottom: 1px;
}

.capture-pill {
    padding: 8px 14px;
    color: #f5f7ff;
    border: 1px solid alpha(#ffffff, 0.08);
    border-radius: 10px;
    box-shadow: inset 0 1px 0 alpha(#ffffff, 0.08);
}

.capture-pill-icon {
    color: inherit;
}

.capture-pill-label {
    font-weight: 600;
}

.capture-pill.fullscreen { background: linear-gradient(135deg, #844dff, #6e3ce8); }
.capture-pill.region     { background: linear-gradient(135deg, #3fb9c8, #2a9f9f); }
.capture-pill.window     { background: linear-gradient(135deg, #e54f8a, #c43c6d); }
.capture-pill.import     { background: linear-gradient(135deg, #f0a73f, #d4791f); }
.capture-pill.utility    { background: #1a2230; }

.capture-pill:hover {
    filter: brightness(1.08);
}

.capture-pill:active {
    filter: brightness(0.96);
}

.capture-pill:disabled {
    opacity: 0.45;
}

/* ── Tool row ───────────────────────────────────────────────────── */
.tool-row {
    padding: 5px 16px 5px 16px;
    border-bottom: 1px solid alpha(#ffffff, 0.05);
}

.tool-row-card {
    background: #111722;
    border: 1px solid alpha(#ffffff, 0.06);
    border-radius: 14px;
    padding: 7px 12px;
    box-shadow: inset 0 1px 0 alpha(#ffffff, 0.03);
}

.tool-pill {
    padding: 8px;
    background: transparent;
    color: alpha(#f5f7ff, 0.72);
    border: 1px solid transparent;
    border-radius: 10px;
    transition: 180ms ease;
}

.tool-pill:hover {
    background: alpha(#ffffff, 0.05);
    color: alpha(#f5f7ff, 0.92);
}

.tool-pill:focus-visible {
    outline: 2px solid alpha(#8d5bff, 0.65);
    outline-offset: 1px;
}

.tool-pill:checked {
    background: linear-gradient(135deg, #8d5bff, #643bda);
    color: #ffffff;
    border-color: alpha(#ffffff, 0.16);
    box-shadow:
        inset 0 1px 0 alpha(#ffffff, 0.14),
        0 0 0 1px alpha(#8d5bff, 0.28);
}

/* Color swatch dots */
.color-swatch-btn {
    padding: 2px;
    background: transparent;
    border: 2px solid transparent;
    border-radius: 50%;
    min-width: 24px;
    min-height: 24px;
}

.color-swatch-btn:hover {
    background: alpha(#ffffff, 0.06);
}

.color-swatch-btn.active {
    border-color: white;
    box-shadow: 0 0 0 3px alpha(#ffffff, 0.12);
}

.color-dot {
    border-radius: 50%;
    min-width: 16px;
    min-height: 16px;
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

.tool-delete-btn:disabled {
    opacity: 0.38;
}

/* ── Workspace paned handle ─────────────────────────────────────── */
paned > separator {
    background: alpha(#ffffff, 0.07);
    min-width: 4px;
    margin: 4px 2px;
    border-radius: 2px;
}

paned > separator:hover {
    background: alpha(#8d5bff, 0.55);
}

/* ── Canvas ──────────────────────────────────────────────────────── */
.canvas-card {
    background: #111722;
    border: 1px solid alpha(#ffffff, 0.04);
    border-radius: 12px;
    box-shadow:
        inset 0 1px 0 alpha(#ffffff, 0.03),
        0 14px 36px alpha(#000000, 0.18);
}

.canvas-wrap {
    padding: 2px;
}

.reframe-done-btn {
    min-height: 34px;
    padding: 0 16px;
    border-radius: 999px;
    box-shadow:
        0 12px 30px alpha(#000000, 0.24),
        inset 0 1px 0 alpha(#ffffff, 0.14);
}

.reframe-reset-btn {
    min-height: 34px;
    padding: 0 16px;
    border-radius: 999px;
    background: alpha(#111722, 0.92);
    border: 1px solid alpha(#ffffff, 0.10);
    box-shadow:
        0 12px 30px alpha(#000000, 0.24),
        inset 0 1px 0 alpha(#ffffff, 0.14);
}

.reframe-overlay-actions {
    padding: 8px;
}

/* ── Inspector ───────────────────────────────────────────────────── */
.inspector-card {
    background: #111722;
    border: 1px solid alpha(#ffffff, 0.06);
    border-radius: 18px;
    padding: 14px;
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
    padding: 3px 6px;
    background: alpha(#ffffff, 0.05);
    border: 1px solid alpha(#ffffff, 0.08);
    border-radius: 6px;
    color: alpha(#f5f7ff, 0.65);
    font-size: smaller;
    min-height: 26px;
}

.ratio-btn.selected {
    background: alpha(#8d5bff, 0.35);
    border-color: alpha(#8d5bff, 0.6);
    color: #d0c0ff;
}

/* Shadow direction grid buttons */
.shadow-dir-btn {
    padding: 0;
    background: alpha(#ffffff, 0.05);
    border: 1px solid alpha(#ffffff, 0.08);
    border-radius: 8px;
    color: alpha(#f5f7ff, 0.80);
    font-size: 18px;
    min-width: 38px;
    min-height: 38px;
}

.shadow-dir-btn.selected {
    background: alpha(#8d5bff, 0.35);
    border-color: alpha(#8d5bff, 0.6);
    color: #d0c0ff;
}

/* Background swatches */
.background-swatch {
    min-height: 36px;
    min-width: 36px;
    border: 2px solid transparent;
    border-radius: 8px;
}

.background-swatch-signature {
    min-height: 52px;
    border-radius: 12px;
    border-color: alpha(#ffffff, 0.08);
}

.signature-preview-art {
    border-radius: 10px;
    overflow: hidden;
}

.signature-preview-base {
    border-radius: 10px;
}

.signature-preview-blueprint {
    background:
        linear-gradient(0deg, alpha(#3ddcff, 0.08), alpha(#3ddcff, 0.08)),
        linear-gradient(135deg, #0f1c34, #0a1325);
    border: 1px solid alpha(#59d7ff, 0.16);
}

.signature-preview-midnightpanel {
    background: linear-gradient(135deg, #1b2942, #0c1220);
    border: 1px solid alpha(#8eaaff, 0.14);
}

.signature-preview-cutpaper {
    background: linear-gradient(135deg, #efe3d1, #d9c7b3);
    border: 1px solid alpha(#1d2430, 0.12);
}

.signature-preview-terminalglow {
    background: linear-gradient(135deg, #061413, #0b2420);
    border: 1px solid alpha(#32f5c1, 0.14);
}

.signature-preview-redacted {
    background: linear-gradient(135deg, #20242d, #394253);
    border: 1px solid alpha(#ffffff, 0.10);
}

.signature-layer-panel {
    border-radius: 8px;
    margin: 10px;
}

.signature-layer-midnight-panel {
    border: 2px solid alpha(#d4dcff, 0.16);
    background: alpha(#0b1120, 0.18);
}

.signature-layer-glow {
    border-radius: 0;
}

.signature-layer-midnight-glow {
    background: linear-gradient(90deg, alpha(#000000, 0.0), alpha(#6e8dff, 0.18));
}

.signature-layer-blueprint-block {
    background: alpha(#3ddcff, 0.22);
    border-radius: 6px;
}

.signature-layer-blueprint-bar {
    background: alpha(#6ef2ff, 0.36);
    border-radius: 999px;
}

.signature-layer-cutpaper-shape-a {
    background: alpha(#253040, 0.14);
    border-radius: 0 0 24px 0;
}

.signature-layer-cutpaper-shape-b {
    background: alpha(#d35f55, 0.22);
    border-radius: 6px;
}

.signature-layer-terminal-badge {
    background: alpha(#31f3bf, 0.18);
    border-radius: 999px;
}

.signature-layer-terminal-glow {
    background: linear-gradient(90deg, alpha(#000000, 0.0), alpha(#1ef5aa, 0.12));
}

.signature-layer-terminal-bar {
    background: alpha(#ffbf42, 0.30);
    border-radius: 5px;
}

.signature-layer-redacted-bar-a {
    background: alpha(#0c1017, 0.50);
    border-radius: 4px;
}

.signature-layer-redacted-bar-b {
    background: alpha(#0c1017, 0.40);
    border-radius: 4px;
}

.signature-layer-redacted-badge {
    background: alpha(#ef5757, 0.34);
    border-radius: 4px;
}

.background-swatch-label {
    color: #f5f7ff;
    font-size: smaller;
    font-weight: 700;
}

.background-swatch-label-dark {
    color: #1d2430;
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
.swatch-steel      { background: #475569; }
.swatch-mist       { background: #e2e8f0; }
.swatch-sky        { background: #38bdf8; }
.swatch-emerald    { background: #10b981; }
.swatch-coral      { background: #fb7185; }
.swatch-amber      { background: #f59e0b; }
.swatch-violet     { background: #8b5cf6; }
.swatch-deepspace  { background: linear-gradient(135deg, #1a1a2e, #16213e); }
.swatch-aurora     { background: linear-gradient(135deg, #22d3ee, #10b981); }
.swatch-blueprint  { background: linear-gradient(135deg, #0f1c34, #0a1325); }
.swatch-midnightpanel { background: linear-gradient(135deg, #1b2942, #0c1220); }
.swatch-cutpaper   { background: linear-gradient(135deg, #efe3d1, #d9c7b3); }
.swatch-terminalglow { background: linear-gradient(135deg, #061413, #0b2420); }
.swatch-redacted   { background: linear-gradient(135deg, #20242d, #394253); }

/* ── Bottom bar ──────────────────────────────────────────────────── */
.bottom-bar {
    background: #0d1219;
    border-top: 1px solid alpha(#ffffff, 0.07);
    padding: 0;
    min-height: 22px;
}

.format-pill {
    padding: 5px 12px;
    background: alpha(#ffffff, 0.06);
    border: 1px solid alpha(#ffffff, 0.10);
    border-radius: 8px;
    color: alpha(#f5f7ff, 0.65);
    font-size: smaller;
}

.format-pill:hover {
    background: alpha(#ffffff, 0.10);
    color: alpha(#f5f7ff, 0.88);
}

.format-pill:checked {
    background: alpha(#8d5bff, 0.30);
    border-color: alpha(#8d5bff, 0.55);
    color: #d0c0ff;
}

.bottom-action-btn {
    padding: 6px 16px;
    border-radius: 8px;
    border: 1px solid alpha(#ffffff, 0.08);
    background: alpha(#ffffff, 0.04);
}

.capture-export-row .format-pill,
.capture-export-row .bottom-action-btn {
    padding: 4px 12px;
    min-height: 30px;
}

.bottom-action-btn:hover {
    background: alpha(#ffffff, 0.08);
    border-color: alpha(#ffffff, 0.14);
}

.bottom-action-btn.suggested-action {
    box-shadow: inset 0 1px 0 alpha(#ffffff, 0.18);
}

.bottom-action-btn:disabled {
    opacity: 0.42;
}

/* ── Light Appearance Overrides ─────────────────────────────────── */
.snapix-shell.snapix-light {
    background: #f3f6fb;
}

.snapix-shell.snapix-light .capture-row,
.snapix-shell.snapix-light .tool-row {
    border-color: alpha(#111827, 0.08);
}

.snapix-shell.snapix-light .tool-row-card,
.snapix-shell.snapix-light .capture-cluster,
.snapix-shell.snapix-light .capture-export-row,
.snapix-shell.snapix-light .canvas-card,
.snapix-shell.snapix-light .inspector-card {
    background: #ffffff;
    border: 1px solid alpha(#111827, 0.10);
    box-shadow:
        inset 0 1px 0 alpha(#ffffff, 0.60),
        0 12px 28px alpha(#0f172a, 0.08);
}

.snapix-shell.snapix-light .tool-pill {
    color: alpha(#111827, 0.72);
}

.snapix-shell.snapix-light .tool-pill:hover {
    background: alpha(#111827, 0.06);
    color: alpha(#111827, 0.96);
}

.snapix-shell.snapix-light .tool-pill:checked {
    background: linear-gradient(135deg, #e7eef8, #dbe7f6);
    color: #162033;
    border-color: alpha(#4b6584, 0.28);
    box-shadow:
        inset 0 1px 0 alpha(#ffffff, 0.75),
        0 6px 14px alpha(#0f172a, 0.06);
}

.snapix-shell.snapix-light .tool-pill:checked:hover {
    background: linear-gradient(135deg, #dfe8f5, #d3e1f3);
}

.snapix-shell.snapix-light .tool-delete-btn {
    color: alpha(#111827, 0.48);
}

.snapix-shell.snapix-light .tool-delete-btn:hover {
    background: alpha(#e53935, 0.12);
    color: #c62828;
}

.snapix-shell.snapix-light .section-title {
    color: #18212d;
}

.snapix-shell.snapix-light .cluster-title {
    color: alpha(#111827, 0.44);
}

.snapix-shell.snapix-light .dim-copy {
    color: alpha(#111827, 0.62);
}

.snapix-shell.snapix-light .ratio-btn,
.snapix-shell.snapix-light .shadow-dir-btn,
.snapix-shell.snapix-light .format-pill,
.snapix-shell.snapix-light .bottom-action-btn {
    background: alpha(#111827, 0.04);
    border-color: alpha(#111827, 0.10);
    color: alpha(#111827, 0.70);
}

.snapix-shell.snapix-light .color-swatch-btn.active {
    border-color: #334155;
    box-shadow: 0 0 0 3px alpha(#334155, 0.12);
}

.snapix-shell.snapix-light .ratio-btn.selected,
.snapix-shell.snapix-light .shadow-dir-btn.selected,
.snapix-shell.snapix-light .background-swatch.selected,
.snapix-shell.snapix-light .format-pill:checked {
    background: alpha(#8d5bff, 0.16);
    border-color: alpha(#7c3aed, 0.48);
    color: #4c1d95;
    box-shadow: 0 0 0 1px alpha(#8d5bff, 0.08);
}

.snapix-shell.snapix-light .format-pill:hover,
.snapix-shell.snapix-light .bottom-action-btn:hover {
    background: alpha(#111827, 0.08);
    border-color: alpha(#111827, 0.14);
}

.snapix-shell.snapix-light .capture-pill.utility {
    background: #eef2f8;
    border-color: alpha(#111827, 0.10);
    color: #223045;
    box-shadow: inset 0 1px 0 alpha(#ffffff, 0.70);
}

.snapix-shell.snapix-light .capture-export-row .format-pill:checked,
.snapix-shell.snapix-light .capture-export-row .bottom-action-btn.suggested-action {
    background: alpha(#0f766e, 0.10);
    border-color: alpha(#0f766e, 0.28);
    color: #115e59;
    box-shadow: inset 0 1px 0 alpha(#ffffff, 0.68);
}

.snapix-shell.snapix-light .capture-export-row {
    background: #f8fafc;
}

.snapix-shell.snapix-light .bottom-bar {
    background: #edf2f8;
    border-top: 1px solid alpha(#111827, 0.08);
    min-height: 20px;
    padding: 0;
}

.snapix-shell.snapix-light paned > separator {
    background: alpha(#111827, 0.10);
}

.snapix-shell.snapix-light paned > separator:hover {
    background: alpha(#8d5bff, 0.45);
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
