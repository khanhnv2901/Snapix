use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use snapix_core::canvas::{Background, Color};

use super::super::helpers::{refresh_history_buttons, refresh_subtitle};
use super::labeled_row_with_value;
use crate::editor::state::{same_background, EditorState};
use crate::widgets::DocumentCanvas;

pub(super) struct BackgroundSection {
    pub(super) swatch_buttons: Rc<RefCell<Vec<(Background, gtk4::Button)>>>,
    pub(super) blur_button: gtk4::Button,
    pub(super) blur_radius_scale: gtk4::Scale,
    pub(super) blur_radius_value: gtk4::Label,
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
            .label("Background")
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .build(),
    );

    let current_background = state.borrow().document.background.clone();
    let swatch_buttons: Rc<RefCell<Vec<(Background, gtk4::Button)>>> =
        Rc::new(RefCell::new(Vec::new()));

    let blur_button = gtk4::Button::builder()
        .label("Screenshot Blur")
        .tooltip_text("Use the captured image as a blurred background fill")
        .hexpand(true)
        .build();
    blur_button.add_css_class("ratio-btn");
    if matches!(&current_background, Background::BlurredScreenshot { .. }) {
        blur_button.add_css_class("selected");
    }

    let blur_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    blur_row.append(&blur_button);
    panel.append(&blur_row);

    let current_blur_radius = match &current_background {
        Background::BlurredScreenshot { radius } => *radius,
        _ => 24.0,
    };
    let blur_radius_value = gtk4::Label::builder()
        .label(&format!("{}px", current_blur_radius.round() as u32))
        .css_classes(["dim-copy"])
        .build();
    let blur_radius_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 4.0, 64.0, 1.0);
    blur_radius_scale.set_value(current_blur_radius as f64);
    blur_radius_scale.set_sensitive(matches!(
        &state.borrow().document().background,
        Background::BlurredScreenshot { .. }
    ));

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
    ];

    {
        let swatch_buttons = swatch_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let blur_radius_scale = blur_radius_scale.clone();
        let blur_button_for_handler = blur_button.clone();
        blur_button.connect_clicked(move |_| {
            let radius = match &state.borrow().document().background {
                Background::BlurredScreenshot { radius } => *radius,
                _ => blur_radius_scale.value() as f32,
            };
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                doc.background = Background::BlurredScreenshot { radius };
            }) {
                blur_button_for_handler.add_css_class("selected");
                blur_radius_scale.set_sensitive(true);
                for (_, existing_button) in swatch_buttons.borrow().iter() {
                    existing_button.remove_css_class("selected");
                }
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
        blur_radius_scale.connect_value_changed(move |scale| {
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

    panel.append(&labeled_row_with_value(
        "Blur Radius",
        &blur_radius_scale,
        &blur_radius_value,
    ));

    panel.append(
        &gtk4::Label::builder()
            .label("Presets")
            .xalign(0.0)
            .css_classes(["dim-copy"])
            .margin_top(4)
            .build(),
    );

    let swatch_grid = gtk4::Grid::builder()
        .row_spacing(6)
        .column_spacing(6)
        .build();

    for (index, (label, css_class, background)) in presets.into_iter().enumerate() {
        let button = gtk4::Button::builder()
            .tooltip_text(label)
            .hexpand(true)
            .vexpand(false)
            .build();
        button.add_css_class("background-swatch");
        button.add_css_class(css_class);
        if same_background(&current_background, &background) {
            button.add_css_class("selected");
        }

        swatch_buttons
            .borrow_mut()
            .push((background.clone(), button.clone()));
        let all_buttons = swatch_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let blur_button = blur_button.clone();
        let blur_radius_scale = blur_radius_scale.clone();
        button.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| doc.background = background.clone()) {
                for (existing_background, existing_button) in all_buttons.borrow().iter() {
                    if same_background(existing_background, &background) {
                        existing_button.add_css_class("selected");
                    } else {
                        existing_button.remove_css_class("selected");
                    }
                }
                blur_button.remove_css_class("selected");
                blur_radius_scale.set_sensitive(false);
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
        swatch_grid.attach(&button, (index % 4) as i32, (index / 4) as i32, 1, 1);
    }

    panel.append(&swatch_grid);

    BackgroundSection {
        swatch_buttons,
        blur_button,
        blur_radius_scale,
        blur_radius_value,
    }
}
