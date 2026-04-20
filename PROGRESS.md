# Snapix Development Progress

> Tracking file for Snapix development milestones.

---

## Current Status: **M1 — Wayland Polish** 🚧 In Progress

---

## M0 — Foundation ✅ Complete

### M0 Checklist

| Task | Status | Notes |
|------|--------|-------|
| Setup workspace với 4 crates | ✅ Done | `snapix-core`, `snapix-capture`, `snapix-ui`, `snapix-app` |
| CI (GitHub Actions: build + clippy + test + fmt) | ✅ Done | `.github/workflows/ci.yml` |
| GTK4 + libadwaita hello window | ✅ Done | `snapix-ui/src/app.rs` |
| `CaptureBackend` trait | ✅ Done | `snapix-capture/src/backend.rs` |
| X11 backend (`capture_full`) | ✅ Done | `snapix-capture/src/x11.rs` — BGR→RGBA conversion |
| X11 backend (`capture_region`) | ✅ Done | Works, cần GUI overlay cho interactive selection |
| X11 backend (`capture_window`) | ✅ Done | EWMH `_NET_ACTIVE_WINDOW` |
| Wayland backend (ashpd portal) | ✅ Done | `snapix-capture/src/wayland.rs` |
| Entitlements struct + Feature flags | ✅ Done | `snapix-core/src/entitlements.rs` |
| `LicenseVerifier` trait + StubVerifier | ✅ Done | `snapix-core/src/license.rs` |
| Canvas model (Image, Rect, Annotation, Document) | ✅ Done | `snapix-core/src/canvas.rs` |
| CLI skeleton (`snapix capture --mode full -o out.png`) | ✅ Done | `snapix-app/src/main.rs` |
| Async runtime decision | ✅ Done | `async-std` (compatible với ashpd + GTK) |
| Logging setup | ✅ Done | `tracing` + `tracing-subscriber` |
| Unit tests cho snapix-core | ✅ Done | 15 tests (canvas, entitlements, license) |
| Integration tests cho snapix-capture | ✅ Done | 3 tests (backend detection, creation) |

### M0 Ship Criteria

```
snapix capture --mode full -o test.png
```

- [x] Hoạt động trên X11
- [x] Hoạt động trên Wayland (via ashpd portal)

---

## M1 — Wayland Polish

| Task | Status | Notes |
|------|--------|-------|
| Test trên GNOME Wayland | 🔲 Pending | Manual test needed |
| Test trên KDE Plasma 6 Wayland | 🔲 Pending | Manual test needed |
| Flatpak manifest | ✅ Done | `flatpak/io.github.snapix.Snapix.yml` |
| Desktop file + metainfo | ✅ Done | `data/io.github.snapix.Snapix.*` |
| App icon (placeholder) | ✅ Done | `data/icons/` |
| Runtime detect X11/Wayland (improve) | ✅ Done | `SessionType` enum, multiple detection methods |
| Handle portal permission dialog UX | ✅ Done | `WaylandCaptureError` enum with specific errors |

---

## M2 — Editor MVP

| Task | Status | Notes |
|------|--------|-------|
| GTK4 editor window với `DrawingArea` | 🔲 Pending | |
| Canvas render pipeline | 🔲 Pending | tiny-skia → cairo surface |
| Tool: Crop | 🔲 Pending | |
| Tool: Arrow | 🔲 Pending | |
| Tool: Text | 🔲 Pending | |
| Export PNG + copy clipboard | 🔲 Pending | |
| Undo/Redo stack (command pattern) | 🔲 Pending | |

---

## M3 — Beautify

| Task | Status | Notes |
|------|--------|-------|
| Background: gradient picker | 🔲 Pending | |
| Background: solid color | 🔲 Pending | |
| Background: blur of screenshot | 🔲 Pending | |
| Frame: padding slider | 🔲 Pending | |
| Frame: corner radius | 🔲 Pending | |
| Frame: drop shadow | 🔲 Pending | |
| Preset system (save/load) | 🔲 Pending | |

---

## M4 — Polish & Release v0.1

| Task | Status | Notes |
|------|--------|-------|
| Preferences dialog | 🔲 Pending | |
| i18n (English + Vietnamese) | 🔲 Pending | |
| App icon | 🔲 Pending | |
| .desktop file | 🔲 Pending | |
| AppStream metadata | 🔲 Pending | |
| Website landing page | 🔲 Pending | |
| Gumroad/Lemon Squeezy setup | 🔲 Pending | |
| Ed25519 license key verification | 🔲 Pending | Replace StubVerifier |
| "Unlock Pro" dialog trong app | 🔲 Pending | |
| Flathub submission | 🔲 Pending | |

---

## M5+ — Pro Features (v0.2+)

| Feature | Status | Tier |
|---------|--------|------|
| Auto-redact (OCR + regex) | 🔲 Pending | Pro |
| Window mockup frames | 🔲 Pending | Pro |
| Upload integration (imgur/S3) | 🔲 Pending | Pro |
| Numbered step tool | 🔲 Pending | Free |
| Spotlight tool | 🔲 Pending | Free |
| Scrolling screenshot | 🔲 Pending | Pro |
| Screen recording → GIF/MP4 | 🔲 Pending | Pro |
| OCR copy text | 🔲 Pending | Pro |

---

## Codebase Structure

```
snapix/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── snapix-core/              # Domain logic (no GTK dependency)
│   │   └── src/
│   │       ├── canvas.rs         # Image, Rect, Annotation, Document
│   │       ├── entitlements.rs   # Tier, Feature, Entitlements
│   │       └── license.rs        # LicenseVerifier trait
│   │
│   ├── snapix-capture/           # Screenshot backends
│   │   └── src/
│   │       ├── backend.rs        # CaptureBackend trait + detect_backend()
│   │       ├── x11.rs            # X11Backend (x11rb)
│   │       └── wayland.rs        # WaylandBackend (ashpd portal)
│   │
│   ├── snapix-ui/                # GTK4 + libadwaita UI
│   │   └── src/
│   │       ├── app.rs            # SnapixApp entry point
│   │       ├── editor.rs         # (planned) Editor window
│   │       └── widgets.rs        # (planned) Custom widgets
│   │
│   └── snapix-app/               # Binary entry point
│       └── src/
│           └── main.rs           # CLI parsing + GTK launch
│
├── data/                         # Desktop integration files
│   ├── io.github.snapix.Snapix.desktop
│   ├── io.github.snapix.Snapix.metainfo.xml
│   └── icons/
│
├── flatpak/                      # Flatpak build files
│   └── io.github.snapix.Snapix.yml
│
├── .github/workflows/ci.yml      # CI pipeline
├── Snapix-Plan.md                # Product vision & roadmap
└── PROGRESS.md                   # This file
```

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Done |
| ⚠️ | Partial / Stub |
| 🔲 | Not started |
| 🚧 | In progress |

---

## Notes & Decisions

### Async Runtime
- Chọn `async-std` thay vì `tokio` vì integrate tốt hơn với GTK main loop và `ashpd`.

### Capture Strategy
- **X11:** Dùng `x11rb` gọi `GetImage` trực tiếp, convert BGR→RGBA.
- **Wayland:** Bắt buộc qua XDG portal (`ashpd`). Portal trả về file URI, load bằng `image` crate.

### License Key
- M0-M3: Dùng `StubVerifier` (key `SNAPIX-PRO-DEV` = Pro).
- M4: Implement Ed25519 signing với `ed25519-dalek`.

---

*Last updated: 2026-04-20*

---

## Changelog

### 2026-04-20
- **M0 Complete** 🎉
  - Implemented X11 `capture_window` with EWMH `_NET_ACTIVE_WINDOW`
  - Added 15 unit tests for `snapix-core` (canvas, entitlements, license)
  - Added 3 integration tests for `snapix-capture` (backend detection)
  - Fixed `ashpd` async runtime issue (switched to `async-std` feature)
  - Fixed Wayland detection for empty `WAYLAND_DISPLAY` env var

- **M1 Progress**
  - Created Flatpak manifest (`flatpak/io.github.snapix.Snapix.yml`)
  - Added `.desktop` file and AppStream metainfo
  - Added placeholder SVG icon
  - Improved session detection with `SessionType` enum (WAYLAND_DISPLAY, XDG_SESSION_TYPE, DISPLAY, GDK_BACKEND)
  - Added `WaylandCaptureError` for better portal error handling (Cancelled, PortalUnavailable, PermissionDenied)
  - Added URL decoding for portal file paths
