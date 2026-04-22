mod click;
mod dialog;
mod drag;
mod reframe;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use libadwaita::ToastOverlay;

use crate::editor::{EditorState, ToolKind};

use self::reframe::attach_reframe_support;
use super::geometry::{draw_crop_mode_canvas, draw_crop_overlay};
use super::render::{draw_editor_canvas, BlurSurfaceCache};

pub(crate) type SharedColorButton = ((u8, u8, u8), gtk4::Button);
pub(crate) type SharedColorButtons = Rc<RefCell<Vec<SharedColorButton>>>;

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
    pub(super) shared_color_buttons: SharedColorButtons,
}

#[derive(Clone)]
pub struct DocumentCanvas {
    drawing_area: gtk4::DrawingArea,
}

impl DocumentCanvas {
    #[allow(clippy::too_many_arguments)]
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
        shared_color_buttons: SharedColorButtons,
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
        let reframe = attach_reframe_support(&drawing_area, state.clone());
        let draw_reframe = reframe.clone();
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
                    draw_reframe.overlay_opacity(),
                    &mut draw_blur_cache.borrow_mut(),
                );
            }
        });

        drag::attach_drag_controller(&drawing_area, state.clone(), ui.clone(), reframe.clone());
        click::attach_click_controller(&drawing_area, state, ui, reframe);

        Self { drawing_area }
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }

    pub fn refresh(&self) {
        self.drawing_area.queue_draw();
    }
}
