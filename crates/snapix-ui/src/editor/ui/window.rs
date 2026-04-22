use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToastOverlay, ToolbarView};

use super::super::actions::{
    connect_capture_actions, connect_copy_button, connect_quick_save_button, connect_save_as_button,
};
use super::super::state::ToolKind;
use super::helpers::{
    refresh_history_buttons, refresh_labels, refresh_scope_label, refresh_subtitle,
    refresh_tool_actions,
};
use super::inspector::build_inspector;
use super::toolbar::{build_bottom_bar, build_canvas_panel, build_capture_row, build_tool_row};
use super::{BottomBar, HistoryAction, InspectorControls, SaveFormat};
use crate::app::LaunchContext;
use crate::editor::show_toast;
use crate::editor::state::EditorState;
use crate::widgets::DocumentCanvas;

pub struct EditorWindow {
    window: ApplicationWindow,
}

impl EditorWindow {
    pub fn new(app: &Application, context: LaunchContext) -> Self {
        let state = Rc::new(RefCell::new(EditorState::with_document(context.document)));

        let title_label = gtk4::Label::new(None);
        let scope_label = gtk4::Label::new(None);

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
            .label(super::helpers::width_label_text(&state.borrow()))
            .margin_start(12)
            .margin_end(2)
            .css_classes(["dim-copy"])
            .valign(gtk4::Align::Center)
            .build();

        let toast_overlay = ToastOverlay::new();
        let save_format = Rc::new(RefCell::new(SaveFormat::Png));
        let shared_width_scale: Rc<RefCell<Option<gtk4::Scale>>> = Rc::new(RefCell::new(None));
        let shared_color_buttons: Rc<RefCell<Vec<((u8, u8, u8), gtk4::Button)>>> =
            Rc::new(RefCell::new(Vec::new()));

        let canvas = DocumentCanvas::new(
            state.clone(),
            subtitle_label.clone(),
            scope_label.clone(),
            width_label.clone(),
            undo_button.clone(),
            redo_button.clone(),
            toast_overlay.clone(),
            delete_button.clone(),
            shared_width_scale.clone(),
            shared_color_buttons.clone(),
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
            shared_width_scale,
            shared_color_buttons,
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
            .margin_start(8)
            .margin_end(8)
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
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
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
            gtk4::gdk::Key::Escape if state.is_reframing_image() => {
                state.exit_image_reframe_mode();
                canvas.widget().set_cursor_from_name(None);
                refresh_labels(&state, &title_label, &subtitle_label);
                refresh_scope_label(&state, &scope_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                refresh_export_actions(&state, &bottom_bar);
                refresh_tool_actions(&state, &delete_button);
                inspector.refresh_from_state(&state);
                canvas.refresh();
                show_toast(&toast_overlay, "Image reframe ended");
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Escape if state.active_tool() == ToolKind::Crop => {
                state.cancel_crop_mode();
                refresh_labels(&state, &title_label, &subtitle_label);
                refresh_scope_label(&state, &scope_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                refresh_export_actions(&state, &bottom_bar);
                refresh_tool_actions(&state, &delete_button);
                inspector.refresh_from_state(&state);
                canvas.refresh();
                show_toast(&toast_overlay, "Crop canceled");
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
                } else {
                    show_toast(
                        &toast_overlay,
                        "Crop area is too small. Minimum size is 4×4 px.",
                    );
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
                    show_toast(&toast_overlay, "Annotation deleted");
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

pub(crate) fn refresh_export_actions(state: &EditorState, bottom_bar: &BottomBar) {
    let has_image = super::export_actions_enabled(state.document());
    bottom_bar.copy_button.set_sensitive(has_image);
    bottom_bar.quick_save_button.set_sensitive(has_image);
    bottom_bar.save_as_button.set_sensitive(has_image);
}
