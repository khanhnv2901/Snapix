# Multi-Screenshot Collage Plan

## Goal

Allow users to add multiple screenshots into the same canvas without replacing earlier captures, then arrange them with preset collage layouts such as:

- `Single`
- `Two Horizontal`
- `Two Vertical`
- `Quad`

This plan is intentionally scoped to a preset-based collage system, not a fully freeform layer editor.

## Product Intent

Current behavior:

- Each capture/import replaces `Document.base_image`

Target behavior:

- Each new capture/import appends a new screenshot into the current composition
- Existing screenshots remain visible
- Users can switch layout presets
- Users can select one image slot at a time
- Image-specific operations apply to the selected slot

## Recommended Scope

### Phase 1

- Add multiple screenshots to one document
- Support preset layouts
- Allow selecting an image slot
- Support per-image reframe for the selected slot
- Keep annotations at canvas level
- Make preview and export render the same multi-image composition

### Explicitly Out of Scope for Phase 1

- Arbitrary freeform image positioning
- Resizing images as independent draggable layers
- Layer z-order management
- Per-image annotation ownership
- Full generic layer system

## Why Not Refactor Directly to Freeform Layers

The current editor assumes a single `base_image` in several major areas:

- canvas render
- export render
- crop logic
- reframe logic
- blur background
- blur annotation sampling
- hit testing
- selection logic
- toolbar action enablement
- undo/redo snapshots

Switching immediately to a generic `Vec<ImageLayer>` model would force a broader architecture rewrite than the requested feature needs. A preset collage system delivers the user value with lower risk.

## Architecture Direction

Replace the single-image document model with a composition model centered on placed images plus a layout preset.

### Proposed Document Changes

Current:

```rust
pub struct Document {
    pub base_image: Option<Image>,
    // ...
}
```

Proposed direction:

```rust
pub struct Document {
    pub images: Vec<PlacedImage>,
    pub layout_preset: LayoutPreset,
    pub selected_image_index: Option<usize>,
    // existing background, frame, annotations, ratio, etc.
}
```

### Proposed Types

```rust
pub struct PlacedImage {
    pub image: Image,
    pub scale_mode: ImageScaleMode,
    pub anchor: ImageAnchor,
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: f32,
}
```

```rust
pub enum LayoutPreset {
    Single,
    TwoHorizontal,
    TwoVertical,
    Quad,
}
```

## Key Design Decisions

### 1. Annotation Ownership

Recommendation:

- Keep annotations at canvas level in Phase 1

Reason:

- Lowest-risk path
- Avoids rewriting annotation coordinates and selection behavior
- Avoids deciding whether annotations should follow an image when layout changes

Tradeoff:

- If layout changes significantly, existing annotations may no longer visually align with the same image semantics

### 2. Reframe Ownership

Recommendation:

- Move reframe settings from document-level to image-level

Reason:

- Each screenshot in a collage needs independent fit/fill/anchor/offset/zoom behavior

### 3. Crop Behavior

Recommendation:

- Crop only the selected image in Phase 1

Reason:

- Current crop flow assumes a single source image
- Cropping the full composition would be a different feature

### 4. Overflow Behavior

Recommendation:

- If all layout slots are full, do not silently replace
- Show a toast asking the user to switch to a larger layout

Optional later enhancement:

- Auto-upgrade layout to the next larger preset if a deterministic rule is defined

### 5. Blurred Screenshot Background

Recommendation for Phase 1:

- Use the first image in the composition as the blur source

Reason:

- Minimal refactor

Longer-term better option:

- Blur the whole rendered composition instead of one source image

## UX Plan

### Capture and Import

Toolbar actions remain:

- `Fullscreen`
- `Region`
- `Window`
- `Import`

Behavior change:

- Successful capture/import appends an image instead of replacing the current one

### Layout Selection

Add a new inspector section:

- `Layout`

Controls:

- preset buttons or segmented buttons for `1`, `2H`, `2V`, `4`

### Slot Selection

Canvas behavior:

- clicking a rendered image slot selects it
- selected slot shows a visual highlight
- image-specific controls apply to the selected slot

### Empty Slots

If a layout has more slots than images:

- show empty placeholder slots
- placeholder can read `Add screenshot`

### Removal

Current `Clear` action should be reconsidered.

Recommended split:

- `Remove Selected`
- `Clear All`

## Technical Backlog

### Task 1. Extend the Core Document Model

Files:

- `crates/snapix-core/src/canvas.rs`

Changes:

- replace `base_image: Option<Image>` with `images: Vec<PlacedImage>`
- add `layout_preset`
- add `selected_image_index`
- move image reframe properties to `PlacedImage`
- add helpers:
  - `has_images()`
  - `selected_image()`
  - `selected_image_mut()`
  - `slot_capacity()`
  - `can_append_image()`

Risk:

- High, because this is the root model change and will cascade everywhere

Definition of done:

- project compiles against the new document model
- no leftover direct usage of `base_image`

### Task 2. Add Layout Computation for Multiple Slots

Files:

- `crates/snapix-ui/src/widgets/geometry/layout.rs`
- `crates/snapix-ui/src/widgets/geometry/mod.rs`

Changes:

- add slot layout computation for each preset
- separate:
  - composition bounds
  - slot bounds
  - image-inside-slot bounds
- introduce helpers similar to:
  - `composition_slots(document, bounds)`
  - `slot_at_point(document, bounds, x, y)`
  - `layout_for_placed_image(image, slot_bounds)`

Risk:

- High, because current layout helpers are single-image oriented

Definition of done:

- slot rectangles are deterministic and test-covered
- selected slot can be resolved by pointer position

### Task 3. Render Multiple Images in Preview

Files:

- `crates/snapix-ui/src/widgets/render/canvas.rs`
- potentially `crates/snapix-ui/src/widgets/render/mod.rs`

Changes:

- keep background and frame behavior
- iterate visible slots
- render each `PlacedImage` into its slot bounds
- draw placeholder for empty slots
- draw selected slot highlight
- keep canvas-level annotations drawn last

Risk:

- Medium to high, especially where shadow and clipping assume one image region

Definition of done:

- preview shows all images in the active layout
- empty slots are visibly distinct
- selected slot is highlighted

### Task 4. Render Multiple Images in Export

Files:

- `crates/snapix-ui/src/widgets/render/export.rs`

Changes:

- ensure export uses the same multi-image composition rules as preview

Risk:

- Medium

Definition of done:

- exported PNG/JPEG contains all images and matches preview structure

### Task 5. Convert Capture and Import from Replace to Append

Files:

- `crates/snapix-ui/src/editor/actions.rs`
- `crates/snapix-ui/src/editor/state.rs`

Changes:

- replace `replace_base_image()` usage with append behavior
- add state operations:
  - `append_image(image)`
  - `replace_selected_image(image)` if needed later
- select the newly added image after append
- handle full-layout overflow with a toast

Risk:

- Medium

Definition of done:

- repeated screenshot captures no longer discard earlier images
- imported images behave the same as captured ones

### Task 6. Add Slot Selection Logic

Files:

- `crates/snapix-ui/src/widgets/canvas/click.rs`
- `crates/snapix-ui/src/widgets/canvas/drag.rs`
- `crates/snapix-ui/src/editor/state.rs`

Changes:

- detect which slot is clicked
- set `selected_image_index`
- ignore image-specific drags if no image is selected

Risk:

- Medium

Definition of done:

- clicking a slot changes the active image
- selection is stable and visually reflected

### Task 7. Port Reframe to Selected Image

Files:

- `crates/snapix-ui/src/editor/state.rs`
- `crates/snapix-ui/src/widgets/canvas/reframe.rs`
- `crates/snapix-ui/src/widgets/render/reframe.rs`
- geometry helpers currently used by reframe

Changes:

- reframe calculations use the selected image and its slot layout
- offset and zoom mutate only the selected `PlacedImage`
- overlay appears only over the selected slot

Risk:

- High, because current reframe math is tightly coupled to a single preview image region

Definition of done:

- reframe only affects the selected slot
- other slots remain unchanged

### Task 8. Add Layout Controls to Inspector

Files:

- `crates/snapix-ui/src/editor/ui/inspector/mod.rs`
- likely a new inspector layout section file

Changes:

- add a `Layout` section
- add buttons for supported presets
- add selected image status text, for example `Image 2 of 4`

Risk:

- Low to medium

Definition of done:

- user can switch layouts from the inspector
- current preset is reflected in UI state

### Task 9. Rebind Existing Image Controls to the Selected Image

Files:

- `crates/snapix-ui/src/editor/ui/inspector/frame.rs`
- any inspector helpers tied to image fit/anchor
- `crates/snapix-ui/src/editor/ui/helpers.rs`

Changes:

- image fit/fill, image anchor, and related controls update the selected image instead of document-global image settings
- disable those controls when no image is selected

Risk:

- Medium

Definition of done:

- changing fit/fill or anchor only affects the selected screenshot

### Task 10. Port Crop to Selected Image

Files:

- `crates/snapix-ui/src/widgets/geometry/crop.rs`
- `crates/snapix-ui/src/widgets/canvas/drag.rs`
- `crates/snapix-ui/src/editor/state.rs`

Changes:

- crop math should work against the selected image and selected slot bounds
- crop result replaces only the selected image's pixel data

Risk:

- High

Definition of done:

- crop works on one selected screenshot inside a collage

### Task 11. Audit Background Blur and Blur Annotation Behavior

Files:

- `crates/snapix-ui/src/widgets/render/annotations.rs`
- `crates/snapix-ui/src/widgets/render/canvas.rs`

Changes:

- redefine blur source assumptions that currently use one base image
- verify blur annotations still sample the correct pixels
- fall back conservatively if behavior is ambiguous in Phase 1

Risk:

- High, because blur currently depends on single-image assumptions

Definition of done:

- no broken blur rendering in multi-image documents

### Task 12. Cleanup Actions and Empty-State Rules

Files:

- `crates/snapix-ui/src/editor/actions.rs`
- `crates/snapix-ui/src/editor/ui/helpers.rs`
- toolbar-related UI files

Changes:

- replace `base_image.is_some()` checks with `!images.is_empty()`
- define button enabled states for:
  - no images
  - selected image
  - partially filled layout

Risk:

- Medium

Definition of done:

- tool availability and labels are consistent across empty and non-empty collages

## Module Risk Summary

Highest-risk areas:

- `crates/snapix-core/src/canvas.rs`
- `crates/snapix-ui/src/editor/state.rs`
- `crates/snapix-ui/src/widgets/geometry/layout.rs`
- `crates/snapix-ui/src/widgets/geometry/crop.rs`
- `crates/snapix-ui/src/widgets/canvas/reframe.rs`
- `crates/snapix-ui/src/widgets/render/canvas.rs`
- `crates/snapix-ui/src/widgets/render/annotations.rs`

Moderate-risk areas:

- `crates/snapix-ui/src/editor/actions.rs`
- `crates/snapix-ui/src/editor/ui/helpers.rs`
- `crates/snapix-ui/src/editor/ui/inspector/*`
- `crates/snapix-ui/src/widgets/canvas/click.rs`
- `crates/snapix-ui/src/widgets/canvas/drag.rs`

## Proposed Execution Order

1. Core document model
2. Slot layout geometry
3. Preview render for multiple images
4. Export render parity
5. Append capture/import behavior
6. Slot selection
7. Reframe for selected image
8. Inspector layout controls
9. Rebind image controls to selected image
10. Crop for selected image
11. Blur behavior audit
12. Cleanup and polish

## Testing Plan

### Unit Tests

- layout slot computation for each preset
- append behavior does not remove earlier images
- selected slot logic resolves correct index
- layout switching preserves image order

### Editor State Tests

- selecting a slot updates `selected_image_index`
- reframe modifies only the selected image
- removing selected image updates selection predictably
- clear all resets composition state

### Render/Export Tests

- preview/export parity for `Single`, `TwoHorizontal`, and `Quad`
- empty slot rendering does not crash
- multiple images appear in exported output

### Regression Tests

- fullscreen capture append behavior
- region capture append behavior
- window capture append behavior
- import append behavior
- blur background remains stable in multi-image mode

## Acceptance Criteria

The feature is ready for review when all of the following are true:

- taking a second screenshot no longer removes the first
- layout presets control how images are arranged
- the user can select a specific image slot
- reframe affects only the selected image
- preview and export both show the same collage layout
- existing annotation behavior still works at canvas level
- empty slots and full-layout overflow are handled clearly

## Future Extensions

After Phase 1 is stable, the next logical upgrades are:

- more layout presets
- automatic layout expansion
- per-image crop polish
- blur based on the full rendered composition
- drag-to-swap slot order
- full freeform image layers with resize and z-order
