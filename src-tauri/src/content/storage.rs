//! Persistent storage for installed content and the CurseForge API key.

use crate::app_dirs::{get_app_config_dir, get_app_data_dir};
use crate::content::types::InstalledContent;
use crate::error::{LauncherError, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const INSTALLATIONS_FILE: &str = "content-installations.json";
const CURSEFORGE_KEY_FILE: &str = "curseforge-api-key";

/// Serialized snapshot of all installations, keyed by instance id.
#[derive(Debug, Clone, Serialize, Default)]
struct InstallationsSnapshot(HashMap<String, Vec<InstalledContent>>);

pub struct ContentStorage {
    installations: Arc<RwLock<HashMap<String, Vec<InstalledContent>>>>,
    path: PathBuf,
    curseforge_key_path: PathBuf,
}

impl ContentStorage {
    pub fn new() -> Result<Self> {
        let data_dir = get_app_data_dir()?;
        let config_dir = get_app_config_dir()?;
        Ok(Self {
            installations: Arc::new(RwLock::new(HashMap::new())),
            path: data_dir.join(INSTALLATIONS_FILE),
            curseforge_key_path: config_dir.join(CURSEFORGE_KEY_FILE),
        })
    }

    pub async fn load(&self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(LauncherError::Io)?;
        let snapshot: InstallationsSnapshot =
            serde_json::from_str(&content).map_err(LauncherError::Serde)?;
        *self.installations.write().await = snapshot.0;
        Ok(())
    }

    async fn save_installations(&self, snapshot: &InstallationsSnapshot) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(LauncherError::Io)?;
        }
        let content = serde_json::to_string_pretty(snapshot).map_err(LauncherError::Serde)?;
        tokio::fs::write(&self.path, content)
            .await
            .map_err(LauncherError::Io)?;
        Ok(())
    }

    /// List installed content for an instance.
    pub async fn list(&self, instance_id: &str) -> Vec<InstalledContent> {
        let installations = self.installations.read().await;
        installations.get(instance_id).cloned().unwrap_or_default()
    }

    /// Record a newly installed piece of content for an instance.
    pub async fn add(&self, instance_id: String, content: InstalledContent) -> Result<()> {
        let snapshot = {
            let mut installations = self.installations.write().await;
            installations.entry(instance_id).or_default().push(content);
            InstallationsSnapshot(installations.clone())
        };
        self.save_installations(&snapshot).await
    }

    /// Remove an installed content entry by install id for an instance.
    pub async fn remove(&self, instance_id: &str, install_id: &str) -> Result<()> {
        let snapshot = {
            let mut installations = self.installations.write().await;
            if let Some(entries) = installations.get_mut(instance_id) {
                entries.retain(|e| e.install_id != install_id);
            }
            InstallationsSnapshot(installations.clone())
        };
        self.save_installations(&snapshot).await
    }

    /// Does a CurseForge API key exist on disk?
    pub async fn curseforge_key_status(&self) -> Result<bool> {
        Ok(self.curseforge_key_path.exists())
    }

    /// Store the CurseForge API key (plaintext on disk, local only).
    pub async fn curseforge_key_set(&self, api_key: &str) -> Result<()> {
        if let Some(parent) = self.curseforge_key_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(LauncherError::Io)?;
        }
        tokio::fs::write(&self.curseforge_key_path, api_key)
            .await
            .map_err(LauncherError::Io)?;
        Ok(())
    }

    /// Remove the stored CurseForge API key.
    pub async fn curseforge_key_clear(&self) -> Result<()> {
        if self.curseforge_key_path.exists() {
            tokio::fs::remove_file(&self.curseforge_key_path)
                .await
                .map_err(LauncherError::Io)?;
        }
        Ok(())
    }

    /// Read the stored CurseForge API key, if any. Never logs or echoes the value.
    pub async fn curseforge_key_get(&self) -> Result<Option<String>> {
        if !self.curseforge_key_path.exists() {
            return Ok(None);
        }
        let content = tokio::fs::read_to_string(&self.curseforge_key_path)
            .await
            .map_err(LauncherError::Io)?;
        Ok(Some(content))
    }
}

impl Default for ContentStorage {
    fn default() -> Self {
        Self::new().expect("content storage should initialize")
    }
}
