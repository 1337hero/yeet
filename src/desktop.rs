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

            let app = App {
                name: name.to_string(),
                exec: exec.to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchCommandError {
    ExecParseFailed,
    EmptyExec,
    TerminalParseFailed,
    EmptyTerminal,
}

impl std::fmt::Display for LaunchCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ExecParseFailed => "failed to parse desktop Exec",
            Self::EmptyExec => "desktop Exec is empty after cleaning",
            Self::TerminalParseFailed => "failed to parse terminal command",
            Self::EmptyTerminal => "terminal command is empty",
        };
        write!(f, "{message}")
    }
}

fn clean_desktop_exec_arg(arg: &str) -> String {
    const FIELD_CODES: &[char] = &[
        'f', 'F', 'u', 'U', 'd', 'D', 'n', 'N', 'i', 'c', 'k', 'v', 'm',
    ];

    let mut result = String::with_capacity(arg.len());
    let mut chars = arg.chars();

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

    result
}

fn parse_desktop_exec(exec: &str) -> Result<(String, Vec<String>), LaunchCommandError> {
    let Some(args) = shlex::split(exec) else {
        return Err(LaunchCommandError::ExecParseFailed);
    };

    let mut cleaned = Vec::with_capacity(args.len());
    for arg in args {
        let cleaned_arg = clean_desktop_exec_arg(&arg);
        if !cleaned_arg.is_empty() {
            cleaned.push(cleaned_arg);
        }
    }

    if cleaned.is_empty() {
        return Err(LaunchCommandError::EmptyExec);
    }

    let mut iter = cleaned.into_iter();
    let program = iter.next().ok_or(LaunchCommandError::EmptyExec)?;
    let arguments = iter.collect();
    Ok((program, arguments))
}

fn parse_command(command: &str) -> Result<(String, Vec<String>), LaunchCommandError> {
    let Some(args) = shlex::split(command) else {
        return Err(LaunchCommandError::TerminalParseFailed);
    };

    if args.is_empty() {
        return Err(LaunchCommandError::EmptyTerminal);
    }

    let mut iter = args.into_iter();
    let program = iter.next().ok_or(LaunchCommandError::EmptyTerminal)?;
    let arguments = iter.collect();
    Ok((program, arguments))
}

fn build_launch_command(
    app: &App,
    terminal: &str,
) -> Result<(String, Vec<String>), LaunchCommandError> {
    let (program, args) = parse_desktop_exec(&app.exec)?;

    if !app.terminal {
        return Ok((program, args));
    }

    let (terminal_program, mut terminal_args) = parse_command(terminal)?;
    terminal_args.push(String::from("-e"));
    terminal_args.push(program);
    terminal_args.extend(args);
    Ok((terminal_program, terminal_args))
}

pub fn launch_app(app: &App, terminal: &str) {
    let (final_program, final_args) = match build_launch_command(app, terminal) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Failed to build launch command for {}: {}", app.name, e);
            return;
        }
    };

    if let Err(e) = std::process::Command::new(&final_program)
        .args(&final_args)
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
    fn clean_desktop_exec_arg_removes_field_codes() {
        assert_eq!(clean_desktop_exec_arg("%u"), "");
        assert_eq!(clean_desktop_exec_arg("%F"), "");
        assert_eq!(clean_desktop_exec_arg("--opt=%k"), "--opt=");
    }

    #[test]
    fn clean_desktop_exec_arg_preserves_escaped_percent() {
        assert_eq!(clean_desktop_exec_arg("100%%"), "100%");
        assert_eq!(clean_desktop_exec_arg("--format=%%d"), "--format=%d");
    }

    #[test]
    fn clean_desktop_exec_arg_preserves_unknown_percent_codes() {
        assert_eq!(clean_desktop_exec_arg("--ratio=50%x"), "--ratio=50%x");
        assert_eq!(clean_desktop_exec_arg("%z"), "%z");
    }

    #[test]
    fn parse_desktop_exec_simple_command() {
        let result = parse_desktop_exec("firefox").ok();
        assert_eq!(result, Some(("firefox".to_string(), vec![])));
    }

    #[test]
    fn parse_desktop_exec_with_arguments() {
        let result = parse_desktop_exec("firefox --private-window").ok();
        assert_eq!(
            result,
            Some(("firefox".to_string(), vec!["--private-window".to_string()]))
        );
    }

    #[test]
    fn parse_desktop_exec_with_quoted_args() {
        let result = parse_desktop_exec(r#"app "arg with spaces" --flag"#).ok();
        assert_eq!(
            result,
            Some((
                "app".to_string(),
                vec!["arg with spaces".to_string(), "--flag".to_string()]
            ))
        );
    }

    #[test]
    fn parse_desktop_exec_removes_field_code_arguments() {
        let result = parse_desktop_exec("firefox %u").ok();
        assert_eq!(result, Some(("firefox".to_string(), vec![])));

        let result = parse_desktop_exec(r#"firefox "%u""#).ok();
        assert_eq!(result, Some(("firefox".to_string(), vec![])));

        let result = parse_desktop_exec("code %F --new-window").ok();
        assert_eq!(
            result,
            Some(("code".to_string(), vec!["--new-window".to_string()]))
        );
    }

    #[test]
    fn parse_desktop_exec_shell_metacharacters_not_interpreted() {
        // These shell metacharacters should be treated as literal strings
        // not as shell operators - this is the security fix
        let result = parse_desktop_exec("firefox; rm -rf ~").ok();
        assert_eq!(
            result,
            Some((
                "firefox;".to_string(),
                vec!["rm".to_string(), "-rf".to_string(), "~".to_string()]
            ))
        );

        let result = parse_desktop_exec("app | cat").ok();
        assert_eq!(
            result,
            Some(("app".to_string(), vec!["|".to_string(), "cat".to_string()]))
        );

        let result = parse_desktop_exec("app && malicious").ok();
        assert_eq!(
            result,
            Some((
                "app".to_string(),
                vec!["&&".to_string(), "malicious".to_string()]
            ))
        );
    }

    #[test]
    fn parse_desktop_exec_empty_returns_error() {
        assert_eq!(parse_desktop_exec(""), Err(LaunchCommandError::EmptyExec));
        assert_eq!(
            parse_desktop_exec("   "),
            Err(LaunchCommandError::EmptyExec)
        );
    }

    #[test]
    fn parse_desktop_exec_absolute_path() {
        let result = parse_desktop_exec("/usr/bin/app --config /etc/app.conf").ok();
        assert_eq!(
            result,
            Some((
                "/usr/bin/app".to_string(),
                vec!["--config".to_string(), "/etc/app.conf".to_string()]
            ))
        );
    }

    #[test]
    fn build_launch_command_wraps_terminal_with_args() {
        let app = App {
            name: "Htop".to_string(),
            exec: "htop".to_string(),
            icon: None,
            description: None,
            keywords: vec![],
            terminal: true,
        };

        let (program, args) = build_launch_command(&app, "kitty --single-instance").unwrap();
        assert_eq!(program, "kitty");
        assert_eq!(
            args,
            vec![
                "--single-instance".to_string(),
                "-e".to_string(),
                "htop".to_string()
            ]
        );
    }
}
