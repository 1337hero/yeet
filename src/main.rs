mod config;
mod desktop;
mod history;
mod ui;

use config::Config;
use desktop::discover_apps;
use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4::Application;

const APP_ID: &str = "dev.yeet.launcher";

fn main() {
    let config = Config::load();

    let apps = discover_apps(&config);

    // NON_UNIQUE: launching yeet while a window is already open gets its own
    // instance instead of stacking a second window inside the first one.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| {
        ui::build_ui(app, &config, apps.clone());
    });

    // we don't use GTK's arg parsing
    app.run_with_args::<&str>(&[]);
}
