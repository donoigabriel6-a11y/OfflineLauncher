//! Shared types for the Modrinth / CurseForge content browser.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Content provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentProvider {
    Modrinth,
    CurseForge,
}

/// Content category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentCategory {
    Mods,
    Modpacks,
    Resourcepacks,
    Shaders,
}

impl ContentCategory {
    pub fn as_modrinth_category(&self) -> &'static str {
        match self {
            ContentCategory::Mods => "mods",
            ContentCategory::Modpacks => "modpacks",
            ContentCategory::Resourcepacks => "resourcepacks",
            ContentCategory::Shaders => "shaders",
        }
    }

    /// CurseForge category id for Minecraft (gameId 432). Some categories are
    /// best-effort; CurseForge groups resource packs and shaders under mods.
    pub fn as_curseforge_category_id(&self) -> Option<u32> {
        match self {
            ContentCategory::Mods => Some(6),
            ContentCategory::Modpacks => Some(4427),
            ContentCategory::Resourcepacks => None,
            ContentCategory::Shaders => None,
        }
    }
}

/// A search result project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentProject {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub downloads: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_versions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loaders: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResult {
    pub projects: Vec<ContentProject>,
    pub offset: usize,
    pub total_hits: usize,
}

/// A specific installable version of a project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_versions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loaders: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentInstallResult {
    pub install_id: String,
    pub project_id: String,
    pub version_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledContent {
    pub install_id: String,
    pub provider: ContentProvider,
    pub content_type: ContentCategory,
    pub project_id: String,
    pub version_id: String,
    pub version_number: String,
    pub title: String,
}

/// Command argument structs (deserialized from camelCase JSON sent by the frontend)

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchArgs {
    pub provider: String,
    pub content_type: String,
    pub query: String,
    pub minecraft_version: String,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionsArgs {
    pub provider: String,
    pub project_id: String,
    pub minecraft_version: String,
    #[serde(default)]
    pub loader: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentInstallArgs {
    pub instance_id: String,
    pub provider: String,
    pub content_type: String,
    pub project_id: String,
    pub version_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentUninstallArgs {
    pub instance_id: String,
    pub install_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeKeySetArgs {
    pub api_key: String,
}

impl ContentProvider {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "modrinth" => Ok(ContentProvider::Modrinth),
            "curseforge" => Ok(ContentProvider::CurseForge),
            other => Err(format!("Unknown provider: {other}")),
        }
    }
}

impl ContentCategory {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "mods" => Ok(ContentCategory::Mods),
            "modpacks" => Ok(ContentCategory::Modpacks),
            "resourcepacks" => Ok(ContentCategory::Resourcepacks),
            "shaders" => Ok(ContentCategory::Shaders),
            other => Err(format!("Unknown content type: {other}")),
        }
    }
}

impl InstalledContent {
    pub fn new(
        provider: ContentProvider,
        content_type: ContentCategory,
        project_id: String,
        version_id: String,
        version_number: String,
        title: String,
    ) -> Self {
        Self {
            install_id: Uuid::new_v4().to_string(),
            provider,
            content_type,
            project_id,
            version_id,
            version_number,
            title,
        }
    }
}
