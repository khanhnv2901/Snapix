use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, ApplicationWindow, PreferencesDialog, PreferencesGroup, PreferencesPage, SwitchRow,
};

use crate::editor::i18n::{
    app_window_title, preferences_about_description, preferences_about_title,
    preferences_app_author_title, preferences_app_license_title, preferences_app_name_title,
    preferences_app_repository_title, preferences_app_version_title,
    preferences_appearance_description, preferences_appearance_title,
    preferences_appearance_updated_toast, preferences_auto_copy_after_export_subtitle,
    preferences_auto_copy_after_export_title, preferences_auto_reframe_subtitle,
    preferences_auto_reframe_title, preferences_color_scheme_subtitle,
    preferences_color_scheme_title, preferences_default_format_updated_toast,
    preferences_dialog_title, preferences_editing_description, preferences_editing_title,
    preferences_editing_updated_toast, preferences_export_description,
    preferences_export_preference_updated_toast, preferences_export_title,
    preferences_jpeg_quality_subtitle, preferences_jpeg_quality_title,
    preferences_jpeg_quality_updated_toast, preferences_open_link_label,
    preferences_pro_description, preferences_pro_row_title, preferences_pro_title,
    preferences_quick_save_location_subtitle, preferences_quick_save_location_title,
    preferences_remember_format_subtitle, preferences_remember_format_title,
    preferences_save_format_subtitle, preferences_save_format_title, preferences_storage_subtitle,
    preferences_storage_title,
};
use crate::editor::preferences::{
    apply_style_preferences, save_preferences, AppPreferences, AppearancePreference,
    PreferredSaveFormat,
};
use crate::editor::show_toast;

use super::unlock::{present_unlock_dialog, refresh_unlock_entry};
use super::{BottomBar, SaveFormat};

pub(super) fn present_preferences_window(
    parent: &ApplicationWindow,
    preferences: Rc<RefCell<AppPreferences>>,
    save_format: Rc<RefCell<SaveFormat>>,
    bottom_bar: &BottomBar,
    toast_overlay: &libadwaita::ToastOverlay,
) {
    let dialog = PreferencesDialog::builder()
        .title(preferences_dialog_title())
        .search_enabled(false)
        .follows_content_size(true)
        .content_width(760)
        .build();

    let page = PreferencesPage::new();
    let export_group = PreferencesGroup::builder()
        .title(preferences_export_title())
        .description(preferences_export_description())
        .build();
    let appearance_group = PreferencesGroup::builder()
        .title(preferences_appearance_title())
        .description(preferences_appearance_description())
        .build();
    let editing_group = PreferencesGroup::builder()
        .title(preferences_editing_title())
        .description(preferences_editing_description())
        .build();

    let appearance_row = ActionRow::builder()
        .title(preferences_color_scheme_title())
        .subtitle(preferences_color_scheme_subtitle())
        .build();
    let appearance_dropdown = gtk4::DropDown::from_strings(&["System", "Light", "Dark"]);
    appearance_dropdown.set_valign(gtk4::Align::Center);
    appearance_dropdown.set_selected(match preferences.borrow().appearance {
        AppearancePreference::System => 0,
        AppearancePreference::Light => 1,
        AppearancePreference::Dark => 2,
    });
    appearance_row.add_suffix(&appearance_dropdown);
    appearance_row.set_activatable_widget(Some(&appearance_dropdown));
    appearance_group.add(&appearance_row);

    let format_row = ActionRow::builder()
        .title(preferences_save_format_title())
        .subtitle(preferences_save_format_subtitle())
        .build();
    let format_dropdown = gtk4::DropDown::from_strings(&["PNG", "JPEG"]);
    format_dropdown.set_valign(gtk4::Align::Center);
    format_dropdown.set_selected(match preferences.borrow().default_save_format {
        PreferredSaveFormat::Png => 0,
        PreferredSaveFormat::Jpeg => 1,
    });
    format_row.add_suffix(&format_dropdown);
    format_row.set_activatable_widget(Some(&format_dropdown));
    export_group.add(&format_row);

    let remember_row = SwitchRow::builder()
        .title(preferences_remember_format_title())
        .subtitle(preferences_remember_format_subtitle())
        .active(preferences.borrow().remember_last_export_format)
        .build();
    export_group.add(&remember_row);

    let jpeg_quality_row = ActionRow::builder()
        .title(preferences_jpeg_quality_title())
        .subtitle(preferences_jpeg_quality_subtitle())
        .build();
    let jpeg_quality_value = gtk4::Label::builder()
        .label(format!("{}", preferences.borrow().effective_jpeg_quality()))
        .css_classes(["dim-copy"])
        .build();
    let jpeg_quality_scale =
        gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 60.0, 100.0, 1.0);
    jpeg_quality_scale.set_draw_value(false);
    jpeg_quality_scale.set_value(preferences.borrow().effective_jpeg_quality() as f64);
    jpeg_quality_row.add_suffix(&jpeg_quality_value);
    jpeg_quality_row.add_suffix(&jpeg_quality_scale);
    jpeg_quality_row.set_activatable_widget(Some(&jpeg_quality_scale));
    export_group.add(&jpeg_quality_row);

    let auto_copy_row = SwitchRow::builder()
        .title(preferences_auto_copy_after_export_title())
        .subtitle(preferences_auto_copy_after_export_subtitle())
        .active(preferences.borrow().auto_copy_after_export)
        .build();
    export_group.add(&auto_copy_row);

    let auto_reframe_row = SwitchRow::builder()
        .title(preferences_auto_reframe_title())
        .subtitle(preferences_auto_reframe_subtitle())
        .active(preferences.borrow().auto_reframe_after_load)
        .build();
    editing_group.add(&auto_reframe_row);

    let notes_group = PreferencesGroup::builder()
        .title(preferences_about_title())
        .description(preferences_about_description())
        .build();
    let app_name_row = ActionRow::builder()
        .title(preferences_app_name_title())
        .subtitle(app_window_title())
        .build();
    notes_group.add(&app_name_row);
    let version_row = ActionRow::builder()
        .title(preferences_app_version_title())
        .subtitle(env!("CARGO_PKG_VERSION"))
        .build();
    notes_group.add(&version_row);
    let author_row = ActionRow::builder()
        .title(preferences_app_author_title())
        .subtitle(primary_author_name())
        .build();
    notes_group.add(&author_row);
    let license_row_info = ActionRow::builder()
        .title(preferences_app_license_title())
        .subtitle(env!("CARGO_PKG_LICENSE"))
        .build();
    notes_group.add(&license_row_info);
    let repository_row = ActionRow::builder()
        .title(preferences_app_repository_title())
        .subtitle(env!("CARGO_PKG_REPOSITORY"))
        .build();
    let repository_button = gtk4::LinkButton::builder()
        .uri(env!("CARGO_PKG_REPOSITORY"))
        .label(preferences_open_link_label())
        .valign(gtk4::Align::Center)
        .build();
    repository_row.add_suffix(&repository_button);
    repository_row.set_activatable_widget(Some(&repository_button));
    notes_group.add(&repository_row);
    let notes_row = ActionRow::builder()
        .title(preferences_storage_title())
        .subtitle(preferences_storage_subtitle())
        .build();
    notes_group.add(&notes_row);
    let quick_save_dir = gtk4::glib::user_special_dir(gtk4::glib::UserDirectory::Pictures)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Screenshots");
    let quick_save_row = ActionRow::builder()
        .title(preferences_quick_save_location_title())
        .subtitle(preferences_quick_save_location_subtitle(
            &quick_save_dir.display().to_string(),
        ))
        .build();
    notes_group.add(&quick_save_row);

    let license_group = PreferencesGroup::builder()
        .title(preferences_pro_title())
        .description(preferences_pro_description())
        .build();
    let license_row = ActionRow::builder()
        .title(preferences_pro_row_title())
        .build();
    let license_button = gtk4::Button::new();
    refresh_unlock_entry(&license_button, &license_row, &preferences.borrow());
    license_row.add_suffix(&license_button);
    license_row.set_activatable_widget(Some(&license_button));
    license_group.add(&license_row);

    page.add(&appearance_group);
    page.add(&export_group);
    page.add(&editing_group);
    page.add(&notes_group);
    page.add(&license_group);
    dialog.add(&page);

    {
        let preferences = preferences.clone();
        let parent = parent.clone();
        let toast_overlay = toast_overlay.clone();
        let license_button = license_button.clone();
        let license_row = license_row.clone();
        license_button.clone().connect_clicked(move |_| {
            let license_button = license_button.clone();
            let license_row = license_row.clone();
            present_unlock_dialog(
                &parent,
                preferences.clone(),
                &toast_overlay,
                Rc::new(move |preferences| {
                    refresh_unlock_entry(&license_button, &license_row, preferences);
                }),
            );
        });
    }

    {
        let preferences = preferences.clone();
        let toast_overlay = toast_overlay.clone();
        appearance_dropdown.connect_selected_notify(move |dropdown| {
            let appearance = match dropdown.selected() {
                1 => AppearancePreference::Light,
                2 => AppearancePreference::Dark,
                _ => AppearancePreference::System,
            };
            let mut preferences = preferences.borrow_mut();
            preferences.appearance = appearance;
            apply_style_preferences(&preferences);
            if let Err(error) = save_preferences(&preferences) {
                tracing::warn!("Failed to save preferences: {error:#}");
            }
            show_toast(&toast_overlay, preferences_appearance_updated_toast());
        });
    }

    {
        let preferences = preferences.clone();
        let save_format = save_format.clone();
        let bottom_bar = bottom_bar.clone();
        let toast_overlay = toast_overlay.clone();
        format_dropdown.connect_selected_notify(move |dropdown| {
            let format = if dropdown.selected() == 1 {
                PreferredSaveFormat::Jpeg
            } else {
                PreferredSaveFormat::Png
            };
            {
                let mut preferences = preferences.borrow_mut();
                preferences.default_save_format = format;
                if !preferences.remember_last_export_format {
                    preferences.last_export_format = Some(format);
                }
                if let Err(error) = save_preferences(&preferences) {
                    tracing::warn!("Failed to save preferences: {error:#}");
                }
            }
            if !preferences.borrow().remember_last_export_format {
                apply_save_format(
                    &save_format,
                    &bottom_bar,
                    match format {
                        PreferredSaveFormat::Png => SaveFormat::Png,
                        PreferredSaveFormat::Jpeg => SaveFormat::Jpeg,
                    },
                );
            }
            show_toast(&toast_overlay, preferences_default_format_updated_toast());
        });
    }

    {
        let preferences = preferences.clone();
        let save_format = save_format.clone();
        let bottom_bar = bottom_bar.clone();
        let toast_overlay = toast_overlay.clone();
        remember_row.connect_active_notify(move |row| {
            let mut preferences = preferences.borrow_mut();
            preferences.remember_last_export_format = row.is_active();
            preferences.last_export_format = Some(match *save_format.borrow() {
                SaveFormat::Png => PreferredSaveFormat::Png,
                SaveFormat::Jpeg => PreferredSaveFormat::Jpeg,
            });
            if let Err(error) = save_preferences(&preferences) {
                tracing::warn!("Failed to save preferences: {error:#}");
            }

            if !preferences.remember_last_export_format {
                let format = preferences.default_save_format;
                drop(preferences);
                apply_save_format(
                    &save_format,
                    &bottom_bar,
                    match format {
                        PreferredSaveFormat::Png => SaveFormat::Png,
                        PreferredSaveFormat::Jpeg => SaveFormat::Jpeg,
                    },
                );
            }
            show_toast(
                &toast_overlay,
                preferences_export_preference_updated_toast(),
            );
        });
    }

    {
        let preferences = preferences.clone();
        let toast_overlay = toast_overlay.clone();
        let jpeg_quality_value = jpeg_quality_value.clone();
        jpeg_quality_scale.connect_value_changed(move |scale| {
            let quality = scale.value().round() as u8;
            jpeg_quality_value.set_label(&quality.to_string());
            let mut preferences = preferences.borrow_mut();
            preferences.jpeg_quality = quality;
            if let Err(error) = save_preferences(&preferences) {
                tracing::warn!("Failed to save preferences: {error:#}");
            }
            show_toast(&toast_overlay, preferences_jpeg_quality_updated_toast());
        });
    }

    {
        let preferences = preferences.clone();
        let toast_overlay = toast_overlay.clone();
        auto_copy_row.connect_active_notify(move |row| {
            let mut preferences = preferences.borrow_mut();
            preferences.auto_copy_after_export = row.is_active();
            if let Err(error) = save_preferences(&preferences) {
                tracing::warn!("Failed to save preferences: {error:#}");
            }
            show_toast(
                &toast_overlay,
                preferences_export_preference_updated_toast(),
            );
        });
    }

    {
        let preferences = preferences.clone();
        let toast_overlay = toast_overlay.clone();
        auto_reframe_row.connect_active_notify(move |row| {
            let mut preferences = preferences.borrow_mut();
            preferences.auto_reframe_after_load = row.is_active();
            if let Err(error) = save_preferences(&preferences) {
                tracing::warn!("Failed to save preferences: {error:#}");
            }
            show_toast(&toast_overlay, preferences_editing_updated_toast());
        });
    }

    dialog.present(Some(parent));
}

fn apply_save_format(
    save_format: &Rc<RefCell<SaveFormat>>,
    bottom_bar: &BottomBar,
    format: SaveFormat,
) {
    *save_format.borrow_mut() = format;
    match format {
        SaveFormat::Png => bottom_bar.png_button.set_active(true),
        SaveFormat::Jpeg => bottom_bar.jpeg_button.set_active(true),
    }
}

fn primary_author_name() -> String {
    env!("CARGO_PKG_AUTHORS")
        .split(':')
        .next()
        .unwrap_or("Khanh Nguyen")
        .split('<')
        .next()
        .unwrap_or("Khanh Nguyen")
        .trim()
        .to_string()
}
