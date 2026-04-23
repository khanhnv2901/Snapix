# Snapix Progress

This file tracks the current product and release status at a high level.
Status source of truth for execution progress: use this file first.
Last synced: 2026-04-24.
Date format: `YYYY-MM-DD`.

## Release Snapshot

- Latest tagged release: `0.1.4` (2026-04-24)
- Active milestone: `M4 Packaging Prep`
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

## Milestones

| Milestone | Status | Notes |
|-----------|--------|-------|
| M0 Foundation | ✅ Complete | Workspace, capture backends, CLI entry point, core models |
| M1 Wayland Polish | ✅ Complete | Portal-aware capture flow, desktop integration, Flatpak base |
| M2 Editor MVP | ✅ Complete | Main editor, annotation tools, export flows, undo/redo |
| M3 Beautify | ✅ Complete | Background/frame styling, presets, image reframe |
| M4 Packaging Prep | 🚧 In progress | Release `0.1.4` shipped, Flatpak bundle path working, Flathub submission pending |

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

## Reference

- Main build and run instructions: [README.md](README.md)
- Flatpak manifest: [flatpak/io.github.snapix.Snapix.yml](flatpak/io.github.snapix.Snapix.yml)
- Flathub submission notes: [FLATHUB.md](FLATHUB.md)
- AppStream metadata: [data/io.github.snapix.Snapix.metainfo.xml](data/io.github.snapix.Snapix.metainfo.xml)
