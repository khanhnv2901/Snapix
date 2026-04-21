# Snapix

> Flameshot meets Xnapper — screenshot tool for Linux with fast annotation and beautiful exports. Native GTK4, Wayland-first.

[![CI](https://github.com/khanhnv2901/snapix/actions/workflows/ci.yml/badge.svg)](https://github.com/khanhnv2901/snapix/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Features

- Capture fullscreen / region / active window
- Annotation: crop, arrow, rectangle, ellipse, text, blur
- **Beautify**: gradient/solid background, padding, corner radius, direction-aware drop shadow
- In-editor keyboard shortcuts for crop and undo/redo
- Export PNG/JPG, copy to clipboard
- Freemium: powerful free tier + Pro features via one-time license key

## Status

| Milestone | Status |
|-----------|--------|
| M0 — Foundation (workspace, X11/Wayland capture, entitlements) | ✅ Complete |
| M1 — Wayland Polish (Flatpak, portal UX) | ⚠️ Mostly complete, QA pending |
| M2 — Editor MVP | 🚧 In progress |
| M3 — Beautify | ⏳ Planned |
| M4 — v0.1 release on Flathub | ⏳ Planned |

See [PROGRESS.md](PROGRESS.md) for detailed progress tracking.

Current M2 progress includes a live GTK4 editor shell, startup capture wired into the editor, a redesigned workspace UI, `DrawingArea` canvas rendering, padding/corner-radius/shadow/background controls, PNG/JPEG export, clipboard copy, undo/redo, usable crop, arrow, rectangle, ellipse, text, and blur tools, plus top-row actions for fullscreen/region capture, import, and clear. Recent polish added clearer empty-state guidance, toast feedback for capture/export/annotation actions, selection for existing annotations, inline editing via color/width controls, keyboard deletion with `Backspace`/`Delete`, a resizable settings panel, direction-aware shadow controls with shadow padding, and a unified preview/export composition so the canvas matches `Copy`/`Save` output more closely. The main open M2 gaps are deeper annotation editing, final crop polish, and Wayland capture limitations for true fullscreen/window distinctions.

## Building

### Prerequisites (Ubuntu/Debian)

```bash
sudo apt-get install \
  libgtk-4-dev libadwaita-1-dev libglib2.0-dev \
  libx11-dev libxrandr-dev pkg-config \
  cargo
```

### Build

```bash
git clone https://github.com/khanhnv2901/snapix
cd snapix
cargo build --release
```

### CLI usage

```bash
# Capture full screen to PNG
snapix capture --mode full --output screenshot.png

# Capture a region on X11 by explicit bounds
snapix capture --mode region --x 100 --y 80 --width 1280 --height 720 --output region.png

# Capture active window
snapix capture --mode window --output window.png

# Launch GUI
snapix
```

On Wayland, CLI region capture uses the XDG portal picker when bounds are not provided. CLI window capture is still better handled through the GUI because portal behavior differs across desktops.

## Project Structure

```
snapix/
├── crates/
│   ├── snapix-core/      # Domain logic (canvas, entitlements, license)
│   ├── snapix-capture/   # Screenshot backends (X11, Wayland portal)
│   ├── snapix-ui/        # GTK4 + libadwaita UI
│   └── snapix-app/       # CLI + binary entry point
├── data/                 # Desktop files, icons, metainfo
└── flatpak/              # Flatpak build manifest
```

## License

Apache-2.0 © 2026 Khanh Nguyen
