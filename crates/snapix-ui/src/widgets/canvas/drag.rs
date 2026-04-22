use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::editor::i18n;
use crate::editor::{
    refresh_history_buttons, refresh_scope_label, refresh_tool_actions, show_toast, EditorState,
    ToolKind,
};
use crate::widgets::geometry::{
    adjusted_crop_bounds, canvas_layout, crop_rect_to_image_pixels, crop_selection_widget_bounds,
    hit_arrow_resize_handle, hit_crop_interaction, hit_resize_handle, hit_test_annotation,
    point_in_layout, preview_canvas_layout, resizable_annotation_widget_bounds,
    widget_point_to_image_pixel, widget_rect_to_image_pixels,
};
use crate::widgets::{
    AnnotationMoveSession, AnnotationResizeSession, ArrowResizeSession, CropInteractionSession,
    ANNOTATION_MOVE_THRESHOLD,
};

use super::reframe::ReframePresentation;
use super::CanvasUi;

pub(super) fn attach_drag_controller(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    ui: CanvasUi,
    reframe: ReframePresentation,
) {
    let crop_interaction = Rc::new(RefCell::new(None::<CropInteractionSession>));
    let annotation_move = Rc::new(RefCell::new(None::<AnnotationMoveSession>));
    let annotation_resize = Rc::new(RefCell::new(None::<AnnotationResizeSession>));
    let arrow_resize = Rc::new(RefCell::new(None::<ArrowResizeSession>));
    let image_reframe = Rc::new(RefCell::new(None::<snapix_core::canvas::Document>));
    let drag = gtk4::GestureDrag::new();

    connect_drag_begin(
        &drag,
        drawing_area,
        state.clone(),
        ui.clone(),
        crop_interaction.clone(),
        annotation_move.clone(),
        annotation_resize.clone(),
        arrow_resize.clone(),
        image_reframe.clone(),
        reframe.clone(),
    );
    connect_drag_update(
        &drag,
        drawing_area,
        state.clone(),
        crop_interaction.clone(),
        annotation_move.clone(),
        annotation_resize.clone(),
        arrow_resize.clone(),
        image_reframe.clone(),
        reframe.clone(),
    );
    connect_drag_end(
        &drag,
        drawing_area,
        state,
        ui,
        crop_interaction,
        annotation_move,
        annotation_resize,
        arrow_resize,
        image_reframe,
        reframe,
    );

    drawing_area.add_controller(drag);
}

fn connect_drag_begin(
    drag: &gtk4::GestureDrag,
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    ui: CanvasUi,
    crop_interaction: Rc<RefCell<Option<CropInteractionSession>>>,
    annotation_move: Rc<RefCell<Option<AnnotationMoveSession>>>,
    annotation_resize: Rc<RefCell<Option<AnnotationResizeSession>>>,
    arrow_resize: Rc<RefCell<Option<ArrowResizeSession>>>,
    image_reframe: Rc<RefCell<Option<snapix_core::canvas::Document>>>,
    reframe: ReframePresentation,
) {
    let drawing_area = drawing_area.clone();
    drag.connect_drag_begin(move |_gesture, x, y| {
        let width = drawing_area.allocated_width();
        let height = drawing_area.allocated_height();
        let mut state = state.borrow_mut();

        if state.active_tool() == ToolKind::Select {
            let Some(layout) = preview_canvas_layout(state.document(), width, height) else {
                return;
            };
            if state.is_reframing_image() && point_in_layout(x, y, layout) {
                *image_reframe.borrow_mut() = Some(state.document().clone());
                reframe.begin_drag(&drawing_area);
                drawing_area.grab_focus();
                drawing_area.queue_draw();
                return;
            }
            if let Some(index) = state.selected_annotation() {
                if let Some(annotation) = state.document().annotations.get(index) {
                    if let Some(move_start) = hit_arrow_resize_handle(layout, annotation, x, y) {
                        *arrow_resize.borrow_mut() = Some(ArrowResizeSession {
                            index,
                            move_start,
                            widget_start_x: x,
                            widget_start_y: y,
                            original: annotation.clone(),
                            before_document: state.document().clone(),
                        });
                        refresh_scope_label(&state, &ui.scope_label);
                        refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
                        drawing_area.grab_focus();
                        drawing_area.queue_draw();
                        return;
                    }
                }
            }
            if let Some(index) = state.selected_annotation() {
                if let Some(bounds) =
                    resizable_annotation_widget_bounds(state.document(), layout, index)
                {
                    if let Some(mode) = hit_resize_handle(bounds, x, y) {
                        let Some(original) = state.document().annotations.get(index).cloned()
                        else {
                            return;
                        };
                        *annotation_resize.borrow_mut() = Some(AnnotationResizeSession {
                            index,
                            mode,
                            initial_bounds: bounds,
                            original,
                            before_document: state.document().clone(),
                        });
                        refresh_scope_label(&state, &ui.scope_label);
                        refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
                        drawing_area.grab_focus();
                        drawing_area.queue_draw();
                        return;
                    }
                }
            }
            let Some(index) = hit_test_annotation(state.document(), layout, x, y) else {
                return;
            };
            let Some((image_x, image_y)) =
                widget_point_to_image_pixel(state.document(), layout, x, y)
            else {
                return;
            };
            let Some(original) = state.document().annotations.get(index).cloned() else {
                return;
            };

            state.set_selected_annotation(Some(index));
            *annotation_move.borrow_mut() = Some(AnnotationMoveSession {
                index,
                widget_start_x: x,
                widget_start_y: y,
                image_start_x: image_x,
                image_start_y: image_y,
                original,
                before_document: state.document().clone(),
            });
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            drawing_area.grab_focus();
            drawing_area.queue_draw();
            return;
        }

        if begin_tool_drag(&mut state, width, height, x, y) {
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            drawing_area.grab_focus();
            drawing_area.queue_draw();
            return;
        }

        if state.active_tool() != ToolKind::Crop {
            return;
        }

        let Some(layout) = canvas_layout(state.document(), width, height) else {
            return;
        };

        if let Some(selection) = state.crop_selection() {
            if let Some(selection_bounds) = crop_selection_widget_bounds(layout, selection) {
                if let Some(mode) = hit_crop_interaction(selection_bounds, x, y) {
                    *crop_interaction.borrow_mut() = Some(CropInteractionSession {
                        mode,
                        initial_bounds: selection_bounds,
                    });
                    refresh_scope_label(&state, &ui.scope_label);
                    drawing_area.grab_focus();
                    drawing_area.queue_draw();
                    return;
                }
            }
        }

        if point_in_layout(x, y, layout) {
            state.begin_crop_drag(x, y);
            *crop_interaction.borrow_mut() = None;
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            drawing_area.grab_focus();
            drawing_area.queue_draw();
        }
    });
}

fn connect_drag_update(
    drag: &gtk4::GestureDrag,
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    crop_interaction: Rc<RefCell<Option<CropInteractionSession>>>,
    annotation_move: Rc<RefCell<Option<AnnotationMoveSession>>>,
    annotation_resize: Rc<RefCell<Option<AnnotationResizeSession>>>,
    arrow_resize: Rc<RefCell<Option<ArrowResizeSession>>>,
    image_reframe: Rc<RefCell<Option<snapix_core::canvas::Document>>>,
    _reframe: ReframePresentation,
) {
    let drawing_area = drawing_area.clone();
    drag.connect_drag_update(move |_gesture, offset_x, offset_y| {
        let mut state = state.borrow_mut();

        if let Some(before_document) = image_reframe.borrow().clone() {
            state.preview_pan_image(&before_document, offset_x, offset_y);
        } else if let Some(session) = arrow_resize.borrow().clone() {
            let width = drawing_area.allocated_width();
            let height = drawing_area.allocated_height();
            if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                    state.document(),
                    layout,
                    session.widget_start_x + offset_x,
                    session.widget_start_y + offset_y,
                ) {
                    state.preview_resize_arrow_endpoint(
                        session.index,
                        &session.original,
                        session.move_start,
                        image_x as f32,
                        image_y as f32,
                    );
                }
            }
        } else if let Some(session) = annotation_resize.borrow().clone() {
            let width = drawing_area.allocated_width();
            let height = drawing_area.allocated_height();
            if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                if let Some((x, y, shape_width, shape_height)) = adjusted_crop_bounds(
                    layout,
                    session.initial_bounds,
                    session.mode,
                    offset_x,
                    offset_y,
                ) {
                    if let Some((image_x, image_y, image_width, image_height)) =
                        widget_rect_to_image_pixels(
                            state.document(),
                            layout,
                            x,
                            y,
                            shape_width,
                            shape_height,
                        )
                    {
                        state.preview_resize_annotation(
                            session.index,
                            &session.original,
                            image_x,
                            image_y,
                            image_width,
                            image_height,
                        );
                    }
                }
            }
        } else if let Some(session) = annotation_move.borrow().clone() {
            if offset_x.hypot(offset_y) < ANNOTATION_MOVE_THRESHOLD {
                return;
            }
            let width = drawing_area.allocated_width();
            let height = drawing_area.allocated_height();
            if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                    state.document(),
                    layout,
                    session.widget_start_x + offset_x,
                    session.widget_start_y + offset_y,
                ) {
                    state.preview_move_annotation(
                        session.index,
                        &session.original,
                        image_x as f32 - session.image_start_x as f32,
                        image_y as f32 - session.image_start_y as f32,
                    );
                }
            }
        } else if let Some(arrow_drag) = state.arrow_drag().cloned() {
            let width = drawing_area.allocated_width();
            let height = drawing_area.allocated_height();
            if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                    state.document(),
                    layout,
                    arrow_drag.widget_start_x() + offset_x,
                    arrow_drag.widget_start_y() + offset_y,
                ) {
                    state.update_arrow_drag(image_x as f32, image_y as f32);
                }
            }
        } else if let Some(rect_drag) = state.rect_drag().cloned() {
            state.update_rect_drag(
                rect_drag.start_x() + offset_x,
                rect_drag.start_y() + offset_y,
            );
        } else if let Some(ellipse_drag) = state.ellipse_drag().cloned() {
            state.update_ellipse_drag(
                ellipse_drag.start_x() + offset_x,
                ellipse_drag.start_y() + offset_y,
            );
        } else if let Some(blur_drag) = state.blur_drag().cloned() {
            state.update_blur_drag(
                blur_drag.start_x() + offset_x,
                blur_drag.start_y() + offset_y,
            );
        } else if let Some(crop_drag) = state.crop_drag().cloned() {
            state.update_crop_drag(
                crop_drag.start_x() + offset_x,
                crop_drag.start_y() + offset_y,
            );
        } else if let Some(session) = *crop_interaction.borrow() {
            let width = drawing_area.allocated_width();
            let height = drawing_area.allocated_height();
            if let Some(layout) = canvas_layout(state.document(), width, height) {
                if let Some((x, y, crop_width, crop_height)) = adjusted_crop_bounds(
                    layout,
                    session.initial_bounds,
                    session.mode,
                    offset_x,
                    offset_y,
                ) {
                    if let Some((image_x, image_y, image_width, image_height)) =
                        widget_rect_to_image_pixels(
                            state.document(),
                            layout,
                            x,
                            y,
                            crop_width,
                            crop_height,
                        )
                    {
                        state.set_crop_selection(image_x, image_y, image_width, image_height);
                    }
                }
            }
        } else {
            return;
        }

        drawing_area.queue_draw();
    });
}

fn connect_drag_end(
    drag: &gtk4::GestureDrag,
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    ui: CanvasUi,
    crop_interaction: Rc<RefCell<Option<CropInteractionSession>>>,
    annotation_move: Rc<RefCell<Option<AnnotationMoveSession>>>,
    annotation_resize: Rc<RefCell<Option<AnnotationResizeSession>>>,
    arrow_resize: Rc<RefCell<Option<ArrowResizeSession>>>,
    image_reframe: Rc<RefCell<Option<snapix_core::canvas::Document>>>,
    reframe: ReframePresentation,
) {
    let drawing_area = drawing_area.clone();
    drag.connect_drag_end(move |_gesture, offset_x, offset_y| {
        let width = drawing_area.allocated_width();
        let height = drawing_area.allocated_height();
        let mut state = state.borrow_mut();

        if let Some(before_document) = image_reframe.borrow_mut().take() {
            state.preview_pan_image(&before_document, offset_x, offset_y);
            state.finalize_image_reframe(before_document);
            reframe.end_drag(&drawing_area, &state);
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            refresh_tool_actions(&state, &ui.delete_button);
        } else if let Some(session) = arrow_resize.borrow_mut().take() {
            if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                    state.document(),
                    layout,
                    session.widget_start_x + offset_x,
                    session.widget_start_y + offset_y,
                ) {
                    state.preview_resize_arrow_endpoint(
                        session.index,
                        &session.original,
                        session.move_start,
                        image_x as f32,
                        image_y as f32,
                    );
                }
            }
            state.finalize_annotation_move(session.before_document);
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            refresh_tool_actions(&state, &ui.delete_button);
        } else if let Some(session) = annotation_resize.borrow_mut().take() {
            if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                if let Some((x, y, shape_width, shape_height)) = adjusted_crop_bounds(
                    layout,
                    session.initial_bounds,
                    session.mode,
                    offset_x,
                    offset_y,
                ) {
                    if let Some((image_x, image_y, image_width, image_height)) =
                        widget_rect_to_image_pixels(
                            state.document(),
                            layout,
                            x,
                            y,
                            shape_width,
                            shape_height,
                        )
                    {
                        state.preview_resize_annotation(
                            session.index,
                            &session.original,
                            image_x,
                            image_y,
                            image_width,
                            image_height,
                        );
                    }
                }
            }
            state.finalize_annotation_move(session.before_document);
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            refresh_tool_actions(&state, &ui.delete_button);
        } else if let Some(session) = annotation_move.borrow_mut().take() {
            if offset_x.hypot(offset_y) >= ANNOTATION_MOVE_THRESHOLD {
                if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                    if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                        state.document(),
                        layout,
                        session.widget_start_x + offset_x,
                        session.widget_start_y + offset_y,
                    ) {
                        state.preview_move_annotation(
                            session.index,
                            &session.original,
                            image_x as f32 - session.image_start_x as f32,
                            image_y as f32 - session.image_start_y as f32,
                        );
                    }
                }
                state.finalize_annotation_move(session.before_document);
            }
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            refresh_tool_actions(&state, &ui.delete_button);
        } else if let Some(arrow_drag) = state.arrow_drag().cloned() {
            if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
                if let Some((image_x, image_y)) = widget_point_to_image_pixel(
                    state.document(),
                    layout,
                    arrow_drag.widget_start_x() + offset_x,
                    arrow_drag.widget_start_y() + offset_y,
                ) {
                    state.update_arrow_drag(image_x as f32, image_y as f32);
                }
            }
            if !state.commit_arrow_drag() {
                show_toast(&ui.toast_overlay, i18n::arrow_too_small_toast());
            }
            refresh_scope_label(&state, &ui.scope_label);
            refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
            refresh_tool_actions(&state, &ui.delete_button);
        } else if let Some(rect_drag) = state.rect_drag().cloned() {
            state.update_rect_drag(
                rect_drag.start_x() + offset_x,
                rect_drag.start_y() + offset_y,
            );
            commit_shape_drag(
                &mut state,
                width,
                height,
                &ui,
                &drawing_area,
                &rect_drag,
                i18n::rectangle_too_small_toast(),
                |state, x, y, w, h| state.commit_rect_annotation(x, y, w, h),
                EditorState::clear_rect_drag,
            );
        } else if let Some(ellipse_drag) = state.ellipse_drag().cloned() {
            state.update_ellipse_drag(
                ellipse_drag.start_x() + offset_x,
                ellipse_drag.start_y() + offset_y,
            );
            commit_shape_drag(
                &mut state,
                width,
                height,
                &ui,
                &drawing_area,
                &ellipse_drag,
                i18n::ellipse_too_small_toast(),
                |state, x, y, w, h| state.commit_ellipse_annotation(x, y, w, h),
                EditorState::clear_ellipse_drag,
            );
        } else if let Some(blur_drag) = state.blur_drag().cloned() {
            state.update_blur_drag(
                blur_drag.start_x() + offset_x,
                blur_drag.start_y() + offset_y,
            );
            commit_shape_drag(
                &mut state,
                width,
                height,
                &ui,
                &drawing_area,
                &blur_drag,
                i18n::blur_too_small_toast(),
                |state, x, y, w, h| state.commit_blur_rect(x, y, w, h),
                EditorState::clear_blur_drag,
            );
        } else if let Some(crop_drag) = state.crop_drag().cloned() {
            state.update_crop_drag(
                crop_drag.start_x() + offset_x,
                crop_drag.start_y() + offset_y,
            );
            let final_crop_drag = state.crop_drag().cloned();

            if let Some(layout) = canvas_layout(state.document(), width, height) {
                if let Some((crop_x, crop_y, crop_width, crop_height)) =
                    final_crop_drag.as_ref().and_then(|crop_drag| {
                        crop_rect_to_image_pixels(state.document(), layout, crop_drag)
                    })
                {
                    state.set_crop_selection(crop_x, crop_y, crop_width, crop_height);
                    refresh_scope_label(&state, &ui.scope_label);
                    refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
                    refresh_tool_actions(&state, &ui.delete_button);
                } else {
                    state.clear_crop_drag();
                    state.ensure_default_crop_selection();
                    refresh_scope_label(&state, &ui.scope_label);
                    refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
                    refresh_tool_actions(&state, &ui.delete_button);
                    show_toast(&ui.toast_overlay, i18n::crop_too_small_toast());
                }
            } else {
                state.clear_crop_drag();
                state.ensure_default_crop_selection();
                refresh_scope_label(&state, &ui.scope_label);
                refresh_history_buttons(&state, &ui.undo_button, &ui.redo_button);
                refresh_tool_actions(&state, &ui.delete_button);
                show_toast(&ui.toast_overlay, i18n::crop_too_small_toast());
            }
        }
        *crop_interaction.borrow_mut() = None;

        drawing_area.grab_focus();
        drawing_area.queue_draw();
    });
}

fn begin_tool_drag(state: &mut EditorState, width: i32, height: i32, x: f64, y: f64) -> bool {
    let Some(layout) = preview_canvas_layout(state.document(), width, height) else {
        return false;
    };

    match state.active_tool() {
        ToolKind::Arrow => {
            if let Some((image_x, image_y)) =
                widget_point_to_image_pixel(state.document(), layout, x, y)
            {
                state.begin_arrow_drag(x, y, image_x as f32, image_y as f32);
                true
            } else {
                false
            }
        }
        ToolKind::Rectangle => {
            if point_in_layout(x, y, layout) {
                state.begin_rect_drag(x, y);
                true
            } else {
                false
            }
        }
        ToolKind::Ellipse => {
            if point_in_layout(x, y, layout) {
                state.begin_ellipse_drag(x, y);
                true
            } else {
                false
            }
        }
        ToolKind::Blur => {
            if point_in_layout(x, y, layout) {
                state.begin_blur_drag(x, y);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn commit_shape_drag<F, C>(
    state: &mut EditorState,
    width: i32,
    height: i32,
    ui: &CanvasUi,
    drawing_area: &gtk4::DrawingArea,
    drag: &crate::editor::CropDrag,
    too_small_message: &str,
    commit: F,
    clear: C,
) where
    F: Fn(&mut EditorState, u32, u32, u32, u32) -> bool,
    C: Fn(&mut EditorState),
{
    if let Some(layout) = preview_canvas_layout(state.document(), width, height) {
        if let Some((x, y, w, h)) = crop_rect_to_image_pixels(state.document(), layout, drag) {
            if !commit(state, x, y, w, h) {
                show_toast(&ui.toast_overlay, too_small_message);
            }
            refresh_scope_label(state, &ui.scope_label);
            refresh_history_buttons(state, &ui.undo_button, &ui.redo_button);
            refresh_tool_actions(state, &ui.delete_button);
        } else {
            clear(state);
            show_toast(&ui.toast_overlay, too_small_message);
        }
    } else {
        clear(state);
        show_toast(&ui.toast_overlay, too_small_message);
    }

    drawing_area.queue_draw();
}
