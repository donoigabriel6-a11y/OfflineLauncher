// Unit tests for SHA1 hash verification
use offline_launcher_lib::downloads::hash::{compute_sha1, verify_sha1, verify_sha1_or_error};
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_sha1_verification() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"hello world").unwrap();
    file.flush().unwrap();

    let expected = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"; // SHA1 of "hello world"
    let result = verify_sha1(file.path(), expected).await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_sha1_mismatch() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"hello world").unwrap();
    file.flush().unwrap();

    let expected = "0000000000000000000000000000000000000000"; // Wrong hash
    let result = verify_sha1_or_error(file.path(), expected, "test.txt").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_compute_sha1() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"test content").unwrap();
    file.flush().unwrap();

    let hash = compute_sha1(file.path()).await.unwrap();
    assert_eq!(hash.len(), 40); // SHA1 is 40 hex chars
    // Verify it's valid hex
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn test_sha1_case_insensitive() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"hello world").unwrap();
    file.flush().unwrap();

    // Uppercase expected hash
    let expected = "2AAE6C35C94FCFB415DBE95F408B9CE91EE846ED";
    let result = verify_sha1(file.path(), expected).await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_empty_file() {
    let file = NamedTempFile::new().unwrap();
    // Empty file SHA1
    let expected = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    let result = verify_sha1(file.path(), expected).await.unwrap();
    assert!(result);
}