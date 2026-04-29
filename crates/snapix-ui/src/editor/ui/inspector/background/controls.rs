use std::cell::Cell;
use std::rc::Rc;

use gtk4::gdk;
use gtk4::prelude::*;
use snapix_core::canvas::{Background, Color};

use super::BackgroundSwatchButtons;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundFamily {
    Clean,
    Signature,
    Image,
}

impl BackgroundFamily {
    pub(crate) fn from_background(background: &Background) -> Self {
        match background {
            Background::Solid { .. } | Background::Gradient { .. } => Self::Clean,
            Background::Style { .. } => Self::Signature,
            Background::BlurredScreenshot { .. } | Background::Image { .. } => Self::Image,
        }
    }
}

#[derive(Clone)]
pub(crate) struct BackgroundModeControls {
    pub(crate) clean_family_button: gtk4::Button,
    pub(crate) signature_family_button: gtk4::Button,
    pub(crate) image_family_button: gtk4::Button,

    pub(crate) clean_submode_row: gtk4::Widget,
    pub(crate) gradient_button: gtk4::Button,
    pub(crate) solid_button: gtk4::Button,

    pub(crate) image_submode_row: gtk4::Widget,
    pub(crate) screenshot_blur_button: gtk4::Button,
    pub(crate) custom_image_button: gtk4::Button,

    pub(crate) solid_row: gtk4::Widget,
    pub(crate) gradient_from_row: gtk4::Widget,
    pub(crate) gradient_to_row: gtk4::Widget,
    pub(crate) gradient_angle_row: gtk4::Widget,
    pub(crate) blur_row: gtk4::Widget,
    pub(crate) image_path_row: gtk4::Widget,
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
    pub(crate) image_path_label: gtk4::Label,
    pub(crate) signature_intensity_scale: gtk4::Scale,
    pub(crate) signature_intensity_value: gtk4::Label,
}

#[derive(Clone)]
pub(crate) struct BackgroundPresetControls {
    pub(crate) presets_label: gtk4::Widget,
    pub(crate) gradient_presets_grid: gtk4::Widget,
    pub(crate) solid_presets_grid: gtk4::Widget,
    pub(crate) signature_presets_grid: gtk4::Widget,
}

pub(crate) fn refresh_background_mode_controls(
    background: &Background,
    controls: &BackgroundModeControls,
) {
    let family = BackgroundFamily::from_background(background);

    set_selected(
        &controls.clean_family_button,
        family == BackgroundFamily::Clean,
    );
    set_selected(
        &controls.signature_family_button,
        family == BackgroundFamily::Signature,
    );
    set_selected(
        &controls.image_family_button,
        family == BackgroundFamily::Image,
    );

    let is_gradient = matches!(background, Background::Gradient { .. });
    let is_solid = matches!(background, Background::Solid { .. });
    let is_signature = matches!(background, Background::Style { .. });
    let is_blur = matches!(background, Background::BlurredScreenshot { .. });
    let is_custom_image = matches!(background, Background::Image { .. });

    controls
        .clean_submode_row
        .set_visible(family == BackgroundFamily::Clean);
    if family == BackgroundFamily::Clean {
        set_selected(&controls.gradient_button, is_gradient);
        set_selected(&controls.solid_button, is_solid);
    }

    controls
        .image_submode_row
        .set_visible(family == BackgroundFamily::Image);
    if family == BackgroundFamily::Image {
        set_selected(&controls.screenshot_blur_button, is_blur);
        set_selected(&controls.custom_image_button, is_custom_image);
    }

    controls.solid_row.set_visible(is_solid);
    controls.gradient_from_row.set_visible(is_gradient);
    controls.gradient_to_row.set_visible(is_gradient);
    controls.gradient_angle_row.set_visible(is_gradient);
    controls.blur_row.set_visible(is_blur);
    controls.image_path_row.set_visible(is_custom_image);
    controls.signature_intensity_row.set_visible(is_signature);
}

pub(crate) fn refresh_background_preset_controls(
    background: &Background,
    controls: &BackgroundPresetControls,
    _swatch_buttons: &BackgroundSwatchButtons,
) {
    let family = BackgroundFamily::from_background(background);
    let show_presets = family != BackgroundFamily::Image;

    controls.presets_label.set_visible(show_presets);

    let is_gradient = matches!(background, Background::Gradient { .. });
    let is_solid = matches!(background, Background::Solid { .. });

    controls
        .gradient_presets_grid
        .set_visible(show_presets && is_gradient);
    controls
        .solid_presets_grid
        .set_visible(show_presets && is_solid);
    controls
        .signature_presets_grid
        .set_visible(show_presets && family == BackgroundFamily::Signature);
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
        Background::Image { path } => {
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("image");
            controls.image_path_label.set_label(filename);
            controls.image_path_label.set_tooltip_text(Some(path));
        }
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

fn set_selected(button: &gtk4::Button, selected: bool) {
    if selected {
        button.add_css_class("selected");
    } else {
        button.remove_css_class("selected");
    }
}
