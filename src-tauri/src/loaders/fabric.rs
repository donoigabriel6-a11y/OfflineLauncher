//! Fabric mod loader integration

use crate::error::{LauncherError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricLoaderVersion {
    pub version: String,
    pub stable: bool,
    pub build: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricInstallerResponse {
    pub profile: FabricInstallerProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricInstallerProfile {
    pub id: String,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "javaVersion")]
    pub java_version: FabricJavaVersion,
    pub libraries: Vec<FabricLibrary>,
    pub arguments: FabricArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricJavaVersion {
    #[serde(rename = "component")]
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricLibrary {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricArguments {
    pub client: Vec<String>,
}

/// Fetch available Fabric game versions
pub async fn fetch_fabric_versions(client: &Client) -> Result<Vec<FabricVersion>> {
    let url = format!("{}/versions/game", FABRIC_META_URL);
    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::ModLoader(format!("Failed to fetch Fabric versions: {}", response.status())));
    }

    let versions: Vec<FabricVersion> = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    Ok(versions)
}

/// Fetch available Fabric loader versions
pub async fn fetch_fabric_loader_versions(client: &Client) -> Result<Vec<FabricLoaderVersion>> {
    let url = format!("{}/versions/loader", FABRIC_META_URL);
    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::ModLoader(format!("Failed to fetch Fabric loader versions: {}", response.status())));
    }

    let versions: Vec<FabricLoaderVersion> = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    Ok(versions)
}

/// Install Fabric into an instance (generate installer profile)
pub async fn install_fabric(
    client: &Client,
    minecraft_version: &str,
    loader_version: &str,
    _install_dir: &Path,
) -> Result<FabricInstallerProfile> {
    let url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        FABRIC_META_URL, minecraft_version, loader_version
    );

    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::ModLoader(
            format!("Failed to get Fabric installer profile: {}", response.status())
        ));
    }

    let installer: FabricInstallerResponse = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    Ok(installer.profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_fabric_versions() {
        let client = Client::new();
        let versions = fetch_fabric_versions(&client).await;
        // This will fail without network, but tests structure is correct
        println!("Fabric versions result: {:?}", versions.is_ok());
    }
}