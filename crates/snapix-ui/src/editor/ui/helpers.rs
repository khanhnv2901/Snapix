use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use snapix_core::canvas::{Annotation, Document, FrameSettings, Image, ImageAnchor, ImageScaleMode, OutputRatio};

use crate::editor::state::{EditorState, ToolKind};
use crate::widgets::{composition_size, DocumentCanvas};

pub(crate) fn refresh_labels(
    state: &EditorState,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
) {
    title_label.set_label(&format!("Editor • {}", state.active_tool.label()));
    refresh_subtitle(state, subtitle_label);
}

pub(crate) fn refresh_subtitle(state: &EditorState, subtitle_label: &gtk4::Label) {
    let text = match shortcut_hint_text(state) {
        Some(hint) => format!("{}  •  {}", subtitle_text(state.document()), hint),
        None => subtitle_text(state.document()),
    };
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

pub(crate) fn subtitle_text(document: &Document) -> String {
    match document.base_image.as_ref() {
        Some(Image { width, height, .. }) => {
            let (output_width, output_height) = composition_size(document);
            let annotation_count = document.annotations.len();
            let annotation_text = match annotation_count {
                0 => "no annotations yet".to_string(),
                1 => "1 annotation".to_string(),
                count => format!("{count} annotations"),
            };
            let output_text = format!(
                "output {}×{}",
                output_width.round() as u32,
                output_height.round() as u32
            );
            let ratio_text = match document.output_ratio {
                OutputRatio::Auto => "ratio auto".to_string(),
                ratio => format!("ratio {}", ratio.label()),
            };
            let image_mode_text = image_mode_text(document.image_scale_mode, document.image_anchor);
            format!(
                "Image: {width}×{height} • {output_text} • {annotation_text} • {ratio_text} • {image_mode_text}"
            )
        }
        None => "No image loaded. Capture or import an image to begin.".to_string(),
    }
}

fn image_mode_text(mode: ImageScaleMode, anchor: ImageAnchor) -> String {
    match mode {
        ImageScaleMode::Fit => "image fit".to_string(),
        ImageScaleMode::Fill => format!("image fill {}", image_anchor_label(anchor)),
    }
}

fn image_anchor_label(anchor: ImageAnchor) -> &'static str {
    match anchor {
        ImageAnchor::TopLeft => "top-left",
        ImageAnchor::Top => "top",
        ImageAnchor::TopRight => "top-right",
        ImageAnchor::Left => "left",
        ImageAnchor::Center => "center",
        ImageAnchor::Right => "right",
        ImageAnchor::BottomLeft => "bottom-left",
        ImageAnchor::Bottom => "bottom",
        ImageAnchor::BottomRight => "bottom-right",
    }
}

pub(crate) fn shortcut_hint_text(state: &EditorState) -> Option<String> {
    if state.document().base_image.is_none() {
        return Some("Fullscreen / Region / Import to begin".to_string());
    }

    match state.active_tool() {
        ToolKind::Select => {
            if state.selected_annotation().is_some() {
                Some("Delete remove • Ctrl+Z undo".to_string())
            } else {
                Some("Click annotation to edit • Ctrl+Z undo".to_string())
            }
        }
        ToolKind::Crop => Some(if state.has_pending_crop() {
            "Enter apply • Esc cancel".to_string()
        } else {
            "Drag to select • Esc cancel".to_string()
        }),
        ToolKind::Arrow | ToolKind::Rectangle | ToolKind::Ellipse | ToolKind::Blur => {
            Some("Drag on image to draw • Ctrl+Z undo".to_string())
        }
        ToolKind::Text => Some("Click to place • Double-click text to edit".to_string()),
    }
}

pub(crate) fn export_actions_enabled(document: &Document) -> bool {
    document.base_image.is_some()
}

#[derive(Clone, Copy)]
pub(crate) struct ShadowDirectionPreset {
    pub(crate) label: &'static str,
    pub(crate) tooltip: &'static str,
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
}

pub(crate) const SHADOW_DIRECTION_PRESETS: [ShadowDirectionPreset; 9] = [
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

pub(crate) fn nearest_shadow_direction_index(offset_x: f32, offset_y: f32) -> usize {
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

pub(crate) fn width_label_text(state: &EditorState) -> &'static str {
    match state
        .selected_annotation()
        .and_then(|index| state.document().annotations.get(index))
    {
        Some(Annotation::Text { .. }) => "Size:",
        _ if state.active_tool() == ToolKind::Text => "Size:",
        _ => "Width:",
    }
}

pub(crate) fn scope_text(state: &EditorState) -> String {
    match state.active_tool() {
        ToolKind::Select => match state.selected_annotation() {
            Some(index) => format!(
                "Selected {}. Adjust color or size, or press Delete to remove it.",
                annotation_kind_label(state.document(), index)
            ),
            None => {
                "Select: click an annotation to edit it. Press Delete to remove it, Ctrl+Z to undo."
                    .to_string()
            }
        },
        ToolKind::Crop => {
            if state.document().base_image.is_none() {
                "Crop: capture or import an image first.".to_string()
            } else if state.has_pending_crop() {
                "Crop: drag handles to adjust, press Enter to apply, or Esc to cancel."
                    .to_string()
            } else {
                "Crop: drag on the image to create a selection, then press Enter to apply."
                    .to_string()
            }
        }
        ToolKind::Arrow => "Arrow: drag on the image to place an arrow.".to_string(),
        ToolKind::Rectangle => "Rectangle: drag on the image to draw a box.".to_string(),
        ToolKind::Ellipse => "Ellipse: drag on the image to draw an oval.".to_string(),
        ToolKind::Text => "Text: click on the image to place a label. Double-click text to edit it."
            .to_string(),
        ToolKind::Blur => "Blur: drag on the image to blur part of the image.".to_string(),
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

pub(crate) fn refresh_tool_actions(state: &EditorState, delete_button: &gtk4::Button) {
    delete_button.set_sensitive(state.selected_annotation().is_some());
}

pub(crate) fn refresh_width_label(state: &EditorState, width_label: &gtk4::Label) {
    width_label.set_label(width_label_text(state));
}

pub(crate) fn connect_frame_slider<F>(
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
