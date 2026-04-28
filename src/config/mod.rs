pub mod style;

use std::path::PathBuf;

use serde::Deserialize;

pub use style::{DiffStyle, StatusStyle, Style};

// ── TOML file schema ────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(default)]
struct ConfigFile {
    color_scheme: ColorScheme,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

// ── Runtime config ──────────────────────────────────────────────────

pub struct Config {
    pub style: Style,
}

impl Config {
    pub fn load() -> Self {
        let path = home_config_path().join("gitshot.toml");
        let file_cfg = if path.exists() {
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("warning: failed to read {}: {e}", path.display());
                std::process::exit(1);
            });
            toml::from_str(&text).unwrap_or_else(|e| {
                eprintln!("warning: failed to parse {}: {e}", path.display());
                ConfigFile::default()
            })
        } else {
            ConfigFile::default()
        };

        let style = Style::from_scheme(file_cfg.color_scheme);
        Self { style }
    }
}

fn home_config_path() -> PathBuf {
    dirs::home_dir()
        .expect("cannot determine home directory")
        .join(".config/gitshot")
}
