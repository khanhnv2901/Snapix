use snapix_core::canvas::{ImageAnchor, OutputRatio};

use crate::editor::state::ToolKind;

pub(crate) fn app_window_title() -> &'static str {
    "Snapix"
}

pub(crate) fn editor_header_title(tool: ToolKind) -> String {
    format!("Editor • {}", tool_label(tool))
}

pub(crate) fn tool_label(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Select => "Select",
        ToolKind::Crop => "Crop",
        ToolKind::Arrow => "Arrow",
        ToolKind::Line => "Line",
        ToolKind::Rectangle => "Rect",
        ToolKind::Ellipse => "Ellipse",
        ToolKind::Text => "Text",
        ToolKind::Blur => "Blur",
    }
}

pub(crate) fn tool_tooltip(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Select => "Select and edit annotations",
        ToolKind::Crop => "Crop the image",
        ToolKind::Arrow => "Draw an arrow annotation",
        ToolKind::Line => "Draw a straight line annotation",
        ToolKind::Rectangle => "Draw a rectangle annotation",
        ToolKind::Ellipse => "Draw an ellipse annotation",
        ToolKind::Text => "Add a text label",
        ToolKind::Blur => "Blur part of the image",
    }
}

pub(crate) fn width_label_text(is_text: bool) -> &'static str {
    if is_text {
        "Size:"
    } else {
        "Width:"
    }
}

pub(crate) fn preferences_button_tooltip() -> &'static str {
    "Preferences"
}

pub(crate) fn undo_tooltip() -> &'static str {
    "Undo (Ctrl+Z)"
}

pub(crate) fn redo_tooltip() -> &'static str {
    "Redo (Ctrl+Shift+Z)"
}

pub(crate) fn delete_tooltip() -> &'static str {
    "Delete selected annotation (Backspace/Delete)"
}

pub(crate) fn capture_section_label() -> &'static str {
    "Capture"
}

pub(crate) fn export_section_label() -> &'static str {
    "Export"
}

pub(crate) fn capture_action_label(id: &str) -> &'static str {
    match id {
        "fullscreen" => "Fullscreen",
        "region" => "Region",
        "window" => "Window",
        "import" => "Import",
        "clear" => "Clear",
        _ => "",
    }
}

pub(crate) fn capture_action_tooltip(id: &str) -> &'static str {
    match id {
        "fullscreen" => "Capture the entire screen",
        "region" => "Choose part of the screen to capture",
        "window" => "Capture the active window",
        "import" => "Open an image from disk",
        "clear" => "Clear the current image and annotations",
        _ => "",
    }
}

pub(crate) fn color_name(index: usize) -> &'static str {
    match index {
        0 => "Orange",
        1 => "Red",
        2 => "Pink",
        3 => "Purple",
        4 => "Blue",
        5 => "Teal",
        6 => "Green",
        7 => "Yellow",
        8 => "White",
        9 => "Dark",
        _ => "",
    }
}

pub(crate) fn preferences_dialog_title() -> &'static str {
    "Preferences"
}

pub(crate) fn preferences_export_title() -> &'static str {
    "Export"
}

pub(crate) fn preferences_export_description() -> &'static str {
    "Choose how Snapix should default export behavior in the editor."
}

pub(crate) fn preferences_appearance_title() -> &'static str {
    "Appearance"
}

pub(crate) fn preferences_appearance_description() -> &'static str {
    "Control how Snapix follows or overrides the system color scheme."
}

pub(crate) fn preferences_color_scheme_title() -> &'static str {
    "Color Scheme"
}

pub(crate) fn preferences_color_scheme_subtitle() -> &'static str {
    "Choose whether Snapix follows the system appearance or forces light/dark."
}

pub(crate) fn preferences_save_format_title() -> &'static str {
    "Default Save Format"
}

pub(crate) fn preferences_save_format_subtitle() -> &'static str {
    "Used when Snapix is not remembering the last export format."
}

pub(crate) fn preferences_remember_format_title() -> &'static str {
    "Remember Last Export Format"
}

pub(crate) fn preferences_remember_format_subtitle() -> &'static str {
    "Keep using the most recently selected PNG/JPEG format between launches."
}

pub(crate) fn preferences_about_title() -> &'static str {
    "About"
}

pub(crate) fn preferences_about_description() -> &'static str {
    "Snapix stores local settings here and uses your Pictures folder for Quick Save by default."
}

pub(crate) fn preferences_app_name_title() -> &'static str {
    "Application"
}

pub(crate) fn preferences_app_version_title() -> &'static str {
    "Version"
}

pub(crate) fn preferences_app_author_title() -> &'static str {
    "Author"
}

pub(crate) fn preferences_app_license_title() -> &'static str {
    "License"
}

pub(crate) fn preferences_app_repository_title() -> &'static str {
    "Repository"
}

pub(crate) fn preferences_open_link_label() -> &'static str {
    "Open"
}

pub(crate) fn preferences_storage_title() -> &'static str {
    "Storage"
}

pub(crate) fn preferences_storage_subtitle() -> &'static str {
    "Preferences are stored locally in your user config directory."
}

pub(crate) fn preferences_quick_save_location_title() -> &'static str {
    "Quick Save Location"
}

pub(crate) fn preferences_quick_save_location_subtitle(path: &str) -> String {
    format!("Quick Save writes to {path}")
}

pub(crate) fn preferences_jpeg_quality_title() -> &'static str {
    "JPEG Quality"
}

pub(crate) fn preferences_jpeg_quality_subtitle() -> &'static str {
    "Higher values keep more detail but create larger JPEG files."
}

pub(crate) fn preferences_auto_copy_after_export_title() -> &'static str {
    "Copy After Save"
}

pub(crate) fn preferences_auto_copy_after_export_subtitle() -> &'static str {
    "After Quick Save or Save As, also copy the exported image to the clipboard."
}

pub(crate) fn preferences_pro_title() -> &'static str {
    "Pro"
}

pub(crate) fn preferences_pro_description() -> &'static str {
    "Optional activation for paid features and future release extras."
}

pub(crate) fn preferences_pro_row_title() -> &'static str {
    "Snapix Pro"
}

pub(crate) fn preferences_appearance_updated_toast() -> &'static str {
    "Appearance updated"
}

pub(crate) fn preferences_default_format_updated_toast() -> &'static str {
    "Default export format updated"
}

pub(crate) fn preferences_export_preference_updated_toast() -> &'static str {
    "Export preference updated"
}

pub(crate) fn preferences_jpeg_quality_updated_toast() -> &'static str {
    "JPEG quality updated"
}

pub(crate) fn unlock_manage_button() -> &'static str {
    "Manage"
}

pub(crate) fn unlock_unlock_button() -> &'static str {
    "Unlock"
}

pub(crate) fn unlock_row_subtitle_active() -> &'static str {
    "Snapix Pro is active on this device."
}

pub(crate) fn unlock_row_subtitle_free() -> &'static str {
    "Free tier is active. Enter a key only if you need Pro features."
}

pub(crate) fn unlock_dialog_title() -> &'static str {
    "Unlock Pro"
}

pub(crate) fn unlock_dialog_heading() -> &'static str {
    "Unlock Snapix Pro"
}

pub(crate) fn unlock_dialog_body() -> &'static str {
    "Enter your activation key to unlock Pro features on this device. During M4 this still uses the local development verifier."
}

pub(crate) fn unlock_placeholder() -> &'static str {
    "SNAPIX-PRO-DEV"
}

pub(crate) fn unlock_use_free_tier_button() -> &'static str {
    "Use Free Tier"
}

pub(crate) fn cancel_button_label() -> &'static str {
    "Cancel"
}

pub(crate) fn unlock_activate_button() -> &'static str {
    "Unlock Pro"
}

pub(crate) fn unlock_status_active() -> &'static str {
    "Pro is currently active on this device."
}

pub(crate) fn unlock_status_free() -> &'static str {
    "Free tier is currently active."
}

pub(crate) fn unlock_failed_to_save_activation(error: &str) -> String {
    format!("Failed to save activation state: {error}")
}

pub(crate) fn unlock_deactivated_toast() -> &'static str {
    "Pro deactivated on this device"
}

pub(crate) fn unlock_activated_toast() -> &'static str {
    "Snapix Pro activated"
}

pub(crate) fn capture_wayland_fullscreen_tooltip() -> &'static str {
    "Hides Snapix, captures the full screen, then restores the window."
}

pub(crate) fn capture_wayland_region_tooltip() -> &'static str {
    "Opens the portal picker — drag to select an area to capture."
}

pub(crate) fn capture_wayland_window_tooltip() -> &'static str {
    "Opens the portal picker — click a window to capture it."
}

pub(crate) fn capture_success_toast(action: &str) -> &'static str {
    match action {
        "fullscreen" => "Full screen captured",
        "region" => "Selection captured",
        "window" => "Window captured",
        _ => "",
    }
}

pub(crate) fn capture_fallback_toast() -> &'static str {
    "Fullscreen capture failed; switched to interactive capture."
}

pub(crate) fn capture_failed_title() -> &'static str {
    "Capture failed"
}

pub(crate) fn capture_failed_detail(action: &str, error: &str) -> String {
    match action {
        "fullscreen" => format!("Fullscreen capture failed: {error}"),
        "region" => format!("Region capture failed: {error}"),
        "window" => format!("Window capture failed: {error}"),
        _ => error.to_string(),
    }
}

pub(crate) fn import_dialog_title() -> &'static str {
    "Import image"
}

pub(crate) fn import_accept_button() -> &'static str {
    "Import"
}

pub(crate) fn images_filter_name() -> &'static str {
    "Images"
}

pub(crate) fn imported_image_toast(path: &str) -> String {
    format!("Imported image from {path}")
}

pub(crate) fn import_failed_title() -> &'static str {
    "Import failed"
}

pub(crate) fn import_failed_open_detail(path: &str, error: &str) -> String {
    format!("Failed to open {path}: {error}")
}

pub(crate) fn import_failed_non_local_detail() -> &'static str {
    "The selected image is not a local file path."
}

pub(crate) fn image_cleared_toast() -> &'static str {
    "Image cleared"
}

pub(crate) fn copy_failed_title() -> &'static str {
    "Copy failed"
}

pub(crate) fn clipboard_read_failed_detail(error: &str) -> String {
    format!("Clipboard read failed: {error}")
}

pub(crate) fn clipboard_image_invalid_detail() -> &'static str {
    "Clipboard image data is invalid or incomplete."
}

pub(crate) fn clipboard_image_missing_detail() -> &'static str {
    "Clipboard does not currently contain an image."
}

pub(crate) fn image_copied_to_clipboard_toast() -> &'static str {
    "Image copied to clipboard"
}

pub(crate) fn image_pasted_from_clipboard_toast() -> &'static str {
    "Image pasted from clipboard"
}

pub(crate) fn paste_failed_title() -> &'static str {
    "Paste Failed"
}

pub(crate) fn quick_save_failed_title() -> &'static str {
    "Quick save failed"
}

pub(crate) fn saved_image_toast(path: &str) -> String {
    format!("Saved image to {path}")
}

pub(crate) fn export_dialog_title(format: &str) -> &'static str {
    match format {
        "png" => "Export PNG",
        "jpeg" => "Export JPEG",
        _ => "",
    }
}

pub(crate) fn export_failed_title() -> &'static str {
    "Export failed"
}

pub(crate) fn exported_image_toast(path: &str) -> String {
    format!("Exported image to {path}")
}

pub(crate) fn export_failed_non_local_detail() -> &'static str {
    "The selected destination is not a local file path."
}

pub(crate) fn text_dialog_cancel_button() -> &'static str {
    "Cancel"
}

pub(crate) fn text_dialog_placeholder() -> &'static str {
    "Type a short label"
}

pub(crate) fn edit_text_dialog_title() -> &'static str {
    "Edit Text"
}

pub(crate) fn edit_text_accept_button() -> &'static str {
    "Update"
}

pub(crate) fn text_content_field_label() -> &'static str {
    "Text content"
}

pub(crate) fn add_text_dialog_title() -> &'static str {
    "Add Text"
}

pub(crate) fn add_button_label() -> &'static str {
    "Add"
}

pub(crate) fn image_view_reset_toast() -> &'static str {
    "Image view reset"
}

pub(crate) fn reframe_active_toast() -> &'static str {
    "Image reframe active: drag to pan, scroll to zoom, Enter to finish"
}

pub(crate) fn reframe_done_toast() -> &'static str {
    "Image reframe finished"
}

pub(crate) fn couldnt_add_text_label_toast() -> &'static str {
    "Couldn't add text label"
}

pub(crate) fn arrow_too_small_toast() -> &'static str {
    "Arrow is too small"
}

pub(crate) fn rectangle_too_small_toast() -> &'static str {
    "Rectangle drag was too small"
}

pub(crate) fn ellipse_too_small_toast() -> &'static str {
    "Ellipse drag was too small"
}

pub(crate) fn blur_too_small_toast() -> &'static str {
    "Blur region was too small"
}

pub(crate) fn crop_too_small_toast() -> &'static str {
    "Crop selection is too small"
}

pub(crate) fn inspector_settings_title() -> &'static str {
    "Settings"
}

pub(crate) fn inspector_background_title() -> &'static str {
    "Background"
}

pub(crate) fn inspector_background_family_clean() -> &'static str {
    "Clean"
}

pub(crate) fn inspector_background_family_signature() -> &'static str {
    "Signature"
}

pub(crate) fn inspector_background_family_image() -> &'static str {
    "Image"
}

pub(crate) fn inspector_background_mode_blur() -> &'static str {
    "Screenshot Blur"
}

pub(crate) fn inspector_background_mode_image() -> &'static str {
    "Custom Image"
}

pub(crate) fn inspector_image_path_label() -> &'static str {
    "Image Path"
}

pub(crate) fn inspector_choose_image_button() -> &'static str {
    "Choose..."
}

pub(crate) fn inspector_background_mode_gradient() -> &'static str {
    "Gradient"
}

pub(crate) fn inspector_background_mode_solid() -> &'static str {
    "Solid"
}

pub(crate) fn inspector_background_blur_tooltip() -> &'static str {
    "Use the captured image as a blurred background fill"
}

pub(crate) fn inspector_pick_background_color() -> &'static str {
    "Pick Background Color"
}

pub(crate) fn inspector_pick_gradient_start() -> &'static str {
    "Pick Gradient Start"
}

pub(crate) fn inspector_pick_gradient_end() -> &'static str {
    "Pick Gradient End"
}

pub(crate) fn inspector_solid_color_label() -> &'static str {
    "Solid Color"
}

pub(crate) fn inspector_gradient_from_label() -> &'static str {
    "Gradient From"
}

pub(crate) fn inspector_gradient_to_label() -> &'static str {
    "Gradient To"
}

pub(crate) fn inspector_gradient_angle_label() -> &'static str {
    "Gradient Angle"
}

pub(crate) fn inspector_blur_radius_label() -> &'static str {
    "Blur Radius"
}

pub(crate) fn inspector_signature_intensity_label() -> &'static str {
    "Style Intensity"
}

pub(crate) fn inspector_saved_presets_title() -> &'static str {
    "Saved Presets"
}

pub(crate) fn inspector_preset_name_placeholder() -> &'static str {
    "Preset name"
}

pub(crate) fn save_button_label() -> &'static str {
    "Save"
}

pub(crate) fn apply_button_label() -> &'static str {
    "Apply"
}

pub(crate) fn delete_button_label() -> &'static str {
    "Delete"
}

pub(crate) fn inspector_padding_label() -> &'static str {
    "Padding"
}

pub(crate) fn inspector_corner_radius_label() -> &'static str {
    "Corner Radius"
}

pub(crate) fn inspector_output_ratio_title() -> &'static str {
    "Output Ratio"
}

pub(crate) fn ratio_tooltip(ratio: OutputRatio) -> &'static str {
    match ratio {
        OutputRatio::Auto => "Match the image's natural aspect ratio",
        OutputRatio::Square => "Square output, useful for thumbnails and social posts",
        OutputRatio::Landscape4x3 => "Classic landscape frame",
        OutputRatio::Landscape3x2 => "Balanced landscape frame",
        OutputRatio::Landscape16x9 => "Wide landscape frame for presentations and video stills",
        OutputRatio::Landscape5x3 => "Extra-wide landscape frame",
        OutputRatio::Portrait9x16 => "Tall portrait frame for stories and reels",
        OutputRatio::Portrait3x4 => "Classic portrait frame",
        OutputRatio::Portrait2x3 => "Photo-style portrait frame",
    }
}

pub(crate) fn inspector_image_reframe_title() -> &'static str {
    "Image Transform"
}

pub(crate) fn inspector_image_reframe_help() -> &'static str {
    "Drag the screenshot to reposition it, use Image Zoom to resize it, or use the mouse wheel while reframing for finer control."
}

pub(crate) fn inspector_image_zoom_label() -> &'static str {
    "Image Zoom"
}

pub(crate) fn reset_view_button_label() -> &'static str {
    "Reset View"
}

pub(crate) fn inspector_reset_view_help() -> &'static str {
    "Reset View returns the screenshot to centered Fit and clears any manual move or zoom."
}

pub(crate) fn inspector_shadow_label() -> &'static str {
    "Shadow"
}

pub(crate) fn inspector_shadow_direction_label() -> &'static str {
    "Shadow Direction"
}

pub(crate) fn shadow_direction_tooltip(index: usize) -> &'static str {
    match index {
        0 => "Shadow toward top left",
        1 => "Shadow toward top",
        2 => "Shadow toward top right",
        3 => "Shadow toward left",
        4 => "Centered glow shadow",
        5 => "Shadow toward right",
        6 => "Shadow toward bottom left",
        7 => "Shadow toward bottom",
        8 => "Shadow toward bottom right",
        _ => "",
    }
}

pub(crate) fn inspector_shadow_padding_label() -> &'static str {
    "Shadow Padding"
}

pub(crate) fn inspector_shadow_blur_label() -> &'static str {
    "Shadow Blur"
}

pub(crate) fn inspector_shadow_strength_label() -> &'static str {
    "Shadow Strength"
}

pub(crate) fn subtitle_text_empty() -> &'static str {
    "No image loaded. Capture or import an image to begin."
}

pub(crate) fn subtitle_annotations(count: usize) -> String {
    match count {
        0 => "no annotations yet".to_string(),
        1 => "1 annotation".to_string(),
        _ => format!("{count} annotations"),
    }
}

pub(crate) fn subtitle_output_text(output_width: f64, output_height: f64) -> String {
    format!(
        "output {}×{}",
        output_width.round() as u32,
        output_height.round() as u32
    )
}

pub(crate) fn subtitle_ratio_text(ratio: OutputRatio) -> String {
    match ratio {
        OutputRatio::Auto => "ratio auto".to_string(),
        _ => format!("ratio {}", ratio.label()),
    }
}

pub(crate) fn image_mode_text_fit() -> &'static str {
    "image fit"
}

pub(crate) fn image_mode_text_fill(anchor: ImageAnchor) -> String {
    format!("image fill {}", image_anchor_label(anchor))
}

pub(crate) fn image_anchor_label(anchor: ImageAnchor) -> &'static str {
    match anchor {
        ImageAnchor::TopLeft => "top-left",
        ImageAnchor::Top => "top",
        ImageAnchor::TopRight => "top-right",
        ImageAnchor::Left => "left",
        ImageAnchor::Center => "center",
        ImageAnchor::Right => "right",
        ImageAnchor::BottomLeft => "bottom-left",
        ImageAnchor::Bottom => "bottom",
        ImageAnchor::BottomRight => "bottom-right",
    }
}

pub(crate) fn shortcut_hint_empty() -> &'static str {
    "Fullscreen / Region / Import / Paste to begin"
}

pub(crate) fn shortcut_hint_reframe() -> &'static str {
    "Drag pan • Scroll zoom • Enter finish • Esc exit"
}

pub(crate) fn shortcut_hint_selected() -> &'static str {
    "Delete remove • Ctrl+Z undo"
}

pub(crate) fn shortcut_hint_select_idle() -> &'static str {
    "Click annotation to edit • Double-click image to reframe • Ctrl+V paste image"
}

pub(crate) fn shortcut_hint_crop_active() -> &'static str {
    "Enter apply • Esc cancel"
}

pub(crate) fn shortcut_hint_crop_idle() -> &'static str {
    "Drag to select • Esc cancel"
}

pub(crate) fn shortcut_hint_draw_shape() -> &'static str {
    "Drag on image to draw • Ctrl+Z undo"
}

pub(crate) fn shortcut_hint_text() -> &'static str {
    "Click to place • Double-click text to edit"
}

pub(crate) fn scope_text_reframe() -> &'static str {
    "Reframe: drag the image to reposition it, use the mouse wheel to zoom, press Enter when done, or Esc to exit."
}

pub(crate) fn scope_text_selected(annotation_kind: &str) -> String {
    format!("Selected {annotation_kind}. Adjust color or size, or press Delete to remove it.")
}

pub(crate) fn scope_text_select_idle() -> &'static str {
    "Select: click an annotation to edit it. Press Delete to remove it, Ctrl+Z to undo."
}

pub(crate) fn scope_text_crop_empty() -> &'static str {
    "Crop: capture or import an image first."
}

pub(crate) fn scope_text_crop_active() -> &'static str {
    "Crop: drag handles to adjust, press Enter to apply, or Esc to cancel."
}

pub(crate) fn scope_text_crop_idle() -> &'static str {
    "Crop: drag on the image to create a selection, then press Enter to apply."
}

pub(crate) fn scope_text_arrow() -> &'static str {
    "Arrow: drag on the image to place an arrow."
}

pub(crate) fn scope_text_line() -> &'static str {
    "Line: drag on the image to draw a straight line."
}

pub(crate) fn scope_text_rectangle() -> &'static str {
    "Rectangle: drag on the image to draw a box."
}

pub(crate) fn scope_text_ellipse() -> &'static str {
    "Ellipse: drag on the image to draw an oval."
}

pub(crate) fn scope_text_text() -> &'static str {
    "Text: click on the image to place a label. Double-click text to edit it."
}

pub(crate) fn scope_text_blur() -> &'static str {
    "Blur: drag on the image to blur part of the image."
}

pub(crate) fn annotation_kind_label(kind: &str) -> String {
    kind.to_string()
}
