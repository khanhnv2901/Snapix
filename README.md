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
| M0 — Foundation (workspace, X11 capture, entitlements) | ✅ In progress |
| M1 — Wayland support | ⏳ Planned |
| M2 — Editor MVP | ⏳ Planned |
| M3 — Beautify | ⏳ Planned |
| M4 — v0.1 release on Flathub | ⏳ Planned |

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

### CLI usage (M0)

```bash
# Capture full screen to PNG
snapix capture --mode full --output screenshot.png

# Launch GUI
snapix
```

## License

Apache-2.0 © 2026 Khanh Nguyen
