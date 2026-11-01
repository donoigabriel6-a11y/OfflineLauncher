use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::app_dirs::{get_app_data_dir, get_app_config_dir};
use crate::error::{LauncherError, Result};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Data directory override (None = use platform default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,

    /// Maximum concurrent downloads
    #[serde(default = "default_concurrent_downloads")]
    pub concurrent_downloads: usize,

    /// Default memory allocation in MB
    #[serde(default = "default_memory")]
    pub default_memory_mb: u32,

    /// Default Java major version (None = auto-detect from version)
    #[serde(default)]
    pub default_java_version: Option<u8>,

    /// Default Java path override (None = auto-detect)
    #[serde(default)]
    pub default_java_path: Option<PathBuf>,

    /// Default JVM arguments
    #[serde(default = "default_jvm_args")]
    pub default_jvm_args: String,

    /// Language code (ISO 639-1)
    #[serde(default = "default_language")]
    pub language: String,

    /// Theme: "dark", "light", or "system"
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Close launcher when game starts
    #[serde(default)]
    pub close_on_launch: bool,

    /// Show advanced settings
    #[serde(default)]
    pub show_advanced: bool,

    /// Last selected instance ID
    #[serde(default)]
    pub last_instance_id: Option<String>,

    /// Last selected account ID
    #[serde(default)]
    pub last_account_id: Option<String>,

    /// Window state
    #[serde(default)]
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: u32,
    #[serde(default = "default_window_height")]
    pub height: u32,
    #[serde(default)]
    pub maximized: bool,
}

fn default_concurrent_downloads() -> usize { 8 }
fn default_memory() -> u32 { 2048 }
fn default_jvm_args() -> String { "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1".into() }
fn default_language() -> String { "en".into() }
fn default_theme() -> String { "dark".into() }
fn default_window_width() -> u32 { 1200 }
fn default_window_height() -> u32 { 800 }

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: None,
            concurrent_downloads: default_concurrent_downloads(),
            default_memory_mb: default_memory(),
            default_java_version: None,
            default_java_path: None,
            default_jvm_args: default_jvm_args(),
            language: default_language(),
            theme: default_theme(),
            close_on_launch: false,
            show_advanced: false,
            last_instance_id: None,
            last_account_id: None,
            window: WindowConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_window_width(),
            height: default_window_height(),
            maximized: false,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(get_app_config_dir()?.join("config.toml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| LauncherError::Io(e))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| LauncherError::Config(e.to_string()))?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LauncherError::Io(e))?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| LauncherError::Config(e.to_string()))?;
        std::fs::write(&path, content)
            .map_err(|e| LauncherError::Io(e))?;
        Ok(())
    }

    /// Get the effective data directory (respects override)
    pub fn effective_data_dir(&self) -> Result<PathBuf> {
        if let Some(dir) = &self.data_dir {
            Ok(dir.clone())
        } else {
            get_app_data_dir()
        }
    }
}