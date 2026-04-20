use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use gio::prelude::FileExt;
use gtk4::cairo;
use gtk4::prelude::*;
use libadwaita::{Application, ApplicationWindow, Bin, HeaderBar, ToolbarView};
use snapix_core::canvas::{
    Annotation, Background, Color, Document, FrameSettings, Image, Point, Rect, TextStyle,
};

use crate::app::LaunchContext;
use crate::widgets::{render_document_rgba, DocumentCanvas};

// ─── Tool kind ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolKind {
    Select,
    Crop,
    Arrow,
    Rectangle,
    Ellipse,
    Text,
    Blur,
}

impl ToolKind {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Crop => "Crop",
            Self::Arrow => "Arrow",
            Self::Rectangle => "Rect",
            Self::Ellipse => "Ellipse",
            Self::Text => "Text",
            Self::Blur => "Blur",
        }
    }
}

// ─── Drag / selection state ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct CropDrag {
    start_x: f64,
    start_y: f64,
    current_x: f64,
    current_y: f64,
}

impl CropDrag {
    pub(crate) fn start_x(&self) -> f64 {
        self.start_x
    }
    pub(crate) fn start_y(&self) -> f64 {
        self.start_y
    }
    pub(crate) fn current_x(&self) -> f64 {
        self.current_x
    }
    pub(crate) fn current_y(&self) -> f64 {
        self.current_y
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CropSelection {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl CropSelection {
    pub(crate) fn x(&self) -> u32 {
        self.x
    }
    pub(crate) fn y(&self) -> u32 {
        self.y
    }
    pub(crate) fn width(&self) -> u32 {
        self.width
    }
    pub(crate) fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ArrowDrag {
    widget_start_x: f64,
    widget_start_y: f64,
    start_x: f32,
    start_y: f32,
    current_x: f32,
    current_y: f32,
}

impl ArrowDrag {
    pub(crate) fn widget_start_x(&self) -> f64 {
        self.widget_start_x
    }
    pub(crate) fn widget_start_y(&self) -> f64 {
        self.widget_start_y
    }
    pub(crate) fn start_x(&self) -> f32 {
        self.start_x
    }
    pub(crate) fn start_y(&self) -> f32 {
        self.start_y
    }
    pub(crate) fn current_x(&self) -> f32 {
        self.current_x
    }
    pub(crate) fn current_y(&self) -> f32 {
        self.current_y
    }
}

// ─── Editor state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct EditorState {
    document: Document,
    active_tool: ToolKind,
    active_color: Color,
    active_width: f32,
    crop_drag: Option<CropDrag>,
    crop_selection: Option<CropSelection>,
    arrow_drag: Option<ArrowDrag>,
    rect_drag: Option<CropDrag>,
    ellipse_drag: Option<CropDrag>,
    blur_drag: Option<CropDrag>,
    undo_stack: Vec<Document>,
    redo_stack: Vec<Document>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            document: Document::default(),
            active_tool: ToolKind::Select,
            active_color: Color {
                r: 255,
                g: 98,
                b: 54,
                a: 255,
            },
            active_width: 6.0,
            crop_drag: None,
            crop_selection: None,
            arrow_drag: None,
            rect_drag: None,
            ellipse_drag: None,
            blur_drag: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl EditorState {
    fn with_document(document: Document) -> Self {
        Self {
            document,
            ..Self::default()
        }
    }

    pub(crate) fn document(&self) -> &Document {
        &self.document
    }
    pub(crate) fn active_tool(&self) -> ToolKind {
        self.active_tool
    }
    pub(crate) fn active_color(&self) -> Color {
        self.active_color.clone()
    }
    pub(crate) fn active_width(&self) -> f32 {
        self.active_width
    }
    pub(crate) fn crop_drag(&self) -> Option<&CropDrag> {
        self.crop_drag.as_ref()
    }
    pub(crate) fn arrow_drag(&self) -> Option<&ArrowDrag> {
        self.arrow_drag.as_ref()
    }
    pub(crate) fn rect_drag(&self) -> Option<&CropDrag> {
        self.rect_drag.as_ref()
    }
    pub(crate) fn ellipse_drag(&self) -> Option<&CropDrag> {
        self.ellipse_drag.as_ref()
    }
    pub(crate) fn blur_drag(&self) -> Option<&CropDrag> {
        self.blur_drag.as_ref()
    }

    pub(crate) fn set_active_color(&mut self, color: Color) {
        self.active_color = color;
    }
    pub(crate) fn set_active_width(&mut self, width: f32) {
        self.active_width = width;
    }

    pub(crate) fn set_active_tool(&mut self, tool: ToolKind) {
        self.active_tool = tool;
        self.arrow_drag = None;
        self.rect_drag = None;
        self.ellipse_drag = None;
        self.blur_drag = None;
        if tool == ToolKind::Crop {
            self.ensure_default_crop_selection();
        } else {
            self.crop_drag = None;
            self.crop_selection = None;
        }
    }

    fn ensure_default_crop_selection(&mut self) {
        if self.crop_selection.is_some() || self.crop_drag.is_some() {
            return;
        }
        let Some(image) = self.document.base_image.as_ref() else {
            return;
        };
        let inset_x = ((image.width as f32) * 0.08).round() as u32;
        let inset_y = ((image.height as f32) * 0.08).round() as u32;
        let width = image.width.saturating_sub(inset_x * 2).max(1);
        let height = image.height.saturating_sub(inset_y * 2).max(1);
        self.crop_selection = Some(CropSelection {
            x: inset_x.min(image.width.saturating_sub(1)),
            y: inset_y.min(image.height.saturating_sub(1)),
            width,
            height,
        });
    }

    pub(crate) fn begin_crop_drag(&mut self, x: f64, y: f64) {
        self.crop_selection = None;
        self.crop_drag = Some(CropDrag {
            start_x: x,
            start_y: y,
            current_x: x,
            current_y: y,
        });
    }

    pub(crate) fn update_crop_drag(&mut self, x: f64, y: f64) {
        if let Some(d) = self.crop_drag.as_mut() {
            d.current_x = x;
            d.current_y = y;
        }
    }

    pub(crate) fn clear_crop_drag(&mut self) {
        self.crop_drag = None;
    }

    pub(crate) fn begin_arrow_drag(&mut self, widget_x: f64, widget_y: f64, x: f32, y: f32) {
        self.arrow_drag = Some(ArrowDrag {
            widget_start_x: widget_x,
            widget_start_y: widget_y,
            start_x: x,
            start_y: y,
            current_x: x,
            current_y: y,
        });
    }

    pub(crate) fn update_arrow_drag(&mut self, x: f32, y: f32) {
        if let Some(d) = self.arrow_drag.as_mut() {
            d.current_x = x;
            d.current_y = y;
        }
    }

    pub(crate) fn begin_blur_drag(&mut self, x: f64, y: f64) {
        self.blur_drag = Some(CropDrag {
            start_x: x,
            start_y: y,
            current_x: x,
            current_y: y,
        });
    }

    pub(crate) fn update_blur_drag(&mut self, x: f64, y: f64) {
        if let Some(d) = self.blur_drag.as_mut() {
            d.current_x = x;
            d.current_y = y;
        }
    }

    pub(crate) fn clear_blur_drag(&mut self) {
        self.blur_drag = None;
    }

    pub(crate) fn begin_rect_drag(&mut self, x: f64, y: f64) {
        self.rect_drag = Some(CropDrag {
            start_x: x,
            start_y: y,
            current_x: x,
            current_y: y,
        });
    }

    pub(crate) fn update_rect_drag(&mut self, x: f64, y: f64) {
        if let Some(d) = self.rect_drag.as_mut() {
            d.current_x = x;
            d.current_y = y;
        }
    }

    pub(crate) fn clear_rect_drag(&mut self) {
        self.rect_drag = None;
    }

    pub(crate) fn begin_ellipse_drag(&mut self, x: f64, y: f64) {
        self.ellipse_drag = Some(CropDrag {
            start_x: x,
            start_y: y,
            current_x: x,
            current_y: y,
        });
    }

    pub(crate) fn update_ellipse_drag(&mut self, x: f64, y: f64) {
        if let Some(d) = self.ellipse_drag.as_mut() {
            d.current_x = x;
            d.current_y = y;
        }
    }

    pub(crate) fn clear_ellipse_drag(&mut self) {
        self.ellipse_drag = None;
    }

    pub(crate) fn crop_selection(&self) -> Option<CropSelection> {
        self.crop_selection
    }

    pub(crate) fn set_crop_selection(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.crop_drag = None;
        self.crop_selection = Some(CropSelection {
            x,
            y,
            width,
            height,
        });
    }

    pub(crate) fn clear_crop_selection(&mut self) {
        self.crop_drag = None;
        self.crop_selection = None;
    }

    pub(crate) fn has_pending_crop(&self) -> bool {
        self.crop_selection.is_some()
    }
    pub(crate) fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    pub(crate) fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub(crate) fn update_document<F>(&mut self, update: F) -> bool
    where
        F: FnOnce(&mut Document),
    {
        let before = self.document.clone();
        update(&mut self.document);
        if self.document_changed(&before) {
            self.undo_stack.push(before);
            self.redo_stack.clear();
            true
        } else {
            false
        }
    }

    pub(crate) fn undo(&mut self) -> bool {
        let Some(previous) = self.undo_stack.pop() else {
            return false;
        };
        self.redo_stack.push(self.document.clone());
        self.document = previous;
        self.crop_drag = None;
        self.crop_selection = None;
        self.rect_drag = None;
        self.ellipse_drag = None;
        self.blur_drag = None;
        true
    }

    pub(crate) fn redo(&mut self) -> bool {
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };
        self.undo_stack.push(self.document.clone());
        self.document = next;
        self.crop_drag = None;
        self.crop_selection = None;
        self.rect_drag = None;
        self.ellipse_drag = None;
        self.blur_drag = None;
        true
    }

    pub(crate) fn apply_crop_selection(&mut self) -> bool {
        let Some(selection) = self.crop_selection else {
            return false;
        };
        let applied = self.apply_crop(selection.x, selection.y, selection.width, selection.height);
        if applied {
            self.crop_selection = None;
            self.active_tool = ToolKind::Select;
        }
        applied
    }

    pub(crate) fn commit_arrow_drag(&mut self) -> bool {
        let Some(arrow_drag) = self.arrow_drag else {
            return false;
        };
        let dx = arrow_drag.current_x - arrow_drag.start_x;
        let dy = arrow_drag.current_y - arrow_drag.start_y;
        if (dx * dx + dy * dy).sqrt() < 8.0 {
            self.arrow_drag = None;
            return false;
        }
        let color = self.active_color.clone();
        let width = self.active_width;
        let changed = self.update_document(|document| {
            document.annotations.push(Annotation::Arrow {
                from: Point {
                    x: arrow_drag.start_x,
                    y: arrow_drag.start_y,
                },
                to: Point {
                    x: arrow_drag.current_x,
                    y: arrow_drag.current_y,
                },
                color,
                width,
            });
        });
        self.arrow_drag = None;
        changed
    }

    pub(crate) fn commit_blur_rect(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        if width < 4 || height < 4 {
            self.blur_drag = None;
            return false;
        }
        let radius = self.active_width.max(6.0) * 1.6;
        let changed = self.update_document(|document| {
            document.annotations.push(Annotation::Blur {
                bounds: Rect {
                    x: x as f32,
                    y: y as f32,
                    width: width as f32,
                    height: height as f32,
                },
                radius,
            });
        });
        self.blur_drag = None;
        changed
    }

    pub(crate) fn commit_rect_annotation(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> bool {
        if width < 4 || height < 4 {
            self.rect_drag = None;
            return false;
        }
        let color = self.active_color.clone();
        let stroke_width = self.active_width;
        let changed = self.update_document(|document| {
            document.annotations.push(Annotation::Rect {
                bounds: Rect {
                    x: x as f32,
                    y: y as f32,
                    width: width as f32,
                    height: height as f32,
                },
                stroke: snapix_core::canvas::Stroke {
                    color,
                    width: stroke_width,
                },
                fill: None,
            });
        });
        self.rect_drag = None;
        changed
    }

    pub(crate) fn commit_ellipse_annotation(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> bool {
        if width < 4 || height < 4 {
            self.ellipse_drag = None;
            return false;
        }
        let color = self.active_color.clone();
        let stroke_width = self.active_width;
        let changed = self.update_document(|document| {
            document.annotations.push(Annotation::Ellipse {
                bounds: Rect {
                    x: x as f32,
                    y: y as f32,
                    width: width as f32,
                    height: height as f32,
                },
                stroke: snapix_core::canvas::Stroke {
                    color,
                    width: stroke_width,
                },
                fill: None,
            });
        });
        self.ellipse_drag = None;
        changed
    }

    pub(crate) fn add_text_annotation(&mut self, x: f32, y: f32, content: String) -> bool {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return false;
        }
        let color = self.active_color.clone();
        self.update_document(|document| {
            document.annotations.push(Annotation::Text {
                pos: Point { x, y },
                content: trimmed.to_string(),
                style: TextStyle {
                    font_family: "Sans".into(),
                    font_size: 28.0,
                    color,
                    bold: true,
                },
            });
        })
    }

    pub(crate) fn replace_base_image(&mut self, image: Image) -> bool {
        let changed = self.update_document(|document| {
            document.base_image = Some(image);
            document.annotations.clear();
        });
        if changed {
            self.crop_drag = None;
            self.crop_selection = None;
            self.arrow_drag = None;
            self.rect_drag = None;
            self.ellipse_drag = None;
            self.blur_drag = None;
            self.active_tool = ToolKind::Select;
        }
        changed
    }

    pub(crate) fn clear_document_contents(&mut self) -> bool {
        let changed = self.update_document(|document| {
            document.base_image = None;
            document.annotations.clear();
        });
        if changed {
            self.crop_drag = None;
            self.crop_selection = None;
            self.arrow_drag = None;
            self.rect_drag = None;
            self.ellipse_drag = None;
            self.blur_drag = None;
            self.active_tool = ToolKind::Select;
        }
        changed
    }

    fn apply_crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        let Some(image) = self.document.base_image.clone() else {
            return false;
        };
        if width == 0 || height == 0 || x >= image.width || y >= image.height {
            return false;
        }
        let crop_w = width.min(image.width - x);
        let crop_h = height.min(image.height - y);
        if crop_w == 0 || crop_h == 0 {
            return false;
        }
        let mut cropped = Vec::with_capacity((crop_w * crop_h * 4) as usize);
        for row in y..(y + crop_h) {
            let start = ((row * image.width + x) * 4) as usize;
            cropped.extend_from_slice(&image.data[start..start + (crop_w * 4) as usize]);
        }
        let changed = self.update_document(|document| {
            document.base_image = Some(Image::new(crop_w, crop_h, cropped));
            document.annotations.clear();
        });
        if changed {
            self.crop_drag = None;
            self.crop_selection = None;
            self.rect_drag = None;
            self.ellipse_drag = None;
            self.blur_drag = None;
        }
        changed
    }

    fn document_changed(&self, previous: &Document) -> bool {
        let cur = self.document.base_image.as_ref();
        let prev = previous.base_image.as_ref();
        !same_optional_image(cur, prev)
            || self.document.frame.padding != previous.frame.padding
            || self.document.frame.corner_radius != previous.frame.corner_radius
            || self.document.frame.shadow != previous.frame.shadow
            || format!("{:?}", self.document.annotations) != format!("{:?}", previous.annotations)
            || !same_background(&self.document.background, &previous.background)
    }
}

// ─── Save format ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum SaveFormat {
    Png,
    Jpeg,
}

// ─── UI component structs ─────────────────────────────────────────────────────

#[derive(Clone)]
struct CaptureActionRow {
    widget: gtk4::Widget,
    fullscreen_button: gtk4::Button,
    region_button: gtk4::Button,
    window_button: gtk4::Button,
    import_button: gtk4::Button,
    clear_button: gtk4::Button,
}

#[derive(Clone)]
struct BottomBar {
    widget: gtk4::Widget,
    copy_button: gtk4::Button,
    quick_save_button: gtk4::Button,
    save_as_button: gtk4::Button,
}

#[derive(Clone, Copy)]
enum HistoryAction {
    Undo,
    Redo,
}

// ─── EditorWindow ─────────────────────────────────────────────────────────────

pub struct EditorWindow {
    window: ApplicationWindow,
}

impl EditorWindow {
    pub fn new(app: &Application, context: LaunchContext) -> Self {
        let state = Rc::new(RefCell::new(EditorState::with_document(context.document)));

        // Hidden internal labels (not in widget tree; kept for refresh-fn compatibility)
        let title_label = gtk4::Label::new(None);
        let scope_label = gtk4::Label::new(None);

        // Visible status label shown in the bottom bar
        let subtitle_label = gtk4::Label::builder()
            .xalign(0.0)
            .css_classes(["dim-copy"])
            .build();

        let undo_button = gtk4::Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text("Undo (Ctrl+Z)")
            .sensitive(false)
            .build();
        let redo_button = gtk4::Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text("Redo (Ctrl+Shift+Z)")
            .sensitive(false)
            .build();

        let canvas = DocumentCanvas::new(
            state.clone(),
            subtitle_label.clone(),
            scope_label.clone(),
            undo_button.clone(),
            redo_button.clone(),
        );
        let canvas_widget = canvas.widget().clone();

        let save_format = Rc::new(RefCell::new(SaveFormat::Png));

        let inspector = build_inspector(
            state.clone(),
            canvas.clone(),
            &subtitle_label,
            &undo_button,
            &redo_button,
        );
        let tool_row = build_tool_row(state.clone(), canvas.clone(), &title_label, &scope_label);
        let capture_row = build_capture_row();
        let canvas_panel = build_canvas_panel(canvas_widget);
        let bottom_bar = build_bottom_bar(&subtitle_label, save_format.clone());

        let workspace = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(16)
            .margin_top(0)
            .margin_start(16)
            .margin_end(16)
            .margin_bottom(0)
            .hexpand(true)
            .vexpand(true)
            .build();
        workspace.append(&canvas_panel);
        workspace.append(&inspector);

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .build();
        content.add_css_class("snapix-shell");
        content.append(&capture_row.widget);
        content.append(&tool_row);
        content.append(&workspace);
        content.append(&bottom_bar.widget);

        let header = HeaderBar::new();
        header.pack_end(&redo_button);
        header.pack_end(&undo_button);

        let toolbar_view = ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&content));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Snapix")
            .default_width(1280)
            .default_height(820)
            .content(&toolbar_view)
            .build();

        connect_crop_shortcuts(
            &window,
            state.clone(),
            canvas.clone(),
            &title_label,
            &subtitle_label,
            &scope_label,
            &undo_button,
            &redo_button,
        );
        connect_history_button(
            &undo_button,
            state.clone(),
            canvas.clone(),
            &title_label,
            &subtitle_label,
            &scope_label,
            &undo_button,
            &redo_button,
            HistoryAction::Undo,
        );
        connect_history_button(
            &redo_button,
            state.clone(),
            canvas.clone(),
            &title_label,
            &subtitle_label,
            &scope_label,
            &undo_button,
            &redo_button,
            HistoryAction::Redo,
        );
        connect_capture_actions(
            &capture_row,
            &window,
            state.clone(),
            canvas.clone(),
            &title_label,
            &subtitle_label,
            &scope_label,
            &undo_button,
            &redo_button,
        );
        connect_copy_button(&bottom_bar.copy_button, &window, state.clone());
        connect_quick_save_button(
            &bottom_bar.quick_save_button,
            &window,
            state.clone(),
            save_format.clone(),
        );
        connect_save_as_button(
            &bottom_bar.save_as_button,
            &window,
            state.clone(),
            save_format,
        );

        refresh_subtitle(&state.borrow(), &subtitle_label);
        refresh_history_buttons(&state.borrow(), &undo_button, &redo_button);
        canvas.refresh();

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

// ─── Keyboard shortcuts ───────────────────────────────────────────────────────

fn connect_crop_shortcuts(
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) {
    let controller = gtk4::EventControllerKey::new();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    controller.connect_key_pressed(move |_controller, key, _keycode, mods| {
        let mut state = state.borrow_mut();
        match key {
            gtk4::gdk::Key::Escape if state.active_tool() == ToolKind::Crop => {
                state.clear_crop_selection();
                refresh_scope_label(&state, &scope_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter
                if state.active_tool() == ToolKind::Crop && state.has_pending_crop() =>
            {
                if state.apply_crop_selection() {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    canvas.refresh();
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::z
                if state.active_tool() != ToolKind::Crop
                    && mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK) =>
            {
                if state.redo() {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    canvas.refresh();
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::z if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) => {
                if state.undo() {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    canvas.refresh();
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    window.add_controller(controller);
}

fn connect_history_button(
    button: &gtk4::Button,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    action: HistoryAction,
) {
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    button.connect_clicked(move |_| {
        let mut state = state.borrow_mut();
        let changed = match action {
            HistoryAction::Undo => state.undo(),
            HistoryAction::Redo => state.redo(),
        };
        if changed {
            refresh_labels(&state, &title_label, &subtitle_label);
            refresh_scope_label(&state, &scope_label);
            refresh_history_buttons(&state, &undo_button, &redo_button);
            canvas.refresh();
        }
    });
}

// ─── Capture actions ──────────────────────────────────────────────────────────

fn connect_capture_actions(
    actions: &CaptureActionRow,
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) {
    let session = snapix_capture::detect_session();
    let backend = snapix_capture::detect_backend();
    if session == snapix_capture::SessionType::Wayland && backend.name() == "ashpd-portal" {
        actions.window_button.set_sensitive(false);
        actions.window_button.set_tooltip_text(Some(
            "Window capture is not available via Wayland portal. Use Region instead.",
        ));
        actions
            .region_button
            .set_tooltip_text(Some("Interactive region capture via XDG portal."));
    }

    for (button, action) in [
        (&actions.fullscreen_button, CaptureAction::Fullscreen),
        (&actions.region_button, CaptureAction::Region),
        (&actions.window_button, CaptureAction::Window),
    ] {
        connect_capture_button(
            button,
            window,
            state.clone(),
            canvas.clone(),
            title_label,
            subtitle_label,
            scope_label,
            undo_button,
            redo_button,
            action,
        );
    }
    connect_import_button(
        &actions.import_button,
        window,
        state.clone(),
        canvas.clone(),
        title_label,
        subtitle_label,
        scope_label,
        undo_button,
        redo_button,
    );
    connect_clear_button(
        &actions.clear_button,
        state,
        canvas,
        title_label,
        subtitle_label,
        scope_label,
        undo_button,
        redo_button,
    );
}

#[derive(Clone, Copy)]
enum CaptureAction {
    Fullscreen,
    Region,
    Window,
}

fn connect_capture_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    action: CaptureAction,
) {
    let window = window.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    button.connect_clicked(move |_| {
        let backend = snapix_capture::detect_backend();
        let result = async_std::task::block_on(async {
            match action {
                CaptureAction::Fullscreen => backend.capture_full().await,
                CaptureAction::Region => backend
                    .capture_region(Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 })
                    .await,
                CaptureAction::Window => backend.capture_window().await,
            }
        });
        match result {
            Ok(image) => {
                let mut state = state.borrow_mut();
                if state.replace_base_image(image) {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    canvas.refresh();
                }
            }
            Err(error) => {
                let detail = match action {
                    CaptureAction::Fullscreen => format!(
                        "Fullscreen capture failed: {error}. On Wayland this can fail even when Region capture works."
                    ),
                    CaptureAction::Region => format!("Region capture failed: {error}"),
                    CaptureAction::Window => format!("Window capture failed: {error}"),
                };
                show_error(&window, "Capture failed", &detail);
            }
        }
    });
}

fn connect_import_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) {
    let window = window.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    button.connect_clicked(move |_| {
        let chooser = gtk4::FileChooserNative::builder()
            .title("Import image")
            .transient_for(&window)
            .action(gtk4::FileChooserAction::Open)
            .accept_label("Import")
            .cancel_label("Cancel")
            .modal(true)
            .build();
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Images"));
        for mime in ["image/png", "image/jpeg", "image/webp"] {
            filter.add_mime_type(mime);
        }
        for pat in ["*.png", "*.jpg", "*.jpeg", "*.webp"] {
            filter.add_pattern(pat);
        }
        chooser.add_filter(&filter);

        let window = window.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let title_label = title_label.clone();
        let subtitle_label = subtitle_label.clone();
        let scope_label = scope_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        chooser.connect_response(move |chooser, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(file) = chooser.file() {
                    match file.path() {
                        Some(path) => match image::open(&path) {
                            Ok(dynamic) => {
                                let mut state = state.borrow_mut();
                                if state.replace_base_image(Image::from_dynamic(dynamic)) {
                                    refresh_labels(&state, &title_label, &subtitle_label);
                                    refresh_scope_label(&state, &scope_label);
                                    refresh_history_buttons(&state, &undo_button, &redo_button);
                                    canvas.refresh();
                                }
                            }
                            Err(error) => show_error(
                                &window,
                                "Import failed",
                                &format!("Failed to open {}: {error}", path.display()),
                            ),
                        },
                        None => show_error(
                            &window,
                            "Import failed",
                            "The selected image is not a local file path.",
                        ),
                    }
                }
            }
            chooser.destroy();
        });
        chooser.show();
    });
}

fn connect_clear_button(
    button: &gtk4::Button,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) {
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    button.connect_clicked(move |_| {
        let mut state = state.borrow_mut();
        if state.clear_document_contents() {
            refresh_labels(&state, &title_label, &subtitle_label);
            refresh_scope_label(&state, &scope_label);
            refresh_history_buttons(&state, &undo_button, &redo_button);
            canvas.refresh();
        }
    });
}

// ─── Export / copy actions ────────────────────────────────────────────────────

fn connect_copy_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
) {
    let window = window.clone();
    button.connect_clicked(move |_| {
        let document = state.borrow().document().clone();
        match render_document_rgba(&document) {
            Ok(rendered) => {
                let mut clipboard = match arboard::Clipboard::new() {
                    Ok(c) => c,
                    Err(error) => {
                        show_error(
                            &window,
                            "Copy failed",
                            &format!("Clipboard unavailable: {error}"),
                        );
                        return;
                    }
                };
                if let Err(error) = clipboard.set_image(arboard::ImageData {
                    width: rendered.width as usize,
                    height: rendered.height as usize,
                    bytes: Cow::Owned(rendered.rgba),
                }) {
                    show_error(
                        &window,
                        "Copy failed",
                        &format!("Clipboard write failed: {error}"),
                    );
                }
            }
            Err(error) => show_error(&window, "Copy failed", &error.to_string()),
        }
    });
}

fn connect_quick_save_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
    save_format: Rc<RefCell<SaveFormat>>,
) {
    let window = window.clone();
    button.connect_clicked(move |_| {
        let document = state.borrow().document().clone();
        let format = *save_format.borrow();
        let pictures_dir = gtk4::glib::user_special_dir(gtk4::glib::UserDirectory::Pictures)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ext = if format == SaveFormat::Jpeg {
            "jpg"
        } else {
            "png"
        };
        let path = pictures_dir.join(format!("snapix-{ts}.{ext}"));
        if let Err(error) = save_image_to_path(&document, &path, format) {
            show_error(&window, "Quick save failed", &error.to_string());
        }
    });
}

fn connect_save_as_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
    save_format: Rc<RefCell<SaveFormat>>,
) {
    let window = window.clone();
    button.connect_clicked(move |_| {
        let format = *save_format.borrow();
        let (title_str, accept_str, default_name, mime, pattern) = match format {
            SaveFormat::Png => (
                "Export PNG",
                "Save",
                "snapix-export.png",
                "image/png",
                "*.png",
            ),
            SaveFormat::Jpeg => (
                "Export JPEG",
                "Save",
                "snapix-export.jpg",
                "image/jpeg",
                "*.jpg",
            ),
        };
        let chooser = gtk4::FileChooserNative::builder()
            .title(title_str)
            .transient_for(&window)
            .action(gtk4::FileChooserAction::Save)
            .accept_label(accept_str)
            .cancel_label("Cancel")
            .modal(true)
            .build();
        chooser.set_current_name(default_name);
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(title_str));
        filter.add_mime_type(mime);
        filter.add_pattern(pattern);
        chooser.add_filter(&filter);

        let state = state.clone();
        let window = window.clone();
        let save_format = save_format.clone();
        chooser.connect_response(move |chooser, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(file) = chooser.file() {
                    match file.path() {
                        Some(path) => {
                            let document = state.borrow().document().clone();
                            let fmt = *save_format.borrow();
                            if let Err(error) = save_image_to_path(&document, &path, fmt) {
                                show_error(&window, "Export failed", &error.to_string());
                            }
                        }
                        None => show_error(
                            &window,
                            "Export failed",
                            "The selected destination is not a local file path.",
                        ),
                    }
                }
            }
            chooser.destroy();
        });
        chooser.show();
    });
}

fn save_image_to_path(
    document: &Document,
    path: &std::path::Path,
    format: SaveFormat,
) -> anyhow::Result<()> {
    let rendered = render_document_rgba(document)?;
    match format {
        SaveFormat::Png => {
            image::save_buffer(
                path,
                &rendered.rgba,
                rendered.width,
                rendered.height,
                image::ColorType::Rgba8,
            )
            .map_err(|e| anyhow::anyhow!("Failed to save PNG: {e}"))?;
        }
        SaveFormat::Jpeg => {
            let rgb: Vec<u8> = rendered
                .rgba
                .chunks_exact(4)
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect();
            image::save_buffer(
                path,
                &rgb,
                rendered.width,
                rendered.height,
                image::ColorType::Rgb8,
            )
            .map_err(|e| anyhow::anyhow!("Failed to save JPEG: {e}"))?;
        }
    }
    Ok(())
}

fn show_error(window: &ApplicationWindow, title: &str, detail: &str) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .message_type(gtk4::MessageType::Error)
        .buttons(gtk4::ButtonsType::Ok)
        .text(title)
        .secondary_text(detail)
        .build();
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}

// ─── UI builders ──────────────────────────────────────────────────────────────

fn build_capture_row() -> CaptureActionRow {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Fill)
        .build();
    row.add_css_class("capture-row");

    let mut built = Vec::new();
    for (label, icon, classes) in [
        (
            "Fullscreen",
            "view-fullscreen-symbolic",
            ["capture-pill", "fullscreen"],
        ),
        (
            "Region",
            "selection-rectangular-symbolic",
            ["capture-pill", "region"],
        ),
        (
            "Window",
            "focus-windows-symbolic",
            ["capture-pill", "window"],
        ),
        (
            "Import",
            "document-open-symbolic",
            ["capture-pill", "import"],
        ),
        ("Clear", "edit-clear-symbolic", ["capture-pill", "utility"]),
    ] {
        let btn = gtk4::Button::builder().label(label).icon_name(icon).build();
        btn.set_css_classes(&classes);
        row.append(&btn);
        built.push(btn);
    }

    CaptureActionRow {
        widget: row.upcast(),
        fullscreen_button: built[0].clone(),
        region_button: built[1].clone(),
        window_button: built[2].clone(),
        import_button: built[3].clone(),
        clear_button: built[4].clone(),
    }
}

fn build_tool_row(
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    scope_label: &gtk4::Label,
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
            .tooltip_text(tool.label())
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
        let all = btn_refs.clone();
        let tool = *tool;
        btn.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            state.set_active_tool(tool);
            title_label.set_label(&format!("Editor • {}", tool.label()));
            refresh_scope_label(&state, &scope_label);
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
    let palette: &[((u8, u8, u8), &str, &str)] = &[
        ((255, 98, 54), "color-dot-0", "Orange"),
        ((229, 57, 53), "color-dot-1", "Red"),
        ((233, 30, 140), "color-dot-2", "Pink"),
        ((124, 77, 255), "color-dot-3", "Purple"),
        ((33, 150, 243), "color-dot-4", "Blue"),
        ((0, 150, 136), "color-dot-5", "Teal"),
        ((76, 175, 80), "color-dot-6", "Green"),
        ((255, 214, 0), "color-dot-7", "Yellow"),
        ((240, 240, 240), "color-dot-8", "White"),
        ((30, 30, 46), "color-dot-9", "Dark"),
    ];

    let color_btns: Rc<RefCell<Vec<gtk4::Button>>> = Rc::new(RefCell::new(Vec::new()));
    let init_color = state.borrow().active_color();

    for (i, ((r, g, b), dot_class, tooltip)) in palette.iter().enumerate() {
        let color = Color {
            r: *r,
            g: *g,
            b: *b,
            a: 255,
        };
        let dot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        dot.set_size_request(18, 18);
        dot.add_css_class("color-dot");
        dot.add_css_class(dot_class);

        let btn = gtk4::Button::builder()
            .tooltip_text(*tooltip)
            .child(&dot)
            .build();
        btn.add_css_class("color-swatch-btn");
        if same_color_rgb(*r, *g, *b, &init_color) {
            btn.add_css_class("active");
        }

        let state = state.clone();
        let canvas = canvas.clone();
        let color_btns_ref = color_btns.clone();
        btn.connect_clicked(move |_| {
            state.borrow_mut().set_active_color(color.clone());
            for (j, b) in color_btns_ref.borrow().iter().enumerate() {
                if j == i {
                    b.add_css_class("active");
                } else {
                    b.remove_css_class("active");
                }
            }
            canvas.refresh();
        });
        color_btns.borrow_mut().push(btn.clone());
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
    let widths: &[(f32, &str, &str)] = &[
        (3.0, "wd-sm", "Thin (3px)"),
        (6.0, "wd-md", "Medium (6px)"),
        (10.0, "wd-lg", "Thick (10px)"),
        (16.0, "wd-xl", "Extra thick (16px)"),
    ];
    let init_width = state.borrow().active_width();
    let width_btns: Rc<RefCell<Vec<gtk4::Button>>> = Rc::new(RefCell::new(Vec::new()));

    for (i, (w, size_class, tooltip)) in widths.iter().enumerate() {
        let dot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        dot.add_css_class("width-dot-inner");
        dot.add_css_class(size_class);
        if (*w - init_width).abs() < 0.1 {
            dot.add_css_class("active");
        }

        let btn = gtk4::Button::builder()
            .tooltip_text(*tooltip)
            .child(&dot)
            .build();
        btn.add_css_class("width-btn");

        let state = state.clone();
        let canvas = canvas.clone();
        let width_btns_ref = width_btns.clone();
        let width_val = *w;
        btn.connect_clicked(move |_| {
            state.borrow_mut().set_active_width(width_val);
            for j in 0..widths.len() {
                if let Some(b) = width_btns_ref.borrow().get(j) {
                    if let Some(child) = b.child() {
                        if let Ok(bx) = child.downcast::<gtk4::Box>() {
                            if j == i {
                                bx.add_css_class("active");
                            } else {
                                bx.remove_css_class("active");
                            }
                        }
                    }
                }
            }
            canvas.refresh();
        });
        width_btns.borrow_mut().push(btn.clone());
        card.append(&btn);
    }

    // ── Spacer + Delete ───────────────────────────────────────────────────────
    let spacer = gtk4::Box::builder().hexpand(true).build();
    card.append(&spacer);

    let delete_btn = gtk4::Button::builder()
        .icon_name("edit-delete-symbolic")
        .tooltip_text("Delete last annotation")
        .css_classes(["tool-delete-btn"])
        .build();
    {
        let state = state.clone();
        let canvas = canvas.clone();
        let title_label = title_label.clone();
        let scope_label = scope_label.clone();
        delete_btn.connect_clicked(move |_| {
            let mut s = state.borrow_mut();
            if s.update_document(|doc| {
                doc.annotations.pop();
            }) {
                refresh_scope_label(&s, &scope_label);
                title_label.set_label(&format!("Editor • {}", s.active_tool().label()));
                canvas.refresh();
            }
        });
    }
    card.append(&delete_btn);

    row.append(&card);
    row.upcast()
}

fn build_tool_icon(tool: ToolKind) -> gtk4::Widget {
    let icon = gtk4::DrawingArea::builder()
        .content_width(18)
        .content_height(18)
        .build();
    icon.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        cr.set_source_rgb(0.95, 0.97, 1.0);
        cr.set_line_width(1.8);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.set_line_join(cairo::LineJoin::Round);

        match tool {
            ToolKind::Select => {
                cr.move_to(3.0, 3.0);
                cr.line_to(13.0, 9.0);
                cr.line_to(8.0, 10.0);
                cr.line_to(10.5, 15.0);
                cr.line_to(8.5, 16.0);
                cr.line_to(6.0, 10.8);
                cr.line_to(3.0, 13.0);
                cr.close_path();
                cr.stroke_preserve().ok();
                cr.set_source_rgba(0.95, 0.97, 1.0, 0.15);
                cr.fill().ok();
            }
            ToolKind::Crop => {
                cr.move_to(5.0, 3.0);
                cr.line_to(5.0, 13.0);
                cr.line_to(15.0, 13.0);
                cr.move_to(8.0, 5.0);
                cr.line_to(15.0, 5.0);
                cr.line_to(15.0, 10.0);
                cr.stroke().ok();
            }
            ToolKind::Arrow => {
                cr.move_to(3.0, h - 4.0);
                cr.line_to(w - 5.0, 5.0);
                cr.stroke().ok();
                cr.move_to(w - 8.5, 5.0);
                cr.line_to(w - 5.0, 5.0);
                cr.line_to(w - 5.0, 8.5);
                cr.stroke().ok();
            }
            ToolKind::Rectangle => {
                cr.rectangle(3.5, 4.0, w - 7.0, h - 8.0);
                cr.stroke().ok();
            }
            ToolKind::Ellipse => {
                cr.save().ok();
                cr.translate(w / 2.0, h / 2.0);
                cr.scale((w - 7.0) / 2.0, (h - 8.0) / 2.0);
                cr.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
                cr.restore().ok();
                cr.stroke().ok();
            }
            ToolKind::Text => {
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
                cr.set_font_size(13.0);
                cr.move_to(4.5, 14.0);
                cr.show_text("T").ok();
            }
            ToolKind::Blur => {
                cr.rectangle(4.0, 4.5, 10.0, 9.0);
                cr.stroke().ok();
                for x in [6.0, 9.0, 12.0] {
                    cr.arc(x, 9.0, 1.1, 0.0, std::f64::consts::TAU);
                    cr.fill().ok();
                }
            }
        }
    });
    icon.upcast()
}

fn build_canvas_panel(canvas_widget: gtk4::DrawingArea) -> gtk4::Widget {
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

fn build_bottom_bar(
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

    // Right side actions
    let actions_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .margin_end(16)
        .build();

    // Format toggle
    let png_btn = gtk4::ToggleButton::builder()
        .label("PNG")
        .active(true)
        .css_classes(["format-pill"])
        .build();
    let jpg_btn = gtk4::ToggleButton::builder()
        .label("JPEG")
        .active(false)
        .css_classes(["format-pill"])
        .build();
    jpg_btn.set_group(Some(&png_btn));

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

    actions_box.append(&png_btn);
    actions_box.append(&jpg_btn);

    let sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(4)
        .margin_end(4)
        .build();
    actions_box.append(&sep);

    let copy_btn = gtk4::Button::builder()
        .label("Copy")
        .css_classes(["bottom-action-btn"])
        .tooltip_text("Copy canvas image to clipboard")
        .build();
    let quick_save_btn = gtk4::Button::builder()
        .label("Save")
        .css_classes(["bottom-action-btn", "suggested-action"])
        .tooltip_text("Quick save to Pictures folder")
        .build();
    let save_as_btn = gtk4::Button::builder()
        .label("Save As…")
        .css_classes(["bottom-action-btn"])
        .tooltip_text("Save to a specific location")
        .build();

    actions_box.append(&copy_btn);
    actions_box.append(&quick_save_btn);
    actions_box.append(&save_as_btn);

    bar.append(&dims_box);
    bar.append(&actions_box);

    BottomBar {
        widget: bar.upcast(),
        copy_button: copy_btn,
        quick_save_button: quick_save_btn,
        save_as_button: save_as_btn,
    }
}

fn build_inspector(
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) -> gtk4::Widget {
    let panel = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(16)
        .width_request(260)
        .valign(gtk4::Align::Fill)
        .build();
    panel.add_css_class("inspector-card");

    panel.append(
        &gtk4::Label::builder()
            .label("Settings")
            .xalign(0.0)
            .css_classes(["title-4", "section-title"])
            .build(),
    );

    // ── Padding ──────────────────────────────────────────────────────────────
    let padding_val = gtk4::Label::builder()
        .label(&format!(
            "{}px",
            state.borrow().document.frame.padding as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let padding = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 160.0, 1.0);
    padding.set_value(state.borrow().document.frame.padding as f64);
    {
        let pv = padding_val.clone();
        connect_frame_slider(
            &padding,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, value| {
                frame.padding = value;
                pv.set_label(&format!("{}px", value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value("Padding", &padding, &padding_val));

    // ── Corner Radius ─────────────────────────────────────────────────────────
    let radius_val = gtk4::Label::builder()
        .label(&format!(
            "{}px",
            state.borrow().document.frame.corner_radius as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let radius = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 48.0, 1.0);
    radius.set_value(state.borrow().document.frame.corner_radius as f64);
    {
        let rv = radius_val.clone();
        connect_frame_slider(
            &radius,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, value| {
                frame.corner_radius = value;
                rv.set_label(&format!("{}px", value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        "Corner Radius",
        &radius,
        &radius_val,
    ));

    // ── Shadow ────────────────────────────────────────────────────────────────
    let shadow = gtk4::Switch::builder()
        .active(state.borrow().document.frame.shadow)
        .halign(gtk4::Align::End)
        .build();
    {
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        shadow.connect_active_notify(move |sw| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| doc.frame.shadow = sw.is_active()) {
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
    }
    panel.append(&labeled_row("Shadow", &shadow));

    panel.append(
        &gtk4::Separator::builder()
            .margin_top(2)
            .margin_bottom(2)
            .build(),
    );

    // ── Output Ratio ──────────────────────────────────────────────────────────
    panel.append(
        &gtk4::Label::builder()
            .label("Output Ratio")
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .build(),
    );
    let ratio_labels = [
        "Auto", "1:1", "4:3", "3:2", "16:9", "5:3", "9:16", "3:4", "2:3",
    ];
    let ratio_grid = gtk4::Grid::builder()
        .row_spacing(6)
        .column_spacing(6)
        .build();
    let ratio_btns: Rc<RefCell<Vec<gtk4::Button>>> = Rc::new(RefCell::new(Vec::new()));
    for (i, lbl) in ratio_labels.iter().enumerate() {
        let btn = gtk4::Button::builder().label(*lbl).hexpand(true).build();
        btn.add_css_class("ratio-btn");
        if i == 0 {
            btn.add_css_class("selected");
        }
        let ratio_btns_ref = ratio_btns.clone();
        btn.connect_clicked(move |_| {
            for (j, b) in ratio_btns_ref.borrow().iter().enumerate() {
                if j == i {
                    b.add_css_class("selected");
                } else {
                    b.remove_css_class("selected");
                }
            }
        });
        ratio_btns.borrow_mut().push(btn.clone());
        ratio_grid.attach(&btn, (i % 3) as i32, (i / 3) as i32, 1, 1);
    }
    panel.append(&ratio_grid);

    panel.append(
        &gtk4::Separator::builder()
            .margin_top(2)
            .margin_bottom(2)
            .build(),
    );

    // ── Background ────────────────────────────────────────────────────────────
    panel.append(
        &gtk4::Label::builder()
            .label("Background")
            .xalign(0.0)
            .css_classes(["heading", "section-title"])
            .build(),
    );

    let current_bg = state.borrow().document.background.clone();
    let swatch_buttons: Rc<RefCell<Vec<(Background, gtk4::Button)>>> =
        Rc::new(RefCell::new(Vec::new()));

    // 12 background presets in a 4×3 grid
    let bg_presets: Vec<(&str, &str, Background)> = vec![
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

    let swatch_grid = gtk4::Grid::builder()
        .row_spacing(8)
        .column_spacing(8)
        .build();

    for (index, (label, css_class, background)) in bg_presets.into_iter().enumerate() {
        let selected = same_background(&current_bg, &background);
        let btn = gtk4::Button::builder()
            .tooltip_text(label)
            .hexpand(true)
            .vexpand(false)
            .build();
        btn.add_css_class("background-swatch");
        btn.add_css_class(css_class);
        if selected {
            btn.add_css_class("selected");
        }

        swatch_buttons
            .borrow_mut()
            .push((background.clone(), btn.clone()));
        let all = swatch_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        btn.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| doc.background = background.clone()) {
                for (sb, sb_btn) in all.borrow().iter() {
                    if same_background(sb, &background) {
                        sb_btn.add_css_class("selected");
                    } else {
                        sb_btn.remove_css_class("selected");
                    }
                }
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });
        swatch_grid.attach(&btn, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }
    panel.append(&swatch_grid);

    panel.upcast()
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

fn connect_frame_slider<F>(
    scale: &gtk4::Scale,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    subtitle_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    update: F,
) where
    F: Fn(&mut FrameSettings, f32) + 'static,
{
    let subtitle_label = subtitle_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    scale.connect_value_changed(move |scale| {
        let mut state = state.borrow_mut();
        if state.update_document(|doc| update(&mut doc.frame, scale.value() as f32)) {
            refresh_subtitle(&state, &subtitle_label);
            refresh_history_buttons(&state, &undo_button, &redo_button);
            canvas.refresh();
        }
    });
}

// ─── Refresh helpers ──────────────────────────────────────────────────────────

fn refresh_labels(state: &EditorState, title_label: &gtk4::Label, subtitle_label: &gtk4::Label) {
    title_label.set_label(&format!("Editor • {}", state.active_tool.label()));
    refresh_subtitle(state, subtitle_label);
}

pub(crate) fn refresh_subtitle(state: &EditorState, subtitle_label: &gtk4::Label) {
    let text = match state.document.base_image.as_ref() {
        Some(Image { width, height, .. }) => format!("Image: {width}×{height}"),
        None => "No image loaded".to_string(),
    };
    subtitle_label.set_label(&text);
}

pub(crate) fn refresh_scope_label(state: &EditorState, scope_label: &gtk4::Label) {
    let text = match state.active_tool() {
        ToolKind::Crop => {
            if state.has_pending_crop() {
                "Crop ready — drag handles to resize, Enter to apply, Esc to cancel."
            } else {
                "Crop mode: default selection shown. Esc to cancel."
            }
        }
        ToolKind::Arrow => "Arrow: drag on the image to place an arrow.",
        ToolKind::Rectangle => "Rectangle: drag on the image to draw a box.",
        ToolKind::Ellipse => "Ellipse: drag on the image to draw an oval.",
        ToolKind::Text => "Text: click on the image to place a label.",
        ToolKind::Blur => "Blur: drag on the image to create a blur region.",
        _ => "",
    };
    scope_label.set_label(text);
}

pub(crate) fn refresh_history_buttons(
    state: &EditorState,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
) {
    undo_button.set_sensitive(state.can_undo());
    redo_button.set_sensitive(state.can_redo());
}

// ─── Equality helpers ─────────────────────────────────────────────────────────

fn same_optional_image(current: Option<&Image>, previous: Option<&Image>) -> bool {
    match (current, previous) {
        (Some(c), Some(p)) => c.width == p.width && c.height == p.height && c.data == p.data,
        (None, None) => true,
        _ => false,
    }
}

pub(crate) fn same_background(current: &Background, previous: &Background) -> bool {
    match (current, previous) {
        (Background::Solid { color: l }, Background::Solid { color: r }) => same_color(l, r),
        (
            Background::Gradient {
                from: lf,
                to: lt,
                angle_deg: la,
            },
            Background::Gradient {
                from: rf,
                to: rt,
                angle_deg: ra,
            },
        ) => same_color(lf, rf) && same_color(lt, rt) && la == ra,
        (Background::Image { path: l }, Background::Image { path: r }) => l == r,
        (
            Background::BlurredScreenshot { radius: l },
            Background::BlurredScreenshot { radius: r },
        ) => l == r,
        _ => false,
    }
}

fn same_color(l: &Color, r: &Color) -> bool {
    l.r == r.r && l.g == r.g && l.b == r.b && l.a == r.a
}

fn same_color_rgb(r: u8, g: u8, b: u8, color: &Color) -> bool {
    color.r == r && color.g == g && color.b == b
}
