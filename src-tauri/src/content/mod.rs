//! Content (mod/modpack/resource pack/shader) marketplace integration.
//!
//! Provides provider-neutral types for browsing and installing Minecraft
//! content from Modrinth and CurseForge into a selected instance.

use crate::error::{LauncherError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Mod content provider.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContentProvider {
    Modrinth,
    CurseForge,
}

impl ContentProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentProvider::Modrinth => "modrinth",
            ContentProvider::CurseForge => "curseforge",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "modrinth" => Ok(ContentProvider::Modrinth),
            "curseforge" => Ok(ContentProvider::CurseForge),
            other => Err(LauncherError::InvalidArgument(format!(
                "Unknown content provider: {other}"
            ))),
        }
    }
}

/// Content category installed into an instance directory.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContentType {
    Mods,
    Modpacks,
    ResourcePacks,
    Shaders,
}

impl ContentType {
    /// Instance subfolder used for this content type.
    pub fn folder_name(&self) -> &'static str {
        match self {
            ContentType::Mods => "mods",
            ContentType::Modpacks => "modpacks",
            ContentType::ResourcePacks => "resourcepacks",
            ContentType::Shaders => "shaderpacks",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "mods" => Ok(ContentType::Mods),
            "modpacks" => Ok(ContentType::Modpacks),
            "resourcepacks" => Ok(ContentType::ResourcePacks),
            "shaderpacks" => Ok(ContentType::Shaders),
            other => Err(LauncherError::InvalidArgument(format!(
                "Unknown content type: {other}"
            ))),
        }
    }
}

/// Search result from a content provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentProject {
    pub provider: ContentProvider,
    pub project_id: String,
    pub slug: Option<String>,
    pub name: String,
    pub icon_url: Option<String>,
    pub downloads: Option<u64>,
    pub summary: Option<String>,
    pub project_type: String,
}

/// Downloadable version of a content project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersion {
    pub version_id: String,
    pub version_name: Option<String>,
    pub version_number: Option<String>,
    pub release_time: Option<String>,
    pub file_name: String,
    pub file_url: String,
    pub file_size: Option<u64>,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub primary_file: bool,
}

/// Search response from a content provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResponse {
    pub projects: Vec<ContentProject>,
    pub offset: usize,
    pub total_hits: Option<u64>,
}

/// Result of installing content into an instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentInstallResult {
    pub install_id: Uuid,
    pub instance_id: Uuid,
    pub content_type: ContentType,
    pub project_id: String,
    pub version_id: String,
    pub files_installed: Vec<String>,
    pub bytes_downloaded: u64,
}

/// Installed content record used for uninstall/update management.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledContent {
    pub install_id: Uuid,
    pub instance_id: Uuid,
    pub content_type: ContentType,
    pub provider: ContentProvider,
    pub project_id: String,
    pub version_id: String,
    pub path: String,
    pub file_name: String,
    pub file_size: u64,
    pub sha1: Option<String>,
    pub installed_at: String,
}

impl InstalledContent {
    pub fn new(
        install_id: Uuid,
        instance_id: Uuid,
        content_type: ContentType,
        provider: ContentProvider,
        project_id: String,
        version_id: String,
        path: PathBuf,
        file_name: String,
        file_size: u64,
        sha1: Option<String>,
    ) -> Self {
        Self {
            install_id,
            instance_id,
            content_type,
            provider,
            project_id,
            version_id,
            path: path.to_string_lossy().to_string(),
            file_name,
            file_size,
            sha1,
            installed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Content provider API used by the launcher.
#[async_trait::async_trait]
pub trait ContentApi: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(
        &self,
        query: &str,
        category: Option<&str>,
        game_version: &str,
        loader: Option<&str>,
        offset: usize,
    ) -> Result<Vec<ContentProject>>;
    async fn versions(
        &self,
        project_id: &str,
        game_version: &str,
        loader: Option<&str>,
    ) -> Result<Vec<ContentVersion>>;
}
