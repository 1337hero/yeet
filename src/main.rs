mod config;
mod desktop;
mod history;
mod ui;

use config::Config;
use desktop::{discover_apps, launch_app, App};
use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4::Application;
use std::cell::Cell;
use std::io::BufRead;
use std::rc::Rc;

const APP_ID: &str = "dev.yeet.launcher";

fn main() {
    let mut dmenu = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-d" | "--dmenu" => dmenu = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-V" | "--version" => {
                println!("yeet {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            other => {
                eprintln!("yeet: unknown option '{other}' (see --help)");
                std::process::exit(2);
            }
        }
    }

    let config = Config::load();

    if dmenu {
        run_dmenu(config);
    } else {
        run_launcher(config);
    }
}

fn print_help() {
    println!(
        "yeet {} - a fast, minimal app launcher for Wayland

Usage: yeet [OPTIONS]

Options:
  -d, --dmenu    read items from stdin, print the selection to stdout
  -h, --help     print this help
  -V, --version  print version",
        env!("CARGO_PKG_VERSION")
    );
}

fn gtk_app() -> Application {
    // NON_UNIQUE: each invocation gets its own window and, in dmenu mode,
    // its own stdin/stdout instead of activating an existing instance.
    Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::NON_UNIQUE)
        .build()
}

fn run_launcher(config: Config) {
    let apps = discover_apps(&config);
    let app = gtk_app();

    app.connect_activate(move |app| {
        let terminal = config.general.terminal.clone();
        let on_select: Rc<dyn Fn(&App)> = Rc::new(move |app| launch_app(app, &terminal));
        ui::build_ui(app, &config, apps.clone(), on_select);
    });

    // we don't use GTK's arg parsing
    app.run_with_args::<&str>(&[]);
}

fn run_dmenu(mut config: Config) {
    // dmenu items are arbitrary lines: show all of them up front and keep
    // launch history out of both ranking and recording.
    config.general.initial_results = 0;
    config.search.use_history = false;

    let items: Vec<App> = std::io::stdin()
        .lock()
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.is_empty())
        .map(App::plain)
        .collect();

    if items.is_empty() {
        eprintln!("yeet: --dmenu expects items on stdin");
        std::process::exit(1);
    }

    let selected = Rc::new(Cell::new(false));
    let app = gtk_app();

    let selected_flag = selected.clone();
    app.connect_activate(move |app| {
        let selected_flag = selected_flag.clone();
        let on_select: Rc<dyn Fn(&App)> = Rc::new(move |item| {
            println!("{}", item.name);
            selected_flag.set(true);
        });
        ui::build_ui(app, &config, items.clone(), on_select);
    });

    app.run_with_args::<&str>(&[]);
    std::process::exit(if selected.get() { 0 } else { 1 });
}
