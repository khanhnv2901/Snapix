use snapix_core::canvas::{Annotation, Background, Color, Document, Image, Point, Rect, TextStyle};

use crate::widgets::{layout_for_document, natural_image_bounds};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClearOutcome {
    None,
    DeletedSelectedAnnotation,
    ClearedDocument,
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
    pub(super) document: Document,
    pub(super) active_tool: ToolKind,
    active_color: Color,
    active_width: f32,
    selected_annotation: Option<usize>,
    crop_drag: Option<CropDrag>,
    crop_selection: Option<CropSelection>,
    arrow_drag: Option<ArrowDrag>,
    rect_drag: Option<CropDrag>,
    ellipse_drag: Option<CropDrag>,
    blur_drag: Option<CropDrag>,
    reframing_image: bool,
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
            reframing_image: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl EditorState {
    pub(super) fn with_document(document: Document) -> Self {
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
    pub(crate) fn is_reframing_image(&self) -> bool {
        self.reframing_image
    }

    pub(crate) fn set_active_color(&mut self, color: Color) {
        self.active_color = color;
    }
    pub(crate) fn set_active_width(&mut self, width: f32) {
        self.active_width = width;
    }

    pub(crate) fn set_active_tool(&mut self, tool: ToolKind) {
        self.active_tool = tool;
        self.selected_annotation = None;
        self.arrow_drag = None;
        self.rect_drag = None;
        self.ellipse_drag = None;
        self.blur_drag = None;
        self.reframing_image = false;
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

    pub(crate) fn sync_active_style_from_selected(&mut self) {
        let Some(index) = self.selected_annotation else {
            return;
        };
        let Some(annotation) = self.document.annotations.get(index) else {
            return;
        };
        match annotation {
            Annotation::Arrow { color, width, .. } => {
                self.active_color = color.clone();
                self.active_width = *width;
            }
            Annotation::Rect { stroke, .. } | Annotation::Ellipse { stroke, .. } => {
                self.active_color = stroke.color.clone();
                self.active_width = stroke.width;
            }
            Annotation::Text { style, .. } => {
                self.active_color = style.color.clone();
                self.active_width = (style.font_size / 4.0).clamp(1.0, 30.0);
            }
            _ => {}
        }
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
        let (x, y, width, height) = if let Some(image) = self.document.base_image.as_ref() {
            let cx = x.min(image.width.saturating_sub(1));
            let cy = y.min(image.height.saturating_sub(1));
            let cw = width.min(image.width - cx).max(1);
            let ch = height.min(image.height - cy).max(1);
            (cx, cy, cw, ch)
        } else {
            (x, y, width, height)
        };
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

    pub(crate) fn enter_image_reframe_mode(&mut self) -> bool {
        if self.document.base_image.is_none() {
            return false;
        }
        self.reframing_image = true;
        self.selected_annotation = None;
        self.document.image_scale_mode = snapix_core::canvas::ImageScaleMode::Fill;
        true
    }

    pub(crate) fn exit_image_reframe_mode(&mut self) {
        self.reframing_image = false;
    }

    pub(crate) fn reset_image_reframe(&mut self) -> bool {
        let changed = self.update_document(|document| {
            document.image_scale_mode = snapix_core::canvas::ImageScaleMode::Fit;
            document.image_anchor = snapix_core::canvas::ImageAnchor::Center;
            document.image_zoom = 1.0;
            document.image_offset_x = 0.0;
            document.image_offset_y = 0.0;
        });
        if changed {
            self.reframing_image = false;
        }
        changed
    }

    pub(crate) fn recenter_image_reframe(&mut self) -> bool {
        if self.document.base_image.is_none() {
            return false;
        }
        let changed = self.update_document(|document| {
            document.image_scale_mode = snapix_core::canvas::ImageScaleMode::Fill;
            document.image_anchor = snapix_core::canvas::ImageAnchor::Center;
            document.image_zoom = 1.0;
            document.image_offset_x = 0.0;
            document.image_offset_y = 0.0;
        });
        if changed {
            self.reframing_image = true;
        }
        changed
    }

    pub(crate) fn preview_pan_image(&mut self, before: &Document, offset_x: f64, offset_y: f64) {
        let Some(image) = before.base_image.as_ref() else {
            return;
        };
        self.document = before.clone();
        self.document.image_scale_mode = snapix_core::canvas::ImageScaleMode::Fill;
        let bounds = natural_image_bounds(&self.document);
        let Some(layout) = layout_for_document(image, bounds, &self.document) else {
            return;
        };
        self.document.image_offset_x += (offset_x / layout.image_scale) as f32;
        self.document.image_offset_y += (offset_y / layout.image_scale) as f32;
    }

    pub(crate) fn preview_zoom_image(&mut self, before: &Document, scale_delta: f64) {
        self.preview_zoom_image_at(before, scale_delta, 0.5, 0.5);
    }

    pub(crate) fn preview_zoom_image_at(
        &mut self,
        before: &Document,
        scale_delta: f64,
        focus_ratio_x: f64,
        focus_ratio_y: f64,
    ) {
        apply_zoom_with_focus(
            &mut self.document,
            before,
            scale_delta,
            focus_ratio_x,
            focus_ratio_y,
        );
    }

    pub(crate) fn finalize_image_reframe(&mut self, before: Document) -> bool {
        if self.document_changed(&before) {
            self.undo_stack.push(before);
            self.redo_stack.clear();
            true
        } else {
            false
        }
    }

    pub(crate) fn zoom_reframed_image(&mut self, direction: f64) -> bool {
        self.zoom_reframed_image_at(direction, 0.5, 0.5)
    }

    pub(crate) fn zoom_reframed_image_at(
        &mut self,
        direction: f64,
        focus_ratio_x: f64,
        focus_ratio_y: f64,
    ) -> bool {
        if self.document.base_image.is_none() {
            return false;
        }
        let scale_delta = if direction < 0.0 {
            1.12
        } else if direction > 0.0 {
            1.0 / 1.12
        } else {
            1.0
        };
        let before = self.document.clone();
        apply_zoom_with_focus(
            &mut self.document,
            &before,
            scale_delta,
            focus_ratio_x,
            focus_ratio_y,
        );
        if self.document_changed(&before) {
            self.undo_stack.push(before);
            self.redo_stack.clear();
            true
        } else {
            false
        }
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
        self.reframing_image = false;
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
        self.reframing_image = false;
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
            self.reframing_image = false;
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
            self.reframing_image = false;
            self.selected_annotation = None;
        }
        changed
    }

    pub(crate) fn clear_action(&mut self) -> ClearOutcome {
        if self.selected_annotation.is_some() {
            if self.delete_selected_annotation() {
                return ClearOutcome::DeletedSelectedAnnotation;
            }
            return ClearOutcome::None;
        }

        if self.clear_document_contents() {
            return ClearOutcome::ClearedDocument;
        }

        ClearOutcome::None
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
        if crop_w < 4 || crop_h < 4 {
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
            || self.document.output_ratio != previous.output_ratio
            || self.document.image_scale_mode != previous.image_scale_mode
            || self.document.image_anchor != previous.image_anchor
            || self.document.image_zoom != previous.image_zoom
            || self.document.image_offset_x != previous.image_offset_x
            || self.document.image_offset_y != previous.image_offset_y
            || format!("{:?}", self.document.annotations) != format!("{:?}", previous.annotations)
            || !same_background(&self.document.background, &previous.background)
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
            Background::Style {
                id: lid,
                intensity: li,
            },
            Background::Style {
                id: rid,
                intensity: ri,
            },
        ) => std::mem::discriminant(lid) == std::mem::discriminant(rid) && li == ri,
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

pub(crate) fn same_color_rgb(r: u8, g: u8, b: u8, color: &Color) -> bool {
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

fn same_optional_image(current: Option<&Image>, previous: Option<&Image>) -> bool {
    match (current, previous) {
        (Some(c), Some(p)) => c.width == p.width && c.height == p.height && c.data == p.data,
        (None, None) => true,
        _ => false,
    }
}

fn apply_zoom_with_focus(
    document: &mut Document,
    before: &Document,
    scale_delta: f64,
    focus_ratio_x: f64,
    focus_ratio_y: f64,
) {
    let Some(image) = before.base_image.as_ref() else {
        return;
    };

    *document = before.clone();
    document.image_scale_mode = snapix_core::canvas::ImageScaleMode::Fill;
    let bounds = natural_image_bounds(document);
    let focus_x = bounds.0 + bounds.2 * focus_ratio_x.clamp(0.0, 1.0);
    let focus_y = bounds.1 + bounds.3 * focus_ratio_y.clamp(0.0, 1.0);

    let Some(before_layout) = layout_for_document(image, bounds, document) else {
        return;
    };

    let image_x = ((focus_x - before_layout.image_x) / before_layout.image_scale)
        .clamp(0.0, image.width.saturating_sub(1) as f64);
    let image_y = ((focus_y - before_layout.image_y) / before_layout.image_scale)
        .clamp(0.0, image.height.saturating_sub(1) as f64);

    let current = before.image_zoom.max(1.0);
    document.image_zoom = (current * scale_delta as f32).clamp(1.0, 6.0);

    let Some(after_layout) = layout_for_document(image, bounds, document) else {
        return;
    };

    document.image_offset_x += ((focus_x
        - (after_layout.image_x + image_x * after_layout.image_scale))
        / after_layout.image_scale) as f32;
    document.image_offset_y += ((focus_y
        - (after_layout.image_y + image_y * after_layout.image_scale))
        / after_layout.image_scale) as f32;
}
