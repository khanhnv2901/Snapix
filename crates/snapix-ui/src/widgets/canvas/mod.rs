mod click;
mod dialog;
mod drag;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use libadwaita::ToastOverlay;
use snapix_core::canvas::Document;

use crate::editor::{EditorState, ToolKind};

use super::geometry::{draw_crop_mode_canvas, draw_crop_overlay};
use super::render::{draw_editor_canvas, BlurSurfaceCache};

#[derive(Clone)]
pub(super) struct CanvasUi {
    pub(super) subtitle_label: gtk4::Label,
    pub(super) scope_label: gtk4::Label,
    pub(super) width_label: gtk4::Label,
    pub(super) undo_button: gtk4::Button,
    pub(super) redo_button: gtk4::Button,
    pub(super) toast_overlay: ToastOverlay,
    pub(super) delete_button: gtk4::Button,
    pub(super) shared_width_scale: Rc<RefCell<Option<gtk4::Scale>>>,
    pub(super) shared_color_buttons: Rc<RefCell<Vec<((u8, u8, u8), gtk4::Button)>>>,
}

#[derive(Clone)]
pub struct DocumentCanvas {
    drawing_area: gtk4::DrawingArea,
}

impl DocumentCanvas {
    pub fn new(
        state: Rc<RefCell<EditorState>>,
        subtitle_label: gtk4::Label,
        scope_label: gtk4::Label,
        width_label: gtk4::Label,
        undo_button: gtk4::Button,
        redo_button: gtk4::Button,
        toast_overlay: ToastOverlay,
        delete_button: gtk4::Button,
        shared_width_scale: Rc<RefCell<Option<gtk4::Scale>>>,
        shared_color_buttons: Rc<RefCell<Vec<((u8, u8, u8), gtk4::Button)>>>,
    ) -> Self {
        let drawing_area = gtk4::DrawingArea::builder()
            .content_width(720)
            .content_height(480)
            .hexpand(true)
            .vexpand(true)
            .focusable(true)
            .build();

        let ui = CanvasUi {
            subtitle_label,
            scope_label,
            width_label,
            undo_button,
            redo_button,
            toast_overlay,
            delete_button,
            shared_width_scale,
            shared_color_buttons,
        };
        let blur_surface_cache = Rc::new(RefCell::new(BlurSurfaceCache::default()));
        let cache_for_updates = blur_surface_cache.clone();
        let drawing_area_weak = drawing_area.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
            let Some(drawing_area) = drawing_area_weak.upgrade() else {
                return glib::ControlFlow::Break;
            };

            if cache_for_updates.borrow_mut().poll_background_updates() {
                drawing_area.queue_draw();
            }

            glib::ControlFlow::Continue
        });

        let draw_state = state.clone();
        let scroll_state = state.clone();
        let zoom_state = state.clone();
        let draw_blur_cache = blur_surface_cache.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let state = draw_state.borrow();
            if state.active_tool() == ToolKind::Crop {
                draw_crop_mode_canvas(cr, width, height, state.document());
                draw_crop_overlay(cr, &state, width, height);
            } else {
                draw_editor_canvas(cr, width, height, &state, &mut draw_blur_cache.borrow_mut());
            }
        });

        drag::attach_drag_controller(&drawing_area, state.clone(), ui.clone());
        click::attach_click_controller(&drawing_area, state, ui);
        attach_scroll_controller(&drawing_area, scroll_state);
        attach_zoom_gesture(&drawing_area, zoom_state);

        Self { drawing_area }
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }

    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
    }
}

fn attach_scroll_controller(drawing_area: &gtk4::DrawingArea, state: Rc<RefCell<EditorState>>) {
    let scroll = gtk4::EventControllerScroll::new(
        gtk4::EventControllerScrollFlags::VERTICAL
            | gtk4::EventControllerScrollFlags::DISCRETE
            | gtk4::EventControllerScrollFlags::KINETIC,
    );
    let draw_target = drawing_area.clone();
    scroll.connect_scroll(move |_controller, _dx, dy| {
        let mut state = state.borrow_mut();
        if !state.is_reframing_image() {
            return glib::Propagation::Proceed;
        }

        if state.zoom_reframed_image(dy) {
            draw_target.queue_draw();
        }
        glib::Propagation::Stop
    });
    drawing_area.add_controller(scroll);
}

fn attach_zoom_gesture(drawing_area: &gtk4::DrawingArea, state: Rc<RefCell<EditorState>>) {
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
        zoom.connect_scale_changed(move |_gesture, scale| {
            let Some(before) = before_document.borrow().clone() else {
                return;
            };
            let mut state = state.borrow_mut();
            if !state.is_reframing_image() {
                return;
            }
            state.preview_zoom_image(&before, scale);
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
