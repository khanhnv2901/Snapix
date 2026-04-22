# Snapix Progress

This file tracks the current product and release status at a high level.

## Current State

- Core screenshot capture is implemented for X11 and Wayland portal flows
- The GTK4/libadwaita editor is functional and used as the default app experience
- Annotation tools are available: crop, arrow, rectangle, ellipse, text, and blur
- Beautify controls are available: gradient, solid color, screenshot blur, padding, radius, and shadow
- Export flows are available: quick save, save as, copy to clipboard, PNG, and JPEG
- Local preferences and Pro activation flow are implemented

## Milestones

| Milestone | Status | Notes |
|-----------|--------|-------|
| M0 Foundation | ✅ Complete | Workspace, capture backends, CLI entry point, core models |
| M1 Wayland Polish | ✅ Complete | Portal-aware capture flow, desktop integration, Flatpak base |
| M2 Editor MVP | ✅ Complete | Main editor, annotation tools, export flows, undo/redo |
| M3 Beautify | ✅ Complete | Background/frame styling, presets, image reframe |
| M4 Packaging Prep | 🚧 In progress | Final release polish, Flathub packaging submission path |

## Current Packaging Notes

- The app metadata, desktop file, icon, and Flatpak manifest live in `data/` and `flatpak/`
- Quick Save now targets `~/Pictures/Screenshots` to match Flatpak sandbox permissions
- The main repo is kept lightweight for normal Cargo builds
- Flathub packaging should provide Cargo sources separately via `cargo-sources.json`

## Remaining M4 Work

- Final Flathub submission and review
- External distribution tasks such as landing page and payment setup
- Final QA on target Linux desktop environments

## Reference

- Main build and run instructions: [README.md](README.md)
- Flatpak manifest: [flatpak/io.github.snapix.Snapix.yml](flatpak/io.github.snapix.Snapix.yml)
- AppStream metadata: [data/io.github.snapix.Snapix.metainfo.xml](data/io.github.snapix.Snapix.metainfo.xml)
