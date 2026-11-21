//! API clients for Modrinth and CurseForge content providers.

use crate::content::types::{
    ContentCategory, ContentProject, ContentProvider, ContentSearchResult, ContentVersion,
};
use crate::error::{LauncherError, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";
const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const CURSEFORGE_GAME_ID: u32 = 432; // Minecraft

// ---------------------------------------------------------------------------
// Modrinth
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
    offset: usize,
    total_hits: usize,
}

#[derive(Debug, Deserialize)]
struct ModrinthHit {
    project_id: String,
    title: String,
    description: String,
    author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    downloads: u64,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    id: String,
    name: String,
    version_number: String,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    files: Vec<ModrinthFile>,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary: Option<bool>,
}

pub async fn modrinth_search(
    client: &Client,
    query: &str,
    category: &ContentCategory,
    minecraft_version: &str,
    loader: Option<&str>,
    offset: usize,
) -> Result<ContentSearchResult> {
    let mut facet_groups: Vec<Vec<String>> = Vec::new();

    if let Some(loader) = loader.filter(|l| !l.is_empty()) {
        facet_groups.push(vec![format!("loader:{loader}")]);
    }
    if !minecraft_version.is_empty() {
        facet_groups.push(vec![format!("game_versions:{minecraft_version}")]);
    }
    facet_groups.push(vec![format!(
        "categories:{}",
        category.as_modrinth_category()
    )]);

    let facets_json = serde_json::to_string(&facet_groups).map_err(LauncherError::Serde)?;

    let mut params = HashMap::new();
    params.insert("query", query);
    params.insert("facets", &facets_json);
    params.insert("index", "relevance");
    params.insert("limit", "20");
    params.insert("offset", offset.to_string().as_str());
    params.insert(
        "categories",
        &format!("[\"{}\"]", category.as_modrinth_category()),
    );

    let url = format!("{MODRINTH_API}/search");
    let response: ModrinthSearchResponse = client
        .get(&url)
        .query(&params)
        .header("User-Agent", "OfflineLauncher")
        .send()
        .await
        .map_err(LauncherError::Http)?
        .error_for_status()
        .map_err(LauncherError::Http)?
        .json()
        .await?;

    let projects = response
        .hits
        .into_iter()
        .map(|h| ContentProject {
            id: h.project_id,
            title: h.title,
            description: h.description,
            author: h.author,
            icon_url: h.icon_url,
            downloads: h.downloads,
            game_versions: if h.game_versions.is_empty() {
                None
            } else {
                Some(h.game_versions)
            },
            loaders: if h.loaders.is_empty() {
                None
            } else {
                Some(h.loaders)
            },
        })
        .collect();

    Ok(ContentSearchResult { projects, offset, total_hits: response.total_hits })
}

pub async fn modrinth_versions(
    client: &Client,
    project_id: &str,
    minecraft_version: &str,
    loader: Option<&str>,
) -> Result<Vec<ContentVersion>> {
    let mut params = HashMap::new();
    if !minecraft_version.is_empty() {
        params.insert("game_versions", &format!("[\"{minecraft_version}\"]"));
    }
    if let Some(loader) = loader.filter(|l| !l.is_empty()) {
        params.insert("loaders", &format!("[\"{loader}\"]"));
    }

    let url = format!("{MODRINTH_API}/project/{project_id}/version");
    let versions: Vec<ModrinthVersion> = client
        .get(&url)
        .query(&params)
        .header("User-Agent", "OfflineLauncher")
        .send()
        .await
        .map_err(LauncherError::Http)?
        .error_for_status()
        .map_err(LauncherError::Http)?
        .json()
        .await?;

    Ok(versions
        .into_iter()
        .map(|v| ContentVersion {
            id: v.id,
            name: v.name,
            version_number: v.version_number,
            game_versions: if v.game_versions.is_empty() {
                None
            } else {
                Some(v.game_versions)
            },
            loaders: if v.loaders.is_empty() {
                None
            } else {
                Some(v.loaders)
            },
        })
        .collect())
}

pub async fn modrinth_version_file(
    client: &Client,
    project_id: &str,
    version_id: &str,
) -> Result<(String, String)> {
    let url = format!("{MODRINTH_API}/project/{project_id}/version/{version_id}");
    let version: ModrinthVersion = client
        .get(&url)
        .header("User-Agent", "OfflineLauncher")
        .send()
        .await
        .map_err(LauncherError::Http)?
        .error_for_status()
        .map_err(LauncherError::Http)?
        .json()
        .await?;

    let file = version
        .files
        .into_iter()
        .find(|f| f.primary.unwrap_or(false))
        .or_else(|| version.files.into_iter().next())
        .ok_or_else(|| LauncherError::NotFound(format!("No files for version {version_id}")))?;

    Ok((file.url, file.filename))
}

// ---------------------------------------------------------------------------
// CurseForge
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CurseforgeSearchResponse {
    data: Vec<CurseforgeMod>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeMod {
    id: u32,
    name: String,
    summary: String,
    authors: Vec<CurseforgeAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo: Option<CurseforgeLogo>,
    download_count: u64,
}

#[derive(Debug, Deserialize)]
struct CurseforgeAuthor {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CurseforgeLogo {
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeFilesResponse {
    data: Vec<CurseforgeFile>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeFile {
    id: u32,
    name: String,
    display_name: String,
    file_name: String,
    #[serde(default)]
    game_versions: Vec<String>,
    download_url: String,
}

pub async fn curseforge_search(
    client: &Client,
    api_key: &str,
    query: &str,
    category: &ContentCategory,
    minecraft_version: &str,
    offset: usize,
) -> Result<ContentSearchResult> {
    let page_index = offset / 20;

    let mut params: HashMap<String, String> = HashMap::new();
    params.insert("gameId".to_string(), CURSEFORGE_GAME_ID.to_string());
    params.insert("searchFilter".to_string(), query.to_string());
    params.insert("pageSize".to_string(), "20".to_string());
    params.insert("pageIndex".to_string(), page_index.to_string());

    if let Some(category_id) = category.as_curseforge_category_id() {
        params.insert("categoryId".to_string(), category_id.to_string());
    }
    if !minecraft_version.is_empty() {
        params.insert("gameVersion".to_string(), minecraft_version.to_string());
    }

    let url = format!("{CURSEFORGE_API}/mods/search");
    let response: CurseforgeSearchResponse = client
        .get(&url)
        .query(&params)
        .header("x-api-key", api_key)
        .header("User-Agent", "OfflineLauncher")
        .send()
        .await
        .map_err(LauncherError::Http)?
        .error_for_status()
        .map_err(LauncherError::Http)?
        .json()
        .await?;

    let projects = response
        .data
        .into_iter()
        .map(|m| ContentProject {
            id: m.id.to_string(),
            title: m.name,
            description: m.summary,
            author: m
                .authors
                .first()
                .map(|a| a.name.clone())
                .unwrap_or_default(),
            icon_url: m.logo.and_then(|l| l.thumbnail_url),
            downloads: m.download_count,
            game_versions: None,
            loaders: None,
        })
        .collect();

    Ok(ContentSearchResult { projects, offset, total_hits: projects.len() })
}

pub async fn curseforge_versions(
    client: &Client,
    api_key: &str,
    project_id: &str,
    minecraft_version: &str,
    _loader: Option<&str>,
) -> Result<Vec<ContentVersion>> {
    let mut params: HashMap<String, String> = HashMap::new();
    if !minecraft_version.is_empty() {
        params.insert("gameVersion".to_string(), minecraft_version.to_string());
    }
    params.insert("pageSize".to_string(), "100".to_string());
    params.insert("pageIndex".to_string(), "0".to_string());

    let url = format!("{CURSEFORGE_API}/mods/{project_id}/files");
    let response: CurseforgeFilesResponse = client
        .get(&url)
        .query(&params)
        .header("x-api-key", api_key)
        .header("User-Agent", "OfflineLauncher")
        .send()
        .await
        .map_err(LauncherError::Http)?
        .error_for_status()
        .map_err(LauncherError::Http)?
        .json()
        .await?;

    Ok(response
        .data
        .into_iter()
        .map(|f| ContentVersion {
            id: f.id.to_string(),
            name: f.display_name,
            version_number: f.name,
            game_versions: if f.game_versions.is_empty() {
                None
            } else {
                Some(f.game_versions)
            },
            loaders: None,
        })
        .collect())
}

pub async fn curseforge_version_file(
    client: &Client,
    api_key: &str,
    project_id: &str,
    version_id: &str,
) -> Result<(String, String)> {
    let file_id: u32 = version_id.parse().map_err(|_| {
        LauncherError::NotFound(format!("Invalid CurseForge file id: {version_id}"))
    })?;

    let url = format!("{CURSEFORGE_API}/mods/{project_id}/files/{file_id}");
    #[derive(Debug, Deserialize)]
    struct CurseforgeFileResponse {
        data: CurseforgeFile,
    }
    let response: CurseforgeFileResponse = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("User-Agent", "OfflineLauncher")
        .send()
        .await
        .map_err(LauncherError::Http)?
        .error_for_status()
        .map_err(LauncherError::Http)?
        .json()
        .await?;

    Ok((response.data.download_url, response.data.file_name))
}

// ---------------------------------------------------------------------------
// Shared download helper
// ---------------------------------------------------------------------------

pub async fn download_content_file(
    client: &Client,
    url: &str,
    destination: std::path::PathBuf,
) -> Result<()> {
    let response = client
        .get(url)
        .header("User-Agent", "OfflineLauncher")
        .send()
        .await
        .map_err(LauncherError::Http)?
        .error_for_status()
        .map_err(LauncherError::Http)?;

    let mut stream = response.bytes_stream();
    use futures::StreamExt;
    let mut file = tokio::fs::File::create(&destination)
        .await
        .map_err(LauncherError::Io)?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(LauncherError::Http)?;
        file.write_all(&chunk).await.map_err(LauncherError::Io)?;
    }

    file.flush().await.map_err(LauncherError::Io)?;
    Ok(())
}

/// Resolve a provider enum from a string.
pub fn provider_from_str(s: &str) -> Result<ContentProvider> {
    match s {
        "modrinth" => Ok(ContentProvider::Modrinth),
        "curseforge" => Ok(ContentProvider::CurseForge),
        other => Err(LauncherError::InvalidArgument(format!(
            "Unknown provider: {other}"
        ))),
    }
}

pub fn content_type_from_str(s: &str) -> Result<ContentCategory> {
    match s {
        "mods" => Ok(ContentCategory::Mods),
        "modpacks" => Ok(ContentCategory::Modpacks),
        "resourcepacks" => Ok(ContentCategory::Resourcepacks),
        "shaders" => Ok(ContentCategory::Shaders),
        other => Err(LauncherError::InvalidArgument(format!(
            "Unknown content type: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_from_str_works() {
        assert!(matches!(
            provider_from_str("modrinth").unwrap(),
            ContentProvider::Modrinth
        ));
        assert!(matches!(
            provider_from_str("curseforge").unwrap(),
            ContentProvider::CurseForge
        ));
        assert!(provider_from_str("invalid").is_err());
    }

    #[test]
    fn content_type_from_str_works() {
        assert!(matches!(
            content_type_from_str("mods").unwrap(),
            ContentCategory::Mods
        ));
        assert!(content_type_from_str("invalid").is_err());
    }
}
