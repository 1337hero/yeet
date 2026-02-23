use crate::config::Config;
use crate::desktop::{launch_app, App};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gtk4::gdk::{Display, ModifierType};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, CssProvider, Entry, Label, ListBox, ListBoxRow,
    Orientation,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const DEFAULT_STYLE: &str = include_str!("../defaults/style.css");

pub fn build_ui(app: &Application, config: &Config, apps: Vec<App>) {
    load_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("yeet")
        .default_width(config.appearance.width)
        .decorated(false)
        .resizable(false)
        .build();

    if gtk4_layer_shell::is_supported() {
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_namespace(Some("yeet"));
        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, config.appearance.anchor_top);
    }

    window.add_css_class("yeet-window");

    let vbox = GtkBox::new(Orientation::Vertical, 0);
    vbox.add_css_class("yeet-container");

    let entry = Entry::builder().placeholder_text("Search...").build();
    entry.add_css_class("yeet-entry");

    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    list_box.add_css_class("yeet-list");

    let list_height = config.appearance.row_height * config.general.max_results as i32;
    list_box.set_size_request(-1, list_height);

    vbox.append(&entry);
    vbox.append(&list_box);
    window.set_child(Some(&vbox));

    let apps = Rc::new(apps);
    let app_names_lower: Rc<Vec<String>> =
        Rc::new(apps.iter().map(|a| a.name.to_lowercase()).collect());
    let app_name_keyword_texts: Rc<Vec<String>> = Rc::new(
        apps.iter()
            .map(|a| {
                let mut text = a.name.clone();
                for kw in &a.keywords {
                    text.push(' ');
                    text.push_str(kw);
                }
                text
            })
            .collect(),
    );
    let app_name_keyword_texts_lower: Rc<Vec<String>> = Rc::new(
        app_name_keyword_texts
            .iter()
            .map(|t| t.to_lowercase())
            .collect(),
    );
    let filtered_apps: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let matcher = Rc::new(SkimMatcherV2::default());
    let max_results = config.general.max_results;
    let initial_results = config.general.initial_results;
    let min_score = config.search.min_score;
    let score_threshold = config.search.score_threshold;
    let prefer_prefix = config.search.prefer_prefix;
    let terminal = config.general.terminal.clone();
    let use_history = config.search.use_history;
    let history: Rc<HashMap<String, u64>> = Rc::new(if use_history {
        crate::history::load_history()
    } else {
        HashMap::new()
    });
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    {
        let mut filtered = filtered_apps.borrow_mut();
        filtered.clear();
        for (i, _) in apps.iter().enumerate().take(initial_results) {
            filtered.push(i);
        }
        if !history.is_empty() {
            filtered.sort_by(|&a, &b| {
                let a_ts = history.get(&apps[a].name).copied().unwrap_or(0);
                let b_ts = history.get(&apps[b].name).copied().unwrap_or(0);
                b_ts.cmp(&a_ts)
            });
        }
        populate_list(&list_box, &apps, &filtered);
    }

    if let Some(row) = list_box.row_at_index(0) {
        list_box.select_row(Some(&row));
    }

    {
        let apps = apps.clone();
        let app_names_lower = app_names_lower.clone();
        let app_name_keyword_texts = app_name_keyword_texts.clone();
        let app_name_keyword_texts_lower = app_name_keyword_texts_lower.clone();
        let filtered_apps = filtered_apps.clone();
        let matcher = matcher.clone();
        let history = history.clone();
        let list_box = list_box.clone();

        entry.connect_changed(move |entry| {
            let query = entry.text();
            let query = query.trim();
            let query_len = query.chars().count();
            let mut filtered = filtered_apps.borrow_mut();
            filtered.clear();

            if query_len == 0 {
                for (i, _) in apps.iter().enumerate().take(initial_results) {
                    filtered.push(i);
                }
                if !history.is_empty() {
                    filtered.sort_by(|&a, &b| {
                        let a_ts = history.get(&apps[a].name).copied().unwrap_or(0);
                        let b_ts = history.get(&apps[b].name).copied().unwrap_or(0);
                        b_ts.cmp(&a_ts)
                    });
                }
                populate_list(&list_box, &apps, &filtered);
                if let Some(row) = list_box.row_at_index(0) {
                    list_box.select_row(Some(&row));
                }
                return;
            }

            let query_lower = query.to_lowercase();
            let has_substring_matches = query_len >= 2
                && app_name_keyword_texts_lower
                    .iter()
                    .any(|t| t.contains(&query_lower));

            let mut scored: Vec<(usize, i64, bool)> = if has_substring_matches {
                app_name_keyword_texts_lower
                    .iter()
                    .enumerate()
                    .filter(|(_, text)| text.contains(&query_lower))
                    .map(|(i, _)| {
                        let score = matcher
                            .fuzzy_match(&app_name_keyword_texts[i], query)
                            .unwrap_or(0);
                        let is_prefix =
                            prefer_prefix && app_names_lower[i].starts_with(&query_lower);
                        let boost = recency_boost(&history, &apps[i].name, now);
                        (i, score + boost, is_prefix)
                    })
                    .collect()
            } else {
                app_name_keyword_texts
                    .iter()
                    .enumerate()
                    .filter_map(|(i, text)| {
                        matcher.fuzzy_match(text, query).map(|score| {
                            let is_prefix =
                                prefer_prefix && app_names_lower[i].starts_with(&query_lower);
                            let boost = recency_boost(&history, &apps[i].name, now);
                            (i, score + boost, is_prefix)
                        })
                    })
                    .collect()
            };

            scored.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.1.cmp(&a.1)));

            if query_len >= 2 && !has_substring_matches {
                scored.retain(|(_, score, _)| *score >= min_score);

                let threshold = score_threshold.clamp(0.0, 1.0);
                let best_score = scored.first().map(|x| x.1).unwrap_or(0);
                let cutoff = (best_score as f64 * threshold) as i64;

                scored.retain(|(_, score, _)| *score >= cutoff);
            }

            for (i, _score, _) in scored.into_iter().take(max_results) {
                filtered.push(i);
            }

            populate_list(&list_box, &apps, &filtered);

            if let Some(row) = list_box.row_at_index(0) {
                list_box.select_row(Some(&row));
            }
        });
    }

    {
        let list_box_enter = list_box.clone();
        let apps_enter = apps.clone();
        let filtered_enter = filtered_apps.clone();
        let window_enter = window.clone();
        let terminal_enter = terminal.clone();

        entry.connect_activate(move |_| {
            if let Some(row) = list_box_enter.selected_row() {
                let idx = row.index() as usize;
                let filtered = filtered_enter.borrow();
                if let Some(&app_idx) = filtered.get(idx) {
                    launch_app(&apps_enter[app_idx], &terminal_enter);
                    window_enter.close();
                }
            }
        });
    }

    {
        let list_box_nav = list_box.clone();
        let window_close = window.clone();
        let apps_shortcut = apps.clone();
        let filtered_shortcut = filtered_apps.clone();
        let terminal_shortcut = terminal.clone();

        let scroll_controller =
            gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
        let list_box_scroll = list_box_nav.clone();
        let accumulated_dy = Rc::new(RefCell::new(0.0_f64));
        let accumulated_dy_reset = accumulated_dy.clone();

        scroll_controller.connect_scroll(move |_, _, dy| {
            let mut acc = accumulated_dy.borrow_mut();
            *acc += dy;
            let mut moved = false;

            while *acc >= 1.0 {
                move_selection(&list_box_scroll, 1);
                *acc -= 1.0;
                moved = true;
            }
            while *acc <= -1.0 {
                move_selection(&list_box_scroll, -1);
                *acc += 1.0;
                moved = true;
            }

            if moved {
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
        });

        scroll_controller.connect_scroll_end(move |_| {
            *accumulated_dy_reset.borrow_mut() = 0.0;
        });

        window.add_controller(scroll_controller);

        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            if modifiers.contains(ModifierType::ALT_MASK) {
                let num = match key {
                    gtk4::gdk::Key::_1 => Some(0),
                    gtk4::gdk::Key::_2 => Some(1),
                    gtk4::gdk::Key::_3 => Some(2),
                    gtk4::gdk::Key::_4 => Some(3),
                    gtk4::gdk::Key::_5 => Some(4),
                    gtk4::gdk::Key::_6 => Some(5),
                    gtk4::gdk::Key::_7 => Some(6),
                    gtk4::gdk::Key::_8 => Some(7),
                    gtk4::gdk::Key::_9 => Some(8),
                    _ => None,
                };

                if let Some(idx) = num {
                    let filtered = filtered_shortcut.borrow();
                    if let Some(&app_idx) = filtered.get(idx) {
                        launch_app(&apps_shortcut[app_idx], &terminal_shortcut);
                        window_close.close();
                        return gtk4::glib::Propagation::Stop;
                    }
                }
            }

            match key {
                gtk4::gdk::Key::Escape => {
                    window_close.close();
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Up => {
                    move_selection(&list_box_nav, -1);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Down => {
                    move_selection(&list_box_nav, 1);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Tab => {
                    move_selection(&list_box_nav, 1);
                    gtk4::glib::Propagation::Stop
                }
                _ => gtk4::glib::Propagation::Proceed,
            }
        });

        window.add_controller(key_controller);
    }

    {
        let apps_click = apps.clone();
        let filtered_click = filtered_apps.clone();
        let window_click = window.clone();

        list_box.connect_row_activated(move |_, row| {
            let idx = row.index() as usize;
            let filtered = filtered_click.borrow();
            if let Some(&app_idx) = filtered.get(idx) {
                launch_app(&apps_click[app_idx], &terminal);
                window_click.close();
            }
        });
    }

    entry.grab_focus();
    window.present();
}

fn populate_list(list_box: &ListBox, apps: &[App], indices: &[usize]) {
    while let Some(row) = list_box.row_at_index(0) {
        list_box.remove(&row);
    }

    for (display_idx, &app_idx) in indices.iter().enumerate() {
        let app = &apps[app_idx];
        let shortcut = if display_idx < 9 {
            Some(display_idx + 1)
        } else {
            None
        };
        let row = create_app_row(app, shortcut);
        list_box.append(&row);
    }
}

fn create_app_row(app: &App, shortcut: Option<usize>) -> ListBoxRow {
    let hbox = GtkBox::new(Orientation::Horizontal, 10);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    hbox.add_css_class("yeet-row-content");

    if let Some(icon_name) = &app.icon {
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(36);
        icon.add_css_class("yeet-icon");
        hbox.append(&icon);
    }

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_valign(gtk4::Align::Center);

    let name_label = Label::new(Some(&app.name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("yeet-app-name");
    text_box.append(&name_label);

    if let Some(desc) = &app.description {
        let desc_label = Label::new(Some(desc));
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        desc_label.add_css_class("yeet-app-desc");
        text_box.append(&desc_label);
    }

    hbox.append(&text_box);

    if let Some(num) = shortcut {
        let shortcut_label = Label::new(Some(&format!("Alt+{}", num)));
        shortcut_label.set_valign(gtk4::Align::Center);
        shortcut_label.add_css_class("yeet-shortcut");
        hbox.append(&shortcut_label);
    }

    let row = ListBoxRow::new();
    row.set_child(Some(&hbox));
    row.add_css_class("yeet-row");
    row
}

fn move_selection(list_box: &ListBox, delta: i32) {
    let current = list_box.selected_row().map(|r| r.index()).unwrap_or(-1);
    let new_idx = (current + delta).max(0);
    if let Some(row) = list_box.row_at_index(new_idx) {
        list_box.select_row(Some(&row));
    }
}

fn recency_boost(history: &HashMap<String, u64>, app_name: &str, now: u64) -> i64 {
    history
        .get(app_name)
        .map(|&last| {
            let age_hours = (now.saturating_sub(last)) / 3600;
            match age_hours {
                0..=24 => 100,
                25..=168 => 50,
                _ => 20,
            }
        })
        .unwrap_or(0)
}

fn load_css() {
    let provider = CssProvider::new();

    if let Some(user_path) = Config::user_style_path() {
        if user_path.exists() {
            provider.load_from_path(&user_path);
        } else {
            provider.load_from_data(DEFAULT_STYLE);
        }
    } else {
        provider.load_from_data(DEFAULT_STYLE);
    }

    gtk4::style_context_add_provider_for_display(
        &Display::default().expect("Could not get default display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );
}
