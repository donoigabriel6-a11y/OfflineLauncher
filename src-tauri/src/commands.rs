//! Tauri commands module

use tauri::{State, AppHandle};
use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    AppState, Config, Account, OfflineAccount, VersionManifest, VersionEntry, VersionJson,
    DownloadStage,
    JavaInstallation, JavaSelection,
    Instance,
    LaunchArguments, launch_game,
    fetch_fabric_versions, fetch_fabric_loader_versions,
    fetch_quilt_versions, fetch_quilt_loader_versions,
    Result, LauncherError,
};
use tracing::{error, info};
use crate::accounts::model::validate_username;
use crate::accounts::uuid::generate_offline_uuid;
use crate::app_dirs::get_app_data_dir;
use crate::events::{
    emit_download_progress, emit_download_complete,
    emit_account_changed, emit_instance_changed, emit_launch_status,
    DownloadProgressEvent, DownloadCompleteEvent,
    AccountChangedEvent, AccountAction, AccountInfo,
    InstanceChangedEvent, InstanceAction, InstanceInfo,
    LaunchStatusEvent, LaunchStatus,
};
use crate::versions::inheritance::load_version_json;
use crate::versions::manifest::fetch_version_manifest;
use crate::launch::classpath::{build_classpath, extract_natives, get_natives_dir};
use crate::java::detect_system_java;

// ==================== CONFIG COMMANDS ====================

#[tauri::command]
pub async fn config_get(state: State<'_, Arc<AppState>>) -> Result<Config> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn config_set(state: State<'_, Arc<AppState>>, config: Config) -> Result<()> {
    config.save()?;
    let mut state_config = state.config.write().await;
    *state_config = config;
    Ok(())
}

#[tauri::command]
pub async fn config_open_data_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?)
}

// ==================== ACCOUNT COMMANDS ====================

#[tauri::command]
pub async fn accounts_list(state: State<'_, Arc<AppState>>) -> Result<Vec<OfflineAccount>> {
    let accounts = state.account_storage.list().await;
    let mut result = Vec::new();
    for acc in accounts {
        if let Some(offline) = acc.as_any().downcast_ref::<OfflineAccount>() {
            result.push(offline.clone());
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn accounts_create(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    username: String,
) -> Result<OfflineAccount> {
    validate_username(&username)?;

    // Check if username already exists
    if state.account_storage.get_by_username(&username).await.is_some() {
        return Err(LauncherError::InvalidArgument("Username already exists".into()));
    }

    let account = OfflineAccount::new(username)?;
    let account_clone = account.clone();
    state.account_storage.add(Box::new(account)).await?;

    // Emit event
    emit_account_changed(&app, AccountChangedEvent {
        action: AccountAction::Created,
        account: AccountInfo {
            id: account_clone.id().to_string(),
            username: account_clone.username().to_string(),
            uuid: account_clone.uuid().to_string(),
            account_type: "offline".to_string(),
        },
    })?;

    Ok(account_clone)
}

#[tauri::command]
pub async fn accounts_update(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    username: Option<String>,
    skin_path: Option<String>,
) -> Result<OfflineAccount> {
    let uuid = Uuid::parse_str(&id)?;
    let mut account = state.account_storage.get(uuid).await
        .ok_or(LauncherError::AccountNotFound(id))?;

    if let Some(username) = username {
        validate_username(&username)?;
        account.set_username(username).await?;
    }

    if let Some(skin_path) = skin_path {
        account.set_skin_path(Some(PathBuf::from(skin_path))).await?;
    }

    let account_clone = account.clone_box();
    state.account_storage.update(account).await?;

    // Emit event
    emit_account_changed(&app, AccountChangedEvent {
        action: AccountAction::Updated,
        account: AccountInfo {
            id: account_clone.id().to_string(),
            username: account_clone.username().to_string(),
            uuid: account_clone.uuid().to_string(),
            account_type: "offline".to_string(),
        },
    })?;

    // Downcast to return OfflineAccount
    account_clone.as_any()
        .downcast_ref::<OfflineAccount>()
        .cloned()
        .ok_or(LauncherError::InvalidArgument("Failed to downcast account".into()))
}

#[tauri::command]
pub async fn accounts_delete(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
) -> Result<()> {
    let uuid = Uuid::parse_str(&id)?;
    let account = state.account_storage.get(uuid).await
        .ok_or(LauncherError::AccountNotFound(id.clone()))?;

    let account_info = AccountInfo {
        id: account.id().to_string(),
        username: account.username().to_string(),
        uuid: account.uuid().to_string(),
        account_type: "offline".to_string(),
    };

    state.account_storage.delete(uuid).await?;

    // Emit event
    emit_account_changed(&app, AccountChangedEvent {
        action: AccountAction::Deleted,
        account: account_info,
    })?;

    Ok(())
}

#[tauri::command]
pub async fn accounts_generate_uuid(username: String) -> Result<String> {
    validate_username(&username)?;
    let uuid = generate_offline_uuid(&username);
    Ok(uuid.to_string())
}

// ==================== VERSION COMMANDS ====================

#[tauri::command]
pub async fn versions_fetch_manifest(state: State<'_, Arc<AppState>>) -> Result<VersionManifest> {
    fetch_version_manifest(&state.http_client).await
}

#[tauri::command]
pub async fn versions_list(
    state: State<'_, Arc<AppState>>,
    version_type: Option<String>,
) -> Result<Vec<VersionEntry>> {
    let manifest = versions_fetch_manifest(state.clone()).await?;
    let filter = version_type.and_then(|t| match t.as_str() {
        "release" => Some(crate::versions::model::VersionType::Release),
        "snapshot" => Some(crate::versions::model::VersionType::Snapshot),
        "old_beta" => Some(crate::versions::model::VersionType::OldBeta),
        "old_alpha" => Some(crate::versions::model::VersionType::OldAlpha),
        _ => None,
    });

    let versions = crate::versions::manifest::filter_versions(&manifest.versions, filter);
    Ok(versions)
}

#[tauri::command]
pub async fn versions_download(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    version_id: String,
) -> Result<()> {
    // A version is considered installed only after every required file has
    // completed successfully. Clear the marker before retrying a partial install.
    let install_dir = crate::app_dirs::get_version_dir(&version_id)?;
    tokio::fs::create_dir_all(&install_dir).await?;
    let install_marker = install_dir.join(".installed");
    let _ = tokio::fs::remove_file(&install_marker).await;

    let manifest = fetch_version_manifest(&state.http_client).await?;

    let version_entry = manifest.versions.iter()
        .find(|v| v.id == version_id)
        .ok_or(LauncherError::VersionNotFound(version_id.clone()))?;

    // Download version JSON
    let response = state.http_client.get(&version_entry.url)
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .send().await
        .map_err(|e| LauncherError::Http(e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(LauncherError::Download(format!(
            "Failed to download version JSON ({}): {}",
            status, text
        )));
    }

    let version_body = response.bytes().await
        .map_err(|e| LauncherError::Download(format!(
            "Failed reading Minecraft {} metadata from {}: {}",
            version_id, version_entry.url, e
        )))?;
    let version_json: VersionJson = serde_json::from_slice(&version_body)
        .map_err(|e| LauncherError::Download(format!(
            "Minecraft {} returned invalid version metadata from {}: {}. Response begins: {}",
            version_id,
            version_entry.url,
            e,
            String::from_utf8_lossy(&version_body[..version_body.len().min(240)])
        )))?;
    if version_json.id != version_id {
        return Err(LauncherError::Download(format!(
            "Mojang metadata ID mismatch: requested {}, received {}",
            version_id, version_json.id
        )));
    }

    // Save version JSON
    let version_dir = crate::app_dirs::get_version_dir(&version_id)?;
    tokio::fs::create_dir_all(&version_dir).await?;
    let json_path = version_dir.join(format!("{}.json", version_id));
    tokio::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?).await?;

    // Emit progress for version JSON
    emit_download_progress(&app, DownloadProgressEvent {
        version_id: version_id.clone(),
        file_name: format!("{}.json", version_id),
        downloaded: 1,
        total: 1,
        stage: DownloadStage::VersionJson,
    })?;

    // Download client jar
    let libraries_dir = crate::app_dirs::get_libraries_dir()?;
    let assets_dir = crate::app_dirs::get_assets_dir()?;

    // Create download tasks for client jar, libraries, and assets
    let mut download_tasks = Vec::new();

    // 1. Client jar
    {
        let client_download = &version_json.downloads.client;
        info!("Downloading client jar: {} -> {}", client_download.url, version_dir.join(format!("{}.jar", version_id)).display());
        let client_jar_path = version_dir.join(format!("{}.jar", version_id));
        download_tasks.push(crate::downloads::DownloadTask {
            url: client_download.url.clone(),
            destination: client_jar_path,
            expected_sha1: Some(client_download.sha1.clone()),
            file_name: format!("{}.jar", version_id),
            stage: DownloadStage::ClientJar,
            version_id: version_id.clone(),
        });
    }

    // 2. Libraries (filter by platform rules)
    let mut library_count = 0;
    for library in &version_json.libraries {
        if !crate::versions::library::evaluate_library_rules(&library.rules, true).unwrap_or(true) {
            continue;
        }

        if let Some(downloads) = &library.downloads {
            // Main artifact
            if let Some(artifact) = &downloads.artifact {
                let artifact_path = crate::launch::classpath::library_artifact_path(library, &libraries_dir)?;
                library_count += 1;
                download_tasks.push(crate::downloads::DownloadTask {
                    url: artifact.url.clone(),
                    destination: artifact_path,
                    expected_sha1: Some(artifact.sha1.clone()),
                    file_name: format!("{}:{}", library.name, artifact.url.split('/').last().unwrap_or("")),
                    stage: DownloadStage::Libraries,
                    version_id: version_id.clone(),
                });
            }

            // Classifiers (including natives)
            if let Some(classifiers) = &downloads.classifiers {
                for (classifier, download_info) in classifiers {
                    let path = crate::launch::classpath::library_native_path(library, &libraries_dir, classifier)?;
                    library_count += 1;
                    download_tasks.push(crate::downloads::DownloadTask {
                        url: download_info.url.clone(),
                        destination: path,
                        expected_sha1: Some(download_info.sha1.clone()),
                        file_name: format!("{}-{}", library.name, classifier),
                        stage: DownloadStage::Libraries,
                        version_id: version_id.clone(),
                    });
                }
            }
        }
    }
    info!("Queued {} library downloads", library_count);

    // 3. Assets index
    let asset_index_path = assets_dir.join("indexes").join(format!("{}.json", version_json.asset_index.id));
    info!("Downloading assets index: {} -> {}", version_json.asset_index.url, asset_index_path.display());
    download_tasks.push(crate::downloads::DownloadTask {
        url: version_json.asset_index.url.clone(),
        destination: asset_index_path,
        expected_sha1: Some(version_json.asset_index.sha1.clone()),
        file_name: format!("{}.json", version_json.asset_index.id),
        stage: DownloadStage::AssetsIndex,
        version_id: version_id.clone(),
    });

    // Execute all downloads
    let download_manager = state.download_manager.clone();
    info!("Starting {} download tasks", download_tasks.len());
    let results = download_manager.download_all(download_tasks, Some(&app)).await;

    // Handle results
    match results {
        Ok(_) => {
            // Now download actual assets from the assets index
            let assets_dir = crate::app_dirs::get_assets_dir()?;
            let asset_index_path = assets_dir.join("indexes").join(format!("{}.json", version_json.asset_index.id));

            // Read the downloaded asset index
            let asset_index_content = match tokio::fs::read_to_string(&asset_index_path).await {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to read asset index file: {}", e);
                    emit_download_complete(&app, DownloadCompleteEvent {
                        version_id: version_id.clone(),
                        success: false,
                        error: Some(e.to_string()),
                    })?;
                    return Err(LauncherError::Io(e));
                }
            };

            let asset_index: serde_json::Value = match serde_json::from_str(&asset_index_content) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse asset index JSON: {}", e);
                    emit_download_complete(&app, DownloadCompleteEvent {
                        version_id: version_id.clone(),
                        success: false,
                        error: Some(e.to_string()),
                    })?;
                    return Err(LauncherError::Serde(e));
                }
            };

            // Parse asset objects
            let objects = match asset_index.get("objects").and_then(|v| v.as_object()) {
                Some(obj) => obj,
                None => {
                    error!("Invalid assets index: missing objects");
                    let error = LauncherError::Launch("Invalid assets index: missing objects".into());
                    emit_download_complete(&app, DownloadCompleteEvent {
                        version_id: version_id.clone(),
                        success: false,
                        error: Some(error.to_string()),
                    })?;
                    return Err(error);
                }
            };

            info!("Found {} assets to download", objects.len());

            // Create download tasks for each asset
            let mut asset_tasks = Vec::new();
            let assets_objects_dir = assets_dir.join("objects");

            for (asset_path, obj_value) in objects {
                if let Some(obj) = obj_value.as_object() {
                    let hash = obj.get("hash").and_then(|v| v.as_str()).unwrap_or("");
                    if hash.len() != 40 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                        let error = LauncherError::Launch(format!(
                            "Invalid SHA-1 hash for asset '{}': {}",
                            asset_path, hash
                        ));
                        emit_download_complete(&app, DownloadCompleteEvent {
                            version_id: version_id.clone(),
                            success: false,
                            error: Some(error.to_string()),
                        })?;
                        return Err(error);
                    }

                    // Asset URL format: https://resources.download.minecraft.net/{first2}/{hash}
                    let url = format!("https://resources.download.minecraft.net/{}/{}", &hash[..2], hash);

                    // Destination: assets/objects/{first2}/{hash}
                    let dest_dir = assets_objects_dir.join(&hash[..2]);
                    let dest_path = dest_dir.join(hash);

                    asset_tasks.push(crate::downloads::DownloadTask {
                        url,
                        destination: dest_path,
                        expected_sha1: Some(hash.to_string()),
                        file_name: asset_path.clone(),
                        stage: DownloadStage::Assets,
                        version_id: version_id.clone(),
                    });
                }
            }

            // Download assets with progress
            if !asset_tasks.is_empty() {
                let download_manager = state.download_manager.clone();

                // Download in batches to avoid too many concurrent downloads
                info!("Downloading {} assets in batches of 50", asset_tasks.len());
                for chunk in asset_tasks.chunks(50) {
                    let chunk_vec = chunk.to_vec();
                    if let Err(e) = download_manager.download_all(chunk_vec, Some(&app)).await {
                        error!("Asset batch download failed: {}", e);
                        emit_download_complete(&app, DownloadCompleteEvent {
                            version_id: version_id.clone(),
                            success: false,
                            error: Some(e.to_string()),
                        })?;
                        return Err(e);
                    }
                }
            }

            // Write the completion marker only after the client, libraries,
            // asset index, and all asset objects have been downloaded and verified.
            tokio::fs::write(&install_marker, b"installed").await?;
            emit_download_complete(&app, DownloadCompleteEvent {
                version_id: version_id.clone(),
                success: true,
                error: None,
            })?;
            Ok(())
        }
        Err(e) => {
            error!("Download failed: {}", e);
            emit_download_complete(&app, DownloadCompleteEvent {
                version_id: version_id.clone(),
                success: false,
                error: Some(e.to_string()),
            })?;
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn versions_is_installed(version_id: String) -> Result<bool> {
    let version_dir = crate::app_dirs::get_version_dir(&version_id)?;
    Ok(tokio::fs::metadata(version_dir.join(".installed")).await.is_ok())
}

#[tauri::command]
pub async fn versions_get_version_json(version_id: String) -> Result<VersionJson> {
    let versions_dir = crate::app_dirs::get_versions_dir()?;
    load_version_json(&version_id, &versions_dir).await
}

// ==================== JAVA COMMANDS ====================

#[tauri::command]
pub async fn java_detect() -> Result<Vec<JavaInstallation>> {
    detect_system_java().await
}

#[tauri::command]
pub async fn java_list_runtimes(state: State<'_, Arc<AppState>>) -> Result<Vec<JavaSelection>> {
    state.java_manager.list_all().await
}

#[tauri::command]
pub async fn java_resolve(
    state: State<'_, Arc<AppState>>,
    required_major: u8,
    override_path: Option<PathBuf>,
) -> Result<JavaSelection> {
    state.java_manager.resolve_java(required_major, override_path.as_ref()).await
}

// ==================== INSTANCE COMMANDS ====================

#[tauri::command]
pub async fn instances_list(state: State<'_, Arc<AppState>>) -> Result<Vec<Instance>> {
    Ok(state.instance_storage.list().await)
}

#[tauri::command]
pub async fn instances_create(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    name: String,
    version_id: String,
) -> Result<Instance> {
    let instance = Instance::new(name, version_id)?;
    let instance_clone = instance.clone();
    state.instance_storage.add(instance).await?;

    emit_instance_changed(&app, InstanceChangedEvent {
        action: InstanceAction::Created,
        instance: InstanceInfo {
            id: instance_clone.id.to_string(),
            name: instance_clone.name.clone(),
            version_id: instance_clone.version_id.clone(),
            loader: Some(instance_clone.loader.as_str().to_string()),
        },
    })?;

    Ok(instance_clone)
}

#[tauri::command]
pub async fn instances_update(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    instance: Instance,
) -> Result<Instance> {
    let instance_clone = instance.clone();
    state.instance_storage.update(instance).await?;

    emit_instance_changed(&app, InstanceChangedEvent {
        action: InstanceAction::Updated,
        instance: InstanceInfo {
            id: instance_clone.id.to_string(),
            name: instance_clone.name.clone(),
            version_id: instance_clone.version_id.clone(),
            loader: Some(instance_clone.loader.as_str().to_string()),
        },
    })?;

    Ok(instance_clone)
}

#[tauri::command]
pub async fn instances_delete(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
) -> Result<()> {
    let uuid = Uuid::parse_str(&id)?;
    let instance = state.instance_storage.get(uuid).await
        .ok_or(LauncherError::InstanceNotFound(id.clone()))?;

    let instance_info = InstanceInfo {
        id: instance.id.to_string(),
        name: instance.name.clone(),
        version_id: instance.version_id.clone(),
        loader: Some(instance.loader.as_str().to_string()),
    };

    state.instance_storage.delete(uuid).await?;

    emit_instance_changed(&app, InstanceChangedEvent {
        action: InstanceAction::Deleted,
        instance: instance_info,
    })?;

    Ok(())
}

#[tauri::command]
pub async fn instances_clone(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    new_name: String,
) -> Result<Instance> {
    let uuid = Uuid::parse_str(&id)?;
    let cloned = state.instance_storage.clone(uuid, new_name).await?;

    emit_instance_changed(&app, InstanceChangedEvent {
        action: InstanceAction::Cloned,
        instance: InstanceInfo {
            id: cloned.id.to_string(),
            name: cloned.name.clone(),
            version_id: cloned.version_id.clone(),
            loader: Some(cloned.loader.as_str().to_string()),
        },
    })?;

    Ok(cloned)
}

#[tauri::command]
pub async fn instances_open_folder(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<()> {
    let uuid = Uuid::parse_str(&id)?;
    state.instance_storage.open_game_dir(uuid).await
}

// ==================== LAUNCH COMMANDS ====================

#[tauri::command]
pub async fn launch_start(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    instance_id: String,
    account_id: String,
) -> Result<()> {
    let instance_uuid = Uuid::parse_str(&instance_id)?;
    let account_uuid = Uuid::parse_str(&account_id)?;

    let instance = state.instance_storage.get(instance_uuid).await
        .ok_or(LauncherError::InstanceNotFound(instance_id))?;

    let account = state.account_storage.get(account_uuid).await
        .ok_or(LauncherError::AccountNotFound(account_id))?;

    // Resolve and validate the instance working directory before launch.
    // Older saved instances may still contain the legacy default "." path.
    let configured_game_dir = instance.effective_game_dir();
    let game_dir = if configured_game_dir.as_os_str().is_empty()
        || configured_game_dir == PathBuf::from(".")
    {
        crate::app_dirs::get_instance_game_dir(&instance.id.to_string())?
    } else {
        configured_game_dir
    };
    tokio::fs::create_dir_all(&game_dir).await.map_err(|e| {
        LauncherError::Launch(format!(
            "Could not create instance game directory '{}': {}",
            game_dir.display(), e
        ))
    })?;
    let game_dir = tokio::fs::canonicalize(&game_dir).await.map_err(|e| {
        LauncherError::Launch(format!(
            "Could not resolve instance game directory '{}': {}",
            game_dir.display(), e
        ))
    })?;
    if !tokio::fs::metadata(&game_dir).await?.is_dir() {
        return Err(LauncherError::Launch(format!(
            "Instance game directory is not a directory: '{}'",
            game_dir.display()
        )));
    }

    // Load version JSON first to get required Java version
    let versions_dir = crate::app_dirs::get_versions_dir()?;
    let version_dir = crate::app_dirs::get_version_dir(&instance.version_id)?;
    let version_json = load_version_json(&instance.version_id, &versions_dir).await?;

    // Resolve Java - use version's required Java version from version JSON
    let required_java = version_json.java_version.major_version;
    let java_selection = state.java_manager.resolve_java(
        required_java,
        instance.java.path.as_ref(),
    ).await?;

    // Build classpath
    let libraries_dir = crate::app_dirs::get_libraries_dir()?;
    let classpath = build_classpath(&version_json, &libraries_dir, &version_dir)?;

    // Extract natives
    let natives_dir = get_natives_dir(&version_dir, &instance.version_id);
    extract_natives(&version_json, &libraries_dir, &natives_dir).await?;

    // Build launch arguments
    let assets_dir = crate::app_dirs::get_assets_dir()?;
    let launch_args = LaunchArguments::from_version(
        &version_json,
        account.as_ref(),
        &game_dir,
        &assets_dir,
        &libraries_dir,
        &version_json.asset_index.id,
        &natives_dir,
        &classpath,
        "OfflineLauncher",
        env!("CARGO_PKG_VERSION"),
    )?;

    // Combine JVM args
    let mut jvm_args = launch_args.jvm_args;

    // Add config default JVM args
    let config = state.config.read().await;
    jvm_args.extend(
        config.default_jvm_args
            .split_whitespace()
            .map(String::from)
    );
    drop(config);

    jvm_args.extend(
        instance.java.jvm_args
            .split_whitespace()
            .map(String::from)
    );
    jvm_args.extend(
        instance.jvm_args
            .split_whitespace()
            .map(String::from)
    );

    // Memory settings
    jvm_args.push(format!("-Xms{}m", instance.memory.min_mb));
    jvm_args.push(format!("-Xmx{}m", instance.memory.max_mb));

    // Deduplicate GC flags to avoid "Multiple garbage collectors selected" error
    crate::launch::arguments::deduplicate_gc_flags(&mut jvm_args);

    // Combine game args
    let mut game_args = launch_args.game_args;
    game_args.extend(
        instance.game_args
            .split_whitespace()
            .map(String::from)
    );

    // Launch game
    let game_process = launch_game(
        app,
        instance_uuid,
        &java_selection.path,
        jvm_args,
        launch_args.main_class,
        game_args,
        &game_dir,
    ).await?;

    // Store the launch handle for kill functionality
    {
        let mut processes = state.running_processes.lock().await;
        processes.insert(instance_uuid, game_process.handle().clone());
    }

    // Update last played
    state.instance_storage.update({
        let mut inst = instance;
        inst.touch_last_played();
        inst
    }).await?;

    Ok(())
}

#[tauri::command]
pub async fn launch_kill(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    instance_id: String,
) -> Result<()> {
    let instance_uuid = Uuid::parse_str(&instance_id)?;

    let handle = {
        let mut processes = state.running_processes.lock().await;
        processes.remove(&instance_uuid)
    };

    if let Some(handle) = handle {
        if handle.is_running().await {
            let pid = handle.pid;
            handle.kill().await?;
            emit_launch_status(&app, LaunchStatusEvent {
                instance_id,
                status: LaunchStatus::Killed,
                exit_code: None,
                pid,
            })?;
        }
    }

    Ok(())
}

// ==================== LOADER COMMANDS ====================

#[tauri::command]
pub async fn loaders_fabric_versions(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::loaders::fabric::FabricVersion>> {
    fetch_fabric_versions(&state.http_client).await
}

#[tauri::command]
pub async fn loaders_fabric_loader_versions(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::loaders::fabric::FabricLoaderVersion>> {
    fetch_fabric_loader_versions(&state.http_client).await
}

#[tauri::command]
pub async fn loaders_quilt_versions(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::loaders::quilt::QuiltVersion>> {
    fetch_quilt_versions(&state.http_client).await
}

#[tauri::command]
pub async fn loaders_quilt_loader_versions(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::loaders::quilt::QuiltLoaderVersion>> {
    fetch_quilt_loader_versions(&state.http_client).await
}