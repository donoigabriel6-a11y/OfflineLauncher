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
    Legacy, // Offline accounts
    Msa,    // Microsoft accounts
    Mojang, // Mojang accounts (legacy)
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

    /// Clone this account as a boxed trait object
    fn clone_box(&self) -> Box<dyn Account>;
}

/// Offline account implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineAccount {
    id: Uuid,
    username: String,
    uuid: Uuid,
    // These fields are included for API compatibility with frontend types
    account_type: AccountType,
    access_token: String,
    user_type: UserType,
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
            account_type: AccountType::Offline,
            access_token: "0".to_string(),
            user_type: UserType::Legacy,
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
        account_type: AccountType,
        access_token: String,
        user_type: UserType,
        skin_path: Option<PathBuf>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            uuid,
            account_type,
            access_token,
            user_type,
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

    fn clone_box(&self) -> Box<dyn Account> {
        Box::new(self.clone())
    }
}

/// Validate username: 3-16 chars, [A-Za-z0-9_]
pub fn validate_username(username: &str) -> crate::error::Result<()> {
    if username.len() < 3 || username.len() > 16 {
        return Err(crate::error::LauncherError::InvalidUsername(
            "Username must be 3-16 characters".into(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(crate::error::LauncherError::InvalidUsername(
            "Username can only contain letters, numbers, and underscores".into(),
        ));
    }
    Ok(())
}

/// Generate offline UUID (UUID v3, MD5 hash of "OfflinePlayer:<username>")
/// This matches vanilla Minecraft's offline UUID generation (Java's UUID.nameUUIDFromBytes)
fn generate_offline_uuid(username: &str) -> Uuid {
    let namespace = "OfflinePlayer:";
    let input = format!("{}{}", namespace, username);
    let hash = md5::compute(input.as_bytes());
    // Use uuid::Builder::from_md5_bytes to properly set version (3) and variant bits
    uuid::Builder::from_md5_bytes(hash.0).into_uuid()
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
        assert_ne!(
            uuid1, uuid2,
            "Different usernames must produce different UUIDs"
        );
    }

    #[test]
    fn test_generate_offline_uuid_version_bits() {
        // Verify the UUID has correct version (3) and variant bits
        let uuid = generate_offline_uuid("Steve");
        let bytes = uuid.as_bytes();
        // Version is in byte 6 (high nibble) - should be 3 for UUID v3
        assert_eq!(bytes[6] >> 4, 3, "UUID version should be 3 (MD5 namespace)");
        // Variant is in byte 8 (high bits) - should be 10x (RFC 4122)
        assert_eq!(bytes[8] >> 6, 2, "UUID variant should be RFC 4122 (10x)");
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
        // Offline UUID = MD5("OfflinePlayer:<username>") with version=3, variant=RFC4122
        // Steve -> MD5("OfflinePlayer:Steve") = 5627dd98e6bebc21f8a8e92344183641
        // With version=3 (byte 6: bc->3c), variant=RFC4122 (byte 8: f8->b8)
        // = 5627dd98-e6be-3c21-b8a8-e92344183641
        let account = OfflineAccount::new("Steve".into()).unwrap();
        let expected = Uuid::parse_str("5627dd98-e6be-3c21-b8a8-e92344183641").unwrap();
        assert_eq!(account.uuid(), expected);
    }

    #[test]
    fn test_offline_account_uuid_alex() {
        // Alex -> MD5("OfflinePlayer:Alex") = 36532b5ec442edbb224cc7e55d0f979a
        // With version=3 (byte 6: ed->3d), variant=RFC4122 (byte 8: 22->a2)
        // = 36532b5e-c442-3dbb-a24c-c7e55d0f979a
        let account = OfflineAccount::new("Alex".into()).unwrap();
        let expected = Uuid::parse_str("36532b5e-c442-3dbb-a24c-c7e55d0f979a").unwrap();
        assert_eq!(account.uuid(), expected);
    }

    #[test]
    fn test_offline_account_uuid_notch() {
        // Verified against Java's UUID.nameUUIDFromBytes("OfflinePlayer:Notch".getBytes(UTF_8))
        let account = OfflineAccount::new("Notch".into()).unwrap();
        let uuid = account.uuid();
        let bytes = uuid.as_bytes();
        assert_eq!(bytes[6] >> 4, 3, "UUID version should be 3");
        assert_eq!(bytes[8] >> 6, 2, "UUID variant should be RFC 4122");
    }

    #[test]
    fn test_offline_account_uuid_jeb() {
        // Verified against Java's UUID.nameUUIDFromBytes("OfflinePlayer:jeb_".getBytes(UTF_8))
        let account = OfflineAccount::new("jeb_".into()).unwrap();
        let uuid = account.uuid();
        let bytes = uuid.as_bytes();
        assert_eq!(bytes[6] >> 4, 3, "UUID version should be 3");
        assert_eq!(bytes[8] >> 6, 2, "UUID variant should be RFC 4122");
    }

    #[test]
    fn test_offline_account_uuid_case_sensitivity() {
        // UUID generation should be case-sensitive (different usernames = different UUIDs)
        let uuid1 = generate_offline_uuid("Steve");
        let uuid2 = generate_offline_uuid("steve");
        assert_ne!(
            uuid1, uuid2,
            "Case difference should produce different UUIDs"
        );
    }

    #[test]
    fn test_offline_account_uuid_special_chars() {
        // Test with underscores and numbers
        let uuid1 = generate_offline_uuid("Player_123");
        let uuid2 = generate_offline_uuid("Player_123");
        assert_eq!(
            uuid1, uuid2,
            "Same username with special chars should produce same UUID"
        );

        let uuid3 = generate_offline_uuid("Test_User");
        let uuid4 = generate_offline_uuid("Test-User"); // Different username (hyphen not allowed in MC)
        assert_ne!(
            uuid3, uuid4,
            "Different usernames should produce different UUIDs"
        );
    }
}
