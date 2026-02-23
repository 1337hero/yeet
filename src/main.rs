#![allow(dead_code)]

mod config;
mod desktop;
pub mod history;
mod ui;

use config::Config;
use desktop::discover_apps;
use gtk4::prelude::*;
use gtk4::Application;

const APP_ID: &str = "dev.yeet.launcher";

fn main() {
    let config = Config::load();

    let apps = discover_apps(&config);

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        ui::build_ui(app, &config, apps.clone());
    });

    // we don't use GTK's arg parsing
    app.run_with_args::<&str>(&[]);
}
