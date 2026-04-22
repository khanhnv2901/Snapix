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
| M2 — Editor MVP | ✅ Complete |
| M3 — Beautify | ✅ Complete |
| M4 — v0.1 release on Flathub | ⏳ Planned |

See [PROGRESS.md](PROGRESS.md) for detailed progress tracking.

Current progress includes the full editor MVP plus completed Beautify features: native gradient and solid background pickers, screenshot blur background, frame padding/radius/shadow controls, local style presets, and direct on-canvas `Image Reframe`. Reframe mode now supports drag-to-pan, wheel zoom, pinch zoom on touchpads, focus-aware zoom toward the cursor or pinch center, visible grid overlay, zoom HUD, reset-by-double-click, and stable `Esc` exit behavior. The main remaining work is M4 release polish: preferences, i18n, branding/assets, production license verification, and distribution polish.

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
