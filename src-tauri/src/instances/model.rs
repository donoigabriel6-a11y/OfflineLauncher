//! Instance (profile) data model

use crate::app_dirs::get_instance_game_dir;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Mod loader type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    None,
    Fabric,
    Quilt,
    // Forge,
    // NeoForge,
}

impl Default for ModLoader {
    fn default() -> Self {
        ModLoader::None
    }
}

impl ModLoader {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModLoader::None => "vanilla",
            ModLoader::Fabric => "fabric",
            ModLoader::Quilt => "quilt",
        }
    }
}

/// Memory settings for JVM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettings {
    #[serde(default = "default_min_memory")]
    pub min_mb: u32,
    #[serde(default = "default_max_memory")]
    pub max_mb: u32,
}

fn default_min_memory() -> u32 { 512 }
fn default_max_memory() -> u32 { 2048 }

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            min_mb: default_min_memory(),
            max_mb: default_max_memory(),
        }
    }
}

/// Java settings per instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaSettings {
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub major_version: Option<u8>,
    #[serde(default)]
    pub jvm_args: String,
}

impl Default for JavaSettings {
    fn default() -> Self {
        Self {
            path: None,
            major_version: None,
            jvm_args: String::new(),
        }
    }
}

/// Window resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub fullscreen: bool,
}

/// Minecraft instance (profile)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>, // Base64 encoded image or path
    pub version_id: String,
    #[serde(default)]
    pub loader: ModLoader,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default = "default_game_dir")]
    pub game_dir: PathBuf,
    #[serde(default)]
    pub java: JavaSettings,
    #[serde(default)]
    pub memory: MemorySettings,
    #[serde(default)]
    pub jvm_args: String,
    #[serde(default)]
    pub game_args: String,
    #[serde(default)]
    pub resolution: Option<Resolution>,
    #[serde(default)]
    pub last_played: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

fn default_game_dir() -> PathBuf {
    // This will be overridden with instance-specific path
    PathBuf::from(".")
}

impl Instance {
    /// Create a new instance
    pub fn new(name: String, version_id: String) -> Result<Self> {
        let id = Uuid::new_v4();
        let game_dir = get_instance_game_dir(&id.to_string())?;
        let now = Utc::now();

        Ok(Self {
            id,
            name,
            icon: None,
            version_id,
            loader: ModLoader::None,
            loader_version: None,
            game_dir,
            java: JavaSettings::default(),
            memory: MemorySettings::default(),
            jvm_args: String::new(),
            game_args: String::new(),
            resolution: None,
            last_played: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Get the effective game directory
    pub fn effective_game_dir(&self) -> PathBuf {
        self.game_dir.clone()
    }

    /// Update last played timestamp
    pub fn touch_last_played(&mut self) {
        self.last_played = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_creation() {
        let instance = Instance::new("Test Instance".into(), "1.20.1".into()).unwrap();
        assert_eq!(instance.name, "Test Instance");
        assert_eq!(instance.version_id, "1.20.1");
        assert_eq!(instance.loader, ModLoader::None);
    }

    #[test]
    fn test_memory_settings_default() {
        let mem = MemorySettings::default();
        assert_eq!(mem.min_mb, 512);
        assert_eq!(mem.max_mb, 2048);
    }
}