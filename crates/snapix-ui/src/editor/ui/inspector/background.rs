use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gdk;
use gtk4::prelude::*;
use snapix_core::canvas::{Background, BackgroundStyleId, Color};

use super::super::helpers::{
    configure_inspector_slider, refresh_history_buttons, refresh_subtitle,
};
use super::labeled_row_with_value;
use crate::editor::i18n;
use crate::editor::state::{same_background, EditorState};
use crate::widgets::{paint_signature_preview_thumbnail, DocumentCanvas};

type BackgroundSwatchButtons = Rc<RefCell<Vec<(Background, gtk4::Button)>>>;
type BackgroundModeControls = (
    gtk4::Button,
    gtk4::Button,
    gtk4::Button,
    gtk4::Button,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
);
type BackgroundEditorControls = (
    gtk4::ColorButton,
    gtk4::ColorButton,
    gtk4::ColorButton,
    gtk4::Scale,
    gtk4::Label,
    gtk4::Scale,
    gtk4::Label,
    gtk4::Scale,
    gtk4::Label,
);

#[derive(Clone)]
pub(super) struct BackgroundSection {
    pub(super) swatch_buttons: BackgroundSwatchButtons,
    pub(super) presets_label: gtk4::Widget,
    pub(super) presets_grid: gtk4::Widget,
    pub(super) signature_presets_grid: gtk4::Widget,
    pub(super) gradient_button: gtk4::Button,
    pub(super) solid_button: gtk4::Button,
    pub(super) signature_button: gtk4::Button,
    pub(super) blur_button: gtk4::Button,
    pub(super) solid_color_button: gtk4::ColorButton,
    pub(super) solid_row: gtk4::Widget,
    pub(super) gradient_from_button: gtk4::ColorButton,
    pub(super) gradient_to_button: gtk4::ColorButton,
    pub(super) gradient_from_row: gtk4::Widget,
    pub(super) gradient_to_row: gtk4::Widget,
    pub(super) gradient_angle_scale: gtk4::Scale,
    pub(super) gradient_angle_value: gtk4::Label,
    pub(super) gradient_angle_row: gtk4::Widget,
    pub(super) blur_radius_scale: gtk4::Scale,
    pub(super) blur_radius_value: gtk4::Label,
    pub(super) blur_row: gtk4::Widget,
    pub(super) signature_intensity_scale: gtk4::Scale,
    pub(super) signature_intensity_value: gtk4::Label,
    pub(super) signature_intensity_row: gtk4::Widget,
    pub(super) suppress_sync_events: Rc<Cell<bool>>,
}

pub(super) fn build_background_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> BackgroundSection {
    panel.append(
        &gtk4::Label::builder()
            .label(i18n::inspector_background_title())
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .build(),
    );

    let current_background = state.borrow().document.background.clone();
    let suppress_sync_events = Rc::new(Cell::new(false));
    let swatch_buttons: BackgroundSwatchButtons = Rc::new(RefCell::new(Vec::new()));

    let mode_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let gradient_button = gtk4::Button::builder()
        .label(i18n::inspector_background_mode_gradient())
        .hexpand(true)
        .build();
    let solid_button = gtk4::Button::builder()
        .label(i18n::inspector_background_mode_solid())
        .hexpand(true)
        .build();
    let signature_button = gtk4::Button::builder()
        .label(i18n::inspector_background_mode_signature())
        .hexpand(true)
        .build();
    let blur_button = gtk4::Button::builder()
        .label(i18n::inspector_background_mode_blur())
        .tooltip_text(i18n::inspector_background_blur_tooltip())
        .hexpand(true)
        .build();
    for button in [&gradient_button, &solid_button, &signature_button, &blur_button] {
        button.add_css_class("ratio-btn");
        mode_row.append(button);
    }
    panel.append(&mode_row);

    let solid_color_button =
        gtk4::ColorButton::with_rgba(&rgba_from_color(&extract_solid_color(&current_background)));
    #[allow(deprecated)]
    solid_color_button.set_title(i18n::inspector_pick_background_color());
    solid_color_button.set_show_editor(true);
    solid_color_button.set_hexpand(true);
    let gradient_from_button = gtk4::ColorButton::with_rgba(&rgba_from_color(
        &extract_gradient_from(&current_background),
    ));
    #[allow(deprecated)]
    gradient_from_button.set_title(i18n::inspector_pick_gradient_start());
    gradient_from_button.set_show_editor(true);
    gradient_from_button.set_hexpand(true);
    let gradient_to_button =
        gtk4::ColorButton::with_rgba(&rgba_from_color(&extract_gradient_to(&current_background)));
    #[allow(deprecated)]
    gradient_to_button.set_title(i18n::inspector_pick_gradient_end());
    gradient_to_button.set_show_editor(true);
    gradient_to_button.set_hexpand(true);

    let solid_row = labeled_row_with_value(
        i18n::inspector_solid_color_label(),
        &solid_color_button,
        &gtk4::Label::builder().label("").build(),
    );

    let gradient_from_row = labeled_row_with_value(
        i18n::inspector_gradient_from_label(),
        &gradient_from_button,
        &gtk4::Label::builder().label("").build(),
    );
    let gradient_to_row = labeled_row_with_value(
        i18n::inspector_gradient_to_label(),
        &gradient_to_button,
        &gtk4::Label::builder().label("").build(),
    );

    let current_gradient_angle = match &current_background {
        Background::Gradient { angle_deg, .. } => *angle_deg,
        _ => 135.0,
    };
    let gradient_angle_value = gtk4::Label::builder()
        .label(format!("{}°", current_gradient_angle.round() as i32))
        .css_classes(["dim-copy"])
        .build();
    let gradient_angle_scale =
        gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 360.0, 1.0);
    gradient_angle_scale.set_value(current_gradient_angle as f64);
    configure_inspector_slider(&gradient_angle_scale);
    let gradient_angle_row = labeled_row_with_value(
        i18n::inspector_gradient_angle_label(),
        &gradient_angle_scale,
        &gradient_angle_value,
    );

    let current_blur_radius = match &current_background {
        Background::BlurredScreenshot { radius } => *radius,
        _ => 24.0,
    };
    let blur_radius_value = gtk4::Label::builder()
        .label(format!("{}px", current_blur_radius.round() as u32))
        .css_classes(["dim-copy"])
        .build();
    let blur_radius_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 4.0, 64.0, 1.0);
    blur_radius_scale.set_value(current_blur_radius as f64);
    configure_inspector_slider(&blur_radius_scale);
    let blur_row = labeled_row_with_value(
        i18n::inspector_blur_radius_label(),
        &blur_radius_scale,
        &blur_radius_value,
    );

    let current_signature_intensity = match &current_background {
        Background::Style { intensity, .. } => *intensity,
        _ => 0.65,
    };
    let signature_intensity_value = gtk4::Label::builder()
        .label(format!("{}%", (current_signature_intensity * 100.0).round() as u32))
        .css_classes(["dim-copy"])
        .build();
    let signature_intensity_scale =
        gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.2, 1.0, 0.05);
    signature_intensity_scale.set_value(current_signature_intensity as f64);
    configure_inspector_slider(&signature_intensity_scale);
    let signature_intensity_row = labeled_row_with_value(
        i18n::inspector_signature_intensity_label(),
        &signature_intensity_scale,
        &signature_intensity_value,
    );

    panel.append(&solid_row);
    panel.append(&gradient_from_row);
    panel.append(&gradient_to_row);
    panel.append(&gradient_angle_row);
    panel.append(&blur_row);
    panel.append(&signature_intensity_row);

    refresh_background_mode_controls(
        &current_background,
        &gradient_button,
        &solid_button,
        &signature_button,
        &blur_button,
        &solid_row,
        &gradient_from_row,
        &gradient_to_row,
        &gradient_angle_row,
        &blur_row,
        &signature_intensity_row,
    );
    sync_background_editor_values(
        &current_background,
        &solid_color_button,
        &gradient_from_button,
        &gradient_to_button,
        &gradient_angle_scale,
        &gradient_angle_value,
        &blur_radius_scale,
        &blur_radius_value,
        &signature_intensity_scale,
        &signature_intensity_value,
        &suppress_sync_events,
    );

    let presets: Vec<(&str, &str, Background)> = vec![
        (
            "Cornflower",
            "swatch-cornflower",
            Background::Gradient {
                from: Color {
                    r: 110,
                    g: 162,
                    b: 255,
                    a: 255,
                },
                to: Color {
                    r: 130,
                    g: 99,
                    b: 245,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Sunset",
            "swatch-sunset",
            Background::Gradient {
                from: Color {
                    r: 255,
                    g: 180,
                    b: 108,
                    a: 255,
                },
                to: Color {
                    r: 232,
                    g: 93,
                    b: 68,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Ocean",
            "swatch-ocean",
            Background::Gradient {
                from: Color {
                    r: 56,
                    g: 189,
                    b: 248,
                    a: 255,
                },
                to: Color {
                    r: 15,
                    g: 118,
                    b: 110,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Forest",
            "swatch-forest",
            Background::Gradient {
                from: Color {
                    r: 74,
                    g: 222,
                    b: 128,
                    a: 255,
                },
                to: Color {
                    r: 21,
                    g: 128,
                    b: 61,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Rose",
            "swatch-rose",
            Background::Gradient {
                from: Color {
                    r: 249,
                    g: 168,
                    b: 212,
                    a: 255,
                },
                to: Color {
                    r: 190,
                    g: 24,
                    b: 93,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Midnight",
            "swatch-midnight",
            Background::Gradient {
                from: Color {
                    r: 99,
                    g: 102,
                    b: 241,
                    a: 255,
                },
                to: Color {
                    r: 30,
                    g: 27,
                    b: 75,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Golden",
            "swatch-golden",
            Background::Gradient {
                from: Color {
                    r: 251,
                    g: 191,
                    b: 36,
                    a: 255,
                },
                to: Color {
                    r: 180,
                    g: 83,
                    b: 9,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Lavender",
            "swatch-lavender",
            Background::Gradient {
                from: Color {
                    r: 196,
                    g: 181,
                    b: 253,
                    a: 255,
                },
                to: Color {
                    r: 124,
                    g: 58,
                    b: 237,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Mint",
            "swatch-mint",
            Background::Gradient {
                from: Color {
                    r: 110,
                    g: 231,
                    b: 183,
                    a: 255,
                },
                to: Color {
                    r: 13,
                    g: 148,
                    b: 136,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Slate",
            "swatch-slate",
            Background::Solid {
                color: Color {
                    r: 31,
                    g: 36,
                    b: 45,
                    a: 255,
                },
            },
        ),
        (
            "Charcoal",
            "swatch-charcoal",
            Background::Solid {
                color: Color {
                    r: 45,
                    g: 55,
                    b: 72,
                    a: 255,
                },
            },
        ),
        (
            "Steel",
            "swatch-steel",
            Background::Solid {
                color: Color {
                    r: 71,
                    g: 85,
                    b: 105,
                    a: 255,
                },
            },
        ),
        (
            "Mist",
            "swatch-mist",
            Background::Solid {
                color: Color {
                    r: 226,
                    g: 232,
                    b: 240,
                    a: 255,
                },
            },
        ),
        (
            "Sky",
            "swatch-sky",
            Background::Solid {
                color: Color {
                    r: 56,
                    g: 189,
                    b: 248,
                    a: 255,
                },
            },
        ),
        (
            "Emerald",
            "swatch-emerald",
            Background::Solid {
                color: Color {
                    r: 16,
                    g: 185,
                    b: 129,
                    a: 255,
                },
            },
        ),
        (
            "Coral",
            "swatch-coral",
            Background::Solid {
                color: Color {
                    r: 251,
                    g: 113,
                    b: 133,
                    a: 255,
                },
            },
        ),
        (
            "Amber",
            "swatch-amber",
            Background::Solid {
                color: Color {
                    r: 245,
                    g: 158,
                    b: 11,
                    a: 255,
                },
            },
        ),
        (
            "Violet",
            "swatch-violet",
            Background::Solid {
                color: Color {
                    r: 139,
                    g: 92,
                    b: 246,
                    a: 255,
                },
            },
        ),
        (
            "Deep Space",
            "swatch-deepspace",
            Background::Gradient {
                from: Color {
                    r: 26,
                    g: 26,
                    b: 46,
                    a: 255,
                },
                to: Color {
                    r: 22,
                    g: 33,
                    b: 62,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Aurora",
            "swatch-aurora",
            Background::Gradient {
                from: Color {
                    r: 34,
                    g: 211,
                    b: 238,
                    a: 255,
                },
                to: Color {
                    r: 16,
                    g: 185,
                    b: 129,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Blueprint",
            "swatch-blueprint",
            Background::Style {
                id: BackgroundStyleId::Blueprint,
                intensity: 0.65,
            },
        ),
        (
            "Midnight Panel",
            "swatch-midnightpanel",
            Background::Style {
                id: BackgroundStyleId::MidnightPanel,
                intensity: 0.65,
            },
        ),
        (
            "Cut Paper",
            "swatch-cutpaper",
            Background::Style {
                id: BackgroundStyleId::CutPaper,
                intensity: 0.65,
            },
        ),
        (
            "Terminal Glow",
            "swatch-terminalglow",
            Background::Style {
                id: BackgroundStyleId::TerminalGlow,
                intensity: 0.65,
            },
        ),
        (
            "Redacted",
            "swatch-redacted",
            Background::Style {
                id: BackgroundStyleId::Redacted,
                intensity: 0.65,
            },
        ),
    ];

    let presets_label = gtk4::Label::builder()
        .label("Presets")
        .xalign(0.0)
        .css_classes(["dim-copy"])
        .margin_top(4)
        .build();

    let swatch_grid = gtk4::Grid::builder()
        .row_spacing(6)
        .column_spacing(6)
        .build();
    let signature_swatch_grid = gtk4::Grid::builder()
        .row_spacing(6)
        .column_spacing(6)
        .build();
    let presets_stack = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .build();
    presets_stack.append(&swatch_grid);
    presets_stack.append(&signature_swatch_grid);

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let gradient_button_for_handler = gradient_button.clone();
        let solid_button_for_handler = solid_button.clone();
        let signature_button_for_handler = signature_button.clone();
        let blur_button_for_handler = blur_button.clone();
        let solid_row_for_handler = solid_row.clone();
        let gradient_from_row_for_handler = gradient_from_row.clone();
        let gradient_to_row_for_handler = gradient_to_row.clone();
        let gradient_angle_row_for_handler = gradient_angle_row.clone();
        let blur_row_for_handler = blur_row.clone();
        let signature_intensity_row_for_handler = signature_intensity_row.clone();
        let swatch_buttons_for_handler = swatch_buttons.clone();
        let solid_color_button_for_handler = solid_color_button.clone();
        let gradient_from_button_for_handler = gradient_from_button.clone();
        let gradient_to_button_for_handler = gradient_to_button.clone();
        let gradient_angle_scale_for_handler = gradient_angle_scale.clone();
        let gradient_angle_value_for_handler = gradient_angle_value.clone();
        let blur_radius_scale_for_handler = blur_radius_scale.clone();
        let blur_radius_value_for_handler = blur_radius_value.clone();
        let signature_intensity_scale_for_handler = signature_intensity_scale.clone();
        let signature_intensity_value_for_handler = signature_intensity_value.clone();
        let suppress_sync_events_for_handler = suppress_sync_events.clone();
        let presets_label_for_handler: gtk4::Widget = presets_label.clone().upcast();
        let presets_grid_for_handler: gtk4::Widget = swatch_grid.clone().upcast();
        let signature_presets_grid_for_handler: gtk4::Widget =
            signature_swatch_grid.clone().upcast();
        gradient_button.connect_clicked(move |_| {
            let next_background = match &state.borrow().document().background {
                Background::Gradient {
                    from,
                    to,
                    angle_deg,
                } => Background::Gradient {
                    from: from.clone(),
                    to: to.clone(),
                    angle_deg: *angle_deg,
                },
                Background::Solid { color } => Background::Gradient {
                    from: color.clone(),
                    to: Color {
                        r: 130,
                        g: 99,
                        b: 245,
                        a: 255,
                    },
                    angle_deg: 135.0,
                },
                _ => Background::Gradient {
                    from: Color {
                        r: 110,
                        g: 162,
                        b: 255,
                        a: 255,
                    },
                    to: Color {
                        r: 130,
                        g: 99,
                        b: 245,
                        a: 255,
                    },
                    angle_deg: 135.0,
                },
            };
            apply_background_change(
                state.clone(),
                canvas.clone(),
                &subtitle_label,
                &undo_button,
                &redo_button,
                next_background,
                Some((
                    gradient_button_for_handler.clone(),
                    solid_button_for_handler.clone(),
                    signature_button_for_handler.clone(),
                    blur_button_for_handler.clone(),
                    solid_row_for_handler.clone(),
                    gradient_from_row_for_handler.clone(),
                    gradient_to_row_for_handler.clone(),
                    gradient_angle_row_for_handler.clone(),
                    blur_row_for_handler.clone(),
                    signature_intensity_row_for_handler.clone(),
                )),
                Some((
                    solid_color_button_for_handler.clone(),
                    gradient_from_button_for_handler.clone(),
                    gradient_to_button_for_handler.clone(),
                    gradient_angle_scale_for_handler.clone(),
                    gradient_angle_value_for_handler.clone(),
                    blur_radius_scale_for_handler.clone(),
                    blur_radius_value_for_handler.clone(),
                    signature_intensity_scale_for_handler.clone(),
                    signature_intensity_value_for_handler.clone(),
                )),
                Some(suppress_sync_events_for_handler.clone()),
                Some(swatch_buttons_for_handler.clone()),
                Some((
                    presets_label_for_handler.clone(),
                    presets_grid_for_handler.clone(),
                    signature_presets_grid_for_handler.clone(),
                )),
            );
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let gradient_button_for_handler = gradient_button.clone();
        let solid_button_for_handler = solid_button.clone();
        let signature_button_for_handler = signature_button.clone();
        let blur_button_for_handler = blur_button.clone();
        let solid_row_for_handler = solid_row.clone();
        let gradient_from_row_for_handler = gradient_from_row.clone();
        let gradient_to_row_for_handler = gradient_to_row.clone();
        let gradient_angle_row_for_handler = gradient_angle_row.clone();
        let blur_row_for_handler = blur_row.clone();
        let signature_intensity_row_for_handler = signature_intensity_row.clone();
        let swatch_buttons_for_handler = swatch_buttons.clone();
        let solid_color_button_for_handler = solid_color_button.clone();
        let gradient_from_button_for_handler = gradient_from_button.clone();
        let gradient_to_button_for_handler = gradient_to_button.clone();
        let gradient_angle_scale_for_handler = gradient_angle_scale.clone();
        let gradient_angle_value_for_handler = gradient_angle_value.clone();
        let blur_radius_scale_for_handler = blur_radius_scale.clone();
        let blur_radius_value_for_handler = blur_radius_value.clone();
        let signature_intensity_scale_for_handler = signature_intensity_scale.clone();
        let signature_intensity_value_for_handler = signature_intensity_value.clone();
        let suppress_sync_events_for_handler = suppress_sync_events.clone();
        let presets_label_for_handler: gtk4::Widget = presets_label.clone().upcast();
        let presets_grid_for_handler: gtk4::Widget = swatch_grid.clone().upcast();
        let signature_presets_grid_for_handler: gtk4::Widget =
            signature_swatch_grid.clone().upcast();
        signature_button.connect_clicked(move |_| {
            let next_background = match &state.borrow().document().background {
                Background::Style { id, intensity } => Background::Style {
                    id: *id,
                    intensity: *intensity,
                },
                _ => Background::Style {
                    id: BackgroundStyleId::Blueprint,
                    intensity: 0.65,
                },
            };
            apply_background_change(
                state.clone(),
                canvas.clone(),
                &subtitle_label,
                &undo_button,
                &redo_button,
                next_background,
                Some((
                    gradient_button_for_handler.clone(),
                    solid_button_for_handler.clone(),
                    signature_button_for_handler.clone(),
                    blur_button_for_handler.clone(),
                    solid_row_for_handler.clone(),
                    gradient_from_row_for_handler.clone(),
                    gradient_to_row_for_handler.clone(),
                    gradient_angle_row_for_handler.clone(),
                    blur_row_for_handler.clone(),
                    signature_intensity_row_for_handler.clone(),
                )),
                Some((
                    solid_color_button_for_handler.clone(),
                    gradient_from_button_for_handler.clone(),
                    gradient_to_button_for_handler.clone(),
                    gradient_angle_scale_for_handler.clone(),
                    gradient_angle_value_for_handler.clone(),
                    blur_radius_scale_for_handler.clone(),
                    blur_radius_value_for_handler.clone(),
                    signature_intensity_scale_for_handler.clone(),
                    signature_intensity_value_for_handler.clone(),
                )),
                Some(suppress_sync_events_for_handler.clone()),
                Some(swatch_buttons_for_handler.clone()),
                Some((
                    presets_label_for_handler.clone(),
                    presets_grid_for_handler.clone(),
                    signature_presets_grid_for_handler.clone(),
                )),
            );
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let gradient_button_for_handler = gradient_button.clone();
        let solid_button_for_handler = solid_button.clone();
        let signature_button_for_handler = signature_button.clone();
        let blur_button_for_handler = blur_button.clone();
        let solid_row_for_handler = solid_row.clone();
        let gradient_from_row_for_handler = gradient_from_row.clone();
        let gradient_to_row_for_handler = gradient_to_row.clone();
        let gradient_angle_row_for_handler = gradient_angle_row.clone();
        let blur_row_for_handler = blur_row.clone();
        let signature_intensity_row_for_handler = signature_intensity_row.clone();
        let swatch_buttons_for_handler = swatch_buttons.clone();
        let solid_color_button_for_handler = solid_color_button.clone();
        let gradient_from_button_for_handler = gradient_from_button.clone();
        let gradient_to_button_for_handler = gradient_to_button.clone();
        let gradient_angle_scale_for_handler = gradient_angle_scale.clone();
        let gradient_angle_value_for_handler = gradient_angle_value.clone();
        let blur_radius_scale_for_handler = blur_radius_scale.clone();
        let blur_radius_value_for_handler = blur_radius_value.clone();
        let signature_intensity_scale_for_handler = signature_intensity_scale.clone();
        let signature_intensity_value_for_handler = signature_intensity_value.clone();
        let suppress_sync_events_for_handler = suppress_sync_events.clone();
        let presets_label_for_handler: gtk4::Widget = presets_label.clone().upcast();
        let presets_grid_for_handler: gtk4::Widget = swatch_grid.clone().upcast();
        let signature_presets_grid_for_handler: gtk4::Widget =
            signature_swatch_grid.clone().upcast();
        solid_button.connect_clicked(move |_| {
            let next_background = match &state.borrow().document().background {
                Background::Solid { color } => Background::Solid {
                    color: color.clone(),
                },
                Background::Gradient { from, .. } => Background::Solid {
                    color: from.clone(),
                },
                _ => Background::Solid {
                    color: Color {
                        r: 31,
                        g: 36,
                        b: 45,
                        a: 255,
                    },
                },
            };
            apply_background_change(
                state.clone(),
                canvas.clone(),
                &subtitle_label,
                &undo_button,
                &redo_button,
                next_background,
                Some((
                    gradient_button_for_handler.clone(),
                    solid_button_for_handler.clone(),
                    signature_button_for_handler.clone(),
                    blur_button_for_handler.clone(),
                    solid_row_for_handler.clone(),
                    gradient_from_row_for_handler.clone(),
                    gradient_to_row_for_handler.clone(),
                    gradient_angle_row_for_handler.clone(),
                    blur_row_for_handler.clone(),
                    signature_intensity_row_for_handler.clone(),
                )),
                Some((
                    solid_color_button_for_handler.clone(),
                    gradient_from_button_for_handler.clone(),
                    gradient_to_button_for_handler.clone(),
                    gradient_angle_scale_for_handler.clone(),
                    gradient_angle_value_for_handler.clone(),
                    blur_radius_scale_for_handler.clone(),
                    blur_radius_value_for_handler.clone(),
                    signature_intensity_scale_for_handler.clone(),
                    signature_intensity_value_for_handler.clone(),
                )),
                Some(suppress_sync_events_for_handler.clone()),
                Some(swatch_buttons_for_handler.clone()),
                Some((
                    presets_label_for_handler.clone(),
                    presets_grid_for_handler.clone(),
                    signature_presets_grid_for_handler.clone(),
                )),
            );
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let gradient_button_for_handler = gradient_button.clone();
        let solid_button_for_handler = solid_button.clone();
        let signature_button_for_handler = signature_button.clone();
        let blur_button_for_handler = blur_button.clone();
        let solid_row_for_handler = solid_row.clone();
        let gradient_from_row_for_handler = gradient_from_row.clone();
        let gradient_to_row_for_handler = gradient_to_row.clone();
        let gradient_angle_row_for_handler = gradient_angle_row.clone();
        let blur_row_for_handler = blur_row.clone();
        let signature_intensity_row_for_handler = signature_intensity_row.clone();
        let swatch_buttons_for_handler = swatch_buttons.clone();
        let solid_color_button_for_handler = solid_color_button.clone();
        let gradient_from_button_for_handler = gradient_from_button.clone();
        let gradient_to_button_for_handler = gradient_to_button.clone();
        let gradient_angle_scale_for_handler = gradient_angle_scale.clone();
        let gradient_angle_value_for_handler = gradient_angle_value.clone();
        let blur_radius_scale_for_handler = blur_radius_scale.clone();
        let blur_radius_value_for_handler = blur_radius_value.clone();
        let signature_intensity_scale_for_handler = signature_intensity_scale.clone();
        let signature_intensity_value_for_handler = signature_intensity_value.clone();
        let suppress_sync_events_for_handler = suppress_sync_events.clone();
        let presets_label_for_handler: gtk4::Widget = presets_label.clone().upcast();
        let presets_grid_for_handler: gtk4::Widget = swatch_grid.clone().upcast();
        let signature_presets_grid_for_handler: gtk4::Widget =
            signature_swatch_grid.clone().upcast();
        blur_button.connect_clicked(move |_| {
            let radius = match &state.borrow().document().background {
                Background::BlurredScreenshot { radius } => *radius,
                _ => 24.0,
            };
            apply_background_change(
                state.clone(),
                canvas.clone(),
                &subtitle_label,
                &undo_button,
                &redo_button,
                Background::BlurredScreenshot { radius },
                Some((
                    gradient_button_for_handler.clone(),
                    solid_button_for_handler.clone(),
                    signature_button_for_handler.clone(),
                    blur_button_for_handler.clone(),
                    solid_row_for_handler.clone(),
                    gradient_from_row_for_handler.clone(),
                    gradient_to_row_for_handler.clone(),
                    gradient_angle_row_for_handler.clone(),
                    blur_row_for_handler.clone(),
                    signature_intensity_row_for_handler.clone(),
                )),
                Some((
                    solid_color_button_for_handler.clone(),
                    gradient_from_button_for_handler.clone(),
                    gradient_to_button_for_handler.clone(),
                    gradient_angle_scale_for_handler.clone(),
                    gradient_angle_value_for_handler.clone(),
                    blur_radius_scale_for_handler.clone(),
                    blur_radius_value_for_handler.clone(),
                    signature_intensity_scale_for_handler.clone(),
                    signature_intensity_value_for_handler.clone(),
                )),
                Some(suppress_sync_events_for_handler.clone()),
                Some(swatch_buttons_for_handler.clone()),
                Some((
                    presets_label_for_handler.clone(),
                    presets_grid_for_handler.clone(),
                    signature_presets_grid_for_handler.clone(),
                )),
            );
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        solid_color_button.connect_color_set(move |button| {
            let color = color_from_rgba(&button.rgba());
            let mut state = state.borrow_mut();
            if state.update_document(|doc| match &mut doc.background {
                Background::Solid { color: current } => *current = color.clone(),
                _ => {
                    doc.background = Background::Solid {
                        color: color.clone(),
                    }
                }
            }) {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        gradient_from_button.connect_color_set(move |button| {
            let color = color_from_rgba(&button.rgba());
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                if let Background::Gradient { from, .. } = &mut doc.background {
                    *from = color.clone();
                }
            }) {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        gradient_to_button.connect_color_set(move |button| {
            let color = color_from_rgba(&button.rgba());
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                if let Background::Gradient { to, .. } = &mut doc.background {
                    *to = color.clone();
                }
            }) {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let value_label = gradient_angle_value.clone();
        let suppress_sync_events = suppress_sync_events.clone();
        gradient_angle_scale.connect_value_changed(move |scale| {
            if suppress_sync_events.get() {
                return;
            }
            let angle = scale.value() as f32;
            value_label.set_label(&format!("{}°", angle.round() as i32));
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                if let Background::Gradient { angle_deg, .. } = &mut doc.background {
                    *angle_deg = angle;
                }
            }) {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let value_label = blur_radius_value.clone();
        let pending_radius = Rc::new(Cell::new(current_blur_radius));
        let change_generation = Rc::new(Cell::new(0u64));
        let suppress_sync_events = suppress_sync_events.clone();
        blur_radius_scale.connect_value_changed(move |scale| {
            if suppress_sync_events.get() {
                return;
            }
            let radius = scale.value() as f32;
            value_label.set_label(&format!("{}px", radius.round() as u32));
            pending_radius.set(radius);

            let generation = change_generation.get().wrapping_add(1);
            change_generation.set(generation);

            let state = state.clone();
            let canvas = canvas.clone();
            let subtitle_label = subtitle_label.clone();
            let undo_button = undo_button.clone();
            let redo_button = redo_button.clone();
            let pending_radius = pending_radius.clone();
            let change_generation = change_generation.clone();
            gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(140), move || {
                if change_generation.get() != generation {
                    return;
                }

                let radius = pending_radius.get();
                let mut state = state.borrow_mut();
                if state.update_document(|doc| {
                    if let Background::BlurredScreenshot { radius: current } = &mut doc.background {
                        *current = radius;
                    }
                }) {
                    refresh_subtitle(&state, &subtitle_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    canvas.refresh();
                }
            });
        });
    }

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let value_label = signature_intensity_value.clone();
        let suppress_sync_events = suppress_sync_events.clone();
        signature_intensity_scale.connect_value_changed(move |scale| {
            if suppress_sync_events.get() {
                return;
            }
            let intensity = scale.value() as f32;
            value_label.set_label(&format!("{}%", (intensity * 100.0).round() as u32));
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                if let Background::Style {
                    intensity: current, ..
                } = &mut doc.background
                {
                    *current = intensity;
                }
            }) {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }

    panel.append(&presets_label);

    let mut clean_index = 0usize;
    let mut signature_index = 0usize;
    for (label, css_class, background) in presets.into_iter() {
        let is_signature_preset = matches!(background, Background::Style { .. });
        let button = gtk4::Button::builder()
            .tooltip_text(label)
            .hexpand(true)
            .vexpand(false)
            .build();
        button.add_css_class("background-swatch");
        if matches!(background, Background::Style { .. }) {
            button.add_css_class("background-swatch-signature");
            let preview = build_signature_preview_card(label, &background);
            button.set_child(Some(&preview));
        }
        button.add_css_class(css_class);
        if same_background(&current_background, &background) {
            button.add_css_class("selected");
        }

        swatch_buttons
            .borrow_mut()
            .push((background.clone(), button.clone()));
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let all_buttons = swatch_buttons.clone();
        let gradient_button = gradient_button.clone();
        let solid_button = solid_button.clone();
        let signature_button = signature_button.clone();
        let blur_button = blur_button.clone();
        let solid_row = solid_row.clone();
        let gradient_from_row = gradient_from_row.clone();
        let gradient_to_row = gradient_to_row.clone();
        let gradient_angle_row = gradient_angle_row.clone();
        let blur_row = blur_row.clone();
        let signature_intensity_row = signature_intensity_row.clone();
        let solid_color_button = solid_color_button.clone();
        let gradient_from_button = gradient_from_button.clone();
        let gradient_to_button = gradient_to_button.clone();
        let gradient_angle_scale = gradient_angle_scale.clone();
        let gradient_angle_value = gradient_angle_value.clone();
        let blur_radius_scale = blur_radius_scale.clone();
        let blur_radius_value = blur_radius_value.clone();
        let signature_intensity_scale = signature_intensity_scale.clone();
        let signature_intensity_value = signature_intensity_value.clone();
        let suppress_sync_events_for_handler = suppress_sync_events.clone();
        let presets_label_for_handler: gtk4::Widget = presets_label.clone().upcast();
        let presets_grid_for_handler: gtk4::Widget = swatch_grid.clone().upcast();
        let signature_presets_grid_for_handler: gtk4::Widget =
            signature_swatch_grid.clone().upcast();
        let background_for_handler = background.clone();
        button.connect_clicked(move |_| {
            let background = background_for_handler.clone();
            let mut state = state.borrow_mut();
            if state.update_document(|doc| doc.background = background.clone()) {
                for (existing_background, existing_button) in all_buttons.borrow().iter() {
                    if same_background(existing_background, &background) {
                        existing_button.add_css_class("selected");
                    } else {
                        existing_button.remove_css_class("selected");
                    }
                }
                refresh_background_mode_controls(
                    &background,
                    &gradient_button,
                    &solid_button,
                    &signature_button,
                    &blur_button,
                    &solid_row,
                    &gradient_from_row,
                    &gradient_to_row,
                    &gradient_angle_row,
                    &blur_row,
                    &signature_intensity_row,
                );
                refresh_background_preset_controls(
                    &background,
                    &presets_label_for_handler,
                    &presets_grid_for_handler,
                    &signature_presets_grid_for_handler,
                    &all_buttons,
                );
                sync_background_editor_values(
                    &background,
                    &solid_color_button,
                    &gradient_from_button,
                    &gradient_to_button,
                    &gradient_angle_scale,
                    &gradient_angle_value,
                    &blur_radius_scale,
                    &blur_radius_value,
                    &signature_intensity_scale,
                    &signature_intensity_value,
                    &suppress_sync_events_for_handler,
                );
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
        if is_signature_preset {
            signature_swatch_grid.attach(
                &button,
                (signature_index % 2) as i32,
                (signature_index / 2) as i32,
                1,
                1,
            );
            signature_index += 1;
        } else {
            swatch_grid.attach(
                &button,
                (clean_index % 4) as i32,
                (clean_index / 4) as i32,
                1,
                1,
            );
            clean_index += 1;
        }
    }

    refresh_background_preset_controls(
        &current_background,
        &presets_label.clone().upcast(),
        &swatch_grid.clone().upcast(),
        &signature_swatch_grid.clone().upcast(),
        &swatch_buttons,
    );
    panel.append(&presets_stack);

    BackgroundSection {
        swatch_buttons,
        presets_label: presets_label.upcast(),
        presets_grid: swatch_grid.upcast(),
        signature_presets_grid: signature_swatch_grid.upcast(),
        gradient_button,
        solid_button,
        signature_button,
        blur_button,
        solid_color_button,
        solid_row,
        gradient_from_button,
        gradient_to_button,
        gradient_from_row,
        gradient_to_row,
        gradient_angle_scale,
        gradient_angle_value,
        gradient_angle_row,
        blur_radius_scale,
        blur_radius_value,
        blur_row,
        signature_intensity_scale,
        signature_intensity_value,
        signature_intensity_row,
        suppress_sync_events,
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_background_change(
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    next_background: Background,
    mode_controls: Option<BackgroundModeControls>,
    editor_controls: Option<BackgroundEditorControls>,
    suppress_sync_events: Option<Rc<Cell<bool>>>,
    swatch_buttons: Option<BackgroundSwatchButtons>,
    preset_controls: Option<(gtk4::Widget, gtk4::Widget, gtk4::Widget)>,
) {
    let mut state = state.borrow_mut();
    if state.update_document(|doc| doc.background = next_background.clone()) {
        if let Some((
            gradient_button,
            solid_button,
            signature_button,
            blur_button,
            solid_row,
            gradient_from_row,
            gradient_to_row,
            gradient_angle_row,
            blur_row,
            signature_intensity_row,
        )) = mode_controls
        {
            refresh_background_mode_controls(
                &next_background,
                &gradient_button,
                &solid_button,
                &signature_button,
                &blur_button,
                &solid_row,
                &gradient_from_row,
                &gradient_to_row,
                &gradient_angle_row,
                &blur_row,
                &signature_intensity_row,
            );
        }
        if let (
            Some((
                solid_color_button,
                gradient_from_button,
                gradient_to_button,
                gradient_angle_scale,
                gradient_angle_value,
                blur_radius_scale,
                blur_radius_value,
                signature_intensity_scale,
                signature_intensity_value,
            )),
            Some(suppress_sync_events),
        ) = (editor_controls, suppress_sync_events)
        {
            sync_background_editor_values(
                &next_background,
                &solid_color_button,
                &gradient_from_button,
                &gradient_to_button,
                &gradient_angle_scale,
                &gradient_angle_value,
                &blur_radius_scale,
                &blur_radius_value,
                &signature_intensity_scale,
                &signature_intensity_value,
                &suppress_sync_events,
            );
        }
        if let Some(buttons) = swatch_buttons.clone() {
            for (_, button) in buttons.borrow().iter() {
                button.remove_css_class("selected");
            }
        }
        if let (Some(buttons), Some((presets_label, presets_grid, signature_presets_grid))) =
            (swatch_buttons, preset_controls)
        {
            refresh_background_preset_controls(
                &next_background,
                &presets_label,
                &presets_grid,
                &signature_presets_grid,
                &buttons,
            );
        }
        refresh_subtitle(&state, subtitle_label);
        refresh_history_buttons(&state, undo_button, redo_button);
        canvas.refresh();
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn refresh_background_mode_controls(
    background: &Background,
    gradient_button: &gtk4::Button,
    solid_button: &gtk4::Button,
    signature_button: &gtk4::Button,
    blur_button: &gtk4::Button,
    solid_row: &gtk4::Widget,
    gradient_from_row: &gtk4::Widget,
    gradient_to_row: &gtk4::Widget,
    gradient_angle_row: &gtk4::Widget,
    blur_row: &gtk4::Widget,
    signature_intensity_row: &gtk4::Widget,
) {
    let is_gradient = matches!(background, Background::Gradient { .. });
    let is_solid = matches!(background, Background::Solid { .. });
    let is_signature = matches!(background, Background::Style { .. });
    let is_blur = matches!(background, Background::BlurredScreenshot { .. });

    set_selected(gradient_button, is_gradient);
    set_selected(solid_button, is_solid);
    set_selected(signature_button, is_signature);
    set_selected(blur_button, is_blur);

    solid_row.set_visible(is_solid);
    gradient_from_row.set_visible(is_gradient);
    gradient_to_row.set_visible(is_gradient);
    gradient_angle_row.set_visible(is_gradient);
    blur_row.set_visible(is_blur);
    signature_intensity_row.set_visible(is_signature);
}

pub(crate) fn refresh_background_preset_controls(
    background: &Background,
    presets_label: &gtk4::Widget,
    presets_grid: &gtk4::Widget,
    signature_presets_grid: &gtk4::Widget,
    swatch_buttons: &BackgroundSwatchButtons,
) {
    let show_presets = !matches!(background, Background::BlurredScreenshot { .. });
    presets_label.set_visible(show_presets);
    let show_signature = matches!(background, Background::Style { .. });
    presets_grid.set_visible(show_presets && !show_signature);
    signature_presets_grid.set_visible(show_presets && show_signature);

    for (preset_background, button) in swatch_buttons.borrow().iter() {
        button.set_visible(show_presets && background_preset_matches_mode(background, preset_background));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_background_editor_values(
    background: &Background,
    solid_color_button: &gtk4::ColorButton,
    gradient_from_button: &gtk4::ColorButton,
    gradient_to_button: &gtk4::ColorButton,
    gradient_angle_scale: &gtk4::Scale,
    gradient_angle_value: &gtk4::Label,
    blur_radius_scale: &gtk4::Scale,
    blur_radius_value: &gtk4::Label,
    signature_intensity_scale: &gtk4::Scale,
    signature_intensity_value: &gtk4::Label,
    suppress_sync_events: &Rc<Cell<bool>>,
) {
    suppress_sync_events.set(true);
    match background {
        Background::Solid { color } => {
            solid_color_button.set_rgba(&rgba_from_color(color));
        }
        Background::Gradient {
            from,
            to,
            angle_deg,
        } => {
            gradient_from_button.set_rgba(&rgba_from_color(from));
            gradient_to_button.set_rgba(&rgba_from_color(to));
            gradient_angle_scale.set_value(*angle_deg as f64);
            gradient_angle_value.set_label(&format!("{}°", angle_deg.round() as i32));
        }
        Background::BlurredScreenshot { radius } => {
            blur_radius_scale.set_value(*radius as f64);
            blur_radius_value.set_label(&format!("{}px", radius.round() as u32));
        }
        Background::Style { intensity, .. } => {
            signature_intensity_scale.set_value(*intensity as f64);
            signature_intensity_value
                .set_label(&format!("{}%", (intensity * 100.0).round() as u32));
        }
        Background::Image { .. } => {}
    }
    suppress_sync_events.set(false);
}

fn rgba_from_color(color: &Color) -> gdk::RGBA {
    gdk::RGBA::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0,
    )
}

fn color_from_rgba(rgba: &gdk::RGBA) -> Color {
    Color {
        r: (rgba.red() * 255.0).round().clamp(0.0, 255.0) as u8,
        g: (rgba.green() * 255.0).round().clamp(0.0, 255.0) as u8,
        b: (rgba.blue() * 255.0).round().clamp(0.0, 255.0) as u8,
        a: (rgba.alpha() * 255.0).round().clamp(0.0, 255.0) as u8,
    }
}

fn build_signature_preview_card(label: &str, background: &Background) -> gtk4::Widget {
    let card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let art = gtk4::DrawingArea::new();
    art.set_height_request(68);
    let preview_background = background.clone();
    art.set_draw_func(move |_, cr, width, height| {
        paint_signature_preview_thumbnail(cr, width as f64, height as f64, &preview_background);
    });

    let title = gtk4::Label::builder()
        .label(label)
        .xalign(0.0)
        .wrap(true)
        .css_classes(["background-swatch-label"])
        .build();
    if matches!(
        background,
        Background::Style {
            id: BackgroundStyleId::CutPaper,
            ..
        }
    ) {
        title.add_css_class("background-swatch-label-dark");
    }

    card.append(&art);
    card.append(&title);
    card.upcast()
}

fn extract_solid_color(background: &Background) -> Color {
    match background {
        Background::Solid { color } => color.clone(),
        Background::Gradient { from, .. } => from.clone(),
        _ => Color {
            r: 31,
            g: 36,
            b: 45,
            a: 255,
        },
    }
}

fn background_preset_matches_mode(current: &Background, preset: &Background) -> bool {
    matches!(
        (current, preset),
        (Background::Gradient { .. }, Background::Gradient { .. })
            | (Background::Solid { .. }, Background::Solid { .. })
            | (Background::Style { .. }, Background::Style { .. })
    )
}

fn extract_gradient_from(background: &Background) -> Color {
    match background {
        Background::Gradient { from, .. } => from.clone(),
        Background::Solid { color } => color.clone(),
        _ => Color {
            r: 110,
            g: 162,
            b: 255,
            a: 255,
        },
    }
}

fn extract_gradient_to(background: &Background) -> Color {
    match background {
        Background::Gradient { to, .. } => to.clone(),
        _ => Color {
            r: 130,
            g: 99,
            b: 245,
            a: 255,
        },
    }
}

fn set_selected(button: &gtk4::Button, selected: bool) {
    if selected {
        button.add_css_class("selected");
    } else {
        button.remove_css_class("selected");
    }
}
