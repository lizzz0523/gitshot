pub mod style;

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use serde::Deserialize;

pub use style::{DiffStyle, StatusStyle, Style};

#[derive(Deserialize)]
#[serde(default)]
pub(crate) struct ConfigFile {
    pub color_scheme: ColorScheme,
    pub font_path: String,
    pub font_size: f32,
    pub line_height: f32,
    pub img_padding: f32,
    pub max_img_width: u32,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            font_path: "/System/Library/Fonts/Monaco.ttf".to_string(),
            font_size: 13.0,
            line_height: 20.0,
            img_padding: 16.0,
            max_img_width: 1800,
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

pub struct Config {
    pub style: Style,
}

impl Config {
    pub fn load() -> Result<Self> {
        let file_cfg = load_config_file().unwrap_or_default();
        Ok(Self {
            style: Style::from_config(&file_cfg),
        })
    }
}

// 配置缺失或损坏都走默认值，只以 warning 提示 —— 不能因为用户 home 下的
// 配置出问题就中断 CLI 主流程
fn load_config_file() -> Option<ConfigFile> {
    let path = home_config_path().ok()?.join("gitshot.toml");
    if !path.exists() {
        return None;
    }

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("warning: failed to read {}: {e}", path.display());
            return None;
        }
    };

    match toml::from_str(&text) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            eprintln!("warning: failed to parse {}: {e}", path.display());
            None
        }
    }
}

fn home_config_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".config/gitshot"))
        .ok_or_else(|| anyhow!("cannot determine home directory"))
}
