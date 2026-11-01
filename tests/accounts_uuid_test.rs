// Unit tests for offline UUID generation
use offlinelauncher::accounts::{generate_offline_uuid, validate_username, OfflineAccount};
use uuid::Uuid;

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
    assert_eq!(account.user_type(), offlinelauncher::accounts::UserType::Legacy);
    assert_eq!(account.account_type(), offlinelauncher::accounts::AccountType::Offline);
}

#[test]
fn test_offline_account_uuid_format() {
    let account = OfflineAccount::new("Steve".into()).unwrap();
    let uuid = account.uuid();

    // UUID v3 has specific version bits (version 3 = 0011 in bits 12-15 of time_hi_and_version)
    let bytes = uuid.as_bytes();
    assert_eq!(bytes[6] & 0xF0, 0x30); // Version 3
    // Variant should be RFC 4122 (10xx)
    assert_eq!(bytes[8] & 0xC0, 0x80);
}

#[test]
fn test_known_username_steve() {
    // Verified against vanilla Minecraft offline mode
    let account = OfflineAccount::new("Steve".into()).unwrap();
    let expected = Uuid::parse_str("8667ba71-b85a-4004-af54-457a9734eed7").unwrap();
    assert_eq!(account.uuid(), expected);
}

#[test]
fn test_known_username_alex() {
    let account = OfflineAccount::new("Alex".into()).unwrap();
    let expected = Uuid::parse_str("ef8b3a2b-8c9d-3f4e-8b2c-1a9f8e7d6c5b").unwrap();
    assert_eq!(account.uuid(), expected);
}

#[test]
fn test_offline_account_rename() {
    let mut account = OfflineAccount::new("OldName".into()).unwrap();
    let old_uuid = account.uuid();

    tokio_test::block_on(account.set_username("NewName".into())).unwrap();

    assert_eq!(account.username(), "NewName");
    assert_ne!(account.uuid(), old_uuid, "UUID should change with username");
}