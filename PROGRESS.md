# Snapix Development Progress

> Tracking file for Snapix development milestones.

---

## Current Status: **M2 — Editor MVP** ✅ Complete

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
| GTK4 editor window với `DrawingArea` | ✅ Done | `EditorWindow` + `DocumentCanvas` are live in `snapix-ui` |
| Canvas render pipeline | ✅ Done | Cairo-based preview/export pipeline renders background, frame, image, crop overlay, arrow, rectangle, ellipse, blur, and text annotations; PNG/JPEG export and clipboard copy use the same composition and layout rules as the preview |
| Tool: Crop | ✅ Done | Non-destructive crop with default selection, move/resize handles, `Enter` apply (min 4×4 px enforced, "too small" toast), `Esc` exit; selection bounds are clamped to image dimensions |
| Tool: Arrow | ✅ Done | Drag on the image to place an arrow; preview, save/copy, undo/redo, and endpoint resize in Select mode are wired |
| Tool: Rectangle | ✅ Done | Drag on the image to draw a rectangle annotation with color/width controls |
| Tool: Ellipse | ✅ Done | Drag on the image to draw an ellipse annotation with color/width controls |
| Tool: Text | ✅ Done | Click on the image, enter text in a dialog, commit a text annotation, and re-edit selected text via dialog in Select mode |
| Tool: Blur | ✅ Done | Drag on the image to create a blur region annotation |
| Export PNG/JPEG + copy clipboard | ✅ Done | `Save`, `Save As`, and `Copy` all use the same rendered canvas output |
| Undo/Redo stack | ✅ Done | Snapshot-based history is wired for crop, frame/background changes, annotation placement, text edit, move, and resize |
| Capture/import action row | ⚠️ Partial | Top-row `Fullscreen / Region / Import / Clear` actions are wired; Wayland fullscreen now falls back to interactive region capture, but true fullscreen/window distinction still depends on portal behavior |
| Editor feedback polish | ✅ Done | Toasts now cover capture, import, copy, save, crop apply, and annotation placement; export actions disable when no image is loaded |
| Annotation selection/editing | ✅ Done | Select, highlight, recolor, move, resize (rect/ellipse/blur handles, arrow endpoints), delete, re-edit text; switching tools clears selection; selecting an annotation syncs active color+width to its values (palette highlight + slider update); toast only fires on meaningful state changes |
| Settings panel resizing | ✅ Done | Main workspace uses a draggable split view so the inspector can be widened or narrowed without breaking the editor |
| Shadow controls | ✅ Done | Inspector supports shadow direction, shadow padding, blur, and strength; `0px` padding keeps the shadow attached to the image and directional padding now respects the chosen side |

### M2 Snapshot

- Editor shell launches as the default GUI window via `snapix_ui::SnapixApp`, and startup capture loads directly into the editor.
- The workspace UI has been reshaped toward an editor-style layout with a top action row, top tool row, central canvas, and right-side settings panel.
- The main workspace now uses a resizable split layout so the right-side settings panel can be widened or narrowed by drag.
- GUI startup on Wayland still falls back from failing full-screen portal capture to interactive capture so the editor opens with a real image when possible.
- Canvas renders beautify output plus committed arrow, rectangle, ellipse, blur, and text annotations, while crop uses its own interaction/overlay layer.
- Inspector controls already update `Document.frame` and `Document.background` in real time, including direction-aware shadow tuning.
- `Save`/`Save As` export the rendered canvas to PNG or JPEG and `Copy` writes the rendered image to the system clipboard.
- Preview canvas layout is now aligned more closely with export/copy output so padding, aspect ratio, and shadow composition stay consistent.
- Crop supports default selection, move/resize handles, `Enter` apply, and `Esc` exit back to Select mode.
- Arrow, rectangle, ellipse, and blur support drag placement with toast feedback when placement succeeds or is too small.
- Text supports click-to-place plus dialog input, re-editing through Select mode, and the empty state now guides the user toward capture/import first.
- Select mode can highlight existing annotations, apply color/width changes, move annotations by drag, resize rectangle/ellipse/blur with corner handles, resize arrows via endpoint handles, and delete the current selection from either the toolbar or the keyboard.
- The top toolbar now switches its slider label between `Width` and `Size` depending on the current tool or selected annotation type.
- Undo/redo is working via whole-document snapshots.
- The remaining M2 risk area is capture UX on Wayland portals, where true fullscreen/window semantics are still inconsistent.

---

## M3 — Beautify

| Task | Status | Notes |
|------|--------|-------|
| Background: gradient picker | ✅ Done | Inspector now supports a native gradient mode with editable start/end colors and angle control |
| Background: solid color | ✅ Done | Inspector now supports a native solid-color mode with direct color picking |
| Background: blur of screenshot | ✅ Done | Inspector can switch to `Screenshot Blur`, adjust blur radius, and preview/export use the same cached blurred background rendering |
| Frame: padding slider | ✅ Done | Live frame padding control is wired in the GTK editor |
| Frame: corner radius | ✅ Done | Live corner radius control is wired in the GTK editor |
| Frame: drop shadow | ✅ Done | Shadow toggle, direction, padding, blur, and strength are all live in the GTK editor |
| Preset system (save/load) | ✅ Done | Saved style presets can now be stored locally, reapplied, overwritten, and deleted from the inspector |

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
│   │       ├── editor.rs         # Editor window, actions, and interaction state
│   │       └── widgets.rs        # Canvas rendering and input handling
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

*Last updated: 2026-04-22*

---

## Changelog

### 2026-04-21
- **M2 Progress**
  - Refreshed the editor shell layout toward a cleaner top-toolbar workspace design
  - Added undo/redo for document snapshots
  - Added usable Arrow tool with preview and committed annotation rendering
  - Added usable Text tool with click placement and dialog input
  - Added usable Rectangle, Ellipse, and Blur annotation tools
  - Added top action row handlers for fullscreen/region/import/clear
  - Added import-from-file flow directly into the editor
  - Improved crop with default selection plus move/resize handles, `Esc` exit, and apply feedback
  - Added empty-state guidance plus toast feedback for capture, import, save, copy, and annotation actions
  - Disabled export actions when no image is loaded
  - Added annotation selection, highlight, toolbar-based color/width edits, and delete-by-selection
  - Added keyboard delete support via `Backspace` / `Delete`
  - Switched the main editor workspace to a draggable split view so the settings panel can be resized
  - Reworked shadow controls with direction presets, shadow padding, blur, and strength
  - Tightened shadow rendering so `0px` padding keeps the shadow attached to the image and directional padding expands toward the chosen side
  - Unified preview/export composition rules so canvas layout matches `Copy`/`Save` output more closely
  - Added tests for CLI region validation, Wayland capture fallback behavior, selection/edit helpers, and editor state helpers
  - Documented current Wayland portal capture limitations in the UI behavior

### 2026-04-22
- **M3 Progress**
  - Added a real `Screenshot Blur` background mode in the inspector with adjustable blur radius
  - Wired blurred screenshot background rendering into both preview and export so output matches the editor
  - Extended the blur surface cache to reuse full-background blur renders in addition to annotation blur regions
  - Kept background preset swatches working alongside blur mode selection
  - Added direct `Gradient` and `Solid` background modes with native color pickers in the inspector
  - Added editable gradient angle control and updated gradient rendering to honor the chosen angle
  - Added a local saved-preset system for beautify settings with save/apply/delete actions in the inspector
  - Persisted style presets to the user config directory as JSON and covered preset roundtrip with tests
  - Verified the workspace with `cargo test`

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

- **M2 Progress**
  - Added `EditorWindow` as the default GTK4 GUI shell
  - Added startup capture -> editor wiring, including Wayland fallback from full-screen capture to interactive window capture
  - Added `DocumentCanvas` with Cairo rendering for document background, frame, image preview, and crop overlay
  - Added inspector controls for padding, corner radius, shadow, and preset background styles
  - Added `Save` PNG export and `Copy` clipboard output from the same render pipeline
  - Added a first non-destructive crop flow with drag selection plus `Enter` apply / `Esc` cancel
  - Left richer annotation editing and portal-specific capture polish for the next M2 steps
