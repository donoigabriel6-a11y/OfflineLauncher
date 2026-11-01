//! Instance storage and persistence

use crate::app_dirs::get_instances_dir;
use crate::error::{LauncherError, Result};
use crate::instances::model::Instance;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct InstanceStorage {
    instances: Arc<RwLock<HashMap<Uuid, Instance>>>,
}

impl InstanceStorage {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load instances from disk
    pub async fn load(&self) -> Result<()> {
        let instances_dir = get_instances_dir()?;
        if !instances_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&instances_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let instance_file = path.join("instance.json");
                if instance_file.exists() {
                    let content = fs::read_to_string(&instance_file).await?;
                    let instance: Instance = serde_json::from_str(&content)?;
                    self.instances.write().await.insert(instance.id, instance);
                }
            }
        }
        Ok(())
    }

    /// Save all instances to disk
    pub async fn save_all(&self) -> Result<()> {
        let instances = self.instances.read().await;
        for instance in instances.values() {
            self.save_instance(instance).await?;
        }
        Ok(())
    }

    /// Save a single instance
    async fn save_instance(&self, instance: &Instance) -> Result<()> {
        let instances_dir = get_instances_dir()?;
        let instance_dir = instances_dir.join(instance.id.to_string());
        fs::create_dir_all(&instance_dir).await?;

        let instance_file = instance_dir.join("instance.json");
        let content = serde_json::to_string_pretty(instance)?;
        fs::write(&instance_file, content).await?;
        Ok(())
    }

    /// Get all instances
    pub async fn list(&self) -> Vec<Instance> {
        let instances = self.instances.read().await;
        instances.values().cloned().collect()
    }

    /// Get an instance by ID
    pub async fn get(&self, id: Uuid) -> Option<Instance> {
        let instances = self.instances.read().await;
        instances.get(&id).cloned()
    }

    /// Add a new instance
    pub async fn add(&self, instance: Instance) -> Result<()> {
        let id = instance.id;
        let mut instances = self.instances.write().await;
        if instances.contains_key(&id) {
            return Err(LauncherError::InvalidArgument("Instance already exists".into()));
        }
        instances.insert(id, instance);
        drop(instances);
        self.save_all().await
    }

    /// Update an existing instance
    pub async fn update(&self, instance: Instance) -> Result<()> {
        let id = instance.id;
        let mut instances = self.instances.write().await;
        if !instances.contains_key(&id) {
            return Err(LauncherError::InstanceNotFound(id.to_string()));
        }
        instances.insert(id, instance);
        drop(instances);
        self.save_all().await
    }

    /// Delete an instance
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let mut instances = self.instances.write().await;
        if instances.remove(&id).is_none() {
            return Err(LauncherError::InstanceNotFound(id.to_string()));
        }
        drop(instances);

        // Also delete the instance directory
        let instances_dir = get_instances_dir()?;
        let instance_dir = instances_dir.join(id.to_string());
        if instance_dir.exists() {
            fs::remove_dir_all(&instance_dir).await?;
        }
        Ok(())
    }

    /// Clone an instance
    pub async fn clone(&self, id: Uuid, new_name: String) -> Result<Instance> {
        let original = self.get(id).await
            .ok_or(LauncherError::InstanceNotFound(id.to_string()))?;

        let mut cloned = Instance::new(new_name, original.version_id.clone())?;
        cloned.loader = original.loader;
        cloned.loader_version = original.loader_version;
        cloned.java = original.java.clone();
        cloned.memory = original.memory.clone();
        cloned.jvm_args = original.jvm_args.clone();
        cloned.game_args = original.game_args.clone();
        cloned.resolution = original.resolution.clone();

        self.add(cloned.clone()).await?;
        Ok(cloned)
    }

    /// Open instance game directory in file explorer
    pub async fn open_game_dir(&self, id: Uuid) -> Result<()> {
        let instance = self.get(id).await
            .ok_or(LauncherError::InstanceNotFound(id.to_string()))?;

        let game_dir = instance.effective_game_dir();
        fs::create_dir_all(&game_dir).await?;

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&game_dir)
                .spawn()?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&game_dir)
                .spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&game_dir)
                .spawn()?;
        }

        Ok(())
    }
}

impl Default for InstanceStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::model::Instance;

    #[tokio::test]
    async fn test_instance_storage_roundtrip() {
        let storage = InstanceStorage::new();
        let instance = Instance::new("Test".into(), "1.20.1".into()).unwrap();
        let id = instance.id;

        storage.add(instance).await.unwrap();

        let loaded = storage.get(id).await.unwrap();
        assert_eq!(loaded.name, "Test");
        assert_eq!(loaded.version_id, "1.20.1");
    }

    #[tokio::test]
    async fn test_instance_storage_delete() {
        let storage = InstanceStorage::new();
        let instance = Instance::new("ToDelete".into(), "1.20.1".into()).unwrap();
        let id = instance.id;
        storage.add(instance).await.unwrap();

        storage.delete(id).await.unwrap();
        assert!(storage.get(id).await.is_none());
    }
}