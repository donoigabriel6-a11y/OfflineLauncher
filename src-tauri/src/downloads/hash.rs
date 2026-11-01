//! SHA1 hash verification

use crate::error::{LauncherError, Result};
use sha1::{Digest, Sha1};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Compute SHA1 hash of a file
pub async fn compute_sha1(path: &Path) -> Result<String> {
    let mut file = File::open(path).await
        .map_err(|e| LauncherError::Io(e))?;

    let mut hasher = Sha1::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let n = file.read(&mut buffer).await
            .map_err(|e| LauncherError::Io(e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Verify a file's SHA1 hash matches expected
pub async fn verify_sha1(path: &Path, expected: &str) -> Result<bool> {
    let actual = compute_sha1(path).await?;
    Ok(actual.eq_ignore_ascii_case(expected))
}

/// Verify and return error if mismatch
pub async fn verify_sha1_or_error(path: &Path, expected: &str, file_name: &str) -> Result<()> {
    let actual = compute_sha1(path).await?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(LauncherError::HashMismatch {
            file: file_name.to_string(),
            expected: expected.to_string(),
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
}