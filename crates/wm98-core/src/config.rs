use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub theme: ThemeConfig,
    pub gaps: GapsConfig,
    pub keybinds: Vec<Keybind>,
    pub autostart: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeConfig {
    /// Which named theme to load (e.g. "bubble98", "aqua2k")
    pub name: String,
    pub border_width: u32,
    pub titlebar_height: u32,
    pub corner_radius: u32,
    pub shadow_blur: u32,
    pub button_size: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GapsConfig {
    pub inner: u32,
    pub outer: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Keybind {
    pub modifiers: Vec<String>,
    pub key: String,
    /// Space-separated action, e.g. "spawn alacritty" or "close_window"
    pub action: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            gaps: GapsConfig { inner: 8, outer: 16 },
            keybinds: vec![
                Keybind {
                    modifiers: vec!["super".into()],
                    key: "Return".into(),
                    action: "spawn alacritty".into(),
                },
                Keybind {
                    modifiers: vec!["super".into()],
                    key: "q".into(),
                    action: "close_window".into(),
                },
                Keybind {
                    modifiers: vec!["super".into(), "shift".into()],
                    key: "q".into(),
                    action: "quit".into(),
                },
            ],
            autostart: vec![],
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "bubble98".into(),
            border_width: 3,
            titlebar_height: 36,
            corner_radius: 12,
            shadow_blur: 20,
            button_size: 18,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&raw)?)
        } else {
            log::info!("No config at {:?}, using defaults", path);
            Ok(Self::default())
        }
    }

    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("98wm").join("config.toml")
    }

    pub fn save_default() -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let toml = toml::to_string_pretty(&Config::default())?;
        std::fs::write(&path, toml)?;
        Ok(())
    }
}
