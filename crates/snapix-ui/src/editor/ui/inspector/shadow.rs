use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use super::super::helpers::{
    connect_frame_slider, nearest_shadow_direction_index, refresh_history_buttons,
    refresh_subtitle, SHADOW_DIRECTION_PRESETS,
};
use super::{labeled_row, labeled_row_with_value};
use crate::editor::i18n;
use crate::editor::state::EditorState;
use crate::widgets::DocumentCanvas;

pub(super) struct ShadowSection {
    pub(super) shadow_switch: gtk4::Switch,
    pub(super) shadow_direction_buttons: Rc<RefCell<Vec<gtk4::Button>>>,
    pub(super) shadow_padding_scale: gtk4::Scale,
    pub(super) shadow_padding_value: gtk4::Label,
    pub(super) shadow_blur_scale: gtk4::Scale,
    pub(super) shadow_blur_value: gtk4::Label,
    pub(super) shadow_strength_scale: gtk4::Scale,
    pub(super) shadow_strength_value: gtk4::Label,
}

pub(super) fn build_shadow_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> ShadowSection {
    let shadow_switch = gtk4::Switch::builder()
        .active(state.borrow().document.frame.shadow)
        .halign(gtk4::Align::End)
        .build();
    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        shadow_switch.connect_active_notify(move |sw| {
            let Ok(mut state) = state.try_borrow_mut() else {
                return;
            };
            if state.update_document(|doc| doc.frame.shadow = sw.is_active()) {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }
    panel.append(&labeled_row(i18n::inspector_shadow_label(), &shadow_switch));

    let shadow_direction_grid = gtk4::Grid::builder()
        .row_spacing(5)
        .column_spacing(5)
        .halign(gtk4::Align::Center)
        .build();
    let shadow_direction_buttons: Rc<RefCell<Vec<gtk4::Button>>> =
        Rc::new(RefCell::new(Vec::new()));
    let selected_direction = nearest_shadow_direction_index(
        state.borrow().document.frame.shadow_offset_x,
        state.borrow().document.frame.shadow_offset_y,
    );
    for (index, preset) in SHADOW_DIRECTION_PRESETS.iter().enumerate() {
        let button = gtk4::Button::builder()
            .label(preset.label)
            .tooltip_text(i18n::shadow_direction_tooltip(index))
            .build();
        button.add_css_class("shadow-dir-btn");
        if index == selected_direction {
            button.add_css_class("selected");
        }

        let all_buttons = shadow_direction_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let offset_x = preset.offset_x;
        let offset_y = preset.offset_y;
        button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                doc.frame.shadow_offset_x = offset_x;
                doc.frame.shadow_offset_y = offset_y;
            }) {
                for (button_index, existing) in all_buttons.borrow().iter().enumerate() {
                    if button_index == index {
                        existing.add_css_class("selected");
                    } else {
                        existing.remove_css_class("selected");
                    }
                }
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });

        shadow_direction_buttons.borrow_mut().push(button.clone());
        shadow_direction_grid.attach(&button, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }
    panel.append(&labeled_row(
        i18n::inspector_shadow_direction_label(),
        &shadow_direction_grid,
    ));

    let shadow_padding_value = gtk4::Label::builder()
        .label(format!(
            "{}px",
            state.borrow().document.frame.shadow_padding as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let shadow_padding_scale =
        gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 32.0, 1.0);
    shadow_padding_scale.set_value(state.borrow().document.frame.shadow_padding as f64);
    {
        let value = shadow_padding_value.clone();
        connect_frame_slider(
            &shadow_padding_scale,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, slider_value| {
                frame.shadow_padding = slider_value;
                value.set_label(&format!("{}px", slider_value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        i18n::inspector_shadow_padding_label(),
        &shadow_padding_scale,
        &shadow_padding_value,
    ));

    let shadow_blur_value = gtk4::Label::builder()
        .label(format!(
            "{}px",
            state.borrow().document.frame.shadow_blur as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let shadow_blur_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 64.0, 1.0);
    shadow_blur_scale.set_value(state.borrow().document.frame.shadow_blur as f64);
    {
        let value = shadow_blur_value.clone();
        connect_frame_slider(
            &shadow_blur_scale,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, slider_value| {
                frame.shadow_blur = slider_value;
                value.set_label(&format!("{}px", slider_value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        i18n::inspector_shadow_blur_label(),
        &shadow_blur_scale,
        &shadow_blur_value,
    ));

    let shadow_strength_value = gtk4::Label::builder()
        .label(format!(
            "{}%",
            (state.borrow().document.frame.shadow_strength * 100.0).round() as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let shadow_strength_scale =
        gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
    shadow_strength_scale.set_value((state.borrow().document.frame.shadow_strength * 100.0) as f64);
    {
        let value = shadow_strength_value.clone();
        connect_frame_slider(
            &shadow_strength_scale,
            state,
            canvas,
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, slider_value| {
                frame.shadow_strength = (slider_value / 100.0).clamp(0.0, 1.0);
                value.set_label(&format!("{}%", slider_value.round() as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        i18n::inspector_shadow_strength_label(),
        &shadow_strength_scale,
        &shadow_strength_value,
    ));

    ShadowSection {
        shadow_switch,
        shadow_direction_buttons,
        shadow_padding_scale,
        shadow_padding_value,
        shadow_blur_scale,
        shadow_blur_value,
        shadow_strength_scale,
        shadow_strength_value,
    }
}
