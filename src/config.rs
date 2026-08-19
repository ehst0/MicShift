use crate::win32::{MOD_CONTROL, VK_F4, VK_F5};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MicSelection {
    pub device_id: String,
    pub device_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub modifiers: u32,
    pub vk: u32,
    pub display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub mic1: Option<MicSelection>,
    pub mic2: Option<MicSelection>,
    pub mic1_hotkey: HotkeyConfig,
    pub mic2_hotkey: HotkeyConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mic1: None,
            mic2: None,
            mic1_hotkey: HotkeyConfig {
                modifiers: MOD_CONTROL,
                vk: VK_F4,
                display: "Ctrl+F4".to_string(),
            },
            mic2_hotkey: HotkeyConfig {
                modifiers: MOD_CONTROL,
                vk: VK_F5,
                display: "Ctrl+F5".to_string(),
            },
        }
    }
}

pub fn config_dir() -> Result<PathBuf, String> {
    let appdata =
        std::env::var_os("APPDATA").ok_or_else(|| "APPDATA is not available".to_string())?;
    Ok(PathBuf::from(appdata).join("MicShift"))
}

fn legacy_config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("DefaultMicSwitcher").join("config.json"))
}

pub fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load_config() -> Result<AppConfig, String> {
    let current_path = config_path()?;
    let path = if current_path.exists() {
        current_path
    } else {
        legacy_config_path()
            .filter(|path| path.exists())
            .unwrap_or(current_path)
    };
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let text =
        fs::read_to_string(&path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("Could not parse {}: {e}", path.display()))
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Could not create {}: {e}", dir.display()))?;

    let path = dir.join("config.json");
    let text = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Could not serialize config: {e}"))?;

    fs::write(&path, text).map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    Ok(())
}

pub fn config_modified_time() -> Option<SystemTime> {
    let current_path = config_path().ok()?;
    let path = if current_path.exists() {
        current_path
    } else {
        legacy_config_path().filter(|path| path.exists())?
    };
    fs::metadata(path).ok()?.modified().ok()
}

pub fn path_modified_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}
