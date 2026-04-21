use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use snapix_core::canvas::{Background, Color};

use super::super::helpers::{refresh_history_buttons, refresh_subtitle};
use crate::editor::state::{same_background, EditorState};
use crate::widgets::DocumentCanvas;

pub(super) fn build_background_section(
    panel: &gtk4::Box,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> Rc<RefCell<Vec<(Background, gtk4::Button)>>> {
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

    let presets: Vec<(&str, &str, Background)> = vec![
        (
            "Cornflower",
            "swatch-cornflower",
            Background::Gradient {
                from: Color { r: 110, g: 162, b: 255, a: 255 },
                to: Color { r: 130, g: 99, b: 245, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Sunset",
            "swatch-sunset",
            Background::Gradient {
                from: Color { r: 255, g: 180, b: 108, a: 255 },
                to: Color { r: 232, g: 93, b: 68, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Ocean",
            "swatch-ocean",
            Background::Gradient {
                from: Color { r: 56, g: 189, b: 248, a: 255 },
                to: Color { r: 15, g: 118, b: 110, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Forest",
            "swatch-forest",
            Background::Gradient {
                from: Color { r: 74, g: 222, b: 128, a: 255 },
                to: Color { r: 21, g: 128, b: 61, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Rose",
            "swatch-rose",
            Background::Gradient {
                from: Color { r: 249, g: 168, b: 212, a: 255 },
                to: Color { r: 190, g: 24, b: 93, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Midnight",
            "swatch-midnight",
            Background::Gradient {
                from: Color { r: 99, g: 102, b: 241, a: 255 },
                to: Color { r: 30, g: 27, b: 75, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Golden",
            "swatch-golden",
            Background::Gradient {
                from: Color { r: 251, g: 191, b: 36, a: 255 },
                to: Color { r: 180, g: 83, b: 9, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Lavender",
            "swatch-lavender",
            Background::Gradient {
                from: Color { r: 196, g: 181, b: 253, a: 255 },
                to: Color { r: 124, g: 58, b: 237, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Mint",
            "swatch-mint",
            Background::Gradient {
                from: Color { r: 110, g: 231, b: 183, a: 255 },
                to: Color { r: 13, g: 148, b: 136, a: 255 },
                angle_deg: 135.0,
            },
        ),
        (
            "Slate",
            "swatch-slate",
            Background::Solid {
                color: Color { r: 31, g: 36, b: 45, a: 255 },
            },
        ),
        (
            "Charcoal",
            "swatch-charcoal",
            Background::Solid {
                color: Color { r: 45, g: 55, b: 72, a: 255 },
            },
        ),
        (
            "Deep Space",
            "swatch-deepspace",
            Background::Gradient {
                from: Color { r: 26, g: 26, b: 46, a: 255 },
                to: Color { r: 22, g: 33, b: 62, a: 255 },
                angle_deg: 135.0,
            },
        ),
    ];

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
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
        swatch_grid.attach(&button, (index % 4) as i32, (index / 4) as i32, 1, 1);
    }

    panel.append(&swatch_grid);
    swatch_buttons
}
