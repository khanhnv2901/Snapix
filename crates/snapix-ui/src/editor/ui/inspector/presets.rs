use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use tracing::warn;

use super::super::helpers::{refresh_history_buttons, refresh_subtitle};
use super::InspectorControls;
use crate::editor::i18n;
use crate::editor::presets::{
    delete_style_preset, load_style_presets, save_style_preset, StylePreset,
};
use crate::editor::state::EditorState;
use crate::widgets::DocumentCanvas;

pub(super) fn build_preset_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    inspector: InspectorControls,
) {
    panel.append(
        &gtk4::Separator::builder()
            .margin_top(2)
            .margin_bottom(2)
            .build(),
    );

    panel.append(
        &gtk4::Label::builder()
            .label(i18n::inspector_saved_presets_title())
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .build(),
    );

    let presets = Rc::new(RefCell::new(load_style_presets_or_warn()));

    let combo = gtk4::ComboBoxText::new();
    combo.set_hexpand(true);
    populate_preset_combo(&combo, &presets.borrow(), None);
    panel.append(&combo);

    let name_entry = gtk4::Entry::builder()
        .placeholder_text(i18n::inspector_preset_name_placeholder())
        .hexpand(true)
        .build();
    panel.append(&name_entry);

    let button_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let save_button = gtk4::Button::builder()
        .label(i18n::save_button_label())
        .hexpand(true)
        .build();
    let apply_button = gtk4::Button::builder()
        .label(i18n::apply_button_label())
        .hexpand(true)
        .build();
    let delete_button = gtk4::Button::builder()
        .label(i18n::delete_button_label())
        .hexpand(true)
        .build();
    button_row.append(&save_button);
    button_row.append(&apply_button);
    button_row.append(&delete_button);
    panel.append(&button_row);

    refresh_preset_actions(&combo, &apply_button, &delete_button);

    {
        let name_entry = name_entry.clone();
        let apply_button = apply_button.clone();
        let delete_button = delete_button.clone();
        combo.connect_changed(move |combo| {
            if let Some(name) = combo.active_text() {
                name_entry.set_text(&name);
            }
            refresh_preset_actions(combo, &apply_button, &delete_button);
        });
    }

    {
        let presets = presets.clone();
        let combo = combo.clone();
        let name_entry = name_entry.clone();
        let apply_button = apply_button.clone();
        let delete_button = delete_button.clone();
        let state = state.clone();
        save_button.connect_clicked(move |_| {
            let name = resolved_preset_name(&name_entry, &combo);
            if name.is_empty() {
                return;
            }

            let preset = StylePreset::from_document(name.clone(), state.borrow().document());
            match save_style_preset(preset) {
                Ok(updated) => {
                    *presets.borrow_mut() = updated;
                    populate_preset_combo(&combo, &presets.borrow(), Some(&name));
                    name_entry.set_text(&name);
                    refresh_preset_actions(&combo, &apply_button, &delete_button);
                }
                Err(error) => warn!("Failed to save preset {name}: {error:#}"),
            }
        });
    }

    {
        let presets = presets.clone();
        let combo = combo.clone();
        let name_entry = name_entry.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let inspector = inspector.clone();
        apply_button.connect_clicked(move |_| {
            let Some(name) = combo.active_text().map(|text| text.to_string()) else {
                return;
            };
            let Some(preset) = presets
                .borrow()
                .iter()
                .find(|preset| preset.name == name)
                .cloned()
            else {
                return;
            };

            let mut state = state.borrow_mut();
            let changed = state.update_document(|document| preset.apply_to_document(document));
            inspector.refresh_from_state(&state);
            if changed {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
            name_entry.set_text(&name);
        });
    }

    {
        let presets = presets.clone();
        let combo = combo.clone();
        let name_entry = name_entry.clone();
        let apply_button = apply_button.clone();
        let delete_button_for_handler = delete_button.clone();
        delete_button.clone().connect_clicked(move |_| {
            let Some(name) = combo.active_text().map(|text| text.to_string()) else {
                return;
            };

            match delete_style_preset(&name) {
                Ok(updated) => {
                    *presets.borrow_mut() = updated;
                    populate_preset_combo(&combo, &presets.borrow(), None);
                    if name_entry.text().as_str() == name {
                        name_entry.set_text("");
                    }
                    refresh_preset_actions(&combo, &apply_button, &delete_button_for_handler);
                }
                Err(error) => warn!("Failed to delete preset {name}: {error:#}"),
            }
        });
    }
}

fn load_style_presets_or_warn() -> Vec<StylePreset> {
    match load_style_presets() {
        Ok(presets) => presets,
        Err(error) => {
            warn!("Failed to load saved presets: {error:#}");
            Vec::new()
        }
    }
}

fn populate_preset_combo(
    combo: &gtk4::ComboBoxText,
    presets: &[StylePreset],
    selected_name: Option<&str>,
) {
    combo.remove_all();
    for preset in presets {
        combo.append_text(&preset.name);
    }

    let active_index = selected_name
        .and_then(|selected| presets.iter().position(|preset| preset.name == selected))
        .or_else(|| (!presets.is_empty()).then_some(0));
    combo.set_active(active_index.map(|index| index as u32));
}

fn refresh_preset_actions(
    combo: &gtk4::ComboBoxText,
    apply_button: &gtk4::Button,
    delete_button: &gtk4::Button,
) {
    let has_selection = combo.active_text().is_some();
    apply_button.set_sensitive(has_selection);
    delete_button.set_sensitive(has_selection);
}

fn resolved_preset_name(entry: &gtk4::Entry, combo: &gtk4::ComboBoxText) -> String {
    let typed = entry.text().trim().to_string();
    if !typed.is_empty() {
        typed
    } else {
        combo
            .active_text()
            .map(|name| name.to_string())
            .unwrap_or_default()
    }
}
