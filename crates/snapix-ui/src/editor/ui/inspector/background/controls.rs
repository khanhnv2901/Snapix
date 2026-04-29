use std::cell::Cell;
use std::rc::Rc;

use gtk4::gdk;
use gtk4::prelude::*;
use snapix_core::canvas::{Background, Color};

use super::BackgroundSwatchButtons;

#[derive(Clone)]
pub(crate) struct BackgroundModeControls {
    pub(crate) gradient_button: gtk4::Button,
    pub(crate) solid_button: gtk4::Button,
    pub(crate) signature_button: gtk4::Button,
    pub(crate) blur_button: gtk4::Button,
    pub(crate) solid_row: gtk4::Widget,
    pub(crate) gradient_from_row: gtk4::Widget,
    pub(crate) gradient_to_row: gtk4::Widget,
    pub(crate) gradient_angle_row: gtk4::Widget,
    pub(crate) blur_row: gtk4::Widget,
    pub(crate) signature_intensity_row: gtk4::Widget,
}

#[derive(Clone)]
pub(crate) struct BackgroundEditorControls {
    pub(crate) solid_color_button: gtk4::ColorButton,
    pub(crate) gradient_from_button: gtk4::ColorButton,
    pub(crate) gradient_to_button: gtk4::ColorButton,
    pub(crate) gradient_angle_scale: gtk4::Scale,
    pub(crate) gradient_angle_value: gtk4::Label,
    pub(crate) blur_radius_scale: gtk4::Scale,
    pub(crate) blur_radius_value: gtk4::Label,
    pub(crate) signature_intensity_scale: gtk4::Scale,
    pub(crate) signature_intensity_value: gtk4::Label,
}

#[derive(Clone)]
pub(crate) struct BackgroundPresetControls {
    pub(crate) presets_label: gtk4::Widget,
    pub(crate) presets_grid: gtk4::Widget,
    pub(crate) signature_presets_grid: gtk4::Widget,
}

pub(crate) fn refresh_background_mode_controls(
    background: &Background,
    controls: &BackgroundModeControls,
) {
    let is_gradient = matches!(background, Background::Gradient { .. });
    let is_solid = matches!(background, Background::Solid { .. });
    let is_signature = matches!(background, Background::Style { .. });
    let is_blur = matches!(background, Background::BlurredScreenshot { .. });

    set_selected(&controls.gradient_button, is_gradient);
    set_selected(&controls.solid_button, is_solid);
    set_selected(&controls.signature_button, is_signature);
    set_selected(&controls.blur_button, is_blur);

    controls.solid_row.set_visible(is_solid);
    controls.gradient_from_row.set_visible(is_gradient);
    controls.gradient_to_row.set_visible(is_gradient);
    controls.gradient_angle_row.set_visible(is_gradient);
    controls.blur_row.set_visible(is_blur);
    controls.signature_intensity_row.set_visible(is_signature);
}

pub(crate) fn refresh_background_preset_controls(
    background: &Background,
    controls: &BackgroundPresetControls,
    swatch_buttons: &BackgroundSwatchButtons,
) {
    let show_presets = !matches!(background, Background::BlurredScreenshot { .. });
    controls.presets_label.set_visible(show_presets);
    let show_signature = matches!(background, Background::Style { .. });
    controls
        .presets_grid
        .set_visible(show_presets && !show_signature);
    controls
        .signature_presets_grid
        .set_visible(show_presets && show_signature);

    for (preset_background, button) in swatch_buttons.borrow().iter() {
        button.set_visible(
            show_presets && background_preset_matches_mode(background, preset_background),
        );
    }
}

pub(crate) fn sync_background_editor_values(
    background: &Background,
    controls: &BackgroundEditorControls,
    suppress_sync_events: &Rc<Cell<bool>>,
) {
    suppress_sync_events.set(true);
    match background {
        Background::Solid { color } => {
            controls
                .solid_color_button
                .set_rgba(&rgba_from_color(color));
        }
        Background::Gradient {
            from,
            to,
            angle_deg,
        } => {
            controls
                .gradient_from_button
                .set_rgba(&rgba_from_color(from));
            controls.gradient_to_button.set_rgba(&rgba_from_color(to));
            controls.gradient_angle_scale.set_value(*angle_deg as f64);
            controls
                .gradient_angle_value
                .set_label(&format!("{}°", angle_deg.round() as i32));
        }
        Background::BlurredScreenshot { radius } => {
            controls.blur_radius_scale.set_value(*radius as f64);
            controls
                .blur_radius_value
                .set_label(&format!("{}px", radius.round() as u32));
        }
        Background::Style { intensity, .. } => {
            controls
                .signature_intensity_scale
                .set_value(*intensity as f64);
            controls
                .signature_intensity_value
                .set_label(&format!("{}%", (intensity * 100.0).round() as u32));
        }
        Background::Image { .. } => {}
    }
    suppress_sync_events.set(false);
}

pub(super) fn rgba_from_color(color: &Color) -> gdk::RGBA {
    gdk::RGBA::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0,
    )
}

pub(super) fn color_from_rgba(rgba: &gdk::RGBA) -> Color {
    Color {
        r: (rgba.red() * 255.0).round().clamp(0.0, 255.0) as u8,
        g: (rgba.green() * 255.0).round().clamp(0.0, 255.0) as u8,
        b: (rgba.blue() * 255.0).round().clamp(0.0, 255.0) as u8,
        a: (rgba.alpha() * 255.0).round().clamp(0.0, 255.0) as u8,
    }
}

pub(super) fn extract_solid_color(background: &Background) -> Color {
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

pub(super) fn extract_gradient_from(background: &Background) -> Color {
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

pub(super) fn extract_gradient_to(background: &Background) -> Color {
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

fn background_preset_matches_mode(current: &Background, preset: &Background) -> bool {
    matches!(
        (current, preset),
        (Background::Gradient { .. }, Background::Gradient { .. })
            | (Background::Solid { .. }, Background::Solid { .. })
            | (Background::Style { .. }, Background::Style { .. })
    )
}

fn set_selected(button: &gtk4::Button, selected: bool) {
    if selected {
        button.add_css_class("selected");
    } else {
        button.remove_css_class("selected");
    }
}
