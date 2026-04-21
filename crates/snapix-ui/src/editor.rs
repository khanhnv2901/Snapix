use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use gio::prelude::FileExt;
use gtk4::cairo;
use gtk4::prelude::*;
use libadwaita::{
    Application, ApplicationWindow, Bin, HeaderBar, Toast, ToastOverlay, ToolbarView,
};
use snapix_capture::{CaptureBackend, SessionType};
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
    selected_annotation: Option<usize>,
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
            selected_annotation: None,
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
    pub(crate) fn selected_annotation(&self) -> Option<usize> {
        self.selected_annotation
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

    pub(crate) fn set_selected_annotation(&mut self, selected: Option<usize>) {
        self.selected_annotation =
            selected.filter(|index| *index < self.document.annotations.len());
    }

    pub(crate) fn delete_selected_annotation(&mut self) -> bool {
        let Some(index) = self.selected_annotation else {
            return false;
        };
        let changed = self.update_document(|document| {
            if index < document.annotations.len() {
                document.annotations.remove(index);
            }
        });
        if changed {
            self.selected_annotation = None;
        }
        changed
    }

    pub(crate) fn apply_active_color_to_selected(&mut self) -> bool {
        let Some(index) = self.selected_annotation else {
            return false;
        };
        let color = self.active_color.clone();
        self.update_document(|document| {
            let Some(annotation) = document.annotations.get_mut(index) else {
                return;
            };
            match annotation {
                Annotation::Arrow { color: current, .. } => *current = color,
                Annotation::Rect { stroke, .. } | Annotation::Ellipse { stroke, .. } => {
                    stroke.color = color
                }
                Annotation::Text { style, .. } => style.color = color,
                Annotation::Blur { .. } | Annotation::Redact { .. } => {}
            }
        })
    }

    pub(crate) fn apply_active_width_to_selected(&mut self) -> bool {
        let Some(index) = self.selected_annotation else {
            return false;
        };
        let width = self.active_width;
        self.update_document(|document| {
            let Some(annotation) = document.annotations.get_mut(index) else {
                return;
            };
            match annotation {
                Annotation::Arrow { width: current, .. } => *current = width,
                Annotation::Rect { stroke, .. } | Annotation::Ellipse { stroke, .. } => {
                    stroke.width = width
                }
                Annotation::Blur { radius, .. } => *radius = width.max(6.0) * 1.6,
                Annotation::Text { style, .. } => style.font_size = width.max(4.0) * 4.0,
                Annotation::Redact { .. } => {}
            }
        })
    }

    pub(crate) fn selected_text_content(&self) -> Option<String> {
        let index = self.selected_annotation?;
        match self.document.annotations.get(index) {
            Some(Annotation::Text { content, .. }) => Some(content.clone()),
            _ => None,
        }
    }

    pub(crate) fn update_selected_text_content(&mut self, content: String) -> bool {
        let Some(index) = self.selected_annotation else {
            return false;
        };
        let trimmed = content.trim().to_string();
        if trimmed.is_empty() {
            return false;
        }
        self.update_document(|document| {
            let Some(annotation) = document.annotations.get_mut(index) else {
                return;
            };
            if let Annotation::Text {
                content: current, ..
            } = annotation
            {
                *current = trimmed;
            }
        })
    }

    pub(crate) fn preview_move_annotation(
        &mut self,
        index: usize,
        original: &Annotation,
        delta_x: f32,
        delta_y: f32,
    ) {
        let Some(image) = self.document.base_image.as_ref() else {
            return;
        };
        let moved =
            move_annotation_within_image(original, delta_x, delta_y, image.width, image.height);
        if let Some(slot) = self.document.annotations.get_mut(index) {
            *slot = moved;
        }
    }

    pub(crate) fn preview_resize_annotation(
        &mut self,
        index: usize,
        original: &Annotation,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        let resized = resize_annotation_bounds(original, x, y, width, height);
        if let Some(slot) = self.document.annotations.get_mut(index) {
            *slot = resized;
        }
    }

    pub(crate) fn preview_resize_arrow_endpoint(
        &mut self,
        index: usize,
        original: &Annotation,
        move_start: bool,
        x: f32,
        y: f32,
    ) {
        let resized = resize_arrow_endpoint(original, move_start, x, y);
        if let Some(slot) = self.document.annotations.get_mut(index) {
            *slot = resized;
        }
    }

    pub(crate) fn finalize_annotation_move(&mut self, before: Document) -> bool {
        if self.document_changed(&before) {
            self.undo_stack.push(before);
            self.redo_stack.clear();
            true
        } else {
            false
        }
    }

    pub(crate) fn ensure_default_crop_selection(&mut self) {
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

    pub(crate) fn cancel_crop_mode(&mut self) {
        self.crop_drag = None;
        self.crop_selection = None;
        self.active_tool = ToolKind::Select;
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
        self.selected_annotation = None;
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
        self.selected_annotation = None;
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
        if changed {
            self.selected_annotation = self.document.annotations.len().checked_sub(1);
        }
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
        if changed {
            self.selected_annotation = self.document.annotations.len().checked_sub(1);
        }
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
        if changed {
            self.selected_annotation = self.document.annotations.len().checked_sub(1);
        }
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
        if changed {
            self.selected_annotation = self.document.annotations.len().checked_sub(1);
        }
        changed
    }

    pub(crate) fn add_text_annotation(&mut self, x: f32, y: f32, content: String) -> bool {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return false;
        }
        let color = self.active_color.clone();
        let changed = self.update_document(|document| {
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
        });
        if changed {
            self.selected_annotation = self.document.annotations.len().checked_sub(1);
        }
        changed
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
            self.selected_annotation = None;
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
            self.selected_annotation = None;
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
            self.selected_annotation = None;
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
            || self.document.frame.shadow_offset_x != previous.frame.shadow_offset_x
            || self.document.frame.shadow_padding != previous.frame.shadow_padding
            || self.document.frame.shadow_blur != previous.frame.shadow_blur
            || self.document.frame.shadow_offset_y != previous.frame.shadow_offset_y
            || self.document.frame.shadow_strength != previous.frame.shadow_strength
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

#[derive(Clone)]
struct InspectorControls {
    widget: gtk4::Widget,
    padding_scale: gtk4::Scale,
    padding_value: gtk4::Label,
    radius_scale: gtk4::Scale,
    radius_value: gtk4::Label,
    shadow_switch: gtk4::Switch,
    shadow_direction_buttons: Rc<RefCell<Vec<gtk4::Button>>>,
    shadow_padding_scale: gtk4::Scale,
    shadow_padding_value: gtk4::Label,
    shadow_blur_scale: gtk4::Scale,
    shadow_blur_value: gtk4::Label,
    shadow_strength_scale: gtk4::Scale,
    shadow_strength_value: gtk4::Label,
    background_buttons: Rc<RefCell<Vec<(Background, gtk4::Button)>>>,
}

impl InspectorControls {
    fn widget(&self) -> gtk4::Widget {
        self.widget.clone()
    }

    fn refresh_from_state(&self, state: &EditorState) {
        let frame = &state.document().frame;
        self.padding_scale.set_value(frame.padding as f64);
        self.padding_value
            .set_label(&format!("{}px", frame.padding as u32));

        self.radius_scale.set_value(frame.corner_radius as f64);
        self.radius_value
            .set_label(&format!("{}px", frame.corner_radius as u32));

        self.shadow_switch.set_active(frame.shadow);

        let selected_shadow_direction =
            nearest_shadow_direction_index(frame.shadow_offset_x, frame.shadow_offset_y);
        for (index, button) in self.shadow_direction_buttons.borrow().iter().enumerate() {
            if index == selected_shadow_direction {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }

        self.shadow_padding_scale
            .set_value(frame.shadow_padding as f64);
        self.shadow_padding_value
            .set_label(&format!("{}px", frame.shadow_padding as u32));

        self.shadow_blur_scale.set_value(frame.shadow_blur as f64);
        self.shadow_blur_value
            .set_label(&format!("{}px", frame.shadow_blur as u32));

        self.shadow_strength_scale
            .set_value((frame.shadow_strength * 100.0) as f64);
        self.shadow_strength_value.set_label(&format!(
            "{}%",
            (frame.shadow_strength * 100.0).round() as u32
        ));

        for (background, button) in self.background_buttons.borrow().iter() {
            if same_background(background, &state.document().background) {
                button.add_css_class("selected");
            } else {
                button.remove_css_class("selected");
            }
        }
    }
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
        let delete_button = gtk4::Button::builder()
            .icon_name("edit-delete-symbolic")
            .tooltip_text("Delete selected annotation (Backspace/Delete)")
            .css_classes(["tool-delete-btn"])
            .sensitive(false)
            .build();
        let width_label = gtk4::Label::builder()
            .label(width_label_text(&state.borrow()))
            .margin_start(12)
            .margin_end(2)
            .css_classes(["dim-copy"])
            .valign(gtk4::Align::Center)
            .build();

        let toast_overlay = ToastOverlay::new();

        let save_format = Rc::new(RefCell::new(SaveFormat::Png));

        let canvas = DocumentCanvas::new(
            state.clone(),
            subtitle_label.clone(),
            scope_label.clone(),
            width_label.clone(),
            undo_button.clone(),
            redo_button.clone(),
            toast_overlay.clone(),
            delete_button.clone(),
        );
        let canvas_widget = canvas.widget().clone();
        let tool_row = build_tool_row(
            state.clone(),
            canvas.clone(),
            &title_label,
            &scope_label,
            &width_label,
            &undo_button,
            &redo_button,
            &delete_button,
        );

        let inspector = build_inspector(
            state.clone(),
            canvas.clone(),
            &subtitle_label,
            &undo_button,
            &redo_button,
        );
        let capture_row = build_capture_row();
        let canvas_panel = build_canvas_panel(canvas_widget);
        let bottom_bar = build_bottom_bar(&subtitle_label, save_format.clone());

        let workspace = gtk4::Paned::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_top(0)
            .margin_start(8)
            .margin_end(8)
            .margin_bottom(0)
            .hexpand(true)
            .vexpand(true)
            .wide_handle(false)
            .build();
        workspace.set_resize_start_child(true);
        workspace.set_resize_end_child(false);
        workspace.set_shrink_start_child(false);
        workspace.set_shrink_end_child(false);
        workspace.set_start_child(Some(&canvas_panel));
        workspace.set_end_child(Some(&inspector.widget()));

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

        toast_overlay.set_child(Some(&content));

        let toolbar_view = ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&toast_overlay));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Snapix")
            .default_width(1280)
            .default_height(820)
            .content(&toolbar_view)
            .build();

        connect_crop_shortcuts(
            &window,
            &toast_overlay,
            state.clone(),
            canvas.clone(),
            &title_label,
            &subtitle_label,
            &scope_label,
            &undo_button,
            &redo_button,
            &bottom_bar,
            &delete_button,
            &inspector,
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
            &bottom_bar,
            &delete_button,
            &inspector,
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
            &bottom_bar,
            &delete_button,
            &inspector,
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
            &toast_overlay,
            &undo_button,
            &redo_button,
            &bottom_bar,
            &delete_button,
            &inspector,
        );
        connect_copy_button(
            &bottom_bar.copy_button,
            &window,
            &toast_overlay,
            state.clone(),
        );
        connect_quick_save_button(
            &bottom_bar.quick_save_button,
            &window,
            &toast_overlay,
            state.clone(),
            save_format.clone(),
        );
        connect_save_as_button(
            &bottom_bar.save_as_button,
            &window,
            &toast_overlay,
            state.clone(),
            save_format,
        );

        if let Some(banner) = context.banner {
            show_toast(&toast_overlay, &banner.text);
        }

        refresh_subtitle(&state.borrow(), &subtitle_label);
        refresh_history_buttons(&state.borrow(), &undo_button, &redo_button);
        refresh_export_actions(&state.borrow(), &bottom_bar);
        refresh_tool_actions(&state.borrow(), &delete_button);
        inspector.refresh_from_state(&state.borrow());
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
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let controller = gtk4::EventControllerKey::new();
    let toast_overlay = toast_overlay.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
    controller.connect_key_pressed(move |_controller, key, _keycode, mods| {
        let mut state = state.borrow_mut();
        match key {
            gtk4::gdk::Key::Escape if state.active_tool() == ToolKind::Crop => {
                state.cancel_crop_mode();
                refresh_labels(&state, &title_label, &subtitle_label);
                refresh_scope_label(&state, &scope_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                refresh_export_actions(&state, &bottom_bar);
                refresh_tool_actions(&state, &delete_button);
                inspector.refresh_from_state(&state);
                canvas.refresh();
                show_toast(&toast_overlay, "Crop cancelled");
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter
                if state.active_tool() == ToolKind::Crop && state.has_pending_crop() =>
            {
                if state.apply_crop_selection() {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    refresh_export_actions(&state, &bottom_bar);
                    refresh_tool_actions(&state, &delete_button);
                    inspector.refresh_from_state(&state);
                    canvas.refresh();
                    show_toast(&toast_overlay, "Crop applied");
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::BackSpace | gtk4::gdk::Key::Delete
                if state.active_tool() != ToolKind::Crop
                    && state.selected_annotation().is_some() =>
            {
                if state.delete_selected_annotation() {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    refresh_export_actions(&state, &bottom_bar);
                    refresh_tool_actions(&state, &delete_button);
                    inspector.refresh_from_state(&state);
                    canvas.refresh();
                    show_toast(&toast_overlay, "Deleted selected annotation");
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
                    refresh_export_actions(&state, &bottom_bar);
                    refresh_tool_actions(&state, &delete_button);
                    inspector.refresh_from_state(&state);
                    canvas.refresh();
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::z if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) => {
                if state.undo() {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    refresh_export_actions(&state, &bottom_bar);
                    refresh_tool_actions(&state, &delete_button);
                    inspector.refresh_from_state(&state);
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
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
    action: HistoryAction,
) {
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
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
            refresh_export_actions(&state, &bottom_bar);
            refresh_tool_actions(&state, &delete_button);
            inspector.refresh_from_state(&state);
            canvas.refresh();
        }
    });
}

fn refresh_export_actions(state: &EditorState, bottom_bar: &BottomBar) {
    let has_image = export_actions_enabled(state.document());
    bottom_bar.copy_button.set_sensitive(has_image);
    bottom_bar.quick_save_button.set_sensitive(has_image);
    bottom_bar.save_as_button.set_sensitive(has_image);
}

pub(crate) fn refresh_tool_actions(state: &EditorState, delete_button: &gtk4::Button) {
    delete_button.set_sensitive(state.selected_annotation().is_some());
}

pub(crate) fn refresh_width_label(state: &EditorState, width_label: &gtk4::Label) {
    width_label.set_label(width_label_text(state));
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
    toast_overlay: &ToastOverlay,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let session = snapix_capture::detect_session();
    let backend = snapix_capture::detect_backend();
    if session == snapix_capture::SessionType::Wayland && backend.name() == "ashpd-portal" {
        actions.window_button.set_sensitive(false);
        actions.window_button.set_tooltip_text(Some(
            "Window capture is not exposed as a distinct action here on Wayland. Use Region and choose a window in the portal picker.",
        ));
        actions.fullscreen_button.set_tooltip_text(Some(
            "Fullscreen capture can fail on some Wayland portal setups. Snapix will fall back to interactive region capture if needed.",
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
            toast_overlay,
            undo_button,
            redo_button,
            bottom_bar,
            delete_button,
            inspector,
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
        toast_overlay,
        undo_button,
        redo_button,
        bottom_bar,
        delete_button,
        inspector,
    );
    connect_clear_button(
        &actions.clear_button,
        toast_overlay,
        state,
        canvas,
        title_label,
        subtitle_label,
        scope_label,
        undo_button,
        redo_button,
        bottom_bar,
        delete_button,
        inspector,
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
    toast_overlay: &ToastOverlay,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
    action: CaptureAction,
) {
    let window = window.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let toast_overlay = toast_overlay.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
    button.connect_clicked(move |_| {
        let session = snapix_capture::detect_session();
        let result = async_std::task::block_on(async {
            let backend = snapix_capture::detect_backend();
            perform_capture_action(backend.as_ref(), session, action).await
        });
        match result {
            Ok((image, message)) => {
                let mut state = state.borrow_mut();
                if state.replace_base_image(image) {
                    refresh_labels(&state, &title_label, &subtitle_label);
                    refresh_scope_label(&state, &scope_label);
                    refresh_history_buttons(&state, &undo_button, &redo_button);
                    refresh_export_actions(&state, &bottom_bar);
                    refresh_tool_actions(&state, &delete_button);
                    inspector.refresh_from_state(&state);
                    canvas.refresh();
                }
                drop(state);
                if let Some(message) = message {
                    show_toast(&toast_overlay, &message);
                } else {
                    let message = match action {
                        CaptureAction::Fullscreen => "Captured full screen",
                        CaptureAction::Region => "Captured selection",
                        CaptureAction::Window => "Captured active window",
                    };
                    show_toast(&toast_overlay, message);
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

async fn perform_capture_action(
    backend: &dyn CaptureBackend,
    session: SessionType,
    action: CaptureAction,
) -> Result<(Image, Option<String>)> {
    match action {
        CaptureAction::Fullscreen => match backend.capture_full().await {
            Ok(image) => Ok((image, None)),
            Err(full_error)
                if session == SessionType::Wayland && backend.name() == "ashpd-portal" =>
            {
                tracing::warn!(
                    "Fullscreen portal capture failed on Wayland, retrying with interactive region capture: {full_error:#}"
                );
                match backend
                    .capture_region(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    })
                    .await
                {
                    Ok(image) => Ok((
                        image,
                        Some("Fullscreen capture failed, switched to region capture.".to_string()),
                    )),
                    Err(region_error) => Err(anyhow::anyhow!(
                        "Fullscreen capture failed: {full_error}. Interactive region fallback also failed: {region_error}"
                    )),
                }
            }
            Err(error) => Err(error),
        },
        CaptureAction::Region => backend
            .capture_region(Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            })
            .await
            .map(|image| (image, None)),
        CaptureAction::Window => backend.capture_window().await.map(|image| (image, None)),
    }
}

fn connect_import_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    toast_overlay: &ToastOverlay,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let window = window.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let toast_overlay = toast_overlay.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
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
        let response_toast_overlay = toast_overlay.clone();
        let response_bottom_bar = bottom_bar.clone();
        let response_delete_button = delete_button.clone();
        let response_inspector = inspector.clone();
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
                                    refresh_export_actions(&state, &response_bottom_bar);
                                    refresh_tool_actions(&state, &response_delete_button);
                                    response_inspector.refresh_from_state(&state);
                                    canvas.refresh();
                                }
                                show_toast(
                                    &response_toast_overlay,
                                    &format!("Imported {}", path.display()),
                                );
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
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let toast_overlay = toast_overlay.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
    button.connect_clicked(move |_| {
        let mut state = state.borrow_mut();
        if state.clear_document_contents() {
            refresh_labels(&state, &title_label, &subtitle_label);
            refresh_scope_label(&state, &scope_label);
            refresh_history_buttons(&state, &undo_button, &redo_button);
            refresh_export_actions(&state, &bottom_bar);
            refresh_tool_actions(&state, &delete_button);
            inspector.refresh_from_state(&state);
            canvas.refresh();
            show_toast(&toast_overlay, "Cleared current image");
        }
    });
}

// ─── Export / copy actions ────────────────────────────────────────────────────

fn connect_copy_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
) {
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();
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
                } else {
                    show_toast(&toast_overlay, "Copied image to clipboard");
                }
            }
            Err(error) => show_error(&window, "Copy failed", &error.to_string()),
        }
    });
}

fn connect_quick_save_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
    save_format: Rc<RefCell<SaveFormat>>,
) {
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();
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
        } else {
            show_toast(&toast_overlay, &format!("Saved to {}", path.display()));
        }
    });
}

fn connect_save_as_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
    save_format: Rc<RefCell<SaveFormat>>,
) {
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();
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
        let toast_overlay = toast_overlay.clone();
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
                            } else {
                                show_toast(
                                    &toast_overlay,
                                    &format!("Exported to {}", path.display()),
                                );
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

fn show_dialog(
    window: &ApplicationWindow,
    message_type: gtk4::MessageType,
    title: &str,
    detail: &str,
) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .message_type(message_type)
        .buttons(gtk4::ButtonsType::Ok)
        .text(title)
        .secondary_text(detail)
        .build();
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}

fn show_error(window: &ApplicationWindow, title: &str, detail: &str) {
    show_dialog(window, gtk4::MessageType::Error, title, detail);
}

pub(crate) fn show_toast(toast_overlay: &ToastOverlay, message: &str) {
    toast_overlay.add_toast(Toast::new(message));
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
    width_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    delete_button: &gtk4::Button,
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
        let width_label = width_label.clone();
        let all = btn_refs.clone();
        let tool = *tool;
        btn.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            state.set_active_tool(tool);
            title_label.set_label(&format!("Editor • {}", tool.label()));
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
        let dot = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();
        dot.set_size_request(16, 16);
        dot.add_css_class("color-dot");
        dot.add_css_class(dot_class);

        let btn = gtk4::Button::builder()
            .tooltip_text(*tooltip)
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
                title_label.set_label(&format!("Editor • {}", s.active_tool().label()));
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

        match tool {
            ToolKind::Select => {
                // Cursor arrow pointing top-left
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.92);
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
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.20);
                cr.fill().ok();
            }
            ToolKind::Crop => {
                // Two L-bracket corners (standard crop icon)
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.92);
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
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.92);
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
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.92);
                cr.set_line_width(1.8);
                cr.rectangle(3.5, 5.0, w - 7.0, h - 10.0);
                cr.stroke().ok();
            }
            ToolKind::Ellipse => {
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.92);
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
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.92);
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
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.92);
                cr.set_line_width(1.7);
                // Outer rectangle
                cr.rectangle(3.5, 5.5, 13.0, 9.0);
                cr.stroke().ok();
                // Horizontal lines inside suggesting blur/scan effect
                cr.set_source_rgba(0.93, 0.95, 1.0, 0.55);
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
        .tooltip_text("Copy the current image to the clipboard")
        .build();
    let quick_save_btn = gtk4::Button::builder()
        .label("Save")
        .css_classes(["bottom-action-btn", "suggested-action"])
        .tooltip_text("Save to the Pictures folder")
        .build();
    let save_as_btn = gtk4::Button::builder()
        .label("Save As…")
        .css_classes(["bottom-action-btn"])
        .tooltip_text("Export to a specific location")
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

    let shadow_direction_grid = gtk4::Grid::builder()
        .row_spacing(5)
        .column_spacing(5)
        .halign(gtk4::Align::Center)
        .build();
    let shadow_direction_buttons: Rc<RefCell<Vec<gtk4::Button>>> =
        Rc::new(RefCell::new(Vec::new()));
    let selected_shadow_direction = nearest_shadow_direction_index(
        state.borrow().document.frame.shadow_offset_x,
        state.borrow().document.frame.shadow_offset_y,
    );
    for (index, preset) in SHADOW_DIRECTION_PRESETS.iter().enumerate() {
        let btn = gtk4::Button::builder()
            .label(preset.label)
            .tooltip_text(preset.tooltip)
            .build();
        btn.add_css_class("shadow-dir-btn");
        if index == selected_shadow_direction {
            btn.add_css_class("selected");
        }

        let all_buttons = shadow_direction_buttons.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let subtitle_label = subtitle_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let offset_x = preset.offset_x;
        let offset_y = preset.offset_y;
        btn.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            if state.update_document(|doc| {
                doc.frame.shadow_offset_x = offset_x;
                doc.frame.shadow_offset_y = offset_y;
            }) {
                for (button_index, button) in all_buttons.borrow().iter().enumerate() {
                    if button_index == index {
                        button.add_css_class("selected");
                    } else {
                        button.remove_css_class("selected");
                    }
                }
                refresh_subtitle(&state, &subtitle_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                canvas.refresh();
            }
        });

        shadow_direction_buttons.borrow_mut().push(btn.clone());
        shadow_direction_grid.attach(&btn, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }
    panel.append(&labeled_row("Shadow Direction", &shadow_direction_grid));

    let shadow_padding_val = gtk4::Label::builder()
        .label(&format!(
            "{}px",
            state.borrow().document.frame.shadow_padding as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let shadow_padding = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 32.0, 1.0);
    shadow_padding.set_value(state.borrow().document.frame.shadow_padding as f64);
    {
        let value_label = shadow_padding_val.clone();
        connect_frame_slider(
            &shadow_padding,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, value| {
                frame.shadow_padding = value;
                value_label.set_label(&format!("{}px", value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        "Shadow Padding",
        &shadow_padding,
        &shadow_padding_val,
    ));

    let shadow_blur_val = gtk4::Label::builder()
        .label(&format!(
            "{}px",
            state.borrow().document.frame.shadow_blur as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let shadow_blur = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 64.0, 1.0);
    shadow_blur.set_value(state.borrow().document.frame.shadow_blur as f64);
    {
        let value_label = shadow_blur_val.clone();
        connect_frame_slider(
            &shadow_blur,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, value| {
                frame.shadow_blur = value;
                value_label.set_label(&format!("{}px", value as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        "Shadow Blur",
        &shadow_blur,
        &shadow_blur_val,
    ));

    let shadow_strength_val = gtk4::Label::builder()
        .label(&format!(
            "{}%",
            (state.borrow().document.frame.shadow_strength * 100.0).round() as u32
        ))
        .css_classes(["dim-copy"])
        .build();
    let shadow_strength = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
    shadow_strength.set_value((state.borrow().document.frame.shadow_strength * 100.0) as f64);
    {
        let value_label = shadow_strength_val.clone();
        connect_frame_slider(
            &shadow_strength,
            state.clone(),
            canvas.clone(),
            subtitle_label,
            undo_button,
            redo_button,
            move |frame, value| {
                frame.shadow_strength = (value / 100.0).clamp(0.0, 1.0);
                value_label.set_label(&format!("{}%", value.round() as u32));
            },
        );
    }
    panel.append(&labeled_row_with_value(
        "Shadow Strength",
        &shadow_strength,
        &shadow_strength_val,
    ));

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
        .row_spacing(4)
        .column_spacing(4)
        .hexpand(true)
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
        .row_spacing(6)
        .column_spacing(6)
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
        swatch_grid.attach(&btn, (index % 4) as i32, (index / 4) as i32, 1, 1);
    }
    panel.append(&swatch_grid);

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
        padding_scale: padding,
        padding_value: padding_val,
        radius_scale: radius,
        radius_value: radius_val,
        shadow_switch: shadow,
        shadow_direction_buttons,
        shadow_padding_scale: shadow_padding,
        shadow_padding_value: shadow_padding_val,
        shadow_blur_scale: shadow_blur,
        shadow_blur_value: shadow_blur_val,
        shadow_strength_scale: shadow_strength,
        shadow_strength_value: shadow_strength_val,
        background_buttons: swatch_buttons,
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
    let text = subtitle_text(state.document());
    subtitle_label.set_label(&text);
}

pub(crate) fn refresh_scope_label(state: &EditorState, scope_label: &gtk4::Label) {
    let text = scope_text(state);
    scope_label.set_label(&text);
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

fn subtitle_text(document: &Document) -> String {
    match document.base_image.as_ref() {
        Some(Image { width, height, .. }) => format!("Image: {width}×{height}"),
        None => "No image loaded. Capture or import an image to begin.".to_string(),
    }
}

fn export_actions_enabled(document: &Document) -> bool {
    document.base_image.is_some()
}

#[derive(Clone, Copy)]
struct ShadowDirectionPreset {
    label: &'static str,
    tooltip: &'static str,
    offset_x: f32,
    offset_y: f32,
}

const SHADOW_DIRECTION_PRESETS: [ShadowDirectionPreset; 9] = [
    ShadowDirectionPreset {
        label: "↖",
        tooltip: "Shadow toward top left",
        offset_x: -18.0,
        offset_y: -18.0,
    },
    ShadowDirectionPreset {
        label: "↑",
        tooltip: "Shadow toward top",
        offset_x: 0.0,
        offset_y: -22.0,
    },
    ShadowDirectionPreset {
        label: "↗",
        tooltip: "Shadow toward top right",
        offset_x: 18.0,
        offset_y: -18.0,
    },
    ShadowDirectionPreset {
        label: "←",
        tooltip: "Shadow toward left",
        offset_x: -24.0,
        offset_y: 0.0,
    },
    ShadowDirectionPreset {
        label: "·",
        tooltip: "Centered glow shadow",
        offset_x: 0.0,
        offset_y: 0.0,
    },
    ShadowDirectionPreset {
        label: "→",
        tooltip: "Shadow toward right",
        offset_x: 24.0,
        offset_y: 0.0,
    },
    ShadowDirectionPreset {
        label: "↙",
        tooltip: "Shadow toward bottom left",
        offset_x: -18.0,
        offset_y: 18.0,
    },
    ShadowDirectionPreset {
        label: "↓",
        tooltip: "Shadow toward bottom",
        offset_x: 0.0,
        offset_y: 22.0,
    },
    ShadowDirectionPreset {
        label: "↘",
        tooltip: "Shadow toward bottom right",
        offset_x: 18.0,
        offset_y: 18.0,
    },
];

fn nearest_shadow_direction_index(offset_x: f32, offset_y: f32) -> usize {
    SHADOW_DIRECTION_PRESETS
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let left_distance =
                ((offset_x - left.offset_x).powi(2) + (offset_y - left.offset_y).powi(2)) as i32;
            let right_distance =
                ((offset_x - right.offset_x).powi(2) + (offset_y - right.offset_y).powi(2)) as i32;
            left_distance.cmp(&right_distance)
        })
        .map(|(index, _)| index)
        .unwrap_or(4)
}

fn width_label_text(state: &EditorState) -> &'static str {
    match state
        .selected_annotation()
        .and_then(|index| state.document().annotations.get(index))
    {
        Some(Annotation::Text { .. }) => "Size:",
        _ if state.active_tool() == ToolKind::Text => "Size:",
        _ => "Width:",
    }
}

fn scope_text(state: &EditorState) -> String {
    match state.active_tool() {
        ToolKind::Select => match state.selected_annotation() {
            Some(index) => format!(
                "Selected {}. Use the color, width, toolbar delete button, or Backspace/Delete.",
                annotation_kind_label(state.document(), index)
            ),
            None => {
                "Select: click an annotation to edit it, then press Backspace/Delete to remove it."
                    .to_string()
            }
        },
        ToolKind::Crop => {
            if state.document().base_image.is_none() {
                "Crop: capture or import an image first.".to_string()
            } else if state.has_pending_crop() {
                "Crop ready. Drag the handles, press Enter to apply, or Esc to exit crop mode."
                    .to_string()
            } else {
                "Crop mode: drag on the image to create a selection, or press Esc to exit."
                    .to_string()
            }
        }
        ToolKind::Arrow => "Arrow: drag on the image to place an arrow.".to_string(),
        ToolKind::Rectangle => "Rectangle: drag on the image to draw a box.".to_string(),
        ToolKind::Ellipse => "Ellipse: drag on the image to draw an oval.".to_string(),
        ToolKind::Text => "Text: click on the image to place a label.".to_string(),
        ToolKind::Blur => "Blur: drag on the image to create a blur region.".to_string(),
    }
}

fn annotation_kind_label(document: &Document, index: usize) -> &'static str {
    match document.annotations.get(index) {
        Some(Annotation::Arrow { .. }) => "arrow",
        Some(Annotation::Rect { .. }) => "rectangle",
        Some(Annotation::Ellipse { .. }) => "ellipse",
        Some(Annotation::Text { .. }) => "text label",
        Some(Annotation::Blur { .. }) => "blur region",
        Some(Annotation::Redact { .. }) => "redaction",
        None => "annotation",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        export_actions_enabled, nearest_shadow_direction_index, perform_capture_action, scope_text,
        subtitle_text, CaptureAction, EditorState, ToolKind,
    };
    use anyhow::{anyhow, Result};
    use snapix_capture::{CaptureBackend, SessionType};
    use snapix_core::canvas::{Annotation, Document, Image, Rect};
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockBackend {
        name: &'static str,
        full_result: Result<Image, String>,
        region_result: Result<Image, String>,
        window_result: Result<Image, String>,
        region_calls: Arc<Mutex<Vec<Rect>>>,
    }

    impl MockBackend {
        fn with_results(
            name: &'static str,
            full_result: Result<Image, String>,
            region_result: Result<Image, String>,
            window_result: Result<Image, String>,
        ) -> Self {
            Self {
                name,
                full_result,
                region_result,
                window_result,
                region_calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn region_calls(&self) -> Vec<Rect> {
            self.region_calls.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl CaptureBackend for MockBackend {
        async fn capture_full(&self) -> Result<Image> {
            self.full_result.clone().map_err(|err| anyhow!(err))
        }

        async fn capture_region(&self, region: Rect) -> Result<Image> {
            self.region_calls.lock().unwrap().push(region);
            self.region_result.clone().map_err(|err| anyhow!(err))
        }

        async fn capture_window(&self) -> Result<Image> {
            self.window_result.clone().map_err(|err| anyhow!(err))
        }

        fn supports_interactive(&self) -> bool {
            false
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    fn sample_image() -> Image {
        Image::new(2, 2, vec![255; 16])
    }

    #[test]
    fn cancel_crop_mode_returns_to_select_tool() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.set_active_tool(ToolKind::Crop);
        assert_eq!(state.active_tool(), ToolKind::Crop);
        assert!(state.has_pending_crop());

        state.cancel_crop_mode();

        assert_eq!(state.active_tool(), ToolKind::Select);
        assert!(!state.has_pending_crop());
        assert!(state.crop_drag().is_none());
    }

    #[test]
    fn crop_scope_text_requires_image_when_empty() {
        let mut state = EditorState::default();
        state.set_active_tool(ToolKind::Crop);

        assert_eq!(
            scope_text(&state),
            "Crop: capture or import an image first."
        );
    }

    #[test]
    fn subtitle_text_guides_empty_state() {
        let text = subtitle_text(&Document::default());

        assert_eq!(
            text,
            "No image loaded. Capture or import an image to begin."
        );
    }

    #[test]
    fn selected_annotation_can_be_deleted() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.commit_rect_annotation(10, 10, 40, 30);
        assert_eq!(state.selected_annotation(), Some(0));

        assert!(state.delete_selected_annotation());
        assert!(state.document().annotations.is_empty());
        assert_eq!(state.selected_annotation(), None);
    }

    #[test]
    fn selected_text_annotation_can_be_edited() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        assert!(state.add_text_annotation(24.0, 42.0, "Old".into()));
        assert_eq!(state.selected_text_content().as_deref(), Some("Old"));

        assert!(state.update_selected_text_content("New value".into()));
        assert_eq!(state.selected_text_content().as_deref(), Some("New value"));
    }

    #[test]
    fn active_color_updates_selected_rect_annotation() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.commit_rect_annotation(10, 10, 40, 30);
        state.set_active_color(snapix_core::canvas::Color {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        });

        assert!(state.apply_active_color_to_selected());

        let Annotation::Rect { stroke, .. } = &state.document().annotations[0] else {
            panic!("expected rectangle annotation");
        };
        assert_eq!(stroke.color.r, 10);
        assert_eq!(stroke.color.g, 20);
        assert_eq!(stroke.color.b, 30);
    }

    #[test]
    fn active_width_updates_selected_arrow_annotation() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.begin_arrow_drag(0.0, 0.0, 5.0, 5.0);
        state.update_arrow_drag(50.0, 45.0);
        assert!(state.commit_arrow_drag());
        state.set_active_width(16.0);

        assert!(state.apply_active_width_to_selected());

        let Annotation::Arrow { width, .. } = &state.document().annotations[0] else {
            panic!("expected arrow annotation");
        };
        assert_eq!(*width, 16.0);
    }

    #[test]
    fn export_actions_disabled_without_image() {
        assert!(!export_actions_enabled(&Document::default()));
    }

    #[test]
    fn export_actions_enabled_with_image() {
        assert!(export_actions_enabled(&Document::new(sample_image())));
    }

    #[test]
    fn fullscreen_wayland_portal_falls_back_to_region() {
        let backend = MockBackend::with_results(
            "ashpd-portal",
            Err("portal fullscreen failed".into()),
            Ok(sample_image()),
            Err("unused".into()),
        );

        let (image, message) = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::Wayland, CaptureAction::Fullscreen)
                .await
                .expect("expected fallback to succeed")
        });

        assert_eq!(image.width, 2);
        assert!(message.is_some());
        assert_eq!(
            message.unwrap(),
            "Fullscreen capture failed, switched to region capture."
        );

        let calls = backend.region_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].x, 0.0);
        assert_eq!(calls[0].y, 0.0);
        assert_eq!(calls[0].width, 0.0);
        assert_eq!(calls[0].height, 0.0);
    }

    #[test]
    fn fullscreen_wayland_portal_reports_both_failures() {
        let backend = MockBackend::with_results(
            "ashpd-portal",
            Err("portal fullscreen failed".into()),
            Err("portal region failed".into()),
            Err("unused".into()),
        );

        let error = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::Wayland, CaptureAction::Fullscreen)
                .await
                .expect_err("expected fallback chain to fail")
        });

        let text = error.to_string();
        assert!(text.contains("Fullscreen capture failed"));
        assert!(text.contains("Interactive region fallback also failed"));
    }

    #[test]
    fn fullscreen_x11_does_not_use_region_fallback() {
        let backend = MockBackend::with_results(
            "x11rb",
            Err("x11 fullscreen failed".into()),
            Ok(sample_image()),
            Err("unused".into()),
        );

        let error = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::X11, CaptureAction::Fullscreen)
                .await
                .expect_err("expected fullscreen error to be returned directly")
        });

        assert!(error.to_string().contains("x11 fullscreen failed"));
        assert!(backend.region_calls().is_empty());
    }

    #[test]
    fn region_action_uses_zero_rect_interactive_request() {
        let backend = MockBackend::with_results(
            "ashpd-portal",
            Err("unused".into()),
            Ok(sample_image()),
            Err("unused".into()),
        );

        let (_image, message) = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::Wayland, CaptureAction::Region)
                .await
                .expect("expected region capture to succeed")
        });

        assert!(message.is_none());
        let calls = backend.region_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].width, 0.0);
        assert_eq!(calls[0].height, 0.0);
    }

    #[test]
    fn nearest_shadow_direction_prefers_bottom_for_default_shadow() {
        assert_eq!(nearest_shadow_direction_index(18.0, 18.0), 8);
    }

    #[test]
    fn nearest_shadow_direction_prefers_center_for_zero_offset() {
        assert_eq!(nearest_shadow_direction_index(0.0, 0.0), 4);
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

fn move_annotation_within_image(
    annotation: &Annotation,
    delta_x: f32,
    delta_y: f32,
    image_width: u32,
    image_height: u32,
) -> Annotation {
    let max_x = image_width.saturating_sub(1) as f32;
    let max_y = image_height.saturating_sub(1) as f32;

    match annotation {
        Annotation::Arrow {
            from,
            to,
            color,
            width,
        } => Annotation::Arrow {
            from: Point {
                x: (from.x + delta_x).clamp(0.0, max_x),
                y: (from.y + delta_y).clamp(0.0, max_y),
            },
            to: Point {
                x: (to.x + delta_x).clamp(0.0, max_x),
                y: (to.y + delta_y).clamp(0.0, max_y),
            },
            color: color.clone(),
            width: *width,
        },
        Annotation::Rect {
            bounds,
            stroke,
            fill,
        } => Annotation::Rect {
            bounds: move_rect_within_image(bounds, delta_x, delta_y, image_width, image_height),
            stroke: stroke.clone(),
            fill: fill.clone(),
        },
        Annotation::Ellipse {
            bounds,
            stroke,
            fill,
        } => Annotation::Ellipse {
            bounds: move_rect_within_image(bounds, delta_x, delta_y, image_width, image_height),
            stroke: stroke.clone(),
            fill: fill.clone(),
        },
        Annotation::Blur { bounds, radius } => Annotation::Blur {
            bounds: move_rect_within_image(bounds, delta_x, delta_y, image_width, image_height),
            radius: *radius,
        },
        Annotation::Redact { bounds } => Annotation::Redact {
            bounds: move_rect_within_image(bounds, delta_x, delta_y, image_width, image_height),
        },
        Annotation::Text {
            pos,
            content,
            style,
        } => Annotation::Text {
            pos: Point {
                x: (pos.x + delta_x).clamp(0.0, max_x),
                y: (pos.y + delta_y).clamp(0.0, max_y),
            },
            content: content.clone(),
            style: style.clone(),
        },
    }
}

fn resize_annotation_bounds(
    annotation: &Annotation,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Annotation {
    let bounds = Rect {
        x: x as f32,
        y: y as f32,
        width: width as f32,
        height: height as f32,
    };

    match annotation {
        Annotation::Rect { stroke, fill, .. } => Annotation::Rect {
            bounds,
            stroke: stroke.clone(),
            fill: fill.clone(),
        },
        Annotation::Ellipse { stroke, fill, .. } => Annotation::Ellipse {
            bounds,
            stroke: stroke.clone(),
            fill: fill.clone(),
        },
        Annotation::Blur { radius, .. } => Annotation::Blur {
            bounds,
            radius: *radius,
        },
        Annotation::Redact { .. } => Annotation::Redact { bounds },
        _ => annotation.clone(),
    }
}

fn resize_arrow_endpoint(annotation: &Annotation, move_start: bool, x: f32, y: f32) -> Annotation {
    match annotation {
        Annotation::Arrow {
            from,
            to,
            color,
            width,
        } => Annotation::Arrow {
            from: if move_start {
                Point { x, y }
            } else {
                from.clone()
            },
            to: if move_start {
                to.clone()
            } else {
                Point { x, y }
            },
            color: color.clone(),
            width: *width,
        },
        _ => annotation.clone(),
    }
}

fn move_rect_within_image(
    bounds: &Rect,
    delta_x: f32,
    delta_y: f32,
    image_width: u32,
    image_height: u32,
) -> Rect {
    let max_x = (image_width as f32 - bounds.width).max(0.0);
    let max_y = (image_height as f32 - bounds.height).max(0.0);
    Rect {
        x: (bounds.x + delta_x).clamp(0.0, max_x),
        y: (bounds.y + delta_y).clamp(0.0, max_y),
        width: bounds.width,
        height: bounds.height,
    }
}
