mod controls;
mod presets;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use snapix_core::canvas::{Background, BackgroundStyleId, Color};

use super::super::helpers::{
    configure_inspector_slider, refresh_history_buttons, refresh_subtitle,
};
use super::labeled_row_with_value;
use crate::editor::i18n;
use crate::editor::state::{same_background, EditorState};
use crate::widgets::DocumentCanvas;
use controls::{
    color_from_rgba, extract_gradient_from, extract_gradient_to, extract_solid_color,
    rgba_from_color,
};
pub(crate) use controls::{
    refresh_background_mode_controls, refresh_background_preset_controls,
    sync_background_editor_values, BackgroundEditorControls, BackgroundModeControls,
    BackgroundPresetControls,
};
use presets::{background_presets, build_signature_preview_card};

pub(crate) type BackgroundSwatchButtons = Rc<RefCell<Vec<(Background, gtk4::Button)>>>;

#[derive(Clone)]
struct BackgroundUi {
    swatch_buttons: BackgroundSwatchButtons,
    mode_controls: BackgroundModeControls,
    editor_controls: BackgroundEditorControls,
    preset_controls: BackgroundPresetControls,
    suppress_sync_events: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct BackgroundChangeContext {
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: gtk4::Label,
    undo_button: gtk4::Button,
    redo_button: gtk4::Button,
    ui: BackgroundUi,
}

#[derive(Clone)]
pub(super) struct BackgroundSection {
    pub(super) swatch_buttons: BackgroundSwatchButtons,
    pub(super) preset_controls: BackgroundPresetControls,
    pub(super) mode_controls: BackgroundModeControls,
    pub(super) editor_controls: BackgroundEditorControls,
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

    let mut mode_controls = build_mode_controls(panel);
    let (
        editor_controls,
        solid_row,
        gradient_from_row,
        gradient_to_row,
        gradient_angle_row,
        blur_row,
        signature_intensity_row,
    ) = build_editor_controls(panel, &current_background, suppress_sync_events.clone());
    mode_controls.solid_row = solid_row;
    mode_controls.gradient_from_row = gradient_from_row;
    mode_controls.gradient_to_row = gradient_to_row;
    mode_controls.gradient_angle_row = gradient_angle_row;
    mode_controls.blur_row = blur_row;
    mode_controls.signature_intensity_row = signature_intensity_row;
    let preset_controls = build_preset_controls(panel);

    let ui = BackgroundUi {
        swatch_buttons: swatch_buttons.clone(),
        mode_controls: mode_controls.clone(),
        editor_controls: editor_controls.clone(),
        preset_controls: preset_controls.clone(),
        suppress_sync_events: suppress_sync_events.clone(),
    };
    let context = BackgroundChangeContext {
        state,
        canvas,
        subtitle_label: subtitle_label.clone(),
        undo_button: undo_button.clone(),
        redo_button: redo_button.clone(),
        ui: ui.clone(),
    };

    refresh_background_mode_controls(&current_background, &ui.mode_controls);
    sync_background_editor_values(
        &current_background,
        &ui.editor_controls,
        &ui.suppress_sync_events,
    );

    connect_mode_handlers(&context);
    connect_editor_handlers(&context, &current_background);
    populate_presets(&current_background, &context);
    refresh_background_preset_controls(
        &current_background,
        &ui.preset_controls,
        &ui.swatch_buttons,
    );

    BackgroundSection {
        swatch_buttons,
        preset_controls,
        mode_controls,
        editor_controls,
        suppress_sync_events,
    }
}

fn build_mode_controls(panel: &gtk4::Box) -> BackgroundModeControls {
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
    for button in [
        &gradient_button,
        &solid_button,
        &signature_button,
        &blur_button,
    ] {
        button.add_css_class("ratio-btn");
        mode_row.append(button);
    }
    panel.append(&mode_row);

    BackgroundModeControls {
        gradient_button,
        solid_button,
        signature_button,
        blur_button,
        solid_row: gtk4::Box::new(gtk4::Orientation::Vertical, 0).upcast(),
        gradient_from_row: gtk4::Box::new(gtk4::Orientation::Vertical, 0).upcast(),
        gradient_to_row: gtk4::Box::new(gtk4::Orientation::Vertical, 0).upcast(),
        gradient_angle_row: gtk4::Box::new(gtk4::Orientation::Vertical, 0).upcast(),
        blur_row: gtk4::Box::new(gtk4::Orientation::Vertical, 0).upcast(),
        signature_intensity_row: gtk4::Box::new(gtk4::Orientation::Vertical, 0).upcast(),
    }
}

fn build_editor_controls(
    panel: &gtk4::Box,
    current_background: &Background,
    _suppress_sync_events: Rc<Cell<bool>>,
) -> (
    BackgroundEditorControls,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
    gtk4::Widget,
) {
    let solid_color_button =
        gtk4::ColorButton::with_rgba(&rgba_from_color(&extract_solid_color(current_background)));
    #[allow(deprecated)]
    solid_color_button.set_title(i18n::inspector_pick_background_color());
    solid_color_button.set_show_editor(true);
    solid_color_button.set_hexpand(true);

    let gradient_from_button =
        gtk4::ColorButton::with_rgba(&rgba_from_color(&extract_gradient_from(current_background)));
    #[allow(deprecated)]
    gradient_from_button.set_title(i18n::inspector_pick_gradient_start());
    gradient_from_button.set_show_editor(true);
    gradient_from_button.set_hexpand(true);

    let gradient_to_button =
        gtk4::ColorButton::with_rgba(&rgba_from_color(&extract_gradient_to(current_background)));
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

    let current_gradient_angle = match current_background {
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

    let current_blur_radius = match current_background {
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

    let current_signature_intensity = match current_background {
        Background::Style { intensity, .. } => *intensity,
        _ => 0.65,
    };
    let signature_intensity_value = gtk4::Label::builder()
        .label(format!(
            "{}%",
            (current_signature_intensity * 100.0).round() as u32
        ))
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

    (
        BackgroundEditorControls {
            solid_color_button,
            gradient_from_button,
            gradient_to_button,
            gradient_angle_scale,
            gradient_angle_value,
            blur_radius_scale,
            blur_radius_value,
            signature_intensity_scale,
            signature_intensity_value,
        },
        solid_row,
        gradient_from_row,
        gradient_to_row,
        gradient_angle_row,
        blur_row,
        signature_intensity_row,
    )
}

fn build_preset_controls(panel: &gtk4::Box) -> BackgroundPresetControls {
    let presets_label = gtk4::Label::builder()
        .label("Presets")
        .xalign(0.0)
        .css_classes(["dim-copy"])
        .margin_top(4)
        .build();
    panel.append(&presets_label);

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
    panel.append(&presets_stack);

    BackgroundPresetControls {
        presets_label: presets_label.upcast(),
        presets_grid: swatch_grid.upcast(),
        signature_presets_grid: signature_swatch_grid.upcast(),
    }
}

fn connect_mode_handlers(context: &BackgroundChangeContext) {
    connect_mode_button(
        &context.ui.mode_controls.gradient_button,
        context.clone(),
        |background| match background {
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
        },
    );
    connect_mode_button(
        &context.ui.mode_controls.solid_button,
        context.clone(),
        |background| match background {
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
        },
    );
    connect_mode_button(
        &context.ui.mode_controls.signature_button,
        context.clone(),
        |background| match background {
            Background::Style { id, intensity } => Background::Style {
                id: *id,
                intensity: *intensity,
            },
            _ => Background::Style {
                id: BackgroundStyleId::Blueprint,
                intensity: 0.65,
            },
        },
    );
    connect_mode_button(
        &context.ui.mode_controls.blur_button,
        context.clone(),
        |background| {
            let radius = match background {
                Background::BlurredScreenshot { radius } => *radius,
                _ => 24.0,
            };
            Background::BlurredScreenshot { radius }
        },
    );
}

fn connect_editor_handlers(context: &BackgroundChangeContext, current_background: &Background) {
    {
        let context = context.clone();
        let button = context.ui.editor_controls.solid_color_button.clone();
        button.connect_color_set(move |button| {
            let color = color_from_rgba(&button.rgba());
            update_background_in_place(&context, move |background| match background {
                Background::Solid { color: current } => *current = color.clone(),
                _ => {
                    *background = Background::Solid {
                        color: color.clone(),
                    };
                }
            });
        });
    }

    {
        let context = context.clone();
        let button = context.ui.editor_controls.gradient_from_button.clone();
        button.connect_color_set(move |button| {
            let color = color_from_rgba(&button.rgba());
            update_background_in_place(&context, move |background| {
                if let Background::Gradient { from, .. } = background {
                    *from = color.clone();
                }
            });
        });
    }

    {
        let context = context.clone();
        let button = context.ui.editor_controls.gradient_to_button.clone();
        button.connect_color_set(move |button| {
            let color = color_from_rgba(&button.rgba());
            update_background_in_place(&context, move |background| {
                if let Background::Gradient { to, .. } = background {
                    *to = color.clone();
                }
            });
        });
    }

    {
        let context = context.clone();
        let value_label = context.ui.editor_controls.gradient_angle_value.clone();
        let suppress_sync_events = context.ui.suppress_sync_events.clone();
        let scale_widget = context.ui.editor_controls.gradient_angle_scale.clone();
        scale_widget.connect_value_changed(move |scale| {
            if suppress_sync_events.get() {
                return;
            }
            let angle = scale.value() as f32;
            value_label.set_label(&format!("{}°", angle.round() as i32));
            update_background_in_place(&context, move |background| {
                if let Background::Gradient { angle_deg, .. } = background {
                    *angle_deg = angle;
                }
            });
        });
    }

    {
        let context = context.clone();
        let value_label = context.ui.editor_controls.blur_radius_value.clone();
        let pending_radius = Rc::new(Cell::new(match current_background {
            Background::BlurredScreenshot { radius } => *radius,
            _ => 24.0,
        }));
        let change_generation = Rc::new(Cell::new(0u64));
        let suppress_sync_events = context.ui.suppress_sync_events.clone();
        let scale_widget = context.ui.editor_controls.blur_radius_scale.clone();
        scale_widget.connect_value_changed(move |scale| {
            if suppress_sync_events.get() {
                return;
            }
            let radius = scale.value() as f32;
            value_label.set_label(&format!("{}px", radius.round() as u32));
            pending_radius.set(radius);

            let generation = change_generation.get().wrapping_add(1);
            change_generation.set(generation);

            let context = context.clone();
            let pending_radius = pending_radius.clone();
            let change_generation = change_generation.clone();
            gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(140), move || {
                if change_generation.get() != generation {
                    return;
                }
                let radius = pending_radius.get();
                update_background_in_place(&context, move |background| {
                    if let Background::BlurredScreenshot { radius: current } = background {
                        *current = radius;
                    }
                });
            });
        });
    }

    {
        let context = context.clone();
        let value_label = context.ui.editor_controls.signature_intensity_value.clone();
        let suppress_sync_events = context.ui.suppress_sync_events.clone();
        let scale_widget = context.ui.editor_controls.signature_intensity_scale.clone();
        scale_widget.connect_value_changed(move |scale| {
            if suppress_sync_events.get() {
                return;
            }
            let intensity = scale.value() as f32;
            value_label.set_label(&format!("{}%", (intensity * 100.0).round() as u32));
            update_background_in_place(&context, move |background| {
                if let Background::Style {
                    intensity: current, ..
                } = background
                {
                    *current = intensity;
                }
            });
        });
    }
}

fn populate_presets(current_background: &Background, context: &BackgroundChangeContext) {
    let presets_grid = context
        .ui
        .preset_controls
        .presets_grid
        .clone()
        .downcast::<gtk4::Grid>()
        .expect("background presets grid should be a gtk4::Grid");
    let signature_grid = context
        .ui
        .preset_controls
        .signature_presets_grid
        .clone()
        .downcast::<gtk4::Grid>()
        .expect("signature presets grid should be a gtk4::Grid");

    let mut clean_index = 0usize;
    let mut signature_index = 0usize;
    for preset in background_presets() {
        let is_signature_preset = matches!(preset.background, Background::Style { .. });
        let button = gtk4::Button::builder()
            .tooltip_text(preset.label)
            .hexpand(true)
            .vexpand(false)
            .build();
        button.add_css_class("background-swatch");
        if is_signature_preset {
            button.add_css_class("background-swatch-signature");
            let preview = build_signature_preview_card(preset.label, &preset.background);
            button.set_child(Some(&preview));
        }
        button.add_css_class(preset.css_class);
        if same_background(current_background, &preset.background) {
            button.add_css_class("selected");
        }

        context
            .ui
            .swatch_buttons
            .borrow_mut()
            .push((preset.background.clone(), button.clone()));
        connect_preset_button(&button, context.clone(), preset.background.clone());

        if is_signature_preset {
            signature_grid.attach(
                &button,
                (signature_index % 2) as i32,
                (signature_index / 2) as i32,
                1,
                1,
            );
            signature_index += 1;
        } else {
            presets_grid.attach(
                &button,
                (clean_index % 4) as i32,
                (clean_index / 4) as i32,
                1,
                1,
            );
            clean_index += 1;
        }
    }
}

fn connect_mode_button<F>(
    button: &gtk4::Button,
    context: BackgroundChangeContext,
    next_background: F,
) where
    F: Fn(&Background) -> Background + 'static,
{
    button.connect_clicked(move |_| {
        let current_background = context.state.borrow().document().background.clone();
        apply_background_change(&context, next_background(&current_background));
    });
}

fn connect_preset_button(
    button: &gtk4::Button,
    context: BackgroundChangeContext,
    background: Background,
) {
    button.connect_clicked(move |_| {
        apply_background_change(&context, background.clone());
    });
}

fn apply_background_change(context: &BackgroundChangeContext, next_background: Background) {
    let mut state = context.state.borrow_mut();
    if state.update_document(|doc| doc.background = next_background.clone()) {
        refresh_background_mode_controls(&next_background, &context.ui.mode_controls);
        sync_background_editor_values(
            &next_background,
            &context.ui.editor_controls,
            &context.ui.suppress_sync_events,
        );
        refresh_background_preset_controls(
            &next_background,
            &context.ui.preset_controls,
            &context.ui.swatch_buttons,
        );
        for (existing_background, button) in context.ui.swatch_buttons.borrow().iter() {
            if same_background(existing_background, &next_background) {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }
        refresh_subtitle(&state, &context.subtitle_label);
        refresh_history_buttons(&state, &context.undo_button, &context.redo_button);
        context.canvas.refresh();
    }
}

fn update_background_in_place<F>(context: &BackgroundChangeContext, update: F)
where
    F: FnOnce(&mut Background),
{
    let mut state = context.state.borrow_mut();
    if state.update_document(|doc| update(&mut doc.background)) {
        refresh_subtitle(&state, &context.subtitle_label);
        refresh_history_buttons(&state, &context.undo_button, &context.redo_button);
        context.canvas.refresh();
    }
}
