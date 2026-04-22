use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::ActionRow;
use libadwaita::ToastOverlay;

use crate::editor::preferences::{save_preferences, AppPreferences};
use crate::editor::show_toast;

pub(super) fn refresh_unlock_entry(
    button: &gtk4::Button,
    row: &ActionRow,
    preferences: &AppPreferences,
) {
    let is_pro = preferences.entitlements().is_pro();
    if is_pro {
        button.set_label("Manage");
        button.remove_css_class("suggested-action");
        row.set_subtitle("Snapix Pro is active on this device.");
    } else {
        button.set_label("Unlock");
        button.add_css_class("suggested-action");
        row.set_subtitle("Free tier is active. Enter a key only if you need Pro features.");
    }
}

pub(super) fn present_unlock_dialog(
    parent: &libadwaita::ApplicationWindow,
    preferences: Rc<RefCell<AppPreferences>>,
    toast_overlay: &ToastOverlay,
    on_change: Rc<dyn Fn(&AppPreferences)>,
) {
    let dialog = gtk4::Window::builder()
        .title("Unlock Pro")
        .transient_for(parent)
        .modal(true)
        .default_width(420)
        .default_height(0)
        .resizable(false)
        .build();

    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(14)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();

    let title = gtk4::Label::builder()
        .label("Unlock Snapix Pro")
        .xalign(0.0)
        .css_classes(["title-3"])
        .build();
    let body = gtk4::Label::builder()
        .label("Enter your activation key to unlock Pro features on this device. During M4 this still uses the local development verifier.")
        .xalign(0.0)
        .wrap(true)
        .build();
    body.add_css_class("dim-label");

    let status = gtk4::Label::builder().xalign(0.0).wrap(true).build();
    status.add_css_class("dim-label");

    let entry = gtk4::Entry::builder()
        .placeholder_text("SNAPIX-PRO-DEV")
        .hexpand(true)
        .build();
    if let Some(existing_key) = preferences.borrow().license_key.clone() {
        entry.set_text(&existing_key);
    }

    let actions = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::End)
        .build();
    let deactivate_button = gtk4::Button::with_label("Use Free Tier");
    let cancel_button = gtk4::Button::with_label("Cancel");
    let activate_button = gtk4::Button::with_label("Unlock Pro");
    activate_button.add_css_class("suggested-action");
    actions.append(&deactivate_button);
    actions.append(&cancel_button);
    actions.append(&activate_button);

    content.append(&title);
    content.append(&body);
    content.append(&entry);
    content.append(&status);
    content.append(&actions);
    dialog.set_child(Some(&content));

    {
        let status = status.clone();
        let is_pro = preferences.borrow().entitlements().is_pro();
        deactivate_button.set_sensitive(is_pro);
        if is_pro {
            status.set_label("Pro is currently active on this device.");
        } else {
            status.set_label("Free tier is currently active.");
        }
    }

    {
        let dialog = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog.close();
        });
    }

    {
        let preferences = preferences.clone();
        let toast_overlay = toast_overlay.clone();
        let dialog = dialog.clone();
        let status = status.clone();
        let on_change = on_change.clone();
        deactivate_button.connect_clicked(move |_| {
            let mut preferences = preferences.borrow_mut();
            preferences.clear_license_key();
            if let Err(error) = save_preferences(&preferences) {
                status.set_label(&format!("Failed to save activation state: {error}"));
                return;
            }
            on_change(&preferences);
            drop(preferences);
            show_toast(&toast_overlay, "Pro deactivated on this device");
            dialog.close();
        });
    }

    {
        let preferences = preferences.clone();
        let toast_overlay = toast_overlay.clone();
        let dialog = dialog.clone();
        let status = status.clone();
        let entry = entry.clone();
        let on_change = on_change.clone();
        activate_button.connect_clicked(move |_| {
            let key = entry.text().to_string();
            let mut preferences = preferences.borrow_mut();
            match preferences.activate_license_key(&key) {
                Ok(_) => {
                    if let Err(error) = save_preferences(&preferences) {
                        status.set_label(&format!("Failed to save activation state: {error}"));
                        return;
                    }
                    on_change(&preferences);
                    drop(preferences);
                    show_toast(&toast_overlay, "Snapix Pro activated");
                    dialog.close();
                }
                Err(error) => {
                    status.set_label(&error.to_string());
                }
            }
        });
    }

    dialog.present();
}
