# Snapix Progress

This file tracks the current product and release status at a high level.
Status source of truth for execution progress: use this file first.
Last synced: 2026-04-28.
Date format: `YYYY-MM-DD`.

## Release Snapshot

- Latest tagged release: `0.1.4` (2026-04-24)
- Active milestone: `M5 Signature Backgrounds`
- Release notes: [CHANGELOG.md](CHANGELOG.md)

## Update Template

Use this checklist when updating status docs:

1. Update `Last synced` in this file.
2. Keep `Latest tagged release` and `Active milestone` in sync with [README.md](README.md) `Current Status`.
3. Add ongoing release work under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md).
4. Use date format `YYYY-MM-DD` across all status and release docs.

## Current State

- Core screenshot capture is implemented for X11 and Wayland portal flows
- The GTK4/libadwaita editor is functional and used as the default app experience
- Annotation tools are available: crop, arrow, rectangle, ellipse, text, and blur
- Beautify controls are available: gradient, solid color, screenshot blur, padding, radius, and shadow
- Export flows are available: quick save, save as, copy to clipboard, PNG, and JPEG
- Local preferences and Pro activation flow are implemented
- Flatpak bundles build successfully against GNOME Platform `50`
- Flathub submission files are prepared, including vendored Cargo sources for offline builds
- Signature background system is implemented: 5 styled presets (Blueprint, Midnight Panel, Cut Paper, Terminal Glow, Redacted) with a dedicated UI tab, intensity slider, and per-style shadow profile tuning

## Milestones

| Milestone | Status | Notes |
|-----------|--------|-------|
| M0 Foundation | ✅ Complete | Workspace, capture backends, CLI entry point, core models |
| M1 Wayland Polish | ✅ Complete | Portal-aware capture flow, desktop integration, Flatpak base |
| M2 Editor MVP | ✅ Complete | Main editor, annotation tools, export flows, undo/redo |
| M3 Beautify | ✅ Complete | Background/frame styling, presets, image reframe |
| M4 Packaging Prep | 🚧 In progress | Release `0.1.4` shipped, Flatpak bundle path working, Flathub submission pending |
| M5 Signature Backgrounds | 🚧 In progress | Style model, 5 renderers, UI tab, intensity slider — pending export QA and commit |

## Current Packaging Notes

- The app metadata, desktop file, icon, and Flatpak manifest live in `data/` and `flatpak/`
- Quick Save now targets `~/Pictures/Screenshots` to match Flatpak sandbox permissions
- The main repo is kept lightweight for normal Cargo builds
- Flatpak bundle builds now generate and use `flatpak/cargo-sources.json` for offline Cargo resolution
- GitHub Releases can ship a `.flatpak` bundle artifact for tester installs before Flathub lands
- `flatpak/flathub.json` currently restricts submission builds to `x86_64`
- `FLATHUB.md` documents the current submission flow and required files

## Remaining M4 Work

- Open the new-app submission PR against `flathub/flathub`
- Respond to Flathub review feedback and run PR test builds
- External distribution tasks such as landing page and payment setup
- Final QA on target Linux desktop environments

## Signature Background Work (M5) — Current Branch

### What is done (uncommitted)

All changes are staged in the working tree, not yet committed.

- **Phase 1 — Internal style model** (`crates/snapix-core/src/canvas.rs`, `editor/state.rs`)
  - Added `Background::Style { id: BackgroundStyleId, intensity: f32 }` variant
  - Added `BackgroundStyleId` enum: `Blueprint`, `MidnightPanel`, `CutPaper`, `TerminalGlow`, `Redacted`
  - Serialization via serde with default intensity `0.65`
  - Updated `same_background` in `state.rs` to handle `Style` equality by discriminant + intensity

- **Phase 2 — Renderer** (`widgets/geometry/paint.rs`, `widgets/render/canvas.rs`)
  - Added `paint_signature_background` dispatcher and five style renderers
  - Added `paint_signature_preview_thumbnail` for inspector preview cards
  - Added `signature_shadow_profile` returning per-style blur/strength scale factors
  - Canvas shadow path uses `signature_shadow_profile` to tune shadow per active style

- **Phase 3 — Presets** (rendered as part of Phase 2)
  - `Blueprint`: deep navy + technical grid + cyan accent block and bars
  - `Midnight Panel`: radial dark gradient + inset panel borders + blue edge glow
  - `Cut Paper`: warm off-white + geometric paper shapes + terracotta accent
  - `Terminal Glow`: dark green-black + scanlines + green/amber accent blocks
  - `Redacted`: charcoal gradient + horizontal bars + red accent + border

- **Phase 4 — UI integration** (`editor/ui/inspector/background.rs`, `app.rs`, `i18n.rs`)
  - Added `Signature` mode button to the background mode row
  - Added signature presets grid with preview cards for all 5 styles
  - Added `Style Intensity` slider (0.2 – 1.0 range)
  - Inspector show/hide logic updated for Signature mode
  - CSS added for signature preview tiles per style
  - i18n strings added: `inspector_background_mode_signature`, `inspector_signature_intensity_label`

- **Partial Phase 5 — Export polish**
  - Per-style shadow profiles tuned with distinct blur/strength scale factors

### What remains

- Export QA: verify preview/export parity for all 5 styles at common canvas sizes
- Phase 5 full texture/anti-aliasing pass
- Phase 6 (Atmosphere) can be deferred — TerminalGlow and Redacted are already in Signature family
- Commit and tag when QA passes

## Reference

- Main build and run instructions: [README.md](README.md)
- Flatpak manifest: [flatpak/io.github.snapix.Snapix.yml](flatpak/io.github.snapix.Snapix.yml)
- Flathub submission notes: [FLATHUB.md](FLATHUB.md)
- AppStream metadata: [data/io.github.snapix.Snapix.metainfo.xml](data/io.github.snapix.Snapix.metainfo.xml)
