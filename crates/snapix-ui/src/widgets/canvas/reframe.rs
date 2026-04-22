use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use snapix_core::canvas::Document;

use crate::editor::EditorState;
use crate::widgets::geometry::{point_in_layout, preview_canvas_layout};

#[derive(Clone)]
pub(super) struct ReframePresentation {
    overlay_opacity: Rc<Cell<f64>>,
    dragging: Rc<Cell<bool>>,
    pointer_over_image: Rc<Cell<bool>>,
}

impl ReframePresentation {
    pub(super) fn overlay_opacity(&self) -> f64 {
        self.overlay_opacity.get()
    }

    pub(super) fn activate(&self, drawing_area: &gtk4::DrawingArea) {
        self.pointer_over_image.set(true);
        self.dragging.set(false);
        refresh_cursor(drawing_area, true, false, true);
    }

    pub(super) fn deactivate(&self, drawing_area: &gtk4::DrawingArea) {
        self.dragging.set(false);
        self.pointer_over_image.set(false);
        refresh_cursor(drawing_area, false, false, false);
    }

    pub(super) fn begin_drag(&self, drawing_area: &gtk4::DrawingArea) {
        self.dragging.set(true);
        refresh_cursor(drawing_area, true, true, self.pointer_over_image.get());
    }

    pub(super) fn end_drag(&self, drawing_area: &gtk4::DrawingArea, state: &EditorState) {
        self.dragging.set(false);
        refresh_cursor(
            drawing_area,
            state.is_reframing_image(),
            false,
            self.pointer_over_image.get(),
        );
    }
}

pub(super) fn attach_reframe_support(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
) -> ReframePresentation {
    let overlay_opacity = Rc::new(Cell::new(0.0));
    let dragging = Rc::new(Cell::new(false));
    let pointer_over_image = Rc::new(Cell::new(false));
    let pointer_position = Rc::new(Cell::new(None));
    let presentation = ReframePresentation {
        overlay_opacity: overlay_opacity.clone(),
        dragging: dragging.clone(),
        pointer_over_image: pointer_over_image.clone(),
    };

    attach_overlay_animation(drawing_area, state.clone(), overlay_opacity);
    attach_scroll_controller(drawing_area, state.clone(), pointer_position.clone());
    attach_zoom_gesture(drawing_area, state.clone(), pointer_position.clone());
    attach_motion_controller(
        drawing_area,
        state,
        dragging,
        pointer_over_image,
        pointer_position,
    );

    presentation
}

fn attach_overlay_animation(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    overlay_opacity: Rc<Cell<f64>>,
) {
    let drawing_area_weak = drawing_area.downgrade();
    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let Some(drawing_area) = drawing_area_weak.upgrade() else {
            return glib::ControlFlow::Break;
        };

        let target = if state.borrow().is_reframing_image() {
            1.0
        } else {
            0.0
        };
        let current = overlay_opacity.get();
        let next = current + (target - current) * 0.28;

        if (next - current).abs() > 0.01 {
            overlay_opacity.set(next);
            drawing_area.queue_draw();
        } else if (current - target).abs() > 0.001 {
            overlay_opacity.set(target);
            drawing_area.queue_draw();
        }

        glib::ControlFlow::Continue
    });
}

fn attach_scroll_controller(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    pointer_position: Rc<Cell<Option<(f64, f64)>>>,
) {
    let scroll = gtk4::EventControllerScroll::new(
        gtk4::EventControllerScrollFlags::VERTICAL
            | gtk4::EventControllerScrollFlags::DISCRETE
            | gtk4::EventControllerScrollFlags::KINETIC,
    );
    let draw_target = drawing_area.clone();
    scroll.connect_scroll(move |_controller, _dx, dy| {
        let focus_ratio = pointer_position.get().and_then(|(x, y)| {
            let state = state.borrow();
            focus_ratio_for_widget_point(&state, &draw_target, x, y)
        });
        let mut state = state.borrow_mut();
        if !state.is_reframing_image() {
            return glib::Propagation::Proceed;
        }

        let changed = if let Some((focus_ratio_x, focus_ratio_y)) = focus_ratio {
            state.zoom_reframed_image_at(dy, focus_ratio_x, focus_ratio_y)
        } else {
            state.zoom_reframed_image(dy)
        };
        if changed {
            draw_target.queue_draw();
        }
        glib::Propagation::Stop
    });
    drawing_area.add_controller(scroll);
}

fn attach_zoom_gesture(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    pointer_position: Rc<Cell<Option<(f64, f64)>>>,
) {
    let zoom = gtk4::GestureZoom::new();
    let before_document = Rc::new(RefCell::new(None::<Document>));

    {
        let state = state.clone();
        let before_document = before_document.clone();
        zoom.connect_begin(move |_gesture, _sequence| {
            let state = state.borrow();
            if !state.is_reframing_image() {
                return;
            }
            *before_document.borrow_mut() = Some(state.document().clone());
        });
    }

    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let before_document = before_document.clone();
        let pointer_position = pointer_position.clone();
        zoom.connect_scale_changed(move |gesture, scale| {
            let Some(before) = before_document.borrow().clone() else {
                return;
            };
            let focus_ratio = gesture
                .bounding_box_center()
                .or(pointer_position.get())
                .and_then(|(x, y)| {
                    let state = state.borrow();
                    focus_ratio_for_widget_point(&state, &drawing_area, x, y)
                });
            let mut state = state.borrow_mut();
            if !state.is_reframing_image() {
                return;
            }
            if let Some((focus_ratio_x, focus_ratio_y)) = focus_ratio {
                state.preview_zoom_image_at(&before, scale, focus_ratio_x, focus_ratio_y);
            } else {
                state.preview_zoom_image(&before, scale);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        zoom.connect_end(move |_gesture, _sequence| {
            let Some(before) = before_document.borrow_mut().take() else {
                return;
            };
            let mut state = state.borrow_mut();
            if state.is_reframing_image() {
                state.finalize_image_reframe(before);
                drawing_area.queue_draw();
            }
        });
    }

    drawing_area.add_controller(zoom);
}

fn attach_motion_controller(
    drawing_area: &gtk4::DrawingArea,
    state: Rc<RefCell<EditorState>>,
    dragging: Rc<Cell<bool>>,
    pointer_over_image: Rc<Cell<bool>>,
    pointer_position: Rc<Cell<Option<(f64, f64)>>>,
) {
    let motion = gtk4::EventControllerMotion::new();

    {
        let drawing_area = drawing_area.clone();
        let state = state.clone();
        let dragging = dragging.clone();
        let pointer_over_image = pointer_over_image.clone();
        let pointer_position = pointer_position.clone();
        motion.connect_motion(move |_controller, x, y| {
            let state = state.borrow();
            let over_image = preview_canvas_layout(
                state.document(),
                drawing_area.allocated_width(),
                drawing_area.allocated_height(),
            )
            .map(|layout| point_in_layout(x, y, layout))
            .unwrap_or(false);
            pointer_position.set(Some((x, y)));
            pointer_over_image.set(over_image);
            refresh_cursor(
                &drawing_area,
                state.is_reframing_image(),
                dragging.get(),
                over_image,
            );
        });
    }

    {
        let drawing_area = drawing_area.clone();
        let state = state.clone();
        let dragging = dragging.clone();
        let pointer_over_image = pointer_over_image.clone();
        let pointer_position = pointer_position.clone();
        motion.connect_enter(move |_controller, x, y| {
            let state = state.borrow();
            let over_image = preview_canvas_layout(
                state.document(),
                drawing_area.allocated_width(),
                drawing_area.allocated_height(),
            )
            .map(|layout| point_in_layout(x, y, layout))
            .unwrap_or(false);
            pointer_position.set(Some((x, y)));
            pointer_over_image.set(over_image);
            refresh_cursor(
                &drawing_area,
                state.is_reframing_image(),
                dragging.get(),
                over_image,
            );
        });
    }

    {
        let drawing_area = drawing_area.clone();
        let state = state.clone();
        let dragging = dragging.clone();
        let pointer_over_image = pointer_over_image.clone();
        motion.connect_leave(move |_controller| {
            pointer_position.set(None);
            pointer_over_image.set(false);
            refresh_cursor(
                &drawing_area,
                state.borrow().is_reframing_image(),
                dragging.get(),
                false,
            );
        });
    }

    drawing_area.add_controller(motion);
}

fn focus_ratio_for_widget_point(
    state: &EditorState,
    drawing_area: &gtk4::DrawingArea,
    x: f64,
    y: f64,
) -> Option<(f64, f64)> {
    let layout = preview_canvas_layout(
        state.document(),
        drawing_area.allocated_width(),
        drawing_area.allocated_height(),
    )?;
    if !point_in_layout(x, y, layout) {
        return None;
    }

    Some((
        ((x - layout.viewport_x) / layout.viewport_width).clamp(0.0, 1.0),
        ((y - layout.viewport_y) / layout.viewport_height).clamp(0.0, 1.0),
    ))
}

fn refresh_cursor(
    drawing_area: &gtk4::DrawingArea,
    active: bool,
    dragging: bool,
    pointer_over_image: bool,
) {
    let cursor_name = if active && dragging {
        Some("grabbing")
    } else if active && pointer_over_image {
        Some("grab")
    } else {
        None
    };
    drawing_area.set_cursor_from_name(cursor_name);
}
