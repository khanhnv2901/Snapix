use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{
    ActionRow, ApplicationWindow, PreferencesDialog, PreferencesGroup, PreferencesPage, SwitchRow,
};

use crate::editor::i18n::{
    preferences_about_description, preferences_about_title, preferences_appearance_description,
    preferences_appearance_title, preferences_appearance_updated_toast,
    preferences_color_scheme_subtitle, preferences_color_scheme_title,
    preferences_default_format_updated_toast, preferences_dialog_title,
    preferences_export_description, preferences_export_preference_updated_toast,
    preferences_export_title, preferences_pro_description, preferences_pro_row_title,
    preferences_pro_title, preferences_remember_format_subtitle, preferences_remember_format_title,
    preferences_save_format_subtitle, preferences_save_format_title,
    preferences_storage_subtitle, preferences_storage_title,
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
        .content_width(560)
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

    let notes_group = PreferencesGroup::builder()
        .title(preferences_about_title())
        .description(preferences_about_description())
        .build();
    let notes_row = ActionRow::builder()
        .title(preferences_storage_title())
        .subtitle(preferences_storage_subtitle())
        .build();
    notes_group.add(&notes_row);

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
            show_toast(&toast_overlay, preferences_export_preference_updated_toast());
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
