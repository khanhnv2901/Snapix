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
            .label("Image Reframe")
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .margin_top(8)
            .build(),
    );

    panel.append(
        &gtk4::Label::builder()
            .label("Double-click the image in Select mode, then drag to reposition and use the mouse wheel to zoom.")
            .xalign(0.0)
            .wrap(true)
            .css_classes(["dim-copy"])
            .build(),
    );

    let reset_button = gtk4::Button::builder()
        .label("Reset View")
        .halign(gtk4::Align::Start)
        .build();
    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        reset_button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.reset_image_reframe() {
                super::super::helpers::refresh_subtitle(&state, &subtitle_label);
                super::super::helpers::refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }
    panel.append(&reset_button);

    Rc::new(RefCell::new(Vec::new()))
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
            .label("Reset View returns the image to Fit and clears any manual pan or zoom.")
            .xalign(0.0)
            .wrap(true)
            .css_classes(["dim-copy"])
            .build(),
    );
    let _ = (state, canvas, subtitle_label, undo_button, redo_button);
    Rc::new(RefCell::new(Vec::new()))
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
