use md5;
use uuid::Uuid;

/// Generate offline UUID (UUID v3, MD5 hash of "OfflinePlayer:<username>")
/// This matches vanilla Minecraft's offline mode UUID generation exactly.
pub fn generate_offline_uuid(username: &str) -> Uuid {
    let namespace = "OfflinePlayer:";
    let input = format!("{}{}", namespace, username);
    let hash = md5::compute(input.as_bytes());
    let mut bytes = *hash;
    // Set version to 3 (MD5-based UUID) - bits 12-15 of byte 6
    bytes[6] = (bytes[6] & 0x0F) | 0x30;
    // Set variant to RFC 4122 (10xx) - bits 6-7 of byte 8
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    Uuid::from_bytes(bytes)
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
        // "Steve" -> 8667ba71-b85a-3340-af54-457a9734eed7
        let steve_uuid = generate_offline_uuid("Steve");
        assert_eq!(steve_uuid.to_string(), "8667ba71-b85a-3340-af54-457a9734eed7");

        // "Alex" -> e70671d5-6c79-3d23-8e76-447a2d8d3b5c
        let alex_uuid = generate_offline_uuid("Alex");
        assert_eq!(alex_uuid.to_string(), "e70671d5-6c79-3d23-8e76-447a2d8d3b5c");
    }
}