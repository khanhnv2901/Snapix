use gtk4::prelude::*;

use crate::editor::i18n;

pub(super) fn present_text_dialog<F>(
    window: &gtk4::ApplicationWindow,
    title: &str,
    accept_label: &str,
    field_label: &str,
    initial_text: &str,
    on_accept: F,
) where
    F: Fn(String) + 'static,
{
    let dialog = gtk4::Dialog::builder()
        .title(title)
        .transient_for(window)
        .modal(true)
        .build();
    dialog.add_button(
        i18n::text_dialog_cancel_button(),
        gtk4::ResponseType::Cancel,
    );
    dialog.add_button(accept_label, gtk4::ResponseType::Accept);
    dialog.set_default_response(gtk4::ResponseType::Accept);

    let content = dialog.content_area();
    content.set_spacing(10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let entry = gtk4::Entry::builder()
        .text(initial_text)
        .placeholder_text(i18n::text_dialog_placeholder())
        .activates_default(true)
        .build();
    entry.select_region(0, -1);
    content.append(
        &gtk4::Label::builder()
            .label(field_label)
            .xalign(0.0)
            .build(),
    );
    content.append(&entry);

    dialog.connect_response(move |dialog, response| {
        if response == gtk4::ResponseType::Accept {
            on_accept(entry.text().to_string());
        }
        dialog.close();
    });
    dialog.present();
}
