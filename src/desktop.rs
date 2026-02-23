use crate::config::{Config, CustomApp};
use freedesktop_desktop_entry::{DesktopEntry, Iter as DesktopIter};
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
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub terminal: bool,
    launch: LaunchCommand,
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
            launch: LaunchCommand::Shell(custom.exec.clone()),
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

            let Some(name) = entry.name(&["en"]) else {
                continue;
            };
            let exec_args = match entry.parse_exec() {
                Ok(args) if !args.is_empty() => args,
                _ => continue,
            };

            if exclude_set.contains(name.as_ref()) {
                continue;
            }

            let app = App {
                name: name.to_string(),
                exec: exec_args.join(" "),
                icon: entry.icon().map(|s| s.to_string()),
                description: entry.comment(&["en"]).map(|s| s.to_string()),
                keywords: entry
                    .keywords(&["en"])
                    .map(|kws| kws.into_iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                terminal: entry.terminal(),
                launch: LaunchCommand::Direct(exec_args),
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
}
