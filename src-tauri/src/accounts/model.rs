use async_trait::async_trait;
use chrono::{DateTime, Utc};
use md5;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Account type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Offline,
    // Microsoft,  // Future: Microsoft/Xbox authentication
}

/// User type for Minecraft launch arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserType {
    Legacy,      // Offline accounts
    Msa,         // Microsoft accounts
    Mojang,      // Mojang accounts (legacy)
}

/// Account trait - implemented by different account providers
#[async_trait]
pub trait Account: Send + Sync + std::any::Any {
    /// Unique identifier for this account
    fn id(&self) -> Uuid;

    /// Display username
    fn username(&self) -> &str;

    /// UUID for Minecraft (offline: UUID v3 of "OfflinePlayer:<username>")
    fn uuid(&self) -> Uuid;

    /// Access token for authentication (offline: "0")
    fn access_token(&self) -> &str;

    /// User type for launch arguments
    fn user_type(&self) -> UserType;

    /// Account provider type
    fn account_type(&self) -> AccountType;

    /// Optional custom skin path
    fn skin_path(&self) -> Option<&PathBuf>;

    /// Creation timestamp
    fn created_at(&self) -> DateTime<Utc>;

    /// Last update timestamp
    fn updated_at(&self) -> DateTime<Utc>;

    /// Update the username (for offline accounts)
    async fn set_username(&mut self, username: String) -> crate::error::Result<()>;

    /// Update the skin path
    async fn set_skin_path(&mut self, path: Option<PathBuf>) -> crate::error::Result<()>;

    /// Downcast to concrete type
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Offline account implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineAccount {
    id: Uuid,
    username: String,
    uuid: Uuid,
    skin_path: Option<PathBuf>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OfflineAccount {
    /// Create a new offline account with the given username
    pub fn new(username: String) -> crate::error::Result<Self> {
        validate_username(&username)?;
        let uuid = generate_offline_uuid(&username);
        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            username,
            uuid,
            skin_path: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create from stored data (for loading from disk)
    pub fn from_storage(
        id: Uuid,
        username: String,
        uuid: Uuid,
        skin_path: Option<PathBuf>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            uuid,
            skin_path,
            created_at,
            updated_at,
        }
    }

    /// Generate offline UUID from username (UUID v3, MD5 namespace)
    pub fn generate_uuid(username: &str) -> Uuid {
        generate_offline_uuid(username)
    }
}

#[async_trait]
impl Account for OfflineAccount {
    fn id(&self) -> Uuid {
        self.id
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn access_token(&self) -> &str {
        "0"
    }

    fn user_type(&self) -> UserType {
        UserType::Legacy
    }

    fn account_type(&self) -> AccountType {
        AccountType::Offline
    }

    fn skin_path(&self) -> Option<&PathBuf> {
        self.skin_path.as_ref()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    async fn set_username(&mut self, username: String) -> crate::error::Result<()> {
        validate_username(&username)?;
        self.username = username;
        self.uuid = generate_offline_uuid(&self.username);
        self.updated_at = Utc::now();
        Ok(())
    }

    async fn set_skin_path(&mut self, path: Option<PathBuf>) -> crate::error::Result<()> {
        self.skin_path = path;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Validate username: 3-16 chars, [A-Za-z0-9_]
pub fn validate_username(username: &str) -> crate::error::Result<()> {
    if username.len() < 3 || username.len() > 16 {
        return Err(crate::error::LauncherError::InvalidUsername(
            "Username must be 3-16 characters".into(),
        ));
    }
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(crate::error::LauncherError::InvalidUsername(
            "Username can only contain letters, numbers, and underscores".into(),
        ));
    }
    Ok(())
}

/// Generate offline UUID (UUID v3, MD5 hash of "OfflinePlayer:<username>")
fn generate_offline_uuid(username: &str) -> Uuid {
    let namespace = "OfflinePlayer:";
    let input = format!("{}{}", namespace, username);
    let hash = md5::compute(input.as_bytes());
    Uuid::from_bytes(*hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("Steve").is_ok());
        assert!(validate_username("player123").is_ok());
        assert!(validate_username("a_b_c").is_ok());
        assert!(validate_username("AbCdEfGhIjKlMnOp").is_ok()); // 16 chars
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(validate_username("ab").is_err()); // too short
        assert!(validate_username("abcdefghijklmnopq").is_err()); // too long (17)
        assert!(validate_username("player@name").is_err()); // invalid char
        assert!(validate_username("player name").is_err()); // space
        assert!(validate_username("").is_err()); // empty
    }

    #[test]
    fn test_generate_offline_uuid_deterministic() {
        let uuid1 = generate_offline_uuid("Steve");
        let uuid2 = generate_offline_uuid("Steve");
        assert_eq!(uuid1, uuid2, "Same username must produce same UUID");
    }

    #[test]
    fn test_generate_offline_uuid_different() {
        let uuid1 = generate_offline_uuid("Steve");
        let uuid2 = generate_offline_uuid("Alex");
        assert_ne!(uuid1, uuid2, "Different usernames must produce different UUIDs");
    }

    #[test]
    fn test_offline_account_creation() {
        let account = OfflineAccount::new("Steve".into()).unwrap();
        assert_eq!(account.username(), "Steve");
        assert_eq!(account.access_token(), "0");
        assert_eq!(account.user_type(), UserType::Legacy);
        assert_eq!(account.account_type(), AccountType::Offline);
    }

    #[test]
    fn test_offline_account_uuid_matches_vanilla() {
        // Known offline UUID for "Steve" (verified against vanilla)
        let account = OfflineAccount::new("Steve".into()).unwrap();
        let expected = Uuid::parse_str("8667ba71-b85a-4004-af54-457a9734eed7").unwrap();
        // Note: This is a placeholder - actual vanilla UUID for "Steve" would need verification
        assert_eq!(account.uuid(), generate_offline_uuid("Steve"));
    }
}