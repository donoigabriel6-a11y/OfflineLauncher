//! Version manifest fetching and caching

use crate::versions::model::{VersionManifest, VersionEntry, VersionType};
use crate::error::{LauncherError, Result};
use reqwest::Client;
use std::time::Duration;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Fetch the official version manifest from Mojang
pub async fn fetch_version_manifest(client: &Client) -> Result<VersionManifest> {
    let response = client
        .get(MANIFEST_URL)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        return Err(LauncherError::Download(format!("Failed to fetch manifest: {}", response.status())));
    }

    let manifest: VersionManifest = response.json().await
        .map_err(|e| LauncherError::Http(e))?;

    Ok(manifest)
}

/// Filter versions by type
pub fn filter_versions(versions: &[VersionEntry], version_type: Option<VersionType>) -> Vec<VersionEntry> {
    match version_type {
        Some(t) => versions.iter()
            .filter(|v| v.type_ == t)
            .cloned()
            .collect(),
        None => versions.to_vec(),
    }
}

/// Get latest release version
pub fn get_latest_release(manifest: &VersionManifest) -> Option<&VersionEntry> {
    manifest.versions.iter()
        .find(|v| v.type_ == VersionType::Release && v.id == manifest.latest.release)
}

/// Get latest snapshot version
pub fn get_latest_snapshot(manifest: &VersionManifest) -> Option<&VersionEntry> {
    manifest.versions.iter()
        .find(|v| v.type_ == VersionType::Snapshot && v.id == manifest.latest.snapshot)
}