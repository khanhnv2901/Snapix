mod click;
mod dialog;
mod drag;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita::ToastOverlay;

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

        let draw_state = state.clone();
        let draw_blur_cache = blur_surface_cache.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let state = draw_state.borrow();
            if state.active_tool() == ToolKind::Crop {
                draw_crop_mode_canvas(cr, width, height, state.document());
                draw_crop_overlay(cr, &state, width, height);
            } else {
                draw_editor_canvas(
                    cr,
                    width,
                    height,
                    &state,
                    &mut draw_blur_cache.borrow_mut(),
                );
            }
        });

        drag::attach_drag_controller(&drawing_area, state.clone(), ui.clone());
        click::attach_click_controller(&drawing_area, state, ui);

        Self { drawing_area }
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }

    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
    }
}
