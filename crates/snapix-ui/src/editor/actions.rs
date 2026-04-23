use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use gdk4::{MemoryFormat, MemoryTexture, Texture, TextureDownloader};
use gio::prelude::FileExt;
use gtk4::prelude::*;
use libadwaita::{ApplicationWindow, Toast, ToastOverlay};
use snapix_capture::{CaptureBackend, SessionType};
use snapix_core::canvas::{Document, Image, Rect};

use super::i18n;
use super::preferences::AppPreferences;
use super::state::EditorState;
use super::ui::{
    refresh_export_actions, refresh_history_buttons, refresh_labels, refresh_scope_label,
    refresh_tool_actions, BottomBar, CaptureActionRow, InspectorControls, SaveFormat,
};
use crate::widgets::{render_document_rgba, DocumentCanvas};

// ─── Capture actions ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(super) fn connect_capture_actions(
    actions: &CaptureActionRow,
    window: &ApplicationWindow,
    preferences: Rc<RefCell<AppPreferences>>,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    toast_overlay: &ToastOverlay,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let session = snapix_capture::detect_session();
    let backend = snapix_capture::detect_backend();
    if session == snapix_capture::SessionType::Wayland && backend.name() == "ashpd-portal" {
        actions
            .fullscreen_button
            .set_tooltip_text(Some(i18n::capture_wayland_fullscreen_tooltip()));
        actions
            .region_button
            .set_tooltip_text(Some(i18n::capture_wayland_region_tooltip()));
        actions
            .window_button
            .set_tooltip_text(Some(i18n::capture_wayland_window_tooltip()));
    }

    for (button, action) in [
        (&actions.fullscreen_button, CaptureAction::Fullscreen),
        (&actions.region_button, CaptureAction::Region),
        (&actions.window_button, CaptureAction::Window),
    ] {
        connect_capture_button(
            button,
            window,
            preferences.clone(),
            state.clone(),
            canvas.clone(),
            title_label,
            subtitle_label,
            scope_label,
            toast_overlay,
            undo_button,
            redo_button,
            bottom_bar,
            delete_button,
            inspector,
            action,
        );
    }
    connect_import_button(
        &actions.import_button,
        window,
        preferences.clone(),
        state.clone(),
        canvas.clone(),
        title_label,
        subtitle_label,
        scope_label,
        toast_overlay,
        undo_button,
        redo_button,
        bottom_bar,
        delete_button,
        inspector,
    );
    connect_clear_button(
        &actions.clear_button,
        toast_overlay,
        state,
        canvas,
        title_label,
        subtitle_label,
        scope_label,
        undo_button,
        redo_button,
        bottom_bar,
        delete_button,
        inspector,
    );
}

#[derive(Clone, Copy)]
pub(crate) enum CaptureAction {
    Fullscreen,
    Region,
    Window,
}

#[allow(clippy::too_many_arguments)]
fn connect_capture_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    preferences: Rc<RefCell<AppPreferences>>,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    toast_overlay: &ToastOverlay,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
    action: CaptureAction,
) {
    let window = window.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let toast_overlay = toast_overlay.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
    button.connect_clicked(move |_| {
        let session = snapix_capture::detect_session();
        let hide_before_capture = matches!(
            action,
            CaptureAction::Fullscreen | CaptureAction::Region | CaptureAction::Window
        );
        let delay_ms = capture_window_hide_delay_ms(action, session);

        // Hide the Snapix window before any capture flow so it doesn't end up
        // in the captured image. Wayland portals and some compositors need a
        // short delay before the window is actually gone from the screen.
        if hide_before_capture {
            window.set_visible(false);
        }

        let window = window.clone();
        let state = state.clone();
        let preferences = preferences.clone();
        let canvas = canvas.clone();
        let title_label = title_label.clone();
        let subtitle_label = subtitle_label.clone();
        let scope_label = scope_label.clone();
        let toast_overlay = toast_overlay.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let bottom_bar = bottom_bar.clone();
        let delete_button = delete_button.clone();
        let inspector = inspector.clone();

        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(delay_ms), move || {
            let result = async_std::task::block_on(async {
                let backend = snapix_capture::detect_backend();
                perform_capture_action(backend.as_ref(), session, action).await
            });

            if hide_before_capture {
                window.set_visible(true);
                window.present();
            }

            match result {
                Ok((image, message)) => {
                    let mut state = state.borrow_mut();
                    if state.replace_base_image(image) {
                        maybe_enter_reframe_after_load(&mut state, &preferences.borrow());
                        refresh_labels(&state, &title_label, &subtitle_label);
                        refresh_scope_label(&state, &scope_label);
                        refresh_history_buttons(&state, &undo_button, &redo_button);
                        refresh_export_actions(&state, &bottom_bar);
                        refresh_tool_actions(&state, &delete_button);
                        inspector.refresh_from_state(&state);
                        canvas.refresh();
                    }
                    drop(state);
                    if let Some(message) = message {
                        show_toast(&toast_overlay, &message);
                    } else {
                        let message = match action {
                            CaptureAction::Fullscreen => i18n::capture_success_toast("fullscreen"),
                            CaptureAction::Region => i18n::capture_success_toast("region"),
                            CaptureAction::Window => i18n::capture_success_toast("window"),
                        };
                        show_toast(&toast_overlay, message);
                    }
                }
                Err(error) => {
                    let detail = match action {
                        CaptureAction::Fullscreen => {
                            i18n::capture_failed_detail("fullscreen", &error.to_string())
                        }
                        CaptureAction::Region => {
                            i18n::capture_failed_detail("region", &error.to_string())
                        }
                        CaptureAction::Window => {
                            i18n::capture_failed_detail("window", &error.to_string())
                        }
                    };
                    show_error(&window, i18n::capture_failed_title(), &detail);
                }
            }
        });
    });
}

fn capture_window_hide_delay_ms(action: CaptureAction, session: SessionType) -> u64 {
    match (session, action) {
        // Wayland/compositor-backed capture tends to need a more noticeable
        // delay so the app window is not included in the screenshot.
        (SessionType::Wayland, CaptureAction::Fullscreen) => 350,
        (SessionType::Wayland, CaptureAction::Region | CaptureAction::Window) => 250,
        // X11 generally reacts faster, but still benefits from a short delay
        // before querying the root/active window.
        (SessionType::X11, _) => 120,
        (SessionType::Unknown, _) => 150,
    }
}

pub(crate) async fn perform_capture_action(
    backend: &dyn CaptureBackend,
    session: SessionType,
    action: CaptureAction,
) -> Result<(Image, Option<String>)> {
    match action {
        CaptureAction::Fullscreen => match backend.capture_full().await {
            Ok(image) => Ok((image, None)),
            // The window is hidden before capture arrives here, so a portal
            // failure is a genuine compositor/permission issue rather than
            // the Snapix window being in the way. Fall back to interactive
            // capture as a last resort so the user still gets something.
            Err(full_error)
                if session == SessionType::Wayland && backend.name() == "ashpd-portal" =>
            {
                tracing::warn!(
                    "Fullscreen portal capture failed, falling back to interactive: {full_error:#}"
                );
                match backend
                    .capture_region(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    })
                    .await
                {
                    Ok(image) => Ok((image, Some(i18n::capture_fallback_toast().to_string()))),
                    Err(region_error) => Err(anyhow::anyhow!(
                        "Fullscreen capture failed: {full_error}. \
                         Interactive fallback also failed: {region_error}"
                    )),
                }
            }
            Err(error) => Err(error),
        },
        CaptureAction::Region => backend
            .capture_region(Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            })
            .await
            .map(|image| (image, None)),
        CaptureAction::Window => backend.capture_window().await.map(|image| (image, None)),
    }
}

#[allow(clippy::too_many_arguments)]
fn connect_import_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    preferences: Rc<RefCell<AppPreferences>>,
    state: Rc<RefCell<EditorState>>,
    canvas: DocumentCanvas,
    title_label: &gtk4::Label,
    subtitle_label: &gtk4::Label,
    scope_label: &gtk4::Label,
    toast_overlay: &ToastOverlay,
    undo_button: &gtk4::Button,
    redo_button: &gtk4::Button,
    bottom_bar: &BottomBar,
    delete_button: &gtk4::Button,
    inspector: &InspectorControls,
) {
    let window = window.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let toast_overlay = toast_overlay.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
    button.connect_clicked(move |_| {
        let chooser = gtk4::FileChooserNative::builder()
            .title(i18n::import_dialog_title())
            .transient_for(&window)
            .action(gtk4::FileChooserAction::Open)
            .accept_label(i18n::import_accept_button())
            .cancel_label(i18n::cancel_button_label())
            .modal(true)
            .build();
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(i18n::images_filter_name()));
        for mime in ["image/png", "image/jpeg", "image/webp"] {
            filter.add_mime_type(mime);
        }
        for pat in ["*.png", "*.jpg", "*.jpeg", "*.webp"] {
            filter.add_pattern(pat);
        }
        chooser.add_filter(&filter);

        let window = window.clone();
        let preferences = preferences.clone();
        let state = state.clone();
        let canvas = canvas.clone();
        let title_label = title_label.clone();
        let subtitle_label = subtitle_label.clone();
        let scope_label = scope_label.clone();
        let undo_button = undo_button.clone();
        let redo_button = redo_button.clone();
        let response_toast_overlay = toast_overlay.clone();
        let response_bottom_bar = bottom_bar.clone();
        let response_delete_button = delete_button.clone();
        let response_inspector = inspector.clone();
        chooser.connect_response(move |chooser, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(file) = chooser.file() {
                    match file.path() {
                        Some(path) => match image::open(&path) {
                            Ok(dynamic) => {
                                let mut state = state.borrow_mut();
                                if state.replace_base_image(Image::from_dynamic(dynamic)) {
                                    maybe_enter_reframe_after_load(&mut state, &preferences.borrow());
                                    refresh_labels(&state, &title_label, &subtitle_label);
                                    refresh_scope_label(&state, &scope_label);
                                    refresh_history_buttons(&state, &undo_button, &redo_button);
                                    refresh_export_actions(&state, &response_bottom_bar);
                                    refresh_tool_actions(&state, &response_delete_button);
                                    response_inspector.refresh_from_state(&state);
                                    canvas.refresh();
                                }
                                show_toast(
                                    &response_toast_overlay,
                                    &i18n::imported_image_toast(&path.display().to_string()),
                                );
                            }
                            Err(error) => show_error(
                                &window,
                                i18n::import_failed_title(),
                                &i18n::import_failed_open_detail(
                                    &path.display().to_string(),
                                    &error.to_string(),
                                ),
                            ),
                        },
                        None => show_error(
                            &window,
                            i18n::import_failed_title(),
                            i18n::import_failed_non_local_detail(),
                        ),
                    }
                }
            }
            chooser.destroy();
        });
        chooser.show();
    });
}

#[allow(clippy::too_many_arguments)]
fn connect_clear_button(
    button: &gtk4::Button,
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
    let toast_overlay = toast_overlay.clone();
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
        let outcome = state.clear_action();
        if !matches!(outcome, super::state::ClearOutcome::None) {
            refresh_labels(&state, &title_label, &subtitle_label);
            refresh_scope_label(&state, &scope_label);
            refresh_history_buttons(&state, &undo_button, &redo_button);
            refresh_export_actions(&state, &bottom_bar);
            refresh_tool_actions(&state, &delete_button);
            inspector.refresh_from_state(&state);
            canvas.refresh();
            match outcome {
                super::state::ClearOutcome::DeletedSelectedAnnotation => {
                    show_toast(&toast_overlay, "Annotation deleted");
                }
                super::state::ClearOutcome::ClearedDocument => {
                    show_toast(&toast_overlay, i18n::image_cleared_toast());
                }
                super::state::ClearOutcome::None => {}
            }
        }
    });
}

// ─── Export / copy actions ────────────────────────────────────────────────────

pub(super) fn connect_copy_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    toast_overlay: &ToastOverlay,
    state: Rc<RefCell<EditorState>>,
) {
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();
    button.connect_clicked(move |_| {
        let document = state.borrow().document().clone();
        match render_document_rgba(&document) {
            Ok(rendered) => {
                let stride = rendered.width as usize * 4;
                let bytes = glib::Bytes::from_owned(rendered.rgba);
                let texture = MemoryTexture::new(
                    rendered.width as i32,
                    rendered.height as i32,
                    MemoryFormat::R8g8b8a8,
                    &bytes,
                    stride,
                );
                let clipboard = gtk4::prelude::WidgetExt::display(&window).clipboard();
                clipboard.set_texture(&texture);
                show_toast(&toast_overlay, i18n::image_copied_to_clipboard_toast());
            }
            Err(error) => show_error(&window, i18n::copy_failed_title(), &error.to_string()),
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paste_image_from_clipboard(
    window: &ApplicationWindow,
    toast_overlay: &ToastOverlay,
    preferences: Rc<RefCell<AppPreferences>>,
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
    let clipboard = gtk4::prelude::WidgetExt::display(window).clipboard();
    let toast_overlay = toast_overlay.clone();
    let canvas = canvas.clone();
    let title_label = title_label.clone();
    let subtitle_label = subtitle_label.clone();
    let scope_label = scope_label.clone();
    let undo_button = undo_button.clone();
    let redo_button = redo_button.clone();
    let bottom_bar = bottom_bar.clone();
    let delete_button = delete_button.clone();
    let inspector = inspector.clone();
    let window = window.clone();

    clipboard.read_texture_async(
        None::<&gio::Cancellable>,
        move |result: Result<Option<Texture>, glib::Error>| {
            let Some(texture) = (match result {
            Ok(texture) => texture,
            Err(error) => {
                show_error(
                    &window,
                    i18n::paste_failed_title(),
                    &i18n::clipboard_read_failed_detail(&error.to_string()),
                );
                return;
            }
            }) else {
                show_error(
                    &window,
                    i18n::paste_failed_title(),
                    i18n::clipboard_image_missing_detail(),
                );
                return;
            };

            let mut downloader = TextureDownloader::new(&texture);
            downloader.set_format(MemoryFormat::R8g8b8a8);
            let (bytes, stride) = downloader.download_bytes();
            let raw: &[u8] = bytes.as_ref();
            let width = texture.width() as u32;
            let height = texture.height() as u32;
            if width == 0 || height == 0 {
                show_error(
                    &window,
                    i18n::paste_failed_title(),
                    i18n::clipboard_image_invalid_detail(),
                );
                return;
            }
            let expected_stride = width as usize * 4;
            if stride < expected_stride {
                show_error(
                    &window,
                    i18n::paste_failed_title(),
                    i18n::clipboard_image_invalid_detail(),
                );
                return;
            }
            let rgba = if stride == expected_stride {
                raw.to_vec()
            } else {
                let mut packed = Vec::with_capacity(width as usize * height as usize * 4);
                for row in raw.chunks(stride).take(height as usize) {
                    packed.extend_from_slice(&row[..expected_stride]);
                }
                packed
            };
            let image = Image::new(width, height, rgba);
            if image.width == 0 || image.height == 0 {
                    show_error(
                        &window,
                        i18n::paste_failed_title(),
                        i18n::clipboard_image_invalid_detail(),
                    );
                    return;
            }

            let mut state = state.borrow_mut();
            if state.replace_base_image(image) {
                maybe_enter_reframe_after_load(&mut state, &preferences.borrow());
                refresh_labels(&state, &title_label, &subtitle_label);
                refresh_scope_label(&state, &scope_label);
                refresh_history_buttons(&state, &undo_button, &redo_button);
                refresh_export_actions(&state, &bottom_bar);
                refresh_tool_actions(&state, &delete_button);
                inspector.refresh_from_state(&state);
                canvas.refresh();
            }
            drop(state);
            show_toast(&toast_overlay, i18n::image_pasted_from_clipboard_toast());
        },
    );
}

pub(super) fn connect_quick_save_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    toast_overlay: &ToastOverlay,
    preferences: Rc<RefCell<AppPreferences>>,
    state: Rc<RefCell<EditorState>>,
    save_format: Rc<RefCell<SaveFormat>>,
) {
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();
    button.connect_clicked(move |_| {
        let document = state.borrow().document().clone();
        let format = *save_format.borrow();
        let preferences_snapshot = preferences.borrow().clone();
        let pictures_dir = gtk4::glib::user_special_dir(gtk4::glib::UserDirectory::Pictures)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let screenshots_dir = pictures_dir.join("Screenshots");
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ext = if format == SaveFormat::Jpeg {
            "jpg"
        } else {
            "png"
        };
        let path = screenshots_dir.join(format!("snapix-{ts}.{ext}"));
        let save_result = std::fs::create_dir_all(&screenshots_dir)
            .map_err(anyhow::Error::from)
            .and_then(|_| save_image_to_path(&document, &path, format, &preferences_snapshot));
        if let Err(error) = save_result {
            show_error(&window, i18n::quick_save_failed_title(), &error.to_string());
        } else {
            if preferences_snapshot.auto_copy_after_export {
                if let Err(error) = copy_document_to_clipboard(&window, &document) {
                    tracing::warn!("Failed to auto-copy exported image: {error:#}");
                }
            }
            show_toast(
                &toast_overlay,
                &i18n::saved_image_toast(&path.display().to_string()),
            );
        }
    });
}

pub(super) fn connect_save_as_button(
    button: &gtk4::Button,
    window: &ApplicationWindow,
    toast_overlay: &ToastOverlay,
    preferences: Rc<RefCell<AppPreferences>>,
    state: Rc<RefCell<EditorState>>,
    save_format: Rc<RefCell<SaveFormat>>,
) {
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();
    button.connect_clicked(move |_| {
        let format = *save_format.borrow();
        let (title_str, accept_str, default_name, mime, pattern) = match format {
            SaveFormat::Png => (
                i18n::export_dialog_title("png"),
                i18n::save_button_label(),
                "snapix-export.png",
                "image/png",
                "*.png",
            ),
            SaveFormat::Jpeg => (
                i18n::export_dialog_title("jpeg"),
                i18n::save_button_label(),
                "snapix-export.jpg",
                "image/jpeg",
                "*.jpg",
            ),
        };
        let chooser = gtk4::FileChooserNative::builder()
            .title(title_str)
            .transient_for(&window)
            .action(gtk4::FileChooserAction::Save)
            .accept_label(accept_str)
            .cancel_label(i18n::cancel_button_label())
            .modal(true)
            .build();
        chooser.set_current_name(default_name);
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(title_str));
        filter.add_mime_type(mime);
        filter.add_pattern(pattern);
        chooser.add_filter(&filter);

        let state = state.clone();
        let window = window.clone();
        let toast_overlay = toast_overlay.clone();
        let save_format = save_format.clone();
        let preferences = preferences.clone();
        chooser.connect_response(move |chooser, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(file) = chooser.file() {
                    match file.path() {
                        Some(path) => {
                            let document = state.borrow().document().clone();
                            let fmt = *save_format.borrow();
                            let preferences_snapshot = preferences.borrow().clone();
                            if let Err(error) =
                                save_image_to_path(&document, &path, fmt, &preferences_snapshot)
                            {
                                show_error(
                                    &window,
                                    i18n::export_failed_title(),
                                    &error.to_string(),
                                );
                            } else {
                                if preferences_snapshot.auto_copy_after_export {
                                    if let Err(error) =
                                        copy_document_to_clipboard(&window, &document)
                                    {
                                        tracing::warn!(
                                            "Failed to auto-copy exported image: {error:#}"
                                        );
                                    }
                                }
                                show_toast(
                                    &toast_overlay,
                                    &i18n::exported_image_toast(&path.display().to_string()),
                                );
                            }
                        }
                        None => show_error(
                            &window,
                            i18n::export_failed_title(),
                            i18n::export_failed_non_local_detail(),
                        ),
                    }
                }
            }
            chooser.destroy();
        });
        chooser.show();
    });
}

fn save_image_to_path(
    document: &Document,
    path: &std::path::Path,
    format: SaveFormat,
    preferences: &AppPreferences,
) -> anyhow::Result<()> {
    let rendered = render_document_rgba(document)?;
    match format {
        SaveFormat::Png => {
            image::save_buffer(
                path,
                &rendered.rgba,
                rendered.width,
                rendered.height,
                image::ColorType::Rgba8,
            )
            .map_err(|e| anyhow::anyhow!("Failed to save PNG: {e}"))?;
        }
        SaveFormat::Jpeg => {
            let rgb: Vec<u8> = rendered
                .rgba
                .chunks_exact(4)
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect();
            let file = std::fs::File::create(path)
                .map_err(|e| anyhow::anyhow!("Failed to open JPEG file for writing: {e}"))?;
            let writer = std::io::BufWriter::new(file);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                writer,
                preferences.effective_jpeg_quality(),
            );
            encoder
                .encode(
                    &rgb,
                    rendered.width,
                    rendered.height,
                    image::ColorType::Rgb8.into(),
                )
                .map_err(|e| anyhow::anyhow!("Failed to save JPEG: {e}"))?;
        }
    }
    Ok(())
}

fn maybe_enter_reframe_after_load(state: &mut EditorState, preferences: &AppPreferences) {
    if preferences.auto_reframe_after_load {
        state.enter_image_reframe_mode();
    }
}

fn copy_document_to_clipboard(
    window: &ApplicationWindow,
    document: &Document,
) -> anyhow::Result<()> {
    let rendered = render_document_rgba(document)?;
    let stride = rendered.width as usize * 4;
    let bytes = glib::Bytes::from_owned(rendered.rgba);
    let texture = MemoryTexture::new(
        rendered.width as i32,
        rendered.height as i32,
        MemoryFormat::R8g8b8a8,
        &bytes,
        stride,
    );
    let clipboard = gtk4::prelude::WidgetExt::display(window).clipboard();
    clipboard.set_texture(&texture);
    Ok(())
}

fn show_dialog(
    window: &ApplicationWindow,
    message_type: gtk4::MessageType,
    title: &str,
    detail: &str,
) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .message_type(message_type)
        .buttons(gtk4::ButtonsType::Ok)
        .text(title)
        .secondary_text(detail)
        .build();
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}

pub(super) fn show_error(window: &ApplicationWindow, title: &str, detail: &str) {
    show_dialog(window, gtk4::MessageType::Error, title, detail);
}

const TOAST_DEDUPE_WINDOW_MS: u128 = 1_200;

thread_local! {
    static LAST_TOAST: RefCell<Option<(String, u128)>> = const { RefCell::new(None) };
}

pub(crate) fn show_toast(toast_overlay: &ToastOverlay, message: &str) {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let should_show = LAST_TOAST.with(|last| {
        let mut last = last.borrow_mut();
        if should_suppress_toast(&last, message, now_ms) {
            return false;
        }
        *last = Some((message.to_string(), now_ms));
        true
    });

    if !should_show {
        return;
    }

    toast_overlay.add_toast(Toast::new(message));
}

fn should_suppress_toast(last_toast: &Option<(String, u128)>, message: &str, now_ms: u128) -> bool {
    last_toast.as_ref().is_some_and(|(last_message, last_ms)| {
        last_message == message && now_ms.saturating_sub(*last_ms) < TOAST_DEDUPE_WINDOW_MS
    })
}

#[cfg(test)]
mod tests {
    use super::should_suppress_toast;

    #[test]
    fn duplicate_toast_within_window_is_suppressed() {
        let last = Some(("Crop selection was too small".to_string(), 1_000));

        assert!(should_suppress_toast(
            &last,
            "Crop selection was too small",
            2_000
        ));
    }

    #[test]
    fn duplicate_toast_after_window_is_allowed() {
        let last = Some(("Crop selection was too small".to_string(), 1_000));

        assert!(!should_suppress_toast(
            &last,
            "Crop selection was too small",
            2_250
        ));
    }

    #[test]
    fn different_toast_message_is_not_suppressed() {
        let last = Some(("Crop selection was too small".to_string(), 1_000));

        assert!(!should_suppress_toast(
            &last,
            "Ellipse drag was too small",
            1_100
        ));
    }
}
