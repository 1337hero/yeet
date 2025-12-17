use serde::Deserialize;
use std::path::PathBuf;

const DEFAULT_CONFIG: &str = include_str!("../defaults/config.toml");

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub apps: AppsConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    #[serde(default)]
    pub monitor: u32,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_initial_results")]
    pub initial_results: usize,
    #[serde(default = "default_terminal")]
    pub terminal: String,
}

#[derive(Debug, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_width")]
    pub width: i32,
    #[serde(default = "default_anchor_top")]
    pub anchor_top: i32,
}

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_min_score")]
    pub min_score: i64,
    #[serde(default = "default_score_threshold")]
    pub score_threshold: f64,
    #[serde(default = "default_true")]
    pub prefer_prefix: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct AppsConfig {
    #[serde(default)]
    pub extra_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub custom: Vec<CustomApp>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomApp {
    pub name: String,
    pub exec: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

fn default_max_results() -> usize {
    8
}
fn default_initial_results() -> usize {
    8
}
fn default_terminal() -> String {
    "alacritty".into()
}
fn default_width() -> i32 {
    500
}
fn default_anchor_top() -> i32 {
    200
}
fn default_min_score() -> i64 {
    30
}
fn default_score_threshold() -> f64 {
    0.6
}
fn default_true() -> bool {
    true
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            monitor: 0,
            max_results: default_max_results(),
            initial_results: default_initial_results(),
            terminal: default_terminal(),
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            anchor_top: default_anchor_top(),
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            min_score: default_min_score(),
            score_threshold: default_score_threshold(),
            prefer_prefix: default_true(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config: Config =
            toml::from_str(DEFAULT_CONFIG).expect("embedded default config should be valid");

        if let Some(user_config_path) = Self::user_config_path() {
            if user_config_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&user_config_path) {
                    match toml::from_str::<Config>(&contents) {
                        Ok(user_config) => config.merge(user_config),
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to parse config at {}",
                                user_config_path.display()
                            );
                            eprintln!("  {e}");
                            eprintln!("  Using default configuration.");
                        }
                    }
                }
            }
        }

        config
    }

    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("yeet"))
    }

    pub fn user_config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("config.toml"))
    }

    pub fn user_style_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("style.css"))
    }

    fn merge(&mut self, user: Config) {
        self.general = user.general;
        self.appearance = user.appearance;
        self.search = user.search;

        if !user.apps.extra_dirs.is_empty() {
            self.apps.extra_dirs = user.apps.extra_dirs;
        }
        if !user.apps.exclude.is_empty() {
            self.apps.exclude = user.apps.exclude;
        }
        if !user.apps.favorites.is_empty() {
            self.apps.favorites = user.apps.favorites;
        }
        if !user.apps.custom.is_empty() {
            self.apps.custom.extend(user.apps.custom);
        }
    }

    #[cfg(test)]
    fn from_toml(toml: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_default_config_is_valid() {
        let config: Config =
            toml::from_str(DEFAULT_CONFIG).expect("embedded default config should parse");

        assert_eq!(config.general.max_results, 8);
        assert_eq!(config.general.terminal, "alacritty");
        assert_eq!(config.appearance.width, 500);
    }

    #[test]
    fn parses_user_config_with_overrides() {
        let user_toml = r#"
            [general]
            max_results = 12
            terminal = "kitty"

            [appearance]
            width = 600
        "#;

        let config = Config::from_toml(user_toml).unwrap();
        assert_eq!(config.general.max_results, 12);
        assert_eq!(config.general.terminal, "kitty");
        assert_eq!(config.appearance.width, 600);
    }

    #[test]
    fn parses_partial_config_with_defaults() {
        let user_toml = r#"
            [general]
            terminal = "wezterm"
        "#;

        let config = Config::from_toml(user_toml).unwrap();
        assert_eq!(config.general.terminal, "wezterm");
        assert_eq!(config.general.max_results, 8);
        assert_eq!(config.appearance.width, 500);
    }

    #[test]
    fn parses_custom_apps() {
        let user_toml = r#"
            [[apps.custom]]
            name = "My Script"
            exec = "/home/user/scripts/my-script.sh"
            icon = "utilities-terminal"
            keywords = ["script", "custom"]
        "#;

        let config = Config::from_toml(user_toml).unwrap();
        assert_eq!(config.apps.custom.len(), 1);
        assert_eq!(config.apps.custom[0].name, "My Script");
        assert_eq!(
            config.apps.custom[0].exec,
            "/home/user/scripts/my-script.sh"
        );
        assert_eq!(
            config.apps.custom[0].icon,
            Some("utilities-terminal".into())
        );
        assert_eq!(config.apps.custom[0].keywords, vec!["script", "custom"]);
    }

    #[test]
    fn parses_favorites_and_excludes() {
        let user_toml = r#"
            [apps]
            favorites = ["Firefox", "Alacritty"]
            exclude = ["htop.desktop", "nvtop.desktop"]
        "#;

        let config = Config::from_toml(user_toml).unwrap();
        assert_eq!(config.apps.favorites, vec!["Firefox", "Alacritty"]);
        assert_eq!(config.apps.exclude, vec!["htop.desktop", "nvtop.desktop"]);
    }

    #[test]
    fn rejects_invalid_toml() {
        let bad_toml = r#"
            [general
            max_results = "not a number"
        "#;

        assert!(Config::from_toml(bad_toml).is_err());
    }

    #[test]
    fn rejects_wrong_types() {
        let bad_toml = r#"
            [general]
            max_results = "eight"
        "#;

        assert!(Config::from_toml(bad_toml).is_err());
    }
}
