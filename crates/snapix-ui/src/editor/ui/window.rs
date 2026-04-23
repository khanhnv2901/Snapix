use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToastOverlay, ToolbarView};

use super::super::actions::{
    connect_capture_actions, connect_copy_button, connect_quick_save_button, connect_save_as_button,
    paste_image_from_clipboard,
};
use super::super::i18n;
use super::super::i18n::{
    app_window_title, delete_tooltip, preferences_button_tooltip, redo_tooltip, undo_tooltip,
};
use super::super::preferences::{
    load_preferences, save_preferences, AppPreferences, PreferredSaveFormat,
};
use super::super::state::ToolKind;
use super::helpers::{
    refresh_history_buttons, refresh_labels, refresh_scope_label, refresh_subtitle,
    refresh_tool_actions,
};
use super::inspector::build_inspector;
use super::preferences::present_preferences_window;
use super::toolbar::{
    build_bottom_bar, build_canvas_panel, build_capture_row, build_reframe_overlay_actions,
    build_tool_row,
};
use super::{BottomBar, HistoryAction, InspectorControls, SaveFormat};
use crate::app::LaunchContext;
use crate::editor::show_toast;
use crate::editor::state::EditorState;
use crate::widgets::{DocumentCanvas, SharedColorButtons};

pub struct EditorWindow {
    window: ApplicationWindow,
}

impl EditorWindow {
    pub fn new(app: &Application, context: LaunchContext) -> Self {
        let state = Rc::new(RefCell::new(EditorState::with_document(context.document)));
        let preferences = Rc::new(RefCell::new(load_preferences().unwrap_or_else(|error| {
            tracing::warn!("Failed to load preferences: {error:#}");
            AppPreferences::default()
        })));

        let title_label = gtk4::Label::new(None);
        let scope_label = gtk4::Label::new(None);

        let subtitle_label = gtk4::Label::builder()
            .xalign(0.0)
            .css_classes(["dim-copy"])
            .build();

        let undo_button = gtk4::Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text(undo_tooltip())
            .sensitive(false)
            .build();
        let redo_button = gtk4::Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text(redo_tooltip())
            .sensitive(false)
            .build();
        let delete_button = gtk4::Button::builder()
            .icon_name("edit-delete-symbolic")
            .tooltip_text(delete_tooltip())
            .css_classes(["tool-delete-btn"])
            .sensitive(false)
            .build();
        let preferences_button = gtk4::Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text(preferences_button_tooltip())
            .build();
        let width_label = gtk4::Label::builder()
            .label(super::helpers::width_label_text(&state.borrow()))
            .margin_start(12)
            .margin_end(2)
            .css_classes(["dim-copy"])
            .valign(gtk4::Align::Center)
            .build();

        let toast_overlay = ToastOverlay::new();
        let save_format = Rc::new(RefCell::new(
            match preferences.borrow().effective_save_format() {
                super::super::preferences::PreferredSaveFormat::Png => SaveFormat::Png,
                super::super::preferences::PreferredSaveFormat::Jpeg => SaveFormat::Jpeg,
            },
        ));
        let shared_width_scale: Rc<RefCell<Option<gtk4::Scale>>> = Rc::new(RefCell::new(None));
        let shared_color_buttons: SharedColorButtons = Rc::new(RefCell::new(Vec::new()));

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
        let bottom_bar = build_bottom_bar(&subtitle_label, save_format.clone());
        let capture_row = build_capture_row(&bottom_bar);
        let canvas_panel = build_canvas_panel(canvas_widget);
        let (reframe_overlay_actions, reframe_reset_button, reframe_done_button) =
            build_reframe_overlay_actions();
        let canvas_overlay = gtk4::Overlay::new();
        canvas_overlay.set_child(Some(&canvas_panel));
        canvas_overlay.add_overlay(&reframe_overlay_actions);
        canvas_overlay.set_measure_overlay(&reframe_overlay_actions, true);

        let workspace = gtk4::Paned::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .hexpand(true)
            .vexpand(true)
            .wide_handle(false)
            .build();
        workspace.set_resize_start_child(true);
        workspace.set_resize_end_child(false);
        workspace.set_shrink_start_child(false);
        workspace.set_shrink_end_child(false);
        workspace.set_start_child(Some(&canvas_overlay));
        workspace.set_end_child(Some(&inspector.widget()));

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .build();
        content.add_css_class("snapix-shell");
        content.append(&capture_row.widget);
        content.append(&tool_row);
        content.append(&workspace);

        let header = HeaderBar::new();
        header.pack_end(&preferences_button);
        header.pack_end(&redo_button);
        header.pack_end(&undo_button);

        toast_overlay.set_child(Some(&content));
        toast_overlay.add_css_class("snapix-shell");

        let toolbar_view = ToolbarView::new();
        toolbar_view.add_css_class("snapix-shell");
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&toast_overlay));
        toolbar_view.add_bottom_bar(&bottom_bar.widget);
        bind_shell_appearance(
            &app.style_manager(),
            &[
                content.clone().upcast(),
                toast_overlay.clone().upcast(),
                toolbar_view.clone().upcast(),
            ],
        );

        let window = ApplicationWindow::builder()
            .application(app)
            .title(app_window_title())
            .default_width(1280)
            .default_height(820)
            .content(&toolbar_view)
            .build();

        {
            let reframe_overlay_actions = reframe_overlay_actions.clone();
            let state = state.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
                reframe_overlay_actions.set_visible(state.borrow().is_reframing_image());
                glib::ControlFlow::Continue
            });
        }

        {
            let toast_overlay = toast_overlay.clone();
            let state = state.clone();
            let canvas = canvas.clone();
            let title_label = title_label.clone();
            let subtitle_label = subtitle_label.clone();
            let scope_label = scope_label.clone();
            let undo_button = undo_button.clone();
            let redo_button = redo_button.clone();
            let bottom_bar = bottom_bar.clone();
            let delete_button = delete_button.clone();
            let inspector = inspector.clone();
            reframe_reset_button.connect_clicked(move |_| {
                reset_reframe_mode(
                    &toast_overlay,
                    state.clone(),
                    &canvas,
                    &title_label,
                    &subtitle_label,
                    &scope_label,
                    &undo_button,
                    &redo_button,
                    &bottom_bar,
                    &delete_button,
                    &inspector,
                );
            });
        }

        {
            let toast_overlay = toast_overlay.clone();
            let state = state.clone();
            let canvas = canvas.clone();
            let title_label = title_label.clone();
            let subtitle_label = subtitle_label.clone();
            let scope_label = scope_label.clone();
            let undo_button = undo_button.clone();
            let redo_button = redo_button.clone();
            let bottom_bar = bottom_bar.clone();
            let delete_button = delete_button.clone();
            let inspector = inspector.clone();
            reframe_done_button.connect_clicked(move |_| {
                finish_reframe_mode(
                    &toast_overlay,
                    state.clone(),
                    &canvas,
                    &title_label,
                    &subtitle_label,
                    &scope_label,
                    &undo_button,
                    &redo_button,
                    &bottom_bar,
                    &delete_button,
                    &inspector,
                );
            });
        }

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
            save_format.clone(),
        );
        connect_export_format_preferences(preferences.clone(), save_format.clone(), &bottom_bar);
        connect_preferences_button(
            &preferences_button,
            &window,
            preferences.clone(),
            save_format,
            bottom_bar.clone(),
            toast_overlay.clone(),
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

fn bind_shell_appearance(style_manager: &libadwaita::StyleManager, widgets: &[gtk4::Widget]) {
    apply_shell_appearance(widgets, style_manager.is_dark());
    let widgets = widgets.to_vec();
    style_manager.connect_dark_notify(move |manager| {
        apply_shell_appearance(&widgets, manager.is_dark());
    });
}

fn apply_shell_appearance(widgets: &[gtk4::Widget], is_dark: bool) {
    for widget in widgets {
        if is_dark {
            widget.remove_css_class("snapix-light");
        } else {
            widget.add_css_class("snapix-light");
        }
    }
}

fn connect_preferences_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    preferences: Rc<RefCell<AppPreferences>>,
    save_format: Rc<RefCell<SaveFormat>>,
    bottom_bar: BottomBar,
    toast_overlay: ToastOverlay,
) {
    let window = window.clone();
    button.connect_clicked(move |_| {
        present_preferences_window(
            &window,
            preferences.clone(),
            save_format.clone(),
            &bottom_bar,
            &toast_overlay,
        );
    });
}

fn connect_export_format_preferences(
    preferences: Rc<RefCell<AppPreferences>>,
    save_format: Rc<RefCell<SaveFormat>>,
    bottom_bar: &BottomBar,
) {
    {
        let preferences = preferences.clone();
        let save_format = save_format.clone();
        bottom_bar.png_button.connect_toggled(move |button| {
            if !button.is_active() {
                return;
            }
            let mut preferences = preferences.borrow_mut();
            if preferences.remember_last_export_format {
                preferences.last_export_format = Some(PreferredSaveFormat::Png);
                if let Err(error) = save_preferences(&preferences) {
                    tracing::warn!("Failed to save preferences: {error:#}");
                }
            }
            *save_format.borrow_mut() = SaveFormat::Png;
        });
    }
    {
        let preferences = preferences.clone();
        let save_format = save_format.clone();
        bottom_bar.jpeg_button.connect_toggled(move |button| {
            if !button.is_active() {
                return;
            }
            let mut preferences = preferences.borrow_mut();
            if preferences.remember_last_export_format {
                preferences.last_export_format = Some(PreferredSaveFormat::Jpeg);
                if let Err(error) = save_preferences(&preferences) {
                    tracing::warn!("Failed to save preferences: {error:#}");
                }
            }
            *save_format.borrow_mut() = SaveFormat::Jpeg;
        });
    }
}

#[allow(clippy::too_many_arguments)]
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
    let window_for_shortcuts = window.clone();
    controller.connect_key_pressed(move |_controller, key, _keycode, mods| {
        if is_text_input_focused(&window_for_shortcuts)
            && mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
            && matches!(
                key,
                gtk4::gdk::Key::c
                    | gtk4::gdk::Key::C
                    | gtk4::gdk::Key::v
                    | gtk4::gdk::Key::V
                    | gtk4::gdk::Key::s
                    | gtk4::gdk::Key::S
                    | gtk4::gdk::Key::z
                    | gtk4::gdk::Key::Z
                    | gtk4::gdk::Key::y
                    | gtk4::gdk::Key::Y
            )
        {
            return glib::Propagation::Proceed;
        }

        if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
            && mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK)
            && matches!(key, gtk4::gdk::Key::s | gtk4::gdk::Key::S)
            && bottom_bar.save_as_button.is_sensitive()
        {
            bottom_bar.save_as_button.emit_clicked();
            return glib::Propagation::Stop;
        }

        if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
            && matches!(key, gtk4::gdk::Key::s | gtk4::gdk::Key::S)
            && bottom_bar.quick_save_button.is_sensitive()
        {
            bottom_bar.quick_save_button.emit_clicked();
            return glib::Propagation::Stop;
        }

        if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
            && matches!(key, gtk4::gdk::Key::c | gtk4::gdk::Key::C)
            && bottom_bar.copy_button.is_sensitive()
        {
            bottom_bar.copy_button.emit_clicked();
            return glib::Propagation::Stop;
        }

        if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
            && matches!(key, gtk4::gdk::Key::v | gtk4::gdk::Key::V)
        {
            paste_image_from_clipboard(
                &window_for_shortcuts,
                &toast_overlay,
                state.clone(),
                &canvas,
                &title_label,
                &subtitle_label,
                &scope_label,
                &undo_button,
                &redo_button,
                &bottom_bar,
                &delete_button,
                &inspector,
            );
            return glib::Propagation::Stop;
        }

        let mut editor_state = state.borrow_mut();
        match key {
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter
                if editor_state.is_reframing_image() =>
            {
                drop(editor_state);
                finish_reframe_mode(
                    &toast_overlay,
                    state.clone(),
                    &canvas,
                    &title_label,
                    &subtitle_label,
                    &scope_label,
                    &undo_button,
                    &redo_button,
                    &bottom_bar,
                    &delete_button,
                    &inspector,
                );
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Escape if editor_state.is_reframing_image() => {
                drop(editor_state);
                finish_reframe_mode(
                    &toast_overlay,
                    state.clone(),
                    &canvas,
                    &title_label,
                    &subtitle_label,
                    &scope_label,
                    &undo_button,
                    &redo_button,
                    &bottom_bar,
                    &delete_button,
                    &inspector,
                );
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Escape if editor_state.active_tool() == ToolKind::Crop => {
                editor_state.cancel_crop_mode();
                refresh_labels(&editor_state, &title_label, &subtitle_label);
                refresh_scope_label(&editor_state, &scope_label);
                refresh_history_buttons(&editor_state, &undo_button, &redo_button);
                refresh_export_actions(&editor_state, &bottom_bar);
                refresh_tool_actions(&editor_state, &delete_button);
                inspector.refresh_from_state(&editor_state);
                canvas.refresh();
                show_toast(&toast_overlay, "Crop canceled");
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter
                if editor_state.active_tool() == ToolKind::Crop && editor_state.has_pending_crop() =>
            {
                if editor_state.apply_crop_selection() {
                    refresh_labels(&editor_state, &title_label, &subtitle_label);
                    refresh_scope_label(&editor_state, &scope_label);
                    refresh_history_buttons(&editor_state, &undo_button, &redo_button);
                    refresh_export_actions(&editor_state, &bottom_bar);
                    refresh_tool_actions(&editor_state, &delete_button);
                    inspector.refresh_from_state(&editor_state);
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
                if editor_state.active_tool() != ToolKind::Crop
                    && editor_state.selected_annotation().is_some() =>
            {
                if editor_state.delete_selected_annotation() {
                    refresh_labels(&editor_state, &title_label, &subtitle_label);
                    refresh_scope_label(&editor_state, &scope_label);
                    refresh_history_buttons(&editor_state, &undo_button, &redo_button);
                    refresh_export_actions(&editor_state, &bottom_bar);
                    refresh_tool_actions(&editor_state, &delete_button);
                    inspector.refresh_from_state(&editor_state);
                    canvas.refresh();
                    show_toast(&toast_overlay, "Annotation deleted");
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::z
                if editor_state.active_tool() != ToolKind::Crop
                    && mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK) =>
            {
                if editor_state.redo() {
                    refresh_labels(&editor_state, &title_label, &subtitle_label);
                    refresh_scope_label(&editor_state, &scope_label);
                    refresh_history_buttons(&editor_state, &undo_button, &redo_button);
                    refresh_export_actions(&editor_state, &bottom_bar);
                    refresh_tool_actions(&editor_state, &delete_button);
                    inspector.refresh_from_state(&editor_state);
                    canvas.refresh();
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::y | gtk4::gdk::Key::Y
                if editor_state.active_tool() != ToolKind::Crop
                    && mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) =>
            {
                if editor_state.redo() {
                    refresh_labels(&editor_state, &title_label, &subtitle_label);
                    refresh_scope_label(&editor_state, &scope_label);
                    refresh_history_buttons(&editor_state, &undo_button, &redo_button);
                    refresh_export_actions(&editor_state, &bottom_bar);
                    refresh_tool_actions(&editor_state, &delete_button);
                    inspector.refresh_from_state(&editor_state);
                    canvas.refresh();
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::z if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) => {
                if editor_state.undo() {
                    refresh_labels(&editor_state, &title_label, &subtitle_label);
                    refresh_scope_label(&editor_state, &scope_label);
                    refresh_history_buttons(&editor_state, &undo_button, &redo_button);
                    refresh_export_actions(&editor_state, &bottom_bar);
                    refresh_tool_actions(&editor_state, &delete_button);
                    inspector.refresh_from_state(&editor_state);
                    canvas.refresh();
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    window.add_controller(controller);
}

#[allow(clippy::too_many_arguments)]
fn finish_reframe_mode(
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
    canvas: &DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let mut state = state.borrow_mut();
    if !state.is_reframing_image() {
        return;
    }
    state.exit_image_reframe_mode();
    canvas.widget().set_cursor_from_name(None);
    refresh_labels(&state, title_label, subtitle_label);
    refresh_scope_label(&state, scope_label);
    refresh_history_buttons(&state, undo_button, redo_button);
    refresh_export_actions(&state, bottom_bar);
    refresh_tool_actions(&state, delete_button);
    inspector.refresh_from_state(&state);
    canvas.refresh();
    show_toast(toast_overlay, i18n::reframe_done_toast());
}

#[allow(clippy::too_many_arguments)]
fn reset_reframe_mode(
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
    canvas: &DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let mut state = state.borrow_mut();
    if !state.reset_image_reframe() {
        return;
    }
    canvas.widget().set_cursor_from_name(None);
    refresh_labels(&state, title_label, subtitle_label);
    refresh_scope_label(&state, scope_label);
    refresh_history_buttons(&state, undo_button, redo_button);
    refresh_export_actions(&state, bottom_bar);
    refresh_tool_actions(&state, delete_button);
    inspector.refresh_from_state(&state);
    canvas.refresh();
    show_toast(toast_overlay, i18n::image_view_reset_toast());
}

fn is_text_input_focused(window: &ApplicationWindow) -> bool {
    gtk4::prelude::GtkWindowExt::focus(window).is_some_and(|widget| {
        widget.is::<gtk4::Entry>()
            || widget.is::<gtk4::TextView>()
            || widget.is::<gtk4::SpinButton>()
            || widget.is::<gtk4::EditableLabel>()
    })
}

#[allow(clippy::too_many_arguments)]
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
