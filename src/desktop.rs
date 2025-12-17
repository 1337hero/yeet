use crate::config::{Config, CustomApp};
use freedesktop_desktop_entry::{DesktopEntry, Iter as DesktopIter};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub terminal: bool,
}

impl App {
    fn from_custom(custom: &CustomApp) -> Self {
        Self {
            name: custom.name.clone(),
            exec: custom.exec.clone(),
            icon: custom.icon.clone(),
            description: None,
            keywords: custom.keywords.clone(),
            terminal: false,
        }
    }

    pub fn search_text(&self) -> String {
        let mut text = self.name.clone();
        if let Some(desc) = &self.description {
            text.push(' ');
            text.push_str(desc);
        }
        for kw in &self.keywords {
            text.push(' ');
            text.push_str(kw);
        }
        text
    }
}

pub fn discover_apps(config: &Config) -> Vec<App> {
    let mut apps = Vec::new();
    let exclude_set: std::collections::HashSet<&str> =
        config.apps.exclude.iter().map(|s| s.as_str()).collect();

    let all_dirs: Vec<PathBuf> = xdg_application_dirs()
        .into_iter()
        .chain(config.apps.extra_dirs.iter().cloned())
        .collect();

    for path in DesktopIter::new(all_dirs.into_iter()) {
        if let Ok(entry) = DesktopEntry::from_path(&path, Some(&["en"])) {
            if entry.no_display() || entry.hidden() {
                continue;
            }

            let Some(exec) = entry.exec() else { continue };
            let Some(name) = entry.name(&["en"]) else {
                continue;
            };

            // Exclude by display name (consistent with favorites)
            if exclude_set.contains(name.as_ref()) {
                continue;
            }

            let exec_clean = clean_exec(exec);

            let app = App {
                name: name.to_string(),
                exec: exec_clean,
                icon: entry.icon().map(|s| s.to_string()),
                description: entry.comment(&["en"]).map(|s| s.to_string()),
                keywords: entry
                    .keywords(&["en"])
                    .map(|kws| kws.into_iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                terminal: entry.terminal(),
            };

            apps.push(app);
        }
    }

    for custom in &config.apps.custom {
        apps.push(App::from_custom(custom));
    }

    let favorites_set: std::collections::HashSet<&str> =
        config.apps.favorites.iter().map(|s| s.as_str()).collect();

    apps.sort_by(|a, b| {
        let a_fav = favorites_set.contains(a.name.as_str());
        let b_fav = favorites_set.contains(b.name.as_str());
        match (a_fav, b_fav) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    apps
}

fn xdg_application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = dirs::data_local_dir() {
        dirs.push(data_home.join("applications"));
    }

    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));

    if let Some(data_home) = dirs::data_local_dir() {
        dirs.push(data_home.join("flatpak/exports/share/applications"));
    }
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

    dirs
}

fn clean_exec(exec: &str) -> String {
    const FIELD_CODES: &[char] = &[
        'f', 'F', 'u', 'U', 'd', 'D', 'n', 'N', 'i', 'c', 'k', 'v', 'm',
    ];

    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(next) = chars.next() {
                if next == '%' {
                    result.push('%');
                } else if !FIELD_CODES.contains(&next) {
                    result.push('%');
                    result.push(next);
                }
            }
        } else {
            result.push(c);
        }
    }

    result.trim().to_string()
}

pub fn launch_app(app: &App, terminal: &str) {
    let exec = if app.terminal {
        format!("{} -e {}", terminal, app.exec)
    } else {
        app.exec.clone()
    };

    if let Err(e) = std::process::Command::new("sh")
        .arg("-c")
        .arg(&exec)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        eprintln!("Failed to launch {}: {}", app.name, e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_exec_preserves_plain_commands() {
        assert_eq!(clean_exec("firefox"), "firefox");
        assert_eq!(clean_exec("/usr/bin/app --flag"), "/usr/bin/app --flag");
    }

    #[test]
    fn clean_exec_removes_field_codes() {
        assert_eq!(clean_exec("firefox %u"), "firefox");
        assert_eq!(clean_exec("app %U"), "app");
        assert_eq!(clean_exec("app %f %F"), "app");
        // Note: leaves space where field code was (shell handles this fine)
        assert_eq!(clean_exec("code %F --new-window"), "code  --new-window");
    }

    #[test]
    fn clean_exec_preserves_escaped_percent() {
        assert_eq!(clean_exec("echo 100%%"), "echo 100%");
        assert_eq!(clean_exec("app --format=%%d"), "app --format=%d");
    }

    #[test]
    fn clean_exec_preserves_non_field_code_percent() {
        assert_eq!(clean_exec("app --ratio=50%x"), "app --ratio=50%x");
        assert_eq!(clean_exec("echo %z"), "echo %z");
    }

    #[test]
    fn clean_exec_handles_trailing_percent() {
        assert_eq!(clean_exec("app %"), "app");
    }
}
