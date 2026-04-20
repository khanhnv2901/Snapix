use gtk4::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, ToolbarView};

const APP_ID: &str = "io.github.snapix.Snapix";

pub struct SnapixApp;

impl SnapixApp {
    pub fn run() -> glib::ExitCode {
        let app = Application::builder().application_id(APP_ID).build();

        app.connect_activate(build_ui);
        app.run()
    }
}

fn build_ui(app: &Application) {
    let header = HeaderBar::new();

    let content = gtk4::Label::builder()
        .label("Snapix — screenshot beautifier")
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(48)
        .margin_end(48)
        .build();

    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&content));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Snapix")
        .default_width(800)
        .default_height(600)
        .content(&toolbar_view)
        .build();

    window.present();
}
