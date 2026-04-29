use std::cell::RefCell;
use std::rc::Rc;

use glib::prelude::Cast;
use gtk4::prelude::*;
use snapix_core::canvas::{
    Annotation, Document, FrameSettings, Image, ImageAnchor, ImageScaleMode,
};

use crate::editor::i18n;
use crate::editor::state::{EditorState, ToolKind};
use crate::widgets::{composition_size, DocumentCanvas};

pub(crate) fn refresh_labels(
    state: &EditorState,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
) {
    title_label.set_label(&i18n::editor_header_title(state.active_tool()));
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
            let annotation_text = i18n::subtitle_annotations(annotation_count);
            let output_text = i18n::subtitle_output_text(output_width, output_height);
            let ratio_text = i18n::subtitle_ratio_text(document.output_ratio);
            let image_mode_text = image_mode_text(document.image_scale_mode, document.image_anchor);
            format!(
                "Image: {width}×{height} • {output_text} • {annotation_text} • {ratio_text} • {image_mode_text}"
            )
        }
        None => i18n::subtitle_text_empty().to_string(),
    }
}

fn image_mode_text(mode: ImageScaleMode, anchor: ImageAnchor) -> String {
    match mode {
        ImageScaleMode::Fit => i18n::image_mode_text_fit().to_string(),
        ImageScaleMode::Fill => i18n::image_mode_text_fill(anchor),
    }
}

pub(crate) fn shortcut_hint_text(state: &EditorState) -> Option<String> {
    if state.document().base_image.is_none() {
        return Some(i18n::shortcut_hint_empty().to_string());
    }

    match state.active_tool() {
        ToolKind::Select => {
            if state.is_reframing_image() {
                Some(i18n::shortcut_hint_reframe().to_string())
            } else if state.selected_annotation().is_some() {
                Some(i18n::shortcut_hint_selected().to_string())
            } else {
                Some(i18n::shortcut_hint_select_idle().to_string())
            }
        }
        ToolKind::Crop => Some(if state.has_pending_crop() {
            i18n::shortcut_hint_crop_active().to_string()
        } else {
            i18n::shortcut_hint_crop_idle().to_string()
        }),
        ToolKind::Arrow
        | ToolKind::Line
        | ToolKind::Rectangle
        | ToolKind::Ellipse
        | ToolKind::Blur => Some(i18n::shortcut_hint_draw_shape().to_string()),
        ToolKind::Text => Some(i18n::shortcut_hint_text().to_string()),
    }
}

pub(crate) fn export_actions_enabled(document: &Document) -> bool {
    document.base_image.is_some()
}

#[derive(Clone, Copy)]
pub(crate) struct ShadowDirectionPreset {
    pub(crate) label: &'static str,
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
}

pub(crate) const SHADOW_DIRECTION_PRESETS: [ShadowDirectionPreset; 9] = [
    ShadowDirectionPreset {
        label: "↖",
        offset_x: -18.0,
        offset_y: -18.0,
    },
    ShadowDirectionPreset {
        label: "↑",
        offset_x: 0.0,
        offset_y: -22.0,
    },
    ShadowDirectionPreset {
        label: "↗",
        offset_x: 18.0,
        offset_y: -18.0,
    },
    ShadowDirectionPreset {
        label: "←",
        offset_x: -24.0,
        offset_y: 0.0,
    },
    ShadowDirectionPreset {
        label: "·",
        offset_x: 0.0,
        offset_y: 0.0,
    },
    ShadowDirectionPreset {
        label: "→",
        offset_x: 24.0,
        offset_y: 0.0,
    },
    ShadowDirectionPreset {
        label: "↙",
        offset_x: -18.0,
        offset_y: 18.0,
    },
    ShadowDirectionPreset {
        label: "↓",
        offset_x: 0.0,
        offset_y: 22.0,
    },
    ShadowDirectionPreset {
        label: "↘",
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
    let is_text = match state
        .selected_annotation()
        .and_then(|index| state.document().annotations.get(index))
    {
        Some(Annotation::Text { .. }) => true,
        _ if state.active_tool() == ToolKind::Text => true,
        _ => false,
    };
    i18n::width_label_text(is_text)
}

pub(crate) fn scope_text(state: &EditorState) -> String {
    match state.active_tool() {
        ToolKind::Select => match state.selected_annotation() {
            _ if state.is_reframing_image() => i18n::scope_text_reframe().to_string(),
            Some(index) => {
                i18n::scope_text_selected(&annotation_kind_label(state.document(), index))
            }
            None => i18n::scope_text_select_idle().to_string(),
        },
        ToolKind::Crop => {
            if state.document().base_image.is_none() {
                i18n::scope_text_crop_empty().to_string()
            } else if state.has_pending_crop() {
                i18n::scope_text_crop_active().to_string()
            } else {
                i18n::scope_text_crop_idle().to_string()
            }
        }
        ToolKind::Arrow => i18n::scope_text_arrow().to_string(),
        ToolKind::Line => i18n::scope_text_line().to_string(),
        ToolKind::Rectangle => i18n::scope_text_rectangle().to_string(),
        ToolKind::Ellipse => i18n::scope_text_ellipse().to_string(),
        ToolKind::Text => i18n::scope_text_text().to_string(),
        ToolKind::Blur => i18n::scope_text_blur().to_string(),
    }
}

fn annotation_kind_label(document: &Document, index: usize) -> String {
    let kind = match document.annotations.get(index) {
        Some(Annotation::Arrow { .. }) => "arrow",
        Some(Annotation::Line { .. }) => "line",
        Some(Annotation::Rect { .. }) => "rectangle",
        Some(Annotation::Ellipse { .. }) => "ellipse",
        Some(Annotation::Text { .. }) => "text label",
        Some(Annotation::Blur { .. }) => "blur region",
        Some(Annotation::Redact { .. }) => "redaction",
        None => "annotation",
    };
    i18n::annotation_kind_label(kind)
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
    configure_inspector_slider(scale);
    let subtitle_label = subtitle_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    scale.connect_value_changed(move |scale| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        if state.update_document(|doc| update(&mut doc.frame, scale.value() as f32)) {
            refresh_subtitle(&state, &subtitle_label);
            refresh_history_buttons(&state, &undo_button, &redo_button);
            canvas.refresh();
        }
    });
}

pub(crate) fn configure_inspector_slider(scale: &gtk4::Scale) {
    disable_slider_scroll_wheel(scale);
    release_slider_focus_after_pointer_up(scale);
}

pub(crate) fn disable_slider_scroll_wheel(scale: &gtk4::Scale) {
    let controller = gtk4::EventControllerScroll::new(
        gtk4::EventControllerScrollFlags::VERTICAL
            | gtk4::EventControllerScrollFlags::DISCRETE
            | gtk4::EventControllerScrollFlags::KINETIC,
    );
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let scale_for_scroll = scale.clone();
    controller.connect_scroll(move |_controller, _dx, dy| {
        if let Some(scroller) = scale_for_scroll
            .ancestor(gtk4::ScrolledWindow::static_type())
            .and_then(|widget| widget.downcast::<gtk4::ScrolledWindow>().ok())
        {
            let adjustment = scroller.vadjustment();
            let page = adjustment.page_increment().max(48.0);
            let delta = if dy.abs() < f64::EPSILON {
                0.0
            } else {
                dy.signum() * page * 0.35
            };
            let upper_bound = (adjustment.upper() - adjustment.page_size()).max(adjustment.lower());
            let next = (adjustment.value() + delta).clamp(adjustment.lower(), upper_bound);
            adjustment.set_value(next);
        }
        glib::Propagation::Stop
    });
    scale.add_controller(controller);
}

fn release_slider_focus_after_pointer_up(scale: &gtk4::Scale) {
    let gesture = gtk4::GestureClick::new();
    let scale_for_release = scale.clone();
    gesture.connect_released(move |_gesture, _n_press, _x, _y| {
        if !scale_for_release.has_focus() {
            return;
        }
        if let Some(window) = scale_for_release
            .root()
            .and_then(|root| root.downcast::<gtk4::Window>().ok())
        {
            gtk4::prelude::GtkWindowExt::set_focus(&window, None::<&gtk4::Widget>);
        }
    });
    scale.add_controller(gesture);
}
