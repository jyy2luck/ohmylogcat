use crate::ui::{LanguagePreference, ThemePreference};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub adb_path: Option<String>,
    pub buffer_capacity: usize,
    #[serde(default = "default_true")]
    pub auto_scroll_to_end: bool,
    #[serde(default)]
    pub soft_wrap: bool,
    #[serde(default)]
    pub theme: ThemePreference,
    #[serde(default)]
    pub language: LanguagePreference,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            adb_path: None,
            buffer_capacity: 200_000,
            auto_scroll_to_end: true,
            soft_wrap: false,
            theme: ThemePreference::default(),
            language: LanguagePreference::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferStats {
    pub lines_per_sec: f64,
    pub memory_estimate_mb: f64,
    pub count: usize,
    pub capacity: usize,
}

impl Default for BufferStats {
    fn default() -> Self {
        Self {
            lines_per_sec: 0.0,
            memory_estimate_mb: 0.0,
            count: 0,
            capacity: 200_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferPreset {
    Light,
    Normal,
    Heavy,
    Marathon,
    Custom,
}

impl BufferPreset {
    pub const ALL: [BufferPreset; 5] = [
        BufferPreset::Light,
        BufferPreset::Normal,
        BufferPreset::Heavy,
        BufferPreset::Marathon,
        BufferPreset::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BufferPreset::Light => "Light (50k)",
            BufferPreset::Normal => "Normal (200k)",
            BufferPreset::Heavy => "Heavy (500k)",
            BufferPreset::Marathon => "Marathon (1M)",
            BufferPreset::Custom => "Custom",
        }
    }

    pub fn capacity(self) -> Option<usize> {
        match self {
            BufferPreset::Light => Some(50_000),
            BufferPreset::Normal => Some(200_000),
            BufferPreset::Heavy => Some(500_000),
            BufferPreset::Marathon => Some(1_000_000),
            BufferPreset::Custom => None,
        }
    }

    pub fn from_capacity(cap: usize) -> Self {
        for preset in Self::ALL {
            if preset.capacity() == Some(cap) {
                return preset;
            }
        }
        BufferPreset::Custom
    }
}

pub fn config_path() -> Result<PathBuf, String> {
    let dir = dirs::config_dir().ok_or_else(|| "Could not resolve config directory".to_string())?;
    Ok(dir.join("ohmylogcat").join("settings.json"))
}

pub fn load_settings() -> Settings {
    let Ok(path) = config_path() else {
        return Settings::default();
    };
    if !path.exists() {
        return Settings::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}
