//! Quilt mod loader integration

use crate::error::{LauncherError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

const QUILT_META_URL: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltVersion {
    pub version: String,
    pub stable: bool,
}

/// Quilt's loader list is scoped to a Minecraft version and wraps the loader
/// coordinate in a `loader` object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltLoaderVersion {
    pub version: String,
    pub build: u32,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
struct QuiltLoaderVersionEntry {
    loader: QuiltLoaderCoordinate,
}

#[derive(Debug, Deserialize)]
struct QuiltLoaderCoordinate {
    version: String,
    build: u32,
}

/// A profile returned by Quilt's `/profile/json` endpoint. The endpoint
/// returns this object directly, not inside a `{ "profile": ... }` wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltInstallerProfile {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "inheritsFrom")]
    pub inherits_from: String,
    #[serde(default, rename = "releaseTime")]
    pub release_time: String,
    #[serde(default, rename = "mainClass")]
    pub main_class: String,
    #[serde(default, rename = "javaVersion")]
    pub java_version: Option<QuiltJavaVersion>,
    #[serde(default)]
    pub libraries: Vec<QuiltLibrary>,
    #[serde(default)]
    pub arguments: QuiltArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltJavaVersion {
    pub component: String,
    pub major_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltLibrary {
    pub name: String,
    /// Maven repository base URL, not a direct JAR URL.
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltArguments {
    #[serde(default)]
    pub game: Vec<String>,
    #[serde(default)]
    pub jvm: Vec<String>,
    /// Older installer profiles use `client` for game arguments.
    #[serde(default)]
    pub client: Vec<String>,
}

fn is_stable_loader(version: &str) -> bool {
    let lower = version.to_ascii_lowercase();
    !["-alpha", "-beta", "-rc", "-pre", "-snapshot"]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn stable_version_components(version: &str) -> Vec<u64> {
    version
        .split(['-', '+'])
        .next()
        .unwrap_or(version)
        .split('.')
        .map(|component| component.parse().unwrap_or(0))
        .collect()
}

/// Pick the newest stable release by semantic version, not Quilt's build number.
/// Build numbers are not globally monotonic across releases (for example, a
/// newer stable may be build 1 while an older release is build 4).
pub fn latest_stable_quilt_loader_version(versions: &[QuiltLoaderVersion]) -> Option<String> {
    versions
        .iter()
        .filter(|version| version.stable)
        .max_by(|left, right| {
            stable_version_components(&left.version)
                .cmp(&stable_version_components(&right.version))
                .then_with(|| left.build.cmp(&right.build))
        })
        .map(|version| version.version.clone())
}

async fn response_text(response: reqwest::Response, operation: &str) -> Result<String> {
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(LauncherError::Http)?;
    if !status.is_success() {
        return Err(LauncherError::ModLoader(format!(
            "{} failed: {} - {}",
            operation, status, text
        )));
    }
    Ok(text)
}

/// Fetch available Quilt game versions.
pub async fn fetch_quilt_versions(client: &Client) -> Result<Vec<QuiltVersion>> {
    let url = format!("{}/versions/game", QUILT_META_URL);
    let response = client.get(&url).send().await.map_err(LauncherError::Http)?;
    let text = response_text(response, "Fetching Quilt game versions").await?;
    serde_json::from_str(&text).map_err(|error| {
        LauncherError::ModLoader(format!(
            "Invalid Quilt game versions response: {}. Response begins: {}",
            error,
            &text[..text.len().min(300)]
        ))
    })
}

/// Fetch Quilt loader versions compatible with the given Minecraft version.
pub async fn fetch_quilt_loader_versions(
    client: &Client,
    minecraft_version: &str,
) -> Result<Vec<QuiltLoaderVersion>> {
    let url = format!("{}/versions/loader/{}", QUILT_META_URL, minecraft_version);
    let response = client.get(&url).send().await.map_err(LauncherError::Http)?;
    let text = response_text(response, "Fetching Quilt loader versions").await?;
    let entries: Vec<QuiltLoaderVersionEntry> = serde_json::from_str(&text).map_err(|error| {
        LauncherError::ModLoader(format!(
            "Invalid Quilt loader versions response for Minecraft {}: {}. Response begins: {}",
            minecraft_version,
            error,
            &text[..text.len().min(300)]
        ))
    })?;

    Ok(entries
        .into_iter()
        .map(|entry| QuiltLoaderVersion {
            stable: is_stable_loader(&entry.loader.version),
            version: entry.loader.version,
            build: entry.loader.build,
        })
        .collect())
}

/// Fetch a Quilt launch profile for a specific Minecraft and loader version.
pub async fn install_quilt(
    client: &Client,
    minecraft_version: &str,
    loader_version: &str,
    _install_dir: &Path,
) -> Result<QuiltInstallerProfile> {
    let url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        QUILT_META_URL, minecraft_version, loader_version
    );
    let response = client.get(&url).send().await.map_err(LauncherError::Http)?;
    let text = response_text(response, "Fetching Quilt installer profile").await?;
    let profile: QuiltInstallerProfile = serde_json::from_str(&text).map_err(|error| {
        LauncherError::ModLoader(format!(
            "Invalid Quilt installer profile for Minecraft {} / loader {}: {}. Response begins: {}",
            minecraft_version,
            loader_version,
            error,
            &text[..text.len().min(500)]
        ))
    })?;

    if profile.id.is_empty() || profile.main_class.is_empty() || profile.libraries.is_empty() {
        return Err(LauncherError::ModLoader(format!(
            "Quilt profile for Minecraft {} and loader {} is missing required launch data",
            minecraft_version, loader_version
        )));
    }

    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_minecraft_specific_loader_entries() {
        let entries: Vec<QuiltLoaderVersionEntry> = serde_json::from_str(
            r#"[{"loader":{"version":"0.28.1","build":7,"maven":"org.quiltmc:quilt-loader:0.28.1"},"intermediary":{"version":"1.21.4"}}]"#,
        )
        .unwrap();
        let entry = entries.into_iter().next().unwrap();
        assert_eq!(entry.loader.version, "0.28.1");
        assert_eq!(entry.loader.build, 7);
        assert!(is_stable_loader(&entry.loader.version));
    }

    #[test]
    fn prerelease_quilt_loader_versions_are_not_stable() {
        assert!(!is_stable_loader("0.20.0-beta.9"));
        assert!(!is_stable_loader("0.20.0-rc.1"));
        assert!(is_stable_loader("0.20.0"));
    }

    #[test]
    fn latest_stable_loader_uses_semver_not_build_number() {
        let versions = vec![
            QuiltLoaderVersion {
                version: "0.26.4".into(),
                build: 4,
                stable: true,
            },
            QuiltLoaderVersion {
                version: "0.31.0-beta.1".into(),
                build: 10,
                stable: false,
            },
            QuiltLoaderVersion {
                version: "0.30.2".into(),
                build: 2,
                stable: true,
            },
        ];

        assert_eq!(latest_stable_quilt_loader_version(&versions).as_deref(), Some("0.30.2"));
    }

    #[test]
    fn parses_top_level_profile_with_optional_java_version() {
        let profile: QuiltInstallerProfile = serde_json::from_str(
            r#"{
                "id":"quilt-loader-0.28.1-1.21.4",
                "inheritsFrom":"1.21.4",
                "mainClass":"org.quiltmc.loader.impl.launch.knot.KnotClient",
                "arguments":{"game":[],"jvm":["-Dquilt.example=true"]},
                "libraries":[{"name":"org.quiltmc:quilt-loader:0.28.1","url":"https://maven.quiltmc.org/repository/release/"}]
            }"#,
        )
        .unwrap();

        assert_eq!(profile.main_class, "org.quiltmc.loader.impl.launch.knot.KnotClient");
        assert!(profile.java_version.is_none());
        assert_eq!(profile.arguments.jvm, vec!["-Dquilt.example=true"]);
        assert_eq!(profile.libraries.len(), 1);
    }
}
