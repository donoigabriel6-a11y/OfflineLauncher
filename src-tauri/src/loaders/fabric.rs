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
pub struct FabricInstallerProfile {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "inheritsFrom")]
    pub inherits_from: String,
    #[serde(default, rename = "releaseTime")]
    pub release_time: String,
    #[serde(default, rename = "mainClass")]
    pub main_class: String,
    #[serde(default, rename = "javaVersion")]
    pub java_version: Option<FabricJavaVersion>,
    #[serde(default)]
    pub libraries: Vec<FabricLibrary>,
    #[serde(default)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricArguments {
    #[serde(default)]
    pub game: Vec<String>,
    #[serde(default)]
    pub jvm: Vec<String>,
    // Some installer profiles use the older `client` field.
    #[serde(default)]
    pub client: Vec<String>,
}

/// Fetch available Fabric game versions
pub async fn fetch_fabric_versions(client: &Client) -> Result<Vec<FabricVersion>> {
    let url = format!("{}/versions/game", FABRIC_META_URL);
    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(LauncherError::ModLoader(format!("Failed to fetch Fabric versions: {} - {}", status, text)));
    }

    // Get the raw text first to debug JSON parsing issues
    let text = response.text().await
        .map_err(|e| LauncherError::Http(e))?;

    // Parse the JSON from the text
    let versions: Vec<FabricVersion> = serde_json::from_str(&text)
        .map_err(|e| LauncherError::ModLoader(
            format!("Failed to parse Fabric versions JSON: {} - Response: {}", e, text)
        ))?;

    Ok(versions)
}

/// Fetch available Fabric loader versions
pub async fn fetch_fabric_loader_versions(client: &Client) -> Result<Vec<FabricLoaderVersion>> {
    let url = format!("{}/versions/loader", FABRIC_META_URL);
    let response = client.get(&url).send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(LauncherError::ModLoader(format!("Failed to fetch Fabric loader versions: {} - {}", status, text)));
    }

    // Get the raw text first to debug JSON parsing issues
    let text = response.text().await
        .map_err(|e| LauncherError::Http(e))?;

    // Parse the JSON from the text
    let versions: Vec<FabricLoaderVersion> = serde_json::from_str(&text)
        .map_err(|e| LauncherError::ModLoader(
            format!("Failed to parse Fabric loader versions JSON: {} - Response: {}", e, text)
        ))?;

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
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(LauncherError::ModLoader(
            format!("Failed to get Fabric installer profile: {} - {}", status, text)
        ));
    }

    // Get the raw text first to debug JSON parsing issues
    let text = response.text().await
        .map_err(|e| LauncherError::Http(e))?;

    // Parse the JSON from the text
    let profile: FabricInstallerProfile = serde_json::from_str(&text)
        .map_err(|e| LauncherError::ModLoader(
            format!("Failed to parse Fabric installer profile JSON: {} - Response: {}", e, text)
        ))?;

    if profile.id.is_empty() || profile.main_class.is_empty() || profile.libraries.is_empty() {
        return Err(LauncherError::ModLoader(format!(
            "Fabric profile for Minecraft {} and loader {} is missing required launch data",
            minecraft_version, loader_version
        )));
    }

    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserializes_top_level_fabric_profile_response() {
        let profile: FabricInstallerProfile = serde_json::from_str(r#"{
            "id": "fabric-loader-1.21.4-0.16.10",
            "inheritsFrom": "1.21.4",
            "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
            "arguments": {"game": [], "jvm": ["-Dfabric.example=true"]},
            "libraries": [{
                "name": "net.fabricmc:fabric-loader:0.16.10",
                "url": "https://maven.fabricmc.net/",
                "size": 100,
                "sha1": "abc"
            }]
        }"#).unwrap();

        assert_eq!(profile.main_class, "net.fabricmc.loader.impl.launch.knot.KnotClient");
        assert_eq!(profile.arguments.jvm, vec!["-Dfabric.example=true"]);
        assert_eq!(profile.libraries.len(), 1);
        assert!(profile.java_version.is_none());
    }

    #[tokio::test]
    async fn test_fetch_fabric_versions() {
        let client = Client::new();
        let versions = fetch_fabric_versions(&client).await;
        println!("Fabric versions result: {:?}", versions.is_ok());
    }
}