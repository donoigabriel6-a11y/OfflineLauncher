use crate::accounts::model::{Account, OfflineAccount, AccountType, UserType};
use crate::app_dirs::get_accounts_file;
use crate::error::{LauncherError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Account storage configuration
#[derive(Debug, Clone)]
pub struct AccountStorageConfig {
    pub accounts_file: PathBuf,
}

impl Default for AccountStorageConfig {
    fn default() -> Self {
        Self {
            accounts_file: get_accounts_file().expect("Failed to get accounts file path"),
        }
    }
}

impl AccountStorageConfig {
    /// Create a test configuration with a temp directory
    pub fn for_test(temp_dir: PathBuf) -> Self {
        Self {
            accounts_file: temp_dir.join("accounts.json"),
        }
    }
}

/// Serialized account data for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StoredAccount {
    Offline {
        id: Uuid,
        username: String,
        uuid: Uuid,
        skin_path: Option<PathBuf>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
    // Microsoft { ... } // Future
}

impl From<&OfflineAccount> for StoredAccount {
    fn from(acc: &OfflineAccount) -> Self {
        StoredAccount::Offline {
            id: acc.id(),
            username: acc.username().to_string(),
            uuid: acc.uuid(),
            skin_path: acc.skin_path().cloned(),
            created_at: acc.created_at(),
            updated_at: acc.updated_at(),
        }
    }
}

impl StoredAccount {
    /// Convert back to an Account trait object
    pub fn into_account(self) -> Box<dyn Account> {
        match self {
            StoredAccount::Offline { id, username, uuid, skin_path, created_at, updated_at } => {
                Box::new(OfflineAccount::from_storage(id, username, uuid, skin_path, created_at, updated_at))
            }
        }
    }
}

/// Account storage manager
pub struct AccountStorage {
    accounts: Arc<RwLock<HashMap<Uuid, Box<dyn Account>>>>,
    config: AccountStorageConfig,
}

impl AccountStorage {
    /// Create a new storage instance with default config
    pub fn new() -> Self {
        Self::with_config(AccountStorageConfig::default())
    }

    /// Create a new storage instance with custom config
    pub fn with_config(config: AccountStorageConfig) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a test storage instance with a temp directory
    pub fn for_test(temp_dir: PathBuf) -> Self {
        Self::with_config(AccountStorageConfig::for_test(temp_dir))
    }

    /// Load accounts from disk
    pub async fn load(&self) -> Result<()> {
        let path = &self.config.accounts_file;
        if !path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| LauncherError::Io(e))?;

        let stored: Vec<StoredAccount> = serde_json::from_str(&content)
            .map_err(|e| LauncherError::Serde(e))?;

        let mut accounts = self.accounts.write().await;
        for stored_acc in stored {
            let account = stored_acc.into_account();
            accounts.insert(account.id(), account);
        }
        Ok(())
    }

    /// Save accounts to disk
    pub async fn save(&self) -> Result<()> {
        let accounts = self.accounts.read().await;
        let stored: Vec<StoredAccount> = accounts.values()
            .map(|acc| {
                // Downcast to OfflineAccount for serialization
                // In the future, we'll handle different account types
                if let Some(offline) = acc.as_any().downcast_ref::<OfflineAccount>() {
                    StoredAccount::from(offline)
                } else {
                    // For now, only offline accounts are supported
                    panic!("Unsupported account type");
                }
            })
            .collect();

        let path = &self.config.accounts_file;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| LauncherError::Io(e))?;
        }

        let content = serde_json::to_string_pretty(&stored)
            .map_err(|e| LauncherError::Serde(e))?;

        tokio::fs::write(path, content).await
            .map_err(|e| LauncherError::Io(e))?;
        Ok(())
    }

    /// Get all accounts
    pub async fn list(&self) -> Vec<Box<dyn Account>> {
        let accounts = self.accounts.read().await;
        accounts.values().map(|a| a.clone_box()).collect()
    }

    /// Get an account by ID
    pub async fn get(&self, id: Uuid) -> Option<Box<dyn Account>> {
        let accounts = self.accounts.read().await;
        accounts.get(&id).map(|a| a.clone_box())
    }

    /// Get an account by username (for offline accounts)
    pub async fn get_by_username(&self, username: &str) -> Option<Box<dyn Account>> {
        let accounts = self.accounts.read().await;
        accounts.values()
            .find(|a| a.username().eq_ignore_ascii_case(username))
            .map(|a| a.clone_box())
    }

    /// Add a new account
    pub async fn add(&self, account: Box<dyn Account>) -> Result<()> {
        let id = account.id();
        let mut accounts = self.accounts.write().await;
        if accounts.contains_key(&id) {
            return Err(LauncherError::InvalidArgument("Account already exists".into()));
        }
        accounts.insert(id, account);
        drop(accounts);
        self.save().await
    }

    /// Update an existing account
    pub async fn update(&self, account: Box<dyn Account>) -> Result<()> {
        let id = account.id();
        let mut accounts = self.accounts.write().await;
        if !accounts.contains_key(&id) {
            return Err(LauncherError::AccountNotFound(id.to_string()));
        }
        accounts.insert(id, account);
        drop(accounts);
        self.save().await
    }

    /// Delete an account by ID
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        if accounts.remove(&id).is_none() {
            return Err(LauncherError::AccountNotFound(id.to_string()));
        }
        drop(accounts);
        self.save().await
    }

    /// Get the default account (first one, or create a default)
    pub async fn get_default(&self) -> Result<Box<dyn Account>> {
        let accounts = self.list().await;
        if let Some(acc) = accounts.first() {
            Ok(acc.clone_box())
        } else {
            // Create a default "Player" account
            let account = OfflineAccount::new("Player".into())?;
            let id = account.id();
            self.add(account).await?;
            self.get(id).await.ok_or(LauncherError::AccountNotFound("default".into()))
        }
    }
}

impl Default for AccountStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::model::OfflineAccount;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_account_storage_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let storage = AccountStorage::for_test(temp_dir.path().to_path_buf());
        let account = OfflineAccount::new("TestUser".into()).unwrap();
        let id = account.id();

        storage.add(Box::new(account)).await.unwrap();

        let loaded = storage.get(id).await.unwrap();
        assert_eq!(loaded.username(), "TestUser");
        assert_eq!(loaded.id(), id);
    }

    #[tokio::test]
    async fn test_account_storage_list() {
        let temp_dir = tempdir().unwrap();
        let storage = AccountStorage::for_test(temp_dir.path().to_path_buf());
        storage.add(Box::new(OfflineAccount::new("User1".into()).unwrap())).await.unwrap();
        storage.add(Box::new(OfflineAccount::new("User2".into()).unwrap())).await.unwrap();

        let accounts = storage.list().await;
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn test_account_storage_delete() {
        let temp_dir = tempdir().unwrap();
        let storage = AccountStorage::for_test(temp_dir.path().to_path_buf());
        let account = OfflineAccount::new("ToDelete".into()).unwrap();
        let id = account.id();
        storage.add(Box::new(account)).await.unwrap();

        storage.delete(id).await.unwrap();
        assert!(storage.get(id).await.is_none());
    }
}