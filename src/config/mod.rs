pub mod style;

use std::path::PathBuf;

use serde::Deserialize;

pub use style::{DiffStyle, StatusStyle, Style};

// ── TOML file schema ────────────────────────────────────────────────

fn default_font_path() -> String {
    "/System/Library/Fonts/Monaco.ttf".to_string()
}

const fn default_font_size() -> f32 {
    13.0
}

const fn default_line_height() -> f32 {
    20.0
}

const fn default_padding() -> f32 {
    16.0
}

const fn default_max_img_width() -> u32 {
    1800
}

#[derive(Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(default)]
    pub color_scheme: ColorScheme,
    #[serde(default = "default_font_path")]
    pub font_path: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,
    #[serde(default = "default_padding")]
    pub padding: f32,
    #[serde(default = "default_max_img_width")]
    pub max_img_width: u32,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            font_path: default_font_path(),
            font_size: default_font_size(),
            line_height: default_line_height(),
            padding: default_padding(),
            max_img_width: default_max_img_width(),
        }
    }
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

        let style = Style::from_config(&file_cfg);
        Self { style }
    }
}

fn home_config_path() -> PathBuf {
    dirs::home_dir()
        .expect("cannot determine home directory")
        .join(".config/gitshot")
}
