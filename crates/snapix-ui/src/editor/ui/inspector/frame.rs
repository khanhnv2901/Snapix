use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use snapix_core::canvas::{ImageAnchor, ImageScaleMode, OutputRatio};

use super::{super::helpers::connect_frame_slider, labeled_row_with_value};
use crate::editor::state::EditorState;
use crate::widgets::DocumentCanvas;

pub(super) struct FrameSection {
    pub(super) padding_scale: gtk4::Scale,
    pub(super) padding_value: gtk4::Label,
    pub(super) radius_scale: gtk4::Scale,
    pub(super) radius_value: gtk4::Label,
}

pub(super) fn build_frame_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> FrameSection {
    let padding_value = gtk4::Label::builder()
        .label(&format!(
            "{}px",
            state.borrow().document.frame.padding as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let padding_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 160.0, 1.0);
    padding_scale.set_value(state.borrow().document.frame.padding as f64);
    {
        let value = padding_value.clone();
        connect_frame_slider(
            &padding_scale,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, slider_value| {
                frame.padding = slider_value;
                value.set_label(&format!("{}px", slider_value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        "Padding",
        &padding_scale,
        &padding_value,
    ));

    let radius_value = gtk4::Label::builder()
        .label(&format!(
            "{}px",
            state.borrow().document.frame.corner_radius as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let radius_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 48.0, 1.0);
    radius_scale.set_value(state.borrow().document.frame.corner_radius as f64);
    {
        let value = radius_value.clone();
        connect_frame_slider(
            &radius_scale,
            state,
            canvas,
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, slider_value| {
                frame.corner_radius = slider_value;
                value.set_label(&format!("{}px", slider_value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        "Corner Radius",
        &radius_scale,
        &radius_value,
    ));

    FrameSection {
        padding_scale,
        padding_value,
        radius_scale,
        radius_value,
    }
}

pub(super) fn build_ratio_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> Rc<RefCell<Vec<(OutputRatio, gtk4::Button)>>> {
    panel.append(
        &gtk4::Label::builder()
            .label("Output Ratio")
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .build(),
    );

    let ratio_options = [
        OutputRatio::Auto,
        OutputRatio::Square,
        OutputRatio::Landscape4x3,
        OutputRatio::Landscape3x2,
        OutputRatio::Landscape16x9,
        OutputRatio::Landscape5x3,
        OutputRatio::Portrait9x16,
        OutputRatio::Portrait3x4,
        OutputRatio::Portrait2x3,
    ];
    let ratio_grid = gtk4::Grid::builder()
        .row_spacing(4)
        .column_spacing(4)
        .hexpand(true)
        .build();
    let ratio_buttons: Rc<RefCell<Vec<(OutputRatio, gtk4::Button)>>> =
        Rc::new(RefCell::new(Vec::new()));
    let current_ratio = state.borrow().document().output_ratio;

    for (index, ratio) in ratio_options.into_iter().enumerate() {
        let button = gtk4::Button::builder()
            .label(ratio.label())
            .tooltip_text(ratio_tooltip(ratio))
            .hexpand(true)
            .build();
        button.add_css_class("ratio-btn");
        if ratio == current_ratio {
            button.add_css_class("selected");
        }
        let all_buttons = ratio_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| doc.output_ratio = ratio) {
                for (existing_ratio, existing_button) in all_buttons.borrow().iter() {
                    if *existing_ratio == ratio {
                        existing_button.add_css_class("selected");
                    } else {
                        existing_button.remove_css_class("selected");
                    }
                }
                super::super::helpers::refresh_subtitle(&state, &subtitle_label);
                super::super::helpers::refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
        ratio_buttons.borrow_mut().push((ratio, button.clone()));
        ratio_grid.attach(&button, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }

    panel.append(&ratio_grid);
    ratio_buttons
}

pub(super) fn build_image_fit_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> Rc<RefCell<Vec<(ImageScaleMode, gtk4::Button)>>> {
    panel.append(
        &gtk4::Label::builder()
            .label("Image Fit")
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .margin_top(8)
            .build(),
    );

    let mode_buttons: Rc<RefCell<Vec<(ImageScaleMode, gtk4::Button)>>> =
        Rc::new(RefCell::new(Vec::new()));
    let current_mode = state.borrow().document().image_scale_mode;
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();

    for mode in [ImageScaleMode::Fit, ImageScaleMode::Fill] {
        let button = gtk4::Button::builder()
            .label(mode.label())
            .tooltip_text(image_scale_mode_tooltip(mode))
            .hexpand(true)
            .build();
        button.add_css_class("ratio-btn");
        if mode == current_mode {
            button.add_css_class("selected");
        }
        let all_buttons = mode_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| doc.image_scale_mode = mode) {
                for (existing_mode, existing_button) in all_buttons.borrow().iter() {
                    if *existing_mode == mode {
                        existing_button.add_css_class("selected");
                    } else {
                        existing_button.remove_css_class("selected");
                    }
                }
                super::super::helpers::refresh_subtitle(&state, &subtitle_label);
                super::super::helpers::refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
        mode_buttons.borrow_mut().push((mode, button.clone()));
        row.append(&button);
    }

    panel.append(&row);
    mode_buttons
}

pub(super) fn build_image_position_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> Rc<RefCell<Vec<(ImageAnchor, gtk4::Button)>>> {
    panel.append(
        &gtk4::Label::builder()
            .label("Image Position")
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .margin_top(8)
            .build(),
    );

    let anchor_buttons: Rc<RefCell<Vec<(ImageAnchor, gtk4::Button)>>> =
        Rc::new(RefCell::new(Vec::new()));
    let current_anchor = state.borrow().document().image_anchor;
    let anchor_grid = gtk4::Grid::builder()
        .row_spacing(4)
        .column_spacing(4)
        .hexpand(true)
        .build();

    let anchor_options = [
        ImageAnchor::TopLeft,
        ImageAnchor::Top,
        ImageAnchor::TopRight,
        ImageAnchor::Left,
        ImageAnchor::Center,
        ImageAnchor::Right,
        ImageAnchor::BottomLeft,
        ImageAnchor::Bottom,
        ImageAnchor::BottomRight,
    ];

    for (index, anchor) in anchor_options.into_iter().enumerate() {
        let button = gtk4::Button::builder()
            .label(anchor.label())
            .tooltip_text(image_anchor_tooltip(anchor))
            .hexpand(true)
            .build();
        button.add_css_class("ratio-btn");
        if anchor == current_anchor {
            button.add_css_class("selected");
        }
        let all_buttons = anchor_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                doc.image_scale_mode = ImageScaleMode::Fill;
                doc.image_anchor = anchor;
            }) {
                for (existing_anchor, existing_button) in all_buttons.borrow().iter() {
                    if *existing_anchor == anchor {
                        existing_button.add_css_class("selected");
                    } else {
                        existing_button.remove_css_class("selected");
                    }
                }
                super::super::helpers::refresh_subtitle(&state, &subtitle_label);
                super::super::helpers::refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
        anchor_buttons.borrow_mut().push((anchor, button.clone()));
        anchor_grid.attach(&button, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }

    panel.append(&anchor_grid);
    anchor_buttons
}

fn ratio_tooltip(ratio: OutputRatio) -> &'static str {
    match ratio {
        OutputRatio::Auto => "Match the image's natural aspect ratio",
        OutputRatio::Square => "Square output, useful for thumbnails and social posts",
        OutputRatio::Landscape4x3 => "Classic landscape frame",
        OutputRatio::Landscape3x2 => "Balanced landscape frame",
        OutputRatio::Landscape16x9 => "Wide landscape frame for presentations and video stills",
        OutputRatio::Landscape5x3 => "Extra-wide landscape frame",
        OutputRatio::Portrait9x16 => "Tall portrait frame for stories and reels",
        OutputRatio::Portrait3x4 => "Classic portrait frame",
        OutputRatio::Portrait2x3 => "Photo-style portrait frame",
    }
}

fn image_scale_mode_tooltip(mode: ImageScaleMode) -> &'static str {
    match mode {
        ImageScaleMode::Fit => "Show the full image inside the output frame",
        ImageScaleMode::Fill => "Cover the whole output frame, cropping overflow if needed",
    }
}

fn image_anchor_tooltip(anchor: ImageAnchor) -> &'static str {
    match anchor {
        ImageAnchor::TopLeft => "Switch to Fill and anchor the image to the top left",
        ImageAnchor::Top => "Switch to Fill and anchor the image to the top edge",
        ImageAnchor::TopRight => "Switch to Fill and anchor the image to the top right",
        ImageAnchor::Left => "Switch to Fill and anchor the image to the left edge",
        ImageAnchor::Center => "Switch to Fill and center the image inside the frame",
        ImageAnchor::Right => "Switch to Fill and anchor the image to the right edge",
        ImageAnchor::BottomLeft => "Switch to Fill and anchor the image to the bottom left",
        ImageAnchor::Bottom => "Switch to Fill and anchor the image to the bottom edge",
        ImageAnchor::BottomRight => "Switch to Fill and anchor the image to the bottom right",
    }
}
