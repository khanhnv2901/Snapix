# Snapix Signature Background Roadmap

## Current Status Snapshot

As of `2026-04-29`, the roadmap is largely implemented in code:

- `Background::Style` is implemented with shipped Signature presets including:
  `Blueprint`, `MidnightPanel`, `CutPaper`, `TerminalGlow`, `Redacted`, `WarningTape`
- The background inspector is family-based: `Clean`, `Signature`, and `Image`; `Clean` includes a `Mesh` sub-mode for Vibrant, Sunset, Candy, Aurora, Peach, and Lagoon mesh color effects
- Custom image backgrounds are implemented with async loading and cache pruning
- Signature render polish is implemented, including grain texture and per-style shadow tuning
- Screenshot composition can be repositioned inside the canvas and clipped to the composition frame

Remaining work is mostly QA and UX polish, not foundational implementation.

## Goal

Define a distinctive background system for Snapix so exported screenshots feel recognizably "Snapix" instead of looking like generic solid or gradient presets.

This document turns the visual idea into an implementation roadmap with concrete phases, technical scope, risks, and acceptance criteria.

## Product Boundary

This roadmap assumes Snapix remains a `screenshot-first composition tool`.

That means:

- the screenshot is always the primary subject
- background and framing systems exist to support the screenshot, not replace it
- annotation and layout features should stay lightweight and presentation-oriented

This roadmap does **not** imply that Snapix should evolve into:

- a general canvas editor
- a slide-design tool
- a whiteboard app
- a PowerPoint/Figma-style freeform composition surface

Boundary test for future work:

- `in scope`: faster screenshot storytelling, clearer emphasis, better framing, stronger export identity
- `out of scope`: features that create pressure for layers, guides, grouping, arbitrary scene building, or non-screenshot-centric workflows

## Product Intent

The current background options are functional, but they are not yet a brand signature. Snapix should offer at least one background family that:

- feels intentional and memorable
- works well for UI screenshots and code screenshots
- exports reliably at different canvas sizes
- stays lightweight enough for real-time preview
- can expand into multiple preset families later

The target is not "more gradients". The target is a reusable background system.

## Recommended Direction

Use a hybrid direction called `Editorial Tech`.

This combines:

- geometric boldness from neo-brutalism
- dark technical atmosphere from terminal/hacker aesthetics
- restrained texture and depth so screenshots remain the hero

This direction fits Snapix better than pure neo-brutalism or pure hacker styling because it can support both:

- clean product screenshots
- dev-facing screenshots with terminal or code UI

## Design Principles

The background system should follow these rules:

1. Screenshot first
The screenshot must remain the focal point. Decorative elements should frame it, not compete with it.

2. Structure over decoration
Backgrounds should rely on compositional motifs like panels, blocks, grid lines, arcs, bars, or cutouts, not only color blends.

3. Controlled contrast
Backgrounds can be visually distinctive, but text and UI inside the screenshot must still read clearly.

4. Predictable rendering
Every style must scale to different aspect ratios and export sizes without broken composition.

5. Preset identity
Each preset should feel like a named style, not just "another color".

## Proposed Background Families

The system should eventually support three families:

### 1. Clean

Purpose: simple, safe, general-purpose output.

Includes:

- solid colors
- standard gradients
- screenshot blur

This is the fallback/default family.

### 2. Signature

Purpose: branded Snapix look.

This should become the hero family.

Visual traits:

- dark or muted base surfaces
- one strong geometric anchor shape
- subtle panel lines or composition marks
- screenshot shadow tuned specifically for the style
- optional texture/noise at very low opacity

### 3. Atmosphere

Purpose: stronger stylistic looks for users who want personality.

Potential substyles:

- neo-brutalist
- dark terminal
- editorial paper
- cinematic panel

This family can arrive after Signature is stable.

Note: `Warning Tape` has already been pulled forward into the current Signature set.

## Candidate Signature Presets

These should be treated as art directions, not just palette entries.

### Blueprint

- Base: deep navy
- Motif: soft technical grid
- Accent: cyan panel or corner block
- Tone: precise, product-design, technical

### Midnight Panel

- Base: graphite black-blue
- Motif: inset panel lines and subtle radial lift
- Accent: faint electric blue edge glow
- Tone: premium tool UI

### Terminal Glow

- Base: green-black or slate-black
- Motif: scanline/noise + soft halo
- Accent: toxic green or amber markers
- Tone: code/dev focused

### Cut Paper

- Base: off-white, ink, or muted clay
- Motif: large cropped geometric sheet shapes
- Accent: bold shadow edge or hard offset
- Tone: editorial, bold, modern

### Warning Tape

- Base: warm yellow or sand
- Motif: black stripes or industrial bars
- Accent: hard shadow, rigid spacing
- Tone: brutalist, loud, memorable
- Status: implemented in the current Signature family

### Redacted

- Base: charcoal or steel
- Motif: horizontal bars and cropped blocks
- Accent: red markers or labels
- Tone: sharp, graphic, poster-like

## Rendering Model

The current background model is too simple for signature styles if it only supports:

- solid
- gradient
- image
- blurred screenshot

To support signature styles cleanly, add a new conceptual mode:

- `Background::Style { id, params }`

Suggested structure:

```rust
pub enum Background {
    Solid { color: Color },
    Gradient { from: Color, to: Color, angle_deg: f32 },
    BlurredScreenshot { radius: f32 },
    Image { path: PathBuf },
    Style {
        id: BackgroundStyleId,
        params: BackgroundStyleParams,
    },
}

pub enum BackgroundStyleId {
    Blueprint,
    MidnightPanel,
    TerminalGlow,
    CutPaper,
    WarningTape,
    Redacted,
}
```

`BackgroundStyleParams` can start minimal:

- base color
- accent color
- intensity
- texture amount

If parameterization feels premature, phase 1 can use fixed preset styles and postpone custom params.

## UI Proposal

The background picker should evolve from mode-based color controls into a family-based system.

Recommended top-level tabs:

- `Clean`
- `Signature`
- `Screenshot Blur`

Possible later expansion:

- `Atmosphere`

Inside each family:

- show style presets as visual cards/swatches
- show only controls relevant to that family
- hide irrelevant controls completely

Examples:

- `Clean > Solid`: color picker + solid presets
- `Clean > Gradient`: start/end/angle + gradient presets
- `Clean > Mesh`: mesh color-effect presets such as Vibrant, Sunset, Candy, Aurora, Peach, and Lagoon
- `Signature`: style cards + optional intensity/detail sliders
- `Screenshot Blur`: blur radius only

This avoids mixing incompatible UI controls in one panel.

## Technical Implementation Phases

### Phase 0. Definition and alignment

Goal: lock direction before coding visuals.

Tasks:

- choose the initial Signature family name and scope
- choose 3 first styles to build
- define palette and motif rules for each style
- decide whether phase 1 includes custom sliders or only fixed presets

Deliverable:

- approved style list with names, colors, and motif descriptions

### Phase 1. Internal style model

Goal: make the background system capable of nontrivial styles.

Tasks:

- extend `snapix-core` background enum or equivalent style representation
- add serialization support if presets are stored
- ensure undo/redo and saved preset flows understand style backgrounds
- update background equality helpers such as `same_background`

Files likely affected:

- `crates/snapix-core/src/canvas.rs`
- `crates/snapix-ui/src/editor/state.rs`
- `crates/snapix-ui/src/editor/presets.rs`

Acceptance criteria:

- app state can represent style-based backgrounds
- switching styles works with undo/redo
- saved presets can store and reload style backgrounds

Implementation status:

- Complete
- Presets also persist screenshot-card composition offsets used by the repositionable canvas layout

### Phase 2. Renderer foundation

Goal: support structured background drawing.

Tasks:

- add style rendering branch in canvas/background paint path
- create reusable drawing helpers for:
  - large blocks
  - panel borders
  - grids
  - stripes
  - arcs
  - noise overlay
- ensure rendering is resolution-independent

Files likely affected:

- `crates/snapix-ui/src/widgets/geometry/paint.rs`
- `crates/snapix-ui/src/widgets/render/canvas.rs`

Acceptance criteria:

- background style renders correctly in preview
- export output matches preview closely
- rendering cost remains acceptable during editor interaction

Implementation status:

- Complete
- Renderer now also supports async custom-image backgrounds and export-safe outer background corners

### Phase 3. First Signature presets

Goal: ship the first recognizable Snapix styles.

Recommended first batch:

- `Blueprint`
- `Midnight Panel`
- `Cut Paper`

Why these three:

- one technical/dark
- one premium/dark
- one brighter/editorial

Tasks:

- implement fixed composition rules for each preset
- tune screenshot shadow per style
- test across landscape and portrait canvas ratios

Acceptance criteria:

- presets are visually distinct from generic solid/gradient
- each preset looks balanced at common export sizes
- screenshot remains legible in all three styles

Implementation status:

- Complete, exceeded MVP scope with 6 shipped presets

### Phase 4. UI integration

Goal: make style backgrounds discoverable and coherent in the inspector.

Tasks:

- add family-level picker or segmented controls
- add Signature preset cards
- show/hide controls per family
- update selected-state behavior
- update saved preset naming/preview logic if needed

Files likely affected:

- `crates/snapix-ui/src/editor/ui/inspector/background.rs`
- `crates/snapix-ui/src/editor/ui/mod.rs`
- `crates/snapix-ui/src/app.rs`

Acceptance criteria:

- UI clearly separates clean presets from signature styles
- no conflicting controls appear for irrelevant modes
- selection state remains correct after mode switches and preset apply

Implementation status:

- Complete
- The shipping UI is family-based: `Clean`, `Signature`, `Image`

### Phase 5. Export polish and quality tuning

Goal: make signature styles production-ready.

Tasks:

- tune anti-aliasing and shape edges
- tune texture opacity to avoid muddy exports
- verify screenshots with light and dark UI
- verify with code screenshots, browser screenshots, and design screenshots
- ensure styles remain attractive at small and large padding values

Acceptance criteria:

- exported images look intentional, not noisy
- background texture does not band or distract
- no visible mismatch between preview and export

Implementation status:

- Mostly complete
- Grain caching, pixel-snapped motifs, and non-blocking custom-image loads are implemented
- Final QA is still recommended for preview/export parity and drag/reframe UX feel

### Phase 6. Atmosphere expansion

Goal: add more aggressive stylistic families after Signature is stable.

Possible additions:

- `Terminal Glow`
- `Warning Tape`
- `Redacted`

This phase should only start after the Signature family feels solid.

Implementation note:

- `Warning Tape` is no longer deferred; it is already part of the current Signature implementation.

## Recommended MVP Scope

Do not build the full style engine all at once.

Recommended MVP:

- add one new `Signature` mode
- implement 3 fixed style presets
- no advanced per-style editor yet
- allow save/apply via existing preset infrastructure

This keeps the scope realistic while still shipping something visibly different.

## Risks

### 1. Over-decorating the canvas

Risk:
Background becomes the main subject instead of the screenshot.

Mitigation:

- cap motif opacity
- test with dense screenshots
- use one major anchor shape, not many

### 2. Renderer complexity

Risk:
Too many special cases create messy paint logic.

Mitigation:

- add style helper functions
- keep each style composed from reusable primitives
- avoid style-specific ad hoc code where possible

### 3. Export mismatch

Risk:
Preview and exported output diverge.

Mitigation:

- use shared drawing logic where possible
- test preview/export parity early in phase 2

### 4. Preset sprawl

Risk:
Too many styles dilute the identity.

Mitigation:

- ship 3 strong styles first
- remove weak or redundant presets aggressively

## Suggested Order of Work

1. Approve the `Editorial Tech` direction
2. Pick the first 3 Signature styles
3. Add the internal background style model
4. Implement shared rendering primitives
5. Build the first 3 style renders
6. Integrate Signature into inspector UI
7. Tune preview/export quality

## Open Decisions

These need a product call before implementation starts:

1. Should `Signature` be a new top-level mode, or a subsection under Background?
2. Should the first release expose style parameters, or only fixed presets?
3. Should saved presets show style names only, or later include thumbnails?
4. Should the default new-document background remain gradient, or move to a Signature preset?

## Recommendation

For the first implementation:

- create a new `Signature` mode
- ship 3 fixed presets: `Blueprint`, `Midnight Panel`, `Cut Paper`
- do not expose advanced sliders yet
- keep `Clean` as the default for safety until Signature quality is proven

This gives Snapix a clear visual identity without turning the first iteration into an overbuilt rendering system.
