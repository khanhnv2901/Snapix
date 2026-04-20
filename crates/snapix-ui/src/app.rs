use gtk4::prelude::*;
use libadwaita::Application;

use crate::editor::EditorWindow;

pub const APP_ID: &str = "io.github.snapix.Snapix";

pub struct SnapixApp;

impl SnapixApp {
    pub fn run() -> glib::ExitCode {
        let app = Application::builder().application_id(APP_ID).build();

        app.connect_activate(build_ui);
        app.run()
    }
}

fn build_ui(app: &Application) {
    let editor = EditorWindow::new(app);
    editor.present();
}
