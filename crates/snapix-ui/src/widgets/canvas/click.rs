use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::editor::i18n;
use crate::editor::{
    refresh_history_buttons, refresh_scope_label, refresh_tool_actions, refresh_width_label,
    same_color_rgb, show_toast, EditorState, ToolKind,
};

use super::dialog::present_text_dialog;
use super::reframe::ReframePresentation;
use super::CanvasUi;
use crate::widgets::geometry::{
    hit_test_annotation, preview_canvas_layout, widget_point_to_image_pixel,
};

pub(super) fn attach_click_controller(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    ui: CanvasUi,
    reframe: ReframePresentation,
) {
    let click = gtk4::GestureClick::new();
    click.set_button(gtk4::gdk::BUTTON_PRIMARY);
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let ui = ui.clone();
        let reframe = reframe.clone();
        click.connect_pressed(move |_gesture, n_press, x, y| {
            let width = drawing_area.allocated_width();
            let height = drawing_area.allocated_height();
            let mut synced_style = None;
            let mut state_ref = state.borrow_mut();
            if state_ref.active_tool() == ToolKind::Select {
                let Some(layout) = preview_canvas_layout(state_ref.document(), width, height)
                else {
                    state_ref.set_selected_annotation(None);
                    refresh_width_label(&state_ref, &ui.width_label);
                    refresh_tool_actions(&state_ref, &ui.delete_button);
                    drawing_area.queue_draw();
                    return;
                };
                let clicked_image = widget_point_to_image_pixel(state_ref.document(), layout, x, y);
                let selected = hit_test_annotation(state_ref.document(), layout, x, y);
                state_ref.set_selected_annotation(selected);
                if state_ref.is_reframing_image() && n_press == 2 && selected.is_none() {
                    if clicked_image.is_some() {
                        if state_ref.recenter_image_reframe() {
                            refresh_scope_label(&state_ref, &ui.scope_label);
                            crate::editor::refresh_subtitle(&state_ref, &ui.subtitle_label);
                            refresh_history_buttons(&state_ref, &ui.undo_button, &ui.redo_button);
                            drawing_area.queue_draw();
                            show_toast(&ui.toast_overlay, i18n::image_view_reset_toast());
                        }
                    } else {
                        state_ref.exit_image_reframe_mode();
                        reframe.deactivate(&drawing_area);
                        refresh_scope_label(&state_ref, &ui.scope_label);
                        drawing_area.queue_draw();
                    }
                    return;
                }
                if selected.is_some() {
                    state_ref.sync_active_style_from_selected();
                    synced_style = Some((state_ref.active_color(), state_ref.active_width()));
                }
                let initial_text = if n_press == 2 {
                    state_ref.selected_text_content()
                } else {
                    None
                };
                let should_enter_reframe =
                    n_press == 2 && selected.is_none() && clicked_image.is_some();
                refresh_scope_label(&state_ref, &ui.scope_label);
                refresh_width_label(&state_ref, &ui.width_label);
                refresh_tool_actions(&state_ref, &ui.delete_button);
                drawing_area.queue_draw();
                if should_enter_reframe {
                    state_ref.enter_image_reframe_mode();
                    drawing_area.grab_focus();
                    reframe.activate(&drawing_area);
                    refresh_scope_label(&state_ref, &ui.scope_label);
                    crate::editor::refresh_subtitle(&state_ref, &ui.subtitle_label);
                    drawing_area.queue_draw();
                }
                drop(state_ref);

                sync_shared_style_controls(&ui, synced_style);

                if let Some(initial_text) = initial_text {
                    let Some(root) = drawing_area.root() else {
                        return;
                    };
                    let Ok(window) = root.downcast::<gtk4::ApplicationWindow>() else {
                        return;
                    };
                    present_text_dialog(
                        &window,
                        i18n::edit_text_dialog_title(),
                        i18n::edit_text_accept_button(),
                        i18n::text_content_field_label(),
                        &initial_text,
                        {
                            let state = state.clone();
                            let drawing_area = drawing_area.clone();
                            let ui = ui.clone();
                            move |content| {
                                let mut state = state.borrow_mut();
                                if state.update_selected_text_content(content) {
                                    refresh_scope_label(&state, &ui.scope_label);
                                    refresh_history_buttons(
                                        &state,
                                        &ui.undo_button,
                                        &ui.redo_button,
                                    );
                                    refresh_tool_actions(&state, &ui.delete_button);
                                    crate::editor::refresh_subtitle(&state, &ui.subtitle_label);
                                    drawing_area.queue_draw();
                                }
                            }
                        },
                    );
                } else if should_enter_reframe {
                    show_toast(&ui.toast_overlay, i18n::reframe_active_toast());
                }
                return;
            }

            if state_ref.active_tool() != ToolKind::Text {
                return;
            }

            let Some(layout) = preview_canvas_layout(state_ref.document(), width, height) else {
                return;
            };
            let Some((image_x, image_y)) =
                widget_point_to_image_pixel(state_ref.document(), layout, x, y)
            else {
                return;
            };
            state_ref.set_selected_annotation(None);
            refresh_width_label(&state_ref, &ui.width_label);
            refresh_tool_actions(&state_ref, &ui.delete_button);
            drop(state_ref);

            let Some(root) = drawing_area.root() else {
                return;
            };
            let Ok(window) = root.downcast::<gtk4::ApplicationWindow>() else {
                return;
            };

            let response_toast_overlay = ui.toast_overlay.clone();
            present_text_dialog(
                &window,
                i18n::add_text_dialog_title(),
                i18n::add_button_label(),
                i18n::text_content_field_label(),
                "",
                {
                    let state = state.clone();
                    let drawing_area = drawing_area.clone();
                    let ui = ui.clone();
                    move |content| {
                        let mut state = state.borrow_mut();
                        if state.add_text_annotation(image_x as f32, image_y as f32, content) {
                            refresh_scope_label(&state, &ui.scope_label);
                            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
                            refresh_tool_actions(&state, &ui.delete_button);
                            crate::editor::refresh_subtitle(&state, &ui.subtitle_label);
                            drawing_area.queue_draw();
                        } else {
                            show_toast(
                                &response_toast_overlay,
                                i18n::couldnt_add_text_label_toast(),
                            );
                        }
                    }
                },
            );
        });
    }
    drawing_area.add_controller(click);
}

fn sync_shared_style_controls(
    ui: &CanvasUi,
    synced_style: Option<(snapix_core::canvas::Color, f32)>,
) {
    if let Some((active_color, active_width)) = synced_style {
        if let Some(scale) = ui.shared_width_scale.borrow().as_ref() {
            scale.set_value(active_width as f64);
        }
        for ((r, g, b), btn) in ui.shared_color_buttons.borrow().iter() {
            if same_color_rgb(*r, *g, *b, &active_color) {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        }
    }
}
