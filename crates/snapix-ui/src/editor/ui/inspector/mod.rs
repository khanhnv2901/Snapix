mod background;
mod frame;
mod shadow;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use super::InspectorControls;
use crate::editor::state::EditorState;
use crate::widgets::DocumentCanvas;

use background::build_background_section;
use frame::{
    build_frame_section, build_image_fit_section, build_image_position_section, build_ratio_section,
};
use shadow::build_shadow_section;

pub(super) fn build_inspector(
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> InspectorControls {
    let panel = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(16)
        .width_request(260)
        .valign(gtk4::Align::Start)
        .build();
    panel.add_css_class("inspector-card");

    panel.append(
        &gtk4::Label::builder()
            .label("Settings")
            .xalign(0.0)
            .css_classes(["title-4", "section-title"])
            .build(),
    );

    let frame_section = build_frame_section(
        &panel,
        state.clone(),
        canvas.clone(),
        subtitle_label,
        undo_button,
        redo_button,
    );
    let shadow_section = build_shadow_section(
        &panel,
        state.clone(),
        canvas.clone(),
        subtitle_label,
        undo_button,
        redo_button,
    );

    panel.append(
        &gtk4::Separator::builder()
            .margin_top(2)
            .margin_bottom(2)
            .build(),
    );

    let ratio_buttons = build_ratio_section(
        &panel,
        state.clone(),
        canvas.clone(),
        subtitle_label,
        undo_button,
        redo_button,
    );

    let image_scale_mode_buttons = build_image_fit_section(
        &panel,
        state.clone(),
        canvas.clone(),
        subtitle_label,
        undo_button,
        redo_button,
    );

    let image_anchor_buttons = build_image_position_section(
        &panel,
        state.clone(),
        canvas.clone(),
        subtitle_label,
        undo_button,
        redo_button,
    );

    panel.append(
        &gtk4::Separator::builder()
            .margin_top(2)
            .margin_bottom(2)
            .build(),
    );

    let background_section = build_background_section(
        &panel,
        state,
        canvas,
        subtitle_label,
        undo_button,
        redo_button,
    );

    let scroller = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(0)
        .propagate_natural_height(false)
        .width_request(260)
        .build();
    scroller.set_child(Some(&panel));

    InspectorControls {
        widget: scroller.upcast(),
        padding_scale: frame_section.padding_scale,
        padding_value: frame_section.padding_value,
        radius_scale: frame_section.radius_scale,
        radius_value: frame_section.radius_value,
        shadow_switch: shadow_section.shadow_switch,
        shadow_direction_buttons: shadow_section.shadow_direction_buttons,
        shadow_padding_scale: shadow_section.shadow_padding_scale,
        shadow_padding_value: shadow_section.shadow_padding_value,
        shadow_blur_scale: shadow_section.shadow_blur_scale,
        shadow_blur_value: shadow_section.shadow_blur_value,
        shadow_strength_scale: shadow_section.shadow_strength_scale,
        shadow_strength_value: shadow_section.shadow_strength_value,
        ratio_buttons,
        image_scale_mode_buttons,
        image_anchor_buttons,
        background_buttons: background_section.swatch_buttons,
        background_blur_button: background_section.blur_button,
        background_blur_scale: background_section.blur_radius_scale,
        background_blur_value: background_section.blur_radius_value,
    }
}

fn labeled_row<W: IsA<gtk4::Widget>>(label: &str, widget: &W) -> gtk4::Widget {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .build();
    row.append(&gtk4::Label::builder().label(label).xalign(0.0).build());
    row.append(widget);
    row.upcast()
}

fn labeled_row_with_value<W: IsA<gtk4::Widget>>(
    label: &str,
    widget: &W,
    value_label: &gtk4::Label,
) -> gtk4::Widget {
    let header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .build();
    header.append(
        &gtk4::Label::builder()
            .label(label)
            .xalign(0.0)
            .hexpand(true)
            .build(),
    );
    header.append(value_label);

    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(4)
        .build();
    row.append(&header);
    row.append(widget);
    row.upcast()
}
