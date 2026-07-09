use crate::config::{Config, CustomApp};
use freedesktop_desktop_entry::{get_languages_from_env, DesktopEntry, Iter as DesktopIter};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
enum LaunchCommand {
    Direct(Vec<String>),
    Shell(String),
}

#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub terminal: bool,
    pub favorite: bool,
    launch: LaunchCommand,
}

impl App {
    fn from_custom(custom: &CustomApp) -> Self {
        Self {
            name: custom.name.clone(),
            icon: custom.icon.clone(),
            description: None,
            keywords: custom.keywords.clone(),
            terminal: false,
            favorite: false,
            launch: LaunchCommand::Shell(custom.exec.clone()),
        }
    }

    /// A bare item with a name only; used in tests.
    #[cfg(test)]
    pub fn plain(name: String) -> Self {
        Self {
            launch: LaunchCommand::Shell(name.clone()),
            name,
            icon: None,
            description: None,
            keywords: Vec::new(),
            terminal: false,
            favorite: false,
        }
    }
}

pub fn discover_apps(config: &Config) -> Vec<App> {
    let exclude_set: HashSet<&str> = config.apps.exclude.iter().map(|s| s.as_str()).collect();

    let all_dirs: Vec<PathBuf> = xdg_application_dirs()
        .into_iter()
        .chain(config.apps.extra_dirs.iter().cloned())
        .collect();

    let locales = get_languages_from_env();
    let mut apps = apps_from_dirs(all_dirs, &exclude_set, &locales);

    for custom in &config.apps.custom {
        apps.push(App::from_custom(custom));
    }

    let favorites_set: HashSet<&str> = config.apps.favorites.iter().map(|s| s.as_str()).collect();
    for app in &mut apps {
        app.favorite = favorites_set.contains(app.name.as_str());
    }

    apps.sort_by_cached_key(|a| (!a.favorite, a.name.to_lowercase()));

    apps
}

fn apps_from_dirs(dirs: Vec<PathBuf>, exclude: &HashSet<&str>, locales: &[String]) -> Vec<App> {
    let mut apps = Vec::new();
    // XDG precedence: a desktop file id seen in an earlier dir shadows later
    // ones entirely, even if the earlier entry is hidden.
    let mut seen: HashSet<OsString> = HashSet::new();

    for path in DesktopIter::new(dirs.into_iter()) {
        let Some(file_name) = path.file_name() else {
            continue;
        };
        if !seen.insert(file_name.to_os_string()) {
            continue;
        }

        if let Ok(entry) = DesktopEntry::from_path(&path, Some(locales)) {
            if entry.no_display() || entry.hidden() {
                continue;
            }

            let Some(name) = entry.name(locales) else {
                continue;
            };
            let exec_args = match entry.parse_exec() {
                Ok(args) if !args.is_empty() => args,
                _ => continue,
            };

            if exclude.contains(name.as_ref()) {
                continue;
            }

            apps.push(App {
                name: name.to_string(),
                icon: entry.icon().map(|s| s.to_string()),
                description: entry.comment(locales).map(|s| s.to_string()),
                keywords: entry
                    .keywords(locales)
                    .map(|kws| kws.into_iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                terminal: entry.terminal(),
                favorite: false,
                launch: LaunchCommand::Direct(exec_args),
            });
        }
    }

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

pub fn launch_app(app: &App, terminal: &str) {
    let command = match &app.launch {
        LaunchCommand::Direct(args) => spawn_direct(args, app.terminal.then_some(terminal)),
        LaunchCommand::Shell(exec) => spawn_shell(exec, app.terminal.then_some(terminal)),
    };

    match command {
        Ok(_) => crate::history::record_launch(&app.name),
        Err(e) => eprintln!("Failed to launch {}: {}", app.name, e),
    }
}

fn spawn_direct(args: &[String], terminal: Option<&str>) -> std::io::Result<std::process::Child> {
    if args.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    }

    let mut cmd = if let Some(terminal) = terminal {
        let mut command = Command::new(terminal);
        command.arg("-e").arg(&args[0]).args(&args[1..]);
        command
    } else {
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        command
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

fn spawn_shell(exec: &str, terminal: Option<&str>) -> std::io::Result<std::process::Child> {
    let mut cmd = if let Some(terminal) = terminal {
        let mut command = Command::new(terminal);
        command.arg("-e").arg("sh").arg("-c").arg(exec);
        command
    } else {
        let mut command = Command::new("sh");
        command.arg("-c").arg(exec);
        command
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn custom_app_uses_shell_launch_mode() {
        let custom = CustomApp {
            name: "My Script".to_string(),
            exec: "echo hello".to_string(),
            icon: None,
            keywords: Vec::new(),
        };

        let app = App::from_custom(&custom);
        match app.launch {
            LaunchCommand::Shell(cmd) => assert_eq!(cmd, "echo hello"),
            LaunchCommand::Direct(_) => panic!("custom entries must use shell launch mode"),
        }
    }

    #[test]
    fn spawn_direct_rejects_empty_command() {
        let err = spawn_direct(&[], None).expect_err("empty command must fail");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    fn write_desktop_file(dir: &std::path::Path, file: &str, name: &str) {
        fs::write(
            dir.join(file),
            format!("[Desktop Entry]\nType=Application\nName={name}\nExec=true\n"),
        )
        .unwrap();
    }

    #[test]
    fn duplicate_desktop_ids_prefer_earlier_dirs() {
        let base = std::env::temp_dir().join("yeet_test_dedup");
        let _ = fs::remove_dir_all(&base);
        let local = base.join("local");
        let system = base.join("system");
        fs::create_dir_all(&local).unwrap();
        fs::create_dir_all(&system).unwrap();

        write_desktop_file(&local, "firefox.desktop", "Firefox Local");
        write_desktop_file(&system, "firefox.desktop", "Firefox System");
        write_desktop_file(&system, "kitty.desktop", "Kitty");

        let apps = apps_from_dirs(vec![local, system], &HashSet::new(), &[]);

        let names: Vec<&str> = apps.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"Firefox Local"));
        assert!(!names.contains(&"Firefox System"));
        assert!(names.contains(&"Kitty"));
        assert_eq!(apps.len(), 2);

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn hidden_local_entry_shadows_system_entry() {
        let base = std::env::temp_dir().join("yeet_test_shadow");
        let _ = fs::remove_dir_all(&base);
        let local = base.join("local");
        let system = base.join("system");
        fs::create_dir_all(&local).unwrap();
        fs::create_dir_all(&system).unwrap();

        fs::write(
            local.join("htop.desktop"),
            "[Desktop Entry]\nType=Application\nName=Htop\nExec=htop\nNoDisplay=true\n",
        )
        .unwrap();
        write_desktop_file(&system, "htop.desktop", "Htop");

        let apps = apps_from_dirs(vec![local, system], &HashSet::new(), &[]);
        assert!(apps.is_empty());

        let _ = fs::remove_dir_all(&base);
    }
}
