use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita::{
    Application, ApplicationWindow, Bin, Clamp, HeaderBar, StatusPage, ToolbarView,
};
use snapix_core::canvas::{Background, Color, Document, FrameSettings, Image};

use crate::widgets::DocumentCanvas;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolKind {
    Select,
    Crop,
    Arrow,
    Text,
    Blur,
}

impl ToolKind {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Crop => "Crop",
            Self::Arrow => "Arrow",
            Self::Text => "Text",
            Self::Blur => "Blur",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EditorState {
    document: Document,
    active_tool: ToolKind,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            document: Document::default(),
            active_tool: ToolKind::Select,
        }
    }
}

impl EditorState {
    pub(crate) fn document(&self) -> &Document {
        &self.document
    }
}

pub struct EditorWindow {
    window: ApplicationWindow,
}

impl EditorWindow {
    pub fn new(app: &Application) -> Self {
        let state = Rc::new(RefCell::new(EditorState::default()));

        let title_label = gtk4::Label::builder()
            .xalign(0.0)
            .css_classes(["title-2"])
            .label("Editor")
            .build();

        let subtitle_label = gtk4::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .css_classes(["dim-label"])
            .build();

        let canvas = DocumentCanvas::new(state.clone());
        let canvas_widget = canvas.widget().clone();

        let inspector = build_inspector(state.clone(), canvas.clone(), &subtitle_label);
        let tools = build_tool_rail(state.clone(), canvas.clone(), &title_label);
        let canvas_panel = build_canvas_panel(canvas_widget);

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(0)
            .build();
        content.append(&tools);
        content.append(&canvas_panel);
        content.append(&inspector);

        let title_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(3)
            .margin_top(18)
            .margin_start(18)
            .margin_end(18)
            .margin_bottom(12)
            .build();
        title_box.append(&title_label);
        title_box.append(&subtitle_label);

        let capture_hint = build_capture_hint();

        let body = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .build();
        body.append(&title_box);
        body.append(&capture_hint);
        body.append(&content);

        let header = HeaderBar::new();
        header.pack_start(
            &gtk4::Button::builder()
                .icon_name("camera-photo-symbolic")
                .tooltip_text("Capture flow will be wired in the next M2 step")
                .sensitive(false)
                .build(),
        );
        header.pack_end(
            &gtk4::Button::builder()
                .label("Copy")
                .tooltip_text("Clipboard export is not wired yet")
                .sensitive(false)
                .build(),
        );
        header.pack_end(
            &gtk4::Button::builder()
                .label("Save")
                .tooltip_text("Export flow is not wired yet")
                .sensitive(false)
                .build(),
        );

        let toolbar_view = ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&body));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Snapix")
            .default_width(1280)
            .default_height(820)
            .content(&toolbar_view)
            .build();

        refresh_labels(&state.borrow(), &title_label, &subtitle_label);
        canvas.refresh();

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn build_capture_hint() -> gtk4::Widget {
    let hint = gtk4::InfoBar::builder()
        .message_type(gtk4::MessageType::Info)
        .revealed(true)
        .show_close_button(false)
        .build();
    hint.add_child(
        &gtk4::Label::builder()
            .label("M2 foundation: editor shell is live. Capture, export, and tool interactions are the next wiring steps.")
            .wrap(true)
            .xalign(0.0)
            .build(),
    );
    hint.upcast()
}

fn build_canvas_panel(canvas_widget: gtk4::DrawingArea) -> gtk4::Widget {
    let frame = gtk4::Frame::builder()
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .build();
    frame.set_child(Some(&canvas_widget));

    let clamp = Clamp::builder()
        .maximum_size(1600)
        .hexpand(true)
        .vexpand(true)
        .child(&frame)
        .build();

    let scroller = gtk4::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_width(720)
        .child(&clamp)
        .build();

    Bin::builder().child(&scroller).build().upcast()
}

fn build_tool_rail(
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
) -> gtk4::Widget {
    let rail = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_top(12)
        .margin_start(12)
        .margin_bottom(12)
        .width_request(140)
        .build();

    rail.append(
        &gtk4::Label::builder()
            .label("Tools")
            .xalign(0.0)
            .css_classes(["heading"])
            .build(),
    );

    for tool in [
        ToolKind::Select,
        ToolKind::Crop,
        ToolKind::Arrow,
        ToolKind::Text,
        ToolKind::Blur,
    ] {
        let state = state.clone();
        let canvas = canvas.clone();
        let title_label = title_label.clone();
        let button = gtk4::ToggleButton::builder()
            .label(tool.label())
            .active(tool == ToolKind::Select)
            .halign(gtk4::Align::Fill)
            .build();

        button.connect_clicked(move |btn| {
            let mut state = state.borrow_mut();
            state.active_tool = tool;
            title_label.set_label(&format!("Editor • {}", state.active_tool.label()));
            canvas.refresh();
            btn.set_active(true);
        });

        rail.append(&button);
    }

    rail.append(
        &gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_top(6)
            .margin_bottom(6)
            .build(),
    );

    rail.append(
        &gtk4::Label::builder()
            .label("Current Scope")
            .xalign(0.0)
            .css_classes(["heading"])
            .build(),
    );

    rail.append(
        &gtk4::Label::builder()
            .label("Editor shell only. No interactive annotations have been wired yet.")
            .wrap(true)
            .xalign(0.0)
            .css_classes(["dim-label"])
            .build(),
    );

    rail.upcast()
}

fn build_inspector(
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
) -> gtk4::Widget {
    let panel = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(12)
        .width_request(280)
        .build();

    panel.append(
        &gtk4::Label::builder()
            .label("Inspector")
            .xalign(0.0)
            .css_classes(["title-4"])
            .build(),
    );

    let padding = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 160.0, 1.0);
    padding.set_value(state.borrow().document.frame.padding as f64);
    connect_frame_slider(
        &padding,
        state.clone(),
        canvas.clone(),
        subtitle_label,
        |frame, value| frame.padding = value,
    );
    panel.append(&labeled_row("Padding", &padding));

    let radius = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 48.0, 1.0);
    radius.set_value(state.borrow().document.frame.corner_radius as f64);
    connect_frame_slider(
        &radius,
        state.clone(),
        canvas.clone(),
        subtitle_label,
        |frame, value| frame.corner_radius = value,
    );
    panel.append(&labeled_row("Corner Radius", &radius));

    let shadow = gtk4::Switch::builder()
        .active(state.borrow().document.frame.shadow)
        .halign(gtk4::Align::End)
        .build();
    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        shadow.connect_active_notify(move |switch| {
            state.borrow_mut().document.frame.shadow = switch.is_active();
            refresh_subtitle(&state.borrow(), &subtitle_label);
            canvas.refresh();
        });
    }
    panel.append(&labeled_row("Shadow", &shadow));

    let bg_group = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .build();
    bg_group.append(
        &gtk4::Label::builder()
            .label("Background")
            .xalign(0.0)
            .css_classes(["heading"])
            .build(),
    );

    for (label, background) in [
        (
            "Cornflower",
            Background::Gradient {
                from: Color {
                    r: 100,
                    g: 149,
                    b: 237,
                    a: 255,
                },
                to: Color {
                    r: 33,
                    g: 53,
                    b: 85,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Sunset",
            Background::Gradient {
                from: Color {
                    r: 255,
                    g: 180,
                    b: 112,
                    a: 255,
                },
                to: Color {
                    r: 230,
                    g: 92,
                    b: 70,
                    a: 255,
                },
                angle_deg: 135.0,
            },
        ),
        (
            "Slate",
            Background::Solid {
                color: Color {
                    r: 31,
                    g: 36,
                    b: 45,
                    a: 255,
                },
            },
        ),
    ] {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let button = gtk4::Button::builder().label(label).halign(gtk4::Align::Fill).build();
        button.connect_clicked(move |_| {
            state.borrow_mut().document.background = background.clone();
            refresh_subtitle(&state.borrow(), &subtitle_label);
            canvas.refresh();
        });
        bg_group.append(&button);
    }
    panel.append(&bg_group);

    panel.append(
        &StatusPage::builder()
            .icon_name("document-properties-symbolic")
            .title("M2 Direction")
            .description("Next steps here are capture wiring, real image rendering, and annotation interactions.")
            .build(),
    );

    panel.upcast()
}

fn labeled_row<W: IsA<gtk4::Widget>>(label: &str, widget: &W) -> gtk4::Widget {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .build();

    row.append(
        &gtk4::Label::builder()
            .label(label)
            .xalign(0.0)
            .build(),
    );
    row.append(widget);
    row.upcast()
}

fn connect_frame_slider<F>(
    scale: &gtk4::Scale,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    update: F,
) where
    F: Fn(&mut FrameSettings, f32) + 'static,
{
    let subtitle_label = subtitle_label.clone();
    scale.connect_value_changed(move |scale| {
        let mut state = state.borrow_mut();
        update(&mut state.document.frame, scale.value() as f32);
        refresh_subtitle(&state, &subtitle_label);
        canvas.refresh();
    });
}

fn refresh_labels(state: &EditorState, title_label: &gtk4::Label, subtitle_label: &gtk4::Label) {
    title_label.set_label(&format!("Editor • {}", state.active_tool.label()));
    refresh_subtitle(state, subtitle_label);
}

fn refresh_subtitle(state: &EditorState, subtitle_label: &gtk4::Label) {
    let frame = &state.document.frame;
    let image_summary = match state.document.base_image.as_ref() {
        Some(Image { width, height, .. }) => format!("{width}×{height} image loaded"),
        None => "No screenshot loaded yet".to_string(),
    };
    subtitle_label.set_label(&format!(
        "{image_summary} • padding {} • radius {} • shadow {}",
        frame.padding as u32,
        frame.corner_radius as u32,
        if frame.shadow { "on" } else { "off" }
    ));
}
