//! Tauri commands for the Modrinth / CurseForge content browser.

use crate::app_dirs::get_instance_game_dir;
use crate::content::client::{
    content_type_from_str, curseforge_search, curseforge_version_file, curseforge_versions,
    download_content_file, modrinth_search, modrinth_version_file, modrinth_versions,
    provider_from_str,
};
use crate::content::storage::ContentStorage;
use crate::content::types::{
    ContentCategory, ContentInstallArgs, ContentInstallResult, ContentSearchArgs,
    ContentSearchResult, ContentUninstallArgs, ContentVersion, ContentVersionsArgs,
    CurseforgeKeySetArgs, InstalledContent,
};
use crate::error::{LauncherError, Result};
use serde::Deserialize;
use tauri::AppHandle;
use tokio::fs;

/// Application state extended with content storage (added by the setup hook).
pub struct ContentState {
    pub storage: ContentStorage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentFileResult {
    url: String,
    file_name: String,
}

/// Parse a string provider/category into enums, validating both.
fn parse_provider_and_type(
    provider: &str,
    content_type: &str,
) -> Result<(crate::content::types::ContentProvider, ContentCategory)> {
    let provider = provider_from_str(provider)?;
    let category = content_type_from_str(content_type)?;
    Ok((provider, category))
}

/// Install a content file into an instance's mod-like directory.
async fn install_file_to_instance(
    http_client: &reqwest::Client,
    content_storage: &ContentStorage,
    instance_id: &str,
    provider: &str,
    content_type: &str,
    project_id: &str,
    version_id: &str,
    title: &str,
    download_result: ContentFileResult,
) -> Result<ContentInstallResult> {
    let instance_dir = get_instance_game_dir(instance_id)?;
    fs::create_dir_all(&instance_dir)
        .await
        .map_err(LauncherError::Io)?;

    // Content subdirectory: content/{provider}/{content_type}/{project_id}/
    let content_dir = instance_dir
        .join("content")
        .join(provider)
        .join(content_type)
        .join(project_id);
    fs::create_dir_all(&content_dir)
        .await
        .map_err(LauncherError::Io)?;

    let destination = content_dir.join(&download_result.file_name);
    if destination.exists() {
        fs::remove_file(&destination)
            .await
            .map_err(LauncherError::Io)?;
    }

    download_content_file(http_client, &download_result.url, destination.clone()).await?;

    let installed = InstalledContent::new(
        provider_from_str(provider)?,
        content_type_from_str(content_type)?,
        project_id.to_string(),
        version_id.to_string(),
        version_id.to_string(),
        title.to_string(),
    );
    content_storage
        .add(instance_id.to_string(), installed.clone())
        .await?;

    Ok(ContentInstallResult {
        install_id: installed.install_id,
        project_id: project_id.to_string(),
        version_id: version_id.to_string(),
        title: title.to_string(),
        file_name: Some(download_result.file_name),
    })
}

#[tauri::command]
pub async fn content_search(
    state: tauri::State<'_, crate::AppState>,
    args: ContentSearchArgs,
) -> Result<ContentSearchResult> {
    let (provider, category) = parse_provider_and_type(&args.provider, &args.content_type)?;
    let client = &state.http_client;

    let result = match provider {
        crate::content::types::ContentProvider::Modrinth => {
            modrinth_search(
                client,
                &args.query,
                &category,
                &args.minecraft_version,
                args.loader.as_deref(),
                args.offset.unwrap_or(0),
            )
            .await?
        },
        crate::content::types::ContentProvider::CurseForge => {
            let api_key = state
                .content_state
                .storage
                .curseforge_key_get()
                .await?
                .ok_or_else(|| {
                    LauncherError::InvalidArgument(
                        "CurseForge API key not configured. Set it in Settings.".into(),
                    )
                })?;
            curseforge_search(
                client,
                &api_key,
                &args.query,
                &category,
                &args.minecraft_version,
                args.offset.unwrap_or(0),
            )
            .await?
        },
    };

    Ok(result)
}

#[tauri::command]
pub async fn content_versions(
    state: tauri::State<'_, crate::AppState>,
    args: ContentVersionsArgs,
) -> Result<Vec<ContentVersion>> {
    let (provider, _category) = parse_provider_and_type(&args.provider, "mods")?;
    let client = &state.http_client;

    match provider {
        crate::content::types::ContentProvider::Modrinth => {
            modrinth_versions(
                client,
                &args.project_id,
                &args.minecraft_version,
                args.loader.as_deref(),
            )
            .await
        },
        crate::content::types::ContentProvider::CurseForge => {
            let api_key = state
                .content_state
                .storage
                .curseforge_key_get()
                .await?
                .ok_or_else(|| {
                    LauncherError::InvalidArgument(
                        "CurseForge API key not configured. Set it in Settings.".into(),
                    )
                })?;
            curseforge_versions(
                client,
                &api_key,
                &args.project_id,
                &args.minecraft_version,
                args.loader.as_deref(),
            )
            .await
        },
    }
}

#[tauri::command]
pub async fn content_install(
    state: tauri::State<'_, crate::AppState>,
    app: AppHandle,
    args: ContentInstallArgs,
) -> Result<ContentInstallResult> {
    let (provider, category) = parse_provider_and_type(&args.provider, &args.content_type)?;
    let client = &state.http_client;

    // Fetch version metadata to get the file URL/name and title.
    let (title, file_info): (String, ContentFileResult) = match provider {
        crate::content::types::ContentProvider::Modrinth => {
            let (url, file_name) =
                modrinth_version_file(client, &args.project_id, &args.version_id).await?;
            // Lookup the project title from versions or project metadata
            let versions =
                modrinth_versions(client, &args.project_id, &args.minecraft_version, None).await?;
            let title = versions
                .iter()
                .find(|v| v.id == args.version_id)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| args.project_id.clone());
            (title, ContentFileResult { url, file_name })
        },
        crate::content::types::ContentProvider::CurseForge => {
            let api_key = state
                .content_state
                .storage
                .curseforge_key_get()
                .await?
                .ok_or_else(|| {
                    LauncherError::InvalidArgument(
                        "CurseForge API key not configured. Set it in Settings.".into(),
                    )
                })?;
            let (url, file_name) =
                curseforge_version_file(client, &api_key, &args.project_id, &args.version_id)
                    .await?;
            let versions = curseforge_versions(
                client,
                &api_key,
                &args.project_id,
                &args.minecraft_version,
                None,
            )
            .await?;
            let title = versions
                .iter()
                .find(|v| v.id == args.version_id)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| args.project_id.clone());
            (title, ContentFileResult { url, file_name })
        },
    };

    install_file_to_instance(
        client,
        &state.content_state.storage,
        &args.instance_id,
        &args.provider,
        &args.content_type,
        &args.project_id,
        &args.version_id,
        &title,
        file_info,
    )
    .await
}

#[tauri::command]
pub async fn content_list_installed(
    state: tauri::State<'_, crate::AppState>,
    instance_id: String,
) -> Result<Vec<InstalledContent>> {
    Ok(state.content_state.storage.list(&instance_id).await)
}

#[tauri::command]
pub async fn content_uninstall(
    state: tauri::State<'_, crate::AppState>,
    args: ContentUninstallArgs,
) -> Result<()> {
    // Verify instance exists.
    let instance_uuid = uuid::Uuid::parse_str(&args.instance_id)
        .map_err(|_| LauncherError::InstanceNotFound(args.instance_id.clone()))?;
    state
        .instance_storage
        .get(instance_uuid)
        .await
        .ok_or(LauncherError::InstanceNotFound(args.instance_id.clone()))?;

    // Remove the installed-content record. The downloaded file itself is left on
    // disk (orphan) since we don't track the exact file name; fine for v1.
    state
        .content_state
        .storage
        .remove(&args.instance_id, &args.install_id)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn curseforge_key_status(state: tauri::State<'_, crate::AppState>) -> Result<bool> {
    state.content_state.storage.curseforge_key_status().await
}

#[tauri::command]
pub async fn curseforge_key_set(
    state: tauri::State<'_, crate::AppState>,
    args: CurseforgeKeySetArgs,
) -> Result<()> {
    state
        .content_state
        .storage
        .curseforge_key_set(&args.api_key)
        .await
}

#[tauri::command]
pub async fn curseforge_key_clear(state: tauri::State<'_, crate::AppState>) -> Result<()> {
    state.content_state.storage.curseforge_key_clear().await
}
