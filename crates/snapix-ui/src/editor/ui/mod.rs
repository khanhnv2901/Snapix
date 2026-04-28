mod helpers;
mod inspector;
mod preferences;
mod toolbar;
mod unlock;
mod window;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use snapix_core::canvas::{Background, ImageAnchor, ImageScaleMode, OutputRatio};

use self::inspector::background::{
    refresh_background_mode_controls, refresh_background_preset_controls,
    sync_background_editor_values,
};
use super::state::{same_background, EditorState};

pub(crate) use helpers::{
    export_actions_enabled, nearest_shadow_direction_index, refresh_history_buttons,
    refresh_labels, refresh_scope_label, refresh_subtitle, refresh_tool_actions,
    refresh_width_label,
};
#[cfg(test)]
pub(crate) use helpers::{scope_text, shortcut_hint_text, subtitle_text};
pub(crate) use window::refresh_export_actions;
pub use window::EditorWindow;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveFormat {
    Png,
    Jpeg,
}

#[derive(Clone)]
pub(super) struct CaptureActionRow {
    pub(super) widget: gtk4::Widget,
    pub(super) fullscreen_button: gtk4::Button,
    pub(super) region_button: gtk4::Button,
    pub(super) window_button: gtk4::Button,
    pub(super) import_button: gtk4::Button,
    pub(super) clear_button: gtk4::Button,
}

#[derive(Clone)]
pub(super) struct BottomBar {
    pub(super) widget: gtk4::Widget,
    pub(super) copy_button: gtk4::Button,
    pub(super) quick_save_button: gtk4::Button,
    pub(super) save_as_button: gtk4::Button,
    pub(super) png_button: gtk4::ToggleButton,
    pub(super) jpeg_button: gtk4::ToggleButton,
}

#[derive(Clone)]
pub(super) struct InspectorControls {
    widget: gtk4::Widget,
    padding_scale: gtk4::Scale,
    padding_value: gtk4::Label,
    radius_scale: gtk4::Scale,
    radius_value: gtk4::Label,
    shadow_switch: gtk4::Switch,
    shadow_direction_buttons: Rc<RefCell<Vec<gtk4::Button>>>,
    shadow_padding_scale: gtk4::Scale,
    shadow_padding_value: gtk4::Label,
    shadow_blur_scale: gtk4::Scale,
    shadow_blur_value: gtk4::Label,
    shadow_strength_scale: gtk4::Scale,
    shadow_strength_value: gtk4::Label,
    ratio_buttons: Rc<RefCell<Vec<(OutputRatio, gtk4::Button)>>>,
    image_scale_mode_buttons: Rc<RefCell<Vec<(ImageScaleMode, gtk4::Button)>>>,
    image_anchor_buttons: Rc<RefCell<Vec<(ImageAnchor, gtk4::Button)>>>,
    background_buttons: Rc<RefCell<Vec<(Background, gtk4::Button)>>>,
    background_presets_label: gtk4::Widget,
    background_presets_grid: gtk4::Widget,
    background_signature_presets_grid: gtk4::Widget,
    background_gradient_button: gtk4::Button,
    background_solid_button: gtk4::Button,
    background_signature_button: gtk4::Button,
    background_blur_button: gtk4::Button,
    background_solid_color_button: gtk4::ColorButton,
    background_solid_row: gtk4::Widget,
    background_gradient_from_button: gtk4::ColorButton,
    background_gradient_to_button: gtk4::ColorButton,
    background_gradient_from_row: gtk4::Widget,
    background_gradient_to_row: gtk4::Widget,
    background_gradient_angle_scale: gtk4::Scale,
    background_gradient_angle_value: gtk4::Label,
    background_gradient_angle_row: gtk4::Widget,
    background_blur_scale: gtk4::Scale,
    background_blur_value: gtk4::Label,
    background_blur_row: gtk4::Widget,
    background_signature_intensity_scale: gtk4::Scale,
    background_signature_intensity_value: gtk4::Label,
    background_signature_intensity_row: gtk4::Widget,
    background_suppress_sync_events: Rc<Cell<bool>>,
}

impl InspectorControls {
    pub(super) fn widget(&self) -> gtk4::Widget {
        self.widget.clone()
    }

    pub(super) fn refresh_from_state(&self, state: &EditorState) {
        let frame = &state.document().frame;
        self.padding_scale.set_value(frame.padding as f64);
        self.padding_value
            .set_label(&format!("{}px", frame.padding as u32));

        self.radius_scale.set_value(frame.corner_radius as f64);
        self.radius_value
            .set_label(&format!("{}px", frame.corner_radius as u32));

        self.shadow_switch.set_active(frame.shadow);

        let selected_shadow_direction =
            nearest_shadow_direction_index(frame.shadow_offset_x, frame.shadow_offset_y);
        for (index, button) in self.shadow_direction_buttons.borrow().iter().enumerate() {
            if index == selected_shadow_direction {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }

        self.shadow_padding_scale
            .set_value(frame.shadow_padding as f64);
        self.shadow_padding_value
            .set_label(&format!("{}px", frame.shadow_padding as u32));

        self.shadow_blur_scale.set_value(frame.shadow_blur as f64);
        self.shadow_blur_value
            .set_label(&format!("{}px", frame.shadow_blur as u32));

        self.shadow_strength_scale
            .set_value((frame.shadow_strength * 100.0) as f64);
        self.shadow_strength_value.set_label(&format!(
            "{}%",
            (frame.shadow_strength * 100.0).round() as u32
        ));

        for (ratio, button) in self.ratio_buttons.borrow().iter() {
            if *ratio == state.document().output_ratio {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }

        for (mode, button) in self.image_scale_mode_buttons.borrow().iter() {
            button.set_sensitive(state.document().output_ratio != OutputRatio::Auto);
            if *mode == state.document().image_scale_mode {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }

        for (anchor, button) in self.image_anchor_buttons.borrow().iter() {
            button.set_sensitive(
                state.document().output_ratio != OutputRatio::Auto
                    && state.document().image_scale_mode == ImageScaleMode::Fill,
            );
            if *anchor == state.document().image_anchor {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }

        for (background, button) in self.background_buttons.borrow().iter() {
            if same_background(background, &state.document().background) {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }
        refresh_background_preset_controls(
            &state.document().background,
            &self.background_presets_label,
            &self.background_presets_grid,
            &self.background_signature_presets_grid,
            &self.background_buttons,
        );
        refresh_background_mode_controls(
            &state.document().background,
            &self.background_gradient_button,
            &self.background_solid_button,
            &self.background_signature_button,
            &self.background_blur_button,
            &self.background_solid_row,
            &self.background_gradient_from_row,
            &self.background_gradient_to_row,
            &self.background_gradient_angle_row,
            &self.background_blur_row,
            &self.background_signature_intensity_row,
        );
        sync_background_editor_values(
            &state.document().background,
            &self.background_solid_color_button,
            &self.background_gradient_from_button,
            &self.background_gradient_to_button,
            &self.background_gradient_angle_scale,
            &self.background_gradient_angle_value,
            &self.background_blur_scale,
            &self.background_blur_value,
            &self.background_signature_intensity_scale,
            &self.background_signature_intensity_value,
            &self.background_suppress_sync_events,
        );
    }
}

#[derive(Clone, Copy)]
pub(super) enum HistoryAction {
    Undo,
    Redo,
}
