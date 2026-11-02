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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltLoaderVersion {
    pub version: String,
    pub build: u32,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltInstallerResponse {
    pub profile: QuiltInstallerProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltInstallerProfile {
    pub id: String,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "javaVersion")]
    pub java_version: QuiltJavaVersion,
    pub libraries: Vec<QuiltLibrary>,
    pub arguments: QuiltArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltJavaVersion {
    #[serde(rename = "component")]
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltLibrary {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuiltArguments {
    pub client: Vec<String>,
}

/// Fetch available Quilt game versions
pub async fn fetch_quilt_versions(client: &Client) -> Result<Vec<QuiltVersion>> {
    let url = format!("{}/versions/game", QUILT_META_URL);
    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::ModLoader(format!("Failed to fetch Quilt versions: {}", response.status())));
    }

    let versions: Vec<QuiltVersion> = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    Ok(versions)
}

/// Fetch available Quilt loader versions
pub async fn fetch_quilt_loader_versions(client: &Client) -> Result<Vec<QuiltLoaderVersion>> {
    let url = format!("{}/versions/loader", QUILT_META_URL);
    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::ModLoader(format!("Failed to fetch Quilt loader versions: {}", response.status())));
    }

    let versions: Vec<QuiltLoaderVersion> = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    Ok(versions)
}

/// Install Quilt into an instance (generate installer profile)
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

    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::ModLoader(
            format!("Failed to get Quilt installer profile: {}", response.status())
        ));
    }

    let installer: QuiltInstallerResponse = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    Ok(installer.profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_quilt_versions() {
        let client = Client::new();
        let versions = fetch_quilt_versions(&client).await;
        println!("Quilt versions result: {:?}", versions.is_ok());
    }
}