# Snapix

> Flameshot meets Xnapper — screenshot tool for Linux with fast annotation and beautiful exports. Native GTK4, Wayland-first.

[![CI](https://github.com/khanhnv2901/snapix/actions/workflows/ci.yml/badge.svg)](https://github.com/khanhnv2901/snapix/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Features

- Capture fullscreen / region / active window
- Global hotkey support (X11 native, Wayland via XDG portal)
- Annotation: crop, arrow, rectangle, text, blur
- **Beautify**: gradient/solid background, padding, corner radius, drop shadow
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

Current M2 progress includes a live GTK4 editor shell, startup capture wired into the editor, a redesigned workspace UI, `DrawingArea` canvas rendering, padding/corner-radius/shadow/background controls, PNG export, clipboard copy, undo/redo, usable crop, arrow, and text tools, plus top-row actions for fullscreen/region capture, import, and clear. The main open M2 gaps are richer annotation editing, better crop polish, and Wayland capture limitations for true fullscreen/window distinctions.

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

# Capture active window
snapix capture --mode window --output window.png

# Launch GUI
snapix
```

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
