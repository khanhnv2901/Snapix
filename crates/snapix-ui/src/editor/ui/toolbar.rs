use std::cell::RefCell;
use std::rc::Rc;

use gtk4::cairo;
use gtk4::prelude::*;
use libadwaita::Bin;
use libadwaita::StyleManager;
use snapix_core::canvas::Color;

use crate::editor::i18n;
use super::helpers::{refresh_history_buttons, refresh_scope_label, refresh_width_label};
use super::{BottomBar, CaptureActionRow, SaveFormat};
use crate::editor::state::{same_color_rgb, EditorState, ToolKind};
use crate::widgets::DocumentCanvas;

pub(super) fn build_capture_row(bottom_bar: &BottomBar) -> CaptureActionRow {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Fill)
        .build();
    row.add_css_class("capture-row");

    let capture_section = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();
    let capture_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .valign(gtk4::Align::Center)
        .build();
    capture_box.add_css_class("capture-cluster");
    let capture_label = gtk4::Label::builder()
        .label(i18n::capture_section_label())
        .xalign(0.0)
        .css_classes(["cluster-title"])
        .build();
    capture_section.append(&capture_label);
    capture_section.append(&capture_box);

    let mut built = Vec::new();
    for (id, icon, classes) in [
        (
            "fullscreen",
            "view-fullscreen-symbolic",
            ["capture-pill", "fullscreen"],
        ),
        (
            "region",
            "view-fullscreen-symbolic",
            ["capture-pill", "region"],
        ),
        (
            "window",
            "focus-windows-symbolic",
            ["capture-pill", "window"],
        ),
        (
            "import",
            "document-open-symbolic",
            ["capture-pill", "import"],
        ),
        ("clear", "edit-clear-symbolic", ["capture-pill", "utility"]),
    ] {
        let label = i18n::capture_action_label(id);
        let tooltip = i18n::capture_action_tooltip(id);
        let icon_widget = build_capture_action_icon(id, icon);
        let text_widget = gtk4::Label::builder().label(label).xalign(0.0).build();
        text_widget.add_css_class("capture-pill-label");
        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .build();
        content.append(&icon_widget);
        content.append(&text_widget);

        let btn = gtk4::Button::builder().tooltip_text(tooltip).build();
        btn.set_child(Some(&content));
        btn.set_css_classes(&classes);
        capture_box.append(&btn);
        built.push(btn);
    }
    row.append(&capture_section);

    let spacer = gtk4::Box::builder().hexpand(true).build();
    row.append(&spacer);

    let export_section = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();
    let export_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .margin_start(12)
        .build();
    export_box.add_css_class("capture-export-row");
    let export_label = gtk4::Label::builder()
        .label(i18n::export_section_label())
        .xalign(0.0)
        .css_classes(["cluster-title"])
        .build();
    export_section.append(&export_label);
    export_section.append(&export_box);

    export_box.append(&bottom_bar.png_button);
    export_box.append(&bottom_bar.jpeg_button);

    let sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(4)
        .margin_end(4)
        .build();
    export_box.append(&sep);
    export_box.append(&bottom_bar.copy_button);
    export_box.append(&bottom_bar.quick_save_button);
    export_box.append(&bottom_bar.save_as_button);
    row.append(&export_section);

    CaptureActionRow {
        widget: row.upcast(),
        fullscreen_button: built[0].clone(),
        region_button: built[1].clone(),
        window_button: built[2].clone(),
        import_button: built[3].clone(),
        clear_button: built[4].clone(),
    }
}

fn build_capture_action_icon(id: &str, fallback_icon: &str) -> gtk4::Widget {
    match id {
        "fullscreen" => {
            let area = gtk4::DrawingArea::builder()
                .content_width(16)
                .content_height(16)
                .build();
            area.add_css_class("capture-pill-icon");
            area.set_draw_func(|_area, cr, width, height| {
                let w = width as f64;
                let h = height as f64;
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.96);
                cr.set_line_width(1.8);
                cr.rectangle(1.5, 3.5, w - 3.0, h - 7.0);
                cr.stroke().ok();
            });
            area.upcast()
        }
        "region" => {
            let area = gtk4::DrawingArea::builder()
                .content_width(16)
                .content_height(16)
                .build();
            area.add_css_class("capture-pill-icon");
            area.set_draw_func(|_area, cr, width, height| {
                let w = width as f64;
                let h = height as f64;
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.96);
                cr.set_line_width(1.8);
                cr.set_line_cap(cairo::LineCap::Round);

                cr.move_to(2.5, 6.0);
                cr.line_to(2.5, 2.5);
                cr.line_to(6.0, 2.5);

                cr.move_to(w - 6.0, 2.5);
                cr.line_to(w - 2.5, 2.5);
                cr.line_to(w - 2.5, 6.0);

                cr.move_to(2.5, h - 6.0);
                cr.line_to(2.5, h - 2.5);
                cr.line_to(6.0, h - 2.5);

                cr.move_to(w - 6.0, h - 2.5);
                cr.line_to(w - 2.5, h - 2.5);
                cr.line_to(w - 2.5, h - 6.0);
                cr.stroke().ok();
            });
            area.upcast()
        }
        _ => {
            let image = gtk4::Image::builder()
                .icon_name(fallback_icon)
                .pixel_size(16)
                .build();
            image.add_css_class("capture-pill-icon");
            image.upcast()
        }
    }
}

pub(super) fn build_tool_row(
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    width_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    delete_button: &gtk4::Button,
    shared_width_scale: Rc<RefCell<Option<gtk4::Scale>>>,
    shared_color_buttons: Rc<RefCell<Vec<((u8, u8, u8), gtk4::Button)>>>,
) -> gtk4::Widget {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
        .halign(gtk4::Align::Fill)
        .build();
    row.add_css_class("tool-row");

    let card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();
    card.add_css_class("tool-row-card");

    let width_label = width_label.clone();

    // ── Tool toggle buttons ──────────────────────────────────────────────────
    let mut tool_buttons: Vec<(ToolKind, gtk4::ToggleButton)> = Vec::new();
    for tool in [
        ToolKind::Select,
        ToolKind::Crop,
        ToolKind::Arrow,
        ToolKind::Rectangle,
        ToolKind::Ellipse,
        ToolKind::Text,
        ToolKind::Blur,
    ] {
        let btn = gtk4::ToggleButton::builder()
            .active(tool == ToolKind::Select)
            .tooltip_text(i18n::tool_tooltip(tool))
            .build();
        btn.set_child(Some(&build_tool_icon(tool)));
        btn.add_css_class("tool-pill");
        card.append(&btn);
        tool_buttons.push((tool, btn));
    }

    let btn_refs: Vec<(ToolKind, gtk4::ToggleButton)> =
        tool_buttons.iter().map(|(t, b)| (*t, b.clone())).collect();
    for (tool, btn) in &tool_buttons {
        let state = state.clone();
        let canvas = canvas.clone();
        let title_label = title_label.clone();
        let scope_label = scope_label.clone();
        let width_label = width_label.clone();
        let all = btn_refs.clone();
        let tool = *tool;
        btn.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            state.set_active_tool(tool);
            title_label.set_label(&i18n::editor_header_title(tool));
            refresh_scope_label(&state, &scope_label);
            refresh_width_label(&state, &width_label);
            for (bt, b) in &all {
                b.set_active(*bt == state.active_tool());
            }
            canvas.refresh();
        });
    }

    // ── Separator ────────────────────────────────────────────────────────────
    card.append(
        &gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_top(6)
            .margin_bottom(6)
            .build(),
    );

    // ── Color palette swatches ───────────────────────────────────────────────
    let palette: &[((u8, u8, u8), &str)] = &[
        ((255, 98, 54), "color-dot-0"),
        ((229, 57, 53), "color-dot-1"),
        ((233, 30, 140), "color-dot-2"),
        ((124, 77, 255), "color-dot-3"),
        ((33, 150, 243), "color-dot-4"),
        ((0, 150, 136), "color-dot-5"),
        ((76, 175, 80), "color-dot-6"),
        ((255, 214, 0), "color-dot-7"),
        ((240, 240, 240), "color-dot-8"),
        ((30, 30, 46), "color-dot-9"),
    ];

    let color_btns: Rc<RefCell<Vec<gtk4::Button>>> = Rc::new(RefCell::new(Vec::new()));
    let init_color = state.borrow().active_color();

    for (i, ((r, g, b), dot_class)) in palette.iter().enumerate() {
        let color = Color {
            r: *r,
            g: *g,
            b: *b,
            a: 255,
        };
        let dot = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();
        dot.set_size_request(16, 16);
        dot.add_css_class("color-dot");
        dot.add_css_class(dot_class);

        let btn = gtk4::Button::builder()
            .tooltip_text(i18n::color_name(i))
            .child(&dot)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .build();
        btn.add_css_class("color-swatch-btn");
        if same_color_rgb(*r, *g, *b, &init_color) {
            btn.add_css_class("active");
        }

        let state = state.clone();
        let canvas = canvas.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let color_btns_ref = color_btns.clone();
        btn.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            state.set_active_color(color.clone());
            let changed = state.apply_active_color_to_selected();
            for (j, b) in color_btns_ref.borrow().iter().enumerate() {
                if j == i {
                    b.add_css_class("active");
                } else {
                    b.remove_css_class("active");
                }
            }
            if changed {
                refresh_history_buttons(&state, &undo_button, &redo_button);
            }
            canvas.refresh();
        });
        color_btns.borrow_mut().push(btn.clone());
        shared_color_buttons
            .borrow_mut()
            .push(((*r, *g, *b), btn.clone()));
        card.append(&btn);
    }

    // ── Separator ────────────────────────────────────────────────────────────
    card.append(
        &gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_top(6)
            .margin_bottom(6)
            .build(),
    );

    // ── Width selector ───────────────────────────────────────────────────────
    let init_width = state.borrow().active_width();
    card.append(&width_label);

    let width_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 1.0, 30.0, 1.0);
    width_scale.set_value(init_width as f64);
    width_scale.set_size_request(200, -1);
    width_scale.set_valign(gtk4::Align::Center);
    *shared_width_scale.borrow_mut() = Some(width_scale.clone());
    card.append(&width_scale);

    let state_w = state.clone();
    let canvas_w = canvas.clone();
    let undo_w = undo_button.clone();
    let redo_w = redo_button.clone();
    let width_label_ref = width_label.clone();
    width_scale.connect_value_changed(move |scale| {
        let val = scale.value() as f32;
        let mut s = state_w.borrow_mut();
        s.set_active_width(val);
        refresh_width_label(&s, &width_label_ref);
        if s.apply_active_width_to_selected() {
            refresh_history_buttons(&s, &undo_w, &redo_w);
        }
        canvas_w.refresh();
    });

    // ── Spacer + Delete ───────────────────────────────────────────────────────
    let spacer = gtk4::Box::builder().hexpand(true).build();
    card.append(&spacer);

    {
        let state = state.clone();
        let canvas = canvas.clone();
        let title_label = title_label.clone();
        let scope_label = scope_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let width_label = width_label.clone();
        let delete_btn_ref = delete_button.clone();
        delete_button.connect_clicked(move |_| {
            let mut s = state.borrow_mut();
            if s.delete_selected_annotation() {
                refresh_scope_label(&s, &scope_label);
                refresh_history_buttons(&s, &undo_button, &redo_button);
                refresh_width_label(&s, &width_label);
                delete_btn_ref.set_sensitive(false);
                title_label.set_label(&i18n::editor_header_title(s.active_tool()));
                canvas.refresh();
            }
        });
    }
    card.append(delete_button);

    row.append(&card);
    row.upcast()
}

fn build_tool_icon(tool: ToolKind) -> gtk4::Widget {
    let icon = gtk4::DrawingArea::builder()
        .content_width(24)
        .content_height(24)
        .build();
    icon.set_draw_func(move |_area, cr, width, height| {
        let actual_w = width as f64;
        let actual_h = height as f64;
        cr.scale(actual_w / 20.0, actual_h / 20.0);
        let w = 20.0;
        let h = 20.0;
        cr.set_line_cap(cairo::LineCap::Round);
        cr.set_line_join(cairo::LineJoin::Round);
        let is_dark = StyleManager::default().is_dark();
        let (r, g, b, alpha, fill_alpha, accent_alpha) = if is_dark {
            (0.93, 0.95, 1.0, 0.92, 0.20, 0.55)
        } else {
            (0.19, 0.24, 0.32, 0.92, 0.10, 0.45)
        };

        match tool {
            ToolKind::Select => {
                // Cursor arrow pointing top-left
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(1.6);
                cr.move_to(3.5, 2.0); // tip
                cr.line_to(3.5, 14.5); // bottom-left
                cr.line_to(7.0, 11.0); // notch
                cr.line_to(9.5, 16.5); // spike bottom
                cr.line_to(11.5, 15.0); // spike right
                cr.line_to(8.5, 9.5); // spike top join
                cr.line_to(13.5, 9.5); // right side
                cr.close_path();
                cr.stroke_preserve().ok();
                cr.set_source_rgba(r, g, b, fill_alpha);
                cr.fill().ok();
            }
            ToolKind::Crop => {
                // Two L-bracket corners (standard crop icon)
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(1.9);
                cr.move_to(4.5, 9.5);
                cr.line_to(4.5, 4.5);
                cr.line_to(9.5, 4.5);
                cr.move_to(10.5, 15.5);
                cr.line_to(15.5, 15.5);
                cr.line_to(15.5, 10.5);
                cr.stroke().ok();
            }
            ToolKind::Arrow => {
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(1.8);
                // Shaft from bottom-left up to arrowhead base
                cr.move_to(4.0, 16.0);
                cr.line_to(11.0, 9.0);
                cr.stroke().ok();
                // Filled arrowhead triangle (45° direction, tip at top-right)
                // tip=(15.5,4.5), wings symmetric around 45° axis
                cr.move_to(15.5, 4.5);
                cr.line_to(13.0, 10.5);
                cr.line_to(9.5, 7.0);
                cr.close_path();
                cr.fill().ok();
            }
            ToolKind::Rectangle => {
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(1.8);
                cr.rectangle(3.5, 5.0, w - 7.0, h - 10.0);
                cr.stroke().ok();
            }
            ToolKind::Ellipse => {
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(1.8);
                cr.save().ok();
                cr.translate(w / 2.0, h / 2.0);
                cr.scale(7.0, 5.5);
                cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
                cr.restore().ok();
                cr.stroke().ok();
            }
            ToolKind::Text => {
                // Stroke-based T (consistent with other icons, no font rendering)
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(2.0);
                cr.move_to(4.5, 5.5);
                cr.line_to(15.5, 5.5);
                cr.stroke().ok();
                cr.move_to(10.0, 5.5);
                cr.line_to(10.0, 16.0);
                cr.stroke().ok();
                cr.set_line_width(1.8);
                cr.move_to(7.5, 16.0);
                cr.line_to(12.5, 16.0);
                cr.stroke().ok();
            }
            ToolKind::Blur => {
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(1.7);
                // Outer rectangle
                cr.rectangle(3.5, 5.5, 13.0, 9.0);
                cr.stroke().ok();
                // Horizontal lines inside suggesting blur/scan effect
                cr.set_source_rgba(r, g, b, accent_alpha);
                cr.set_line_width(1.1);
                for y_pos in [8.0_f64, 10.0, 12.0] {
                    cr.move_to(5.5, y_pos);
                    cr.line_to(14.5, y_pos);
                    cr.stroke().ok();
                }
            }
        }
    });
    icon.upcast()
}

pub(super) fn build_canvas_panel(canvas_widget: gtk4::DrawingArea) -> gtk4::Widget {
    let frame = gtk4::Frame::builder().hexpand(true).vexpand(true).build();
    frame.add_css_class("canvas-card");
    frame.set_child(Some(&canvas_widget));

    let wrap = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    wrap.add_css_class("canvas-wrap");
    wrap.append(&frame);

    Bin::builder().child(&wrap).build().upcast()
}

pub(super) fn build_bottom_bar(
    subtitle_label: &gtk4::Label,
    save_format: Rc<RefCell<SaveFormat>>,
) -> BottomBar {
    let bar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
        .halign(gtk4::Align::Fill)
        .build();
    bar.add_css_class("bottom-bar");

    // Dimensions label (left side)
    let dims_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .margin_start(16)
        .build();
    dims_box.append(subtitle_label);

    // Format toggle
    let png_btn = gtk4::ToggleButton::builder()
        .label("PNG")
        .css_classes(["format-pill"])
        .tooltip_text("Export with lossless quality")
        .build();
    let jpg_btn = gtk4::ToggleButton::builder()
        .label("JPEG")
        .active(false)
        .css_classes(["format-pill"])
        .tooltip_text("Export smaller files with lossy compression")
        .build();
    jpg_btn.set_group(Some(&png_btn));
    match *save_format.borrow() {
        SaveFormat::Png => png_btn.set_active(true),
        SaveFormat::Jpeg => jpg_btn.set_active(true),
    }

    {
        let sf = save_format.clone();
        png_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                *sf.borrow_mut() = SaveFormat::Png;
            }
        });
    }
    {
        let sf = save_format.clone();
        jpg_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                *sf.borrow_mut() = SaveFormat::Jpeg;
            }
        });
    }

    let copy_btn = gtk4::Button::builder()
        .label("Copy")
        .css_classes(["bottom-action-btn"])
        .tooltip_text("Copy the current image to the clipboard")
        .build();
    let quick_save_btn = gtk4::Button::builder()
        .label("Save")
        .css_classes(["bottom-action-btn", "suggested-action"])
        .tooltip_text("Save to your Pictures folder using the selected format")
        .build();
    let save_as_btn = gtk4::Button::builder()
        .label("Save As…")
        .css_classes(["bottom-action-btn"])
        .tooltip_text("Choose where to export the current image")
        .build();

    bar.append(&dims_box);

    BottomBar {
        widget: bar.upcast(),
        copy_button: copy_btn,
        quick_save_button: quick_save_btn,
        save_as_button: save_as_btn,
        png_button: png_btn,
        jpeg_button: jpg_btn,
    }
}
