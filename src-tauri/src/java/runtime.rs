//! Java runtime manifest fetching (Mojang/Adoptium)

use crate::error::{LauncherError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MOJANG_JAVA_MANIFEST: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntimeManifest {
    #[serde(rename = "javaVersion")]
    pub java_version: String,
    pub files: HashMap<String, JavaRuntimeFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntimeFile {
    pub url: String,
    #[serde(rename = "sha1")]
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntime {
    pub version: String,
    pub major_version: u8,
    pub os: String,
    pub arch: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub vendor: String, // "mojang" or "adoptium"
}

/// Fetch Mojang's Java runtime manifest
pub async fn fetch_java_runtimes(client: &Client) -> Result<Vec<JavaRuntime>> {
    let response = client.get(MOJANG_JAVA_MANIFEST).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::Download(format!("Failed to fetch Java manifest: {}", response.status())));
    }

    let manifest: JavaRuntimeManifest = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    let mut runtimes = Vec::new();

    for (key, file) in manifest.files {
        // Parse key like "java-runtime-alpha/windows-x64/21.0.1"
        let parts: Vec<&str> = key.split('/').collect();
        if parts.len() != 3 {
            continue;
        }

        let os_arch = parts[1];
        let version_str = parts[2];

        let Some((os, arch)) = parse_os_arch(os_arch) else {
            continue;
        };
        let Some(major_version) = parse_java_major_version(version_str) else {
            continue;
        };

        // Only include current platform
        if os != current_os() || arch != current_arch() {
            continue;
        }

        runtimes.push(JavaRuntime {
            version: version_str.to_string(),
            major_version,
            os: os.to_string(),
            arch: arch.to_string(),
            url: file.url,
            sha1: file.sha1,
            size: file.size,
            vendor: "mojang".to_string(),
        });
    }

    // Sort by version (newest first)
    runtimes.sort_by(|a, b| b.version.cmp(&a.version));

    Ok(runtimes)
}

fn parse_os_arch(os_arch: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = os_arch.split('-').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

fn parse_java_major_version(version: &str) -> Option<u8> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        if parts[0] == "1" {
            parts[1].parse().ok()
        } else {
            parts[0].parse().ok()
        }
    } else {
        None
    }
}

fn current_os() -> &'static str {
    #[cfg(target_os = "windows")] return "windows";
    #[cfg(target_os = "macos")] return "macos";
    #[cfg(target_os = "linux")] return "linux";
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))] return "unknown";
}

fn current_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")] return "x64";
    #[cfg(target_arch = "aarch64")] return "aarch64";
    #[cfg(target_arch = "arm")] return "arm";
    #[cfg(target_arch = "x86")] return "x86";
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm", target_arch = "x86")))] return "unknown";
}

/// Fetch Adoptium Temurin releases as fallback
pub async fn fetch_adoptium_runtimes(client: &Client, major_version: u8) -> Result<Vec<JavaRuntime>> {
    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&os={}&vendor=eclipse",
        major_version,
        current_arch(),
        current_os()
    );

    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    #[derive(Deserialize)]
    struct AdoptiumResponse {
        #[serde(rename = "binary")]
        binaries: Vec<AdoptiumBinary>,
    }

    #[derive(Deserialize)]
    struct AdoptiumBinary {
        #[serde(rename = "package")]
        pkg: AdoptiumPackage,
    }

    #[derive(Deserialize)]
    struct AdoptiumPackage {
        #[serde(rename = "link")]
        link: String,
        #[serde(rename = "checksum")]
        checksum: String,
        #[serde(rename = "size")]
        size: u64,
        #[serde(rename = "version")]
        version: AdoptiumVersion,
    }

    #[derive(Deserialize)]
    struct AdoptiumVersion {
        #[serde(rename = "major")]
        major: u8,
        #[serde(rename = "minor")]
        minor: u32,
        #[serde(rename = "security")]
        security: u32,
        #[serde(rename = "patch")]
        patch: u32,
    }

    let data: AdoptiumResponse = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    let mut runtimes = Vec::new();
    for binary in data.binaries {
        let v = &binary.pkg.version;
        let version_str = format!("{}.{}.{}.{}", v.major, v.minor, v.security, v.patch);
        runtimes.push(JavaRuntime {
            version: version_str,
            major_version: v.major,
            os: current_os().to_string(),
            arch: current_arch().to_string(),
            url: binary.pkg.link,
            sha1: binary.pkg.checksum,
            size: binary.pkg.size,
            vendor: "adoptium".to_string(),
        });
    }

    Ok(runtimes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_java_major_version() {
        assert_eq!(parse_java_major_version("1.8.0_381"), Some(8));
        assert_eq!(parse_java_major_version("11.0.1"), Some(11));
        assert_eq!(parse_java_major_version("17.0.5"), Some(17));
        assert_eq!(parse_java_major_version("21.0.1"), Some(21));
    }

    #[test]
    fn test_parse_os_arch() {
        assert_eq!(parse_os_arch("windows-x64"), Some(("windows", "x64")));
        assert_eq!(parse_os_arch("linux-aarch64"), Some(("linux", "aarch64")));
        assert_eq!(parse_os_arch("macos-x64"), Some(("macos", "x64")));
    }
}