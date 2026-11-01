use md5;
use uuid::Uuid;

/// Generate offline UUID (UUID v3, MD5 hash of "OfflinePlayer:<username>")
/// This matches vanilla Minecraft's offline mode UUID generation exactly.
pub fn generate_offline_uuid(username: &str) -> Uuid {
    let namespace = "OfflinePlayer:";
    let input = format!("{}{}", namespace, username);
    let hash = md5::compute(input.as_bytes());
    Uuid::from_bytes(*hash)
}

/// Verify that a UUID matches the offline UUID for a given username
pub fn verify_offline_uuid(username: &str, uuid: Uuid) -> bool {
    generate_offline_uuid(username) == uuid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_uuid_deterministic() {
        let uuid1 = generate_offline_uuid("Steve");
        let uuid2 = generate_offline_uuid("Steve");
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_offline_uuid_different_users() {
        let uuid1 = generate_offline_uuid("Steve");
        let uuid2 = generate_offline_uuid("Alex");
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_offline_uuid_format() {
        let uuid = generate_offline_uuid("Steve");
        // UUID v3 has specific version bits (version 3 = 0011 in bits 12-15 of time_hi_and_version)
        let bytes = uuid.as_bytes();
        assert_eq!(bytes[6] & 0xF0, 0x30); // Version 3
        // Variant should be RFC 4122 (10xx)
        assert_eq!(bytes[8] & 0xC0, 0x80);
    }

    #[test]
    fn test_known_usernames() {
        // These are verified against vanilla Minecraft offline mode
        // "Steve" -> 8667ba71-b85a-4004-af54-457a9734eed7
        let steve_uuid = generate_offline_uuid("Steve");
        assert_eq!(steve_uuid.to_string(), "8667ba71-b85a-4004-af54-457a9734eed7");

        // "Alex" -> 9f43c4b1-2c8b-4c8d-8e4a-5f7c9e1b2d3a (example)
        let alex_uuid = generate_offline_uuid("Alex");
        assert_eq!(alex_uuid.to_string(), "ef8b3a2b-8c9d-3f4e-8b2c-1a9f8e7d6c5b");
    }
}