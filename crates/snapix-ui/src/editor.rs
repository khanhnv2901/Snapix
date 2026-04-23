mod actions;
pub(crate) mod i18n;
mod preferences;
mod presets;
mod state;
mod ui;

pub(crate) use actions::show_toast;
pub(crate) use preferences::{apply_style_preferences, load_preferences};
pub(crate) use state::{same_color_rgb, CropDrag, CropSelection, EditorState, ToolKind};
pub(crate) use ui::{
    refresh_history_buttons, refresh_scope_label, refresh_subtitle, refresh_tool_actions,
    refresh_width_label, EditorWindow,
};

#[cfg(test)]
use actions::{perform_capture_action, CaptureAction};
#[cfg(test)]
use ui::{
    export_actions_enabled, nearest_shadow_direction_index, scope_text, shortcut_hint_text,
    subtitle_text,
};

#[cfg(test)]
mod tests {
    use super::{
        export_actions_enabled, nearest_shadow_direction_index, perform_capture_action, scope_text,
        shortcut_hint_text, subtitle_text, CaptureAction, EditorState, ToolKind,
    };
    use anyhow::{anyhow, Result};
    use snapix_capture::{CaptureBackend, SessionType};
    use snapix_core::canvas::{
        Annotation, Document, Image, ImageAnchor, ImageScaleMode, OutputRatio, Rect,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockBackend {
        name: &'static str,
        full_result: Result<Image, String>,
        region_result: Result<Image, String>,
        window_result: Result<Image, String>,
        region_calls: Arc<Mutex<Vec<Rect>>>,
        window_calls: Arc<Mutex<u32>>,
    }

    impl MockBackend {
        fn with_results(
            name: &'static str,
            full_result: Result<Image, String>,
            region_result: Result<Image, String>,
            window_result: Result<Image, String>,
        ) -> Self {
            Self {
                name,
                full_result,
                region_result,
                window_result,
                region_calls: Arc::new(Mutex::new(Vec::new())),
                window_calls: Arc::new(Mutex::new(0)),
            }
        }

        fn region_calls(&self) -> Vec<Rect> {
            self.region_calls.lock().unwrap().clone()
        }

        fn window_call_count(&self) -> u32 {
            *self.window_calls.lock().unwrap()
        }
    }

    #[async_trait::async_trait]
    impl CaptureBackend for MockBackend {
        async fn capture_full(&self) -> Result<Image> {
            self.full_result.clone().map_err(|err| anyhow!(err))
        }

        async fn capture_region(&self, region: Rect) -> Result<Image> {
            self.region_calls.lock().unwrap().push(region);
            self.region_result.clone().map_err(|err| anyhow!(err))
        }

        async fn capture_window(&self) -> Result<Image> {
            *self.window_calls.lock().unwrap() += 1;
            self.window_result.clone().map_err(|err| anyhow!(err))
        }

        fn supports_interactive(&self) -> bool {
            false
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    fn sample_image() -> Image {
        Image::new(2, 2, vec![255; 16])
    }

    fn large_sample_image() -> Image {
        Image::new(32, 24, vec![255; 32 * 24 * 4])
    }

    #[test]
    fn cancel_crop_mode_returns_to_select_tool() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.set_active_tool(ToolKind::Crop);
        assert_eq!(state.active_tool(), ToolKind::Crop);
        assert!(state.has_pending_crop());

        state.cancel_crop_mode();

        assert_eq!(state.active_tool(), ToolKind::Select);
        assert!(!state.has_pending_crop());
        assert!(state.crop_drag().is_none());
    }

    #[test]
    fn crop_scope_text_requires_image_when_empty() {
        let mut state = EditorState::default();
        state.set_active_tool(ToolKind::Crop);

        assert_eq!(
            scope_text(&state),
            "Crop: capture or import an image first."
        );
    }

    #[test]
    fn crop_scope_text_mentions_enter_when_selection_exists() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        state.set_active_tool(ToolKind::Crop);

        let text = scope_text(&state);

        assert!(text.contains("Enter"));
        assert!(text.contains("Esc"));
    }

    #[test]
    fn select_scope_text_mentions_delete_and_undo_when_idle() {
        let state = EditorState::with_document(Document::new(large_sample_image()));

        let text = scope_text(&state);

        assert!(text.contains("Delete"));
        assert!(text.contains("Ctrl+Z"));
    }

    #[test]
    fn subtitle_text_guides_empty_state() {
        let text = subtitle_text(&Document::default());

        assert_eq!(
            text,
            "No image loaded. Capture or import an image to begin."
        );
    }

    #[test]
    fn subtitle_text_includes_annotation_count() {
        let mut document = Document::new(large_sample_image());
        document.annotations.push(Annotation::Text {
            pos: snapix_core::canvas::Point { x: 8.0, y: 8.0 },
            content: "hello".into(),
            style: snapix_core::canvas::TextStyle {
                font_family: "Sans".into(),
                font_size: 24.0,
                color: snapix_core::canvas::Color {
                    r: 255,
                    g: 98,
                    b: 54,
                    a: 255,
                },
                bold: true,
            },
        });

        let text = subtitle_text(&document);

        assert_eq!(
            text,
            "Image: 32×24 • output 118×110 • 1 annotation • ratio auto • image fit"
        );
    }

    #[test]
    fn subtitle_text_includes_selected_output_ratio() {
        let mut document = Document::new(large_sample_image());
        document.output_ratio = OutputRatio::Landscape16x9;

        let text = subtitle_text(&document);

        assert_eq!(
            text,
            "Image: 32×24 • output 191×110 • no annotations yet • ratio 16:9 • image fit"
        );
    }

    #[test]
    fn subtitle_text_includes_fill_mode() {
        let mut document = Document::new(large_sample_image());
        document.image_scale_mode = ImageScaleMode::Fill;

        let text = subtitle_text(&document);

        assert_eq!(
            text,
            "Image: 32×24 • output 118×110 • no annotations yet • ratio auto • image fill center"
        );
    }

    #[test]
    fn subtitle_text_includes_fill_anchor() {
        let mut document = Document::new(large_sample_image());
        document.image_scale_mode = ImageScaleMode::Fill;
        document.image_anchor = ImageAnchor::TopLeft;

        let text = subtitle_text(&document);

        assert_eq!(
            text,
            "Image: 32×24 • output 118×110 • no annotations yet • ratio auto • image fill top-left"
        );
    }

    #[test]
    fn shortcut_hint_text_guides_empty_state() {
        let state = EditorState::default();

        assert_eq!(
            shortcut_hint_text(&state).as_deref(),
            Some("Fullscreen / Region / Import to begin")
        );
    }

    #[test]
    fn shortcut_hint_text_changes_for_crop_mode() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        state.set_active_tool(ToolKind::Crop);

        assert_eq!(
            shortcut_hint_text(&state).as_deref(),
            Some("Enter apply • Esc cancel")
        );
    }

    #[test]
    fn shortcut_hint_text_for_selected_annotation_mentions_delete() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        assert!(state.commit_rect_annotation(4, 4, 12, 10));

        assert_eq!(
            shortcut_hint_text(&state).as_deref(),
            Some("Delete remove • Ctrl+Z undo")
        );
    }

    #[test]
    fn selected_annotation_can_be_deleted() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.commit_rect_annotation(10, 10, 40, 30);
        assert_eq!(state.selected_annotation(), Some(0));

        assert!(state.delete_selected_annotation());
        assert!(state.document().annotations.is_empty());
        assert_eq!(state.selected_annotation(), None);
    }

    #[test]
    fn clear_action_prefers_deleting_selected_annotation() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        assert!(state.commit_rect_annotation(10, 10, 40, 30));
        assert!(state.document().base_image.is_some());
        assert_eq!(state.selected_annotation(), Some(0));

        let outcome = state.clear_action();

        assert_eq!(
            outcome,
            super::state::ClearOutcome::DeletedSelectedAnnotation
        );
        assert!(state.document().base_image.is_some());
        assert!(state.document().annotations.is_empty());
    }

    #[test]
    fn clear_action_clears_document_when_no_selection() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        assert!(state.commit_rect_annotation(10, 10, 40, 30));
        state.set_selected_annotation(None);

        let outcome = state.clear_action();

        assert_eq!(outcome, super::state::ClearOutcome::ClearedDocument);
        assert!(state.document().base_image.is_none());
        assert!(state.document().annotations.is_empty());
    }

    #[test]
    fn clear_action_is_noop_for_empty_document() {
        let mut state = EditorState::default();

        let outcome = state.clear_action();

        assert_eq!(outcome, super::state::ClearOutcome::None);
        assert!(state.document().base_image.is_none());
        assert!(state.document().annotations.is_empty());
    }

    #[test]
    fn selected_text_annotation_can_be_edited() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        assert!(state.add_text_annotation(24.0, 42.0, "Old".into()));
        assert_eq!(state.selected_text_content().as_deref(), Some("Old"));

        assert!(state.update_selected_text_content("New value".into()));
        assert_eq!(state.selected_text_content().as_deref(), Some("New value"));
    }

    #[test]
    fn active_color_updates_selected_rect_annotation() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.commit_rect_annotation(10, 10, 40, 30);
        state.set_active_color(snapix_core::canvas::Color {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        });

        assert!(state.apply_active_color_to_selected());

        let Annotation::Rect { stroke, .. } = &state.document().annotations[0] else {
            panic!("expected rectangle annotation");
        };
        assert_eq!(stroke.color.r, 10);
        assert_eq!(stroke.color.g, 20);
        assert_eq!(stroke.color.b, 30);
    }

    #[test]
    fn active_width_updates_selected_arrow_annotation() {
        let mut state = EditorState::with_document(Document::new(sample_image()));
        state.begin_arrow_drag(0.0, 0.0, 5.0, 5.0);
        state.update_arrow_drag(50.0, 45.0);
        assert!(state.commit_arrow_drag());
        state.set_active_width(16.0);

        assert!(state.apply_active_width_to_selected());

        let Annotation::Arrow { width, .. } = &state.document().annotations[0] else {
            panic!("expected arrow annotation");
        };
        assert_eq!(*width, 16.0);
    }

    #[test]
    fn syncing_selected_text_updates_active_style() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        state.set_active_color(snapix_core::canvas::Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255,
        });
        state.set_active_width(3.0);
        assert!(state.add_text_annotation(12.0, 8.0, "hello".into()));

        state.sync_active_style_from_selected();

        let active = state.active_color();
        assert_eq!((active.r, active.g, active.b, active.a), (1, 2, 3, 255));
        assert_eq!(state.active_width(), 7.0);
    }

    #[test]
    fn syncing_selected_ellipse_updates_active_style() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        state.set_active_color(snapix_core::canvas::Color {
            r: 20,
            g: 40,
            b: 60,
            a: 255,
        });
        state.set_active_width(9.0);
        assert!(state.commit_ellipse_annotation(4, 5, 12, 10));

        state.set_active_color(snapix_core::canvas::Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255,
        });
        state.set_active_width(1.0);
        state.sync_active_style_from_selected();

        let active = state.active_color();
        assert_eq!((active.r, active.g, active.b, active.a), (20, 40, 60, 255));
        assert_eq!(state.active_width(), 9.0);
    }

    #[test]
    fn tiny_crop_selection_does_not_apply_or_exit_crop_mode() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        state.set_active_tool(ToolKind::Crop);
        state.set_crop_selection(3, 4, 3, 2);

        assert!(!state.apply_crop_selection());
        assert_eq!(state.active_tool(), ToolKind::Crop);
        assert_eq!(
            state
                .document()
                .base_image
                .as_ref()
                .map(|img| (img.width, img.height)),
            Some((32, 24))
        );

        let selection = state.crop_selection().expect("selection should remain");
        assert_eq!(
            (
                selection.x(),
                selection.y(),
                selection.width(),
                selection.height()
            ),
            (3, 4, 3, 2)
        );
        assert!(state.document().annotations.is_empty());
    }

    #[test]
    fn preview_move_annotation_clamps_rect_within_image_bounds() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        assert!(state.commit_rect_annotation(20, 16, 12, 8));
        let original = state.document().annotations[0].clone();

        state.preview_move_annotation(0, &original, 50.0, 50.0);

        let Annotation::Rect { bounds, .. } = &state.document().annotations[0] else {
            panic!("expected rectangle annotation");
        };
        assert_eq!(
            (bounds.x, bounds.y, bounds.width, bounds.height),
            (20.0, 16.0, 12.0, 8.0)
        );
    }

    #[test]
    fn preview_resize_annotation_updates_bounds_and_preserves_style() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        state.set_active_color(snapix_core::canvas::Color {
            r: 80,
            g: 90,
            b: 100,
            a: 255,
        });
        state.set_active_width(7.0);
        assert!(state.commit_ellipse_annotation(4, 5, 12, 10));
        let original = state.document().annotations[0].clone();

        state.preview_resize_annotation(0, &original, 8, 9, 14, 11);

        let Annotation::Ellipse {
            bounds,
            stroke,
            fill,
        } = &state.document().annotations[0]
        else {
            panic!("expected ellipse annotation");
        };
        assert_eq!(
            (bounds.x, bounds.y, bounds.width, bounds.height),
            (8.0, 9.0, 14.0, 11.0)
        );
        assert_eq!(
            (stroke.color.r, stroke.color.g, stroke.color.b, stroke.width),
            (80, 90, 100, 7.0)
        );
        assert!(fill.is_none());
    }

    #[test]
    fn finalize_annotation_move_only_records_real_changes() {
        let mut state = EditorState::with_document(Document::new(large_sample_image()));
        assert!(state.commit_rect_annotation(4, 4, 10, 8));
        let before_noop = state.document().clone();

        assert!(!state.finalize_annotation_move(before_noop.clone()));
        assert!(!state.can_redo());

        let original = state.document().annotations[0].clone();
        state.preview_move_annotation(0, &original, -2.0, 3.0);

        assert!(state.finalize_annotation_move(before_noop));
        assert!(state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn export_actions_disabled_without_image() {
        assert!(!export_actions_enabled(&Document::default()));
    }

    #[test]
    fn export_actions_enabled_with_image() {
        assert!(export_actions_enabled(&Document::new(sample_image())));
    }

    #[test]
    fn fullscreen_wayland_portal_falls_back_to_region() {
        let backend = MockBackend::with_results(
            "ashpd-portal",
            Err("portal fullscreen failed".into()),
            Ok(sample_image()),
            Err("unused".into()),
        );

        let (image, message) = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::Wayland, CaptureAction::Fullscreen)
                .await
                .expect("expected fallback to succeed")
        });

        assert_eq!(image.width, 2);
        assert!(message.is_some());
        assert_eq!(
            message.unwrap(),
            "Fullscreen capture failed; switched to interactive capture."
        );

        let calls = backend.region_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].x, 0.0);
        assert_eq!(calls[0].y, 0.0);
        assert_eq!(calls[0].width, 0.0);
        assert_eq!(calls[0].height, 0.0);
    }

    #[test]
    fn fullscreen_wayland_portal_reports_both_failures() {
        let backend = MockBackend::with_results(
            "ashpd-portal",
            Err("portal fullscreen failed".into()),
            Err("portal region failed".into()),
            Err("unused".into()),
        );

        let error = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::Wayland, CaptureAction::Fullscreen)
                .await
                .expect_err("expected fallback chain to fail")
        });

        let text = error.to_string();
        assert!(text.contains("Fullscreen capture failed"));
        assert!(text.contains("Interactive fallback also failed"));
    }

    #[test]
    fn fullscreen_x11_does_not_use_region_fallback() {
        let backend = MockBackend::with_results(
            "x11rb",
            Err("x11 fullscreen failed".into()),
            Ok(sample_image()),
            Err("unused".into()),
        );

        let error = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::X11, CaptureAction::Fullscreen)
                .await
                .expect_err("expected fullscreen error to be returned directly")
        });

        assert!(error.to_string().contains("x11 fullscreen failed"));
        assert!(backend.region_calls().is_empty());
    }

    #[test]
    fn region_action_uses_zero_rect_interactive_request() {
        let backend = MockBackend::with_results(
            "ashpd-portal",
            Err("unused".into()),
            Ok(sample_image()),
            Err("unused".into()),
        );

        let (_image, message) = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::Wayland, CaptureAction::Region)
                .await
                .expect("expected region capture to succeed")
        });

        assert!(message.is_none());
        let calls = backend.region_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].width, 0.0);
        assert_eq!(calls[0].height, 0.0);
    }

    #[test]
    fn window_action_uses_window_backend_path() {
        let backend = MockBackend::with_results(
            "x11rb",
            Err("unused".into()),
            Err("unused".into()),
            Ok(sample_image()),
        );

        let (image, message) = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::X11, CaptureAction::Window)
                .await
                .expect("expected window capture to succeed")
        });

        assert_eq!(image.width, 2);
        assert!(message.is_none());
        assert_eq!(backend.window_call_count(), 1);
        assert!(backend.region_calls().is_empty());
    }

    #[test]
    fn window_action_propagates_window_backend_error() {
        let backend = MockBackend::with_results(
            "x11rb",
            Err("unused".into()),
            Err("unused".into()),
            Err("window capture failed".into()),
        );

        let error = async_std::task::block_on(async {
            perform_capture_action(&backend, SessionType::X11, CaptureAction::Window)
                .await
                .expect_err("expected window capture to fail")
        });

        assert!(error.to_string().contains("window capture failed"));
        assert_eq!(backend.window_call_count(), 1);
        assert!(backend.region_calls().is_empty());
    }

    #[test]
    fn nearest_shadow_direction_prefers_bottom_for_default_shadow() {
        assert_eq!(nearest_shadow_direction_index(18.0, 18.0), 8);
    }

    #[test]
    fn nearest_shadow_direction_prefers_center_for_zero_offset() {
        assert_eq!(nearest_shadow_direction_index(0.0, 0.0), 4);
    }
}
