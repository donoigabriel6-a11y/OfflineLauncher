//! OfflineLauncher - Main entry point

use tauri::{Manager, State, AppHandle, Emitter};
use std::sync::Arc;

mod lib;
use lib::{AppState, initialize_app, Result, LauncherError};
use lib::{
    Config,
    Account, AccountStorage, OfflineAccount, AccountType, UserType,
    VersionManifest, VersionEntry, VersionJson, fetch_version_manifest,
    DownloadManager, DownloadOptions, DownloadTask, DownloadStage,
    JavaManager, JavaInstallation, JavaSelection,
    InstanceStorage, Instance, ModLoader, MemorySettings, JavaSettings,
    LaunchArguments, launch_game,
    fetch_fabric_versions, fetch_fabric_loader_versions, install_fabric,
    fetch_quilt_versions, fetch_quilt_loader_versions, install_quilt,
};
use crate::accounts::model::validate_username;
use crate::accounts::uuid::generate_offline_uuid;
use crate::app_dirs::get_app_data_dir;
use crate::events::{
    emit_download_progress, emit_download_complete,
    emit_launch_output, emit_launch_status,
    emit_account_changed, emit_instance_changed,
    DownloadProgressEvent, DownloadCompleteEvent,
    LaunchOutputEvent, LaunchStatusEvent, LaunchStatus, OutputStream,
    AccountChangedEvent, AccountAction, AccountInfo,
    InstanceChangedEvent, InstanceAction, InstanceInfo,
};
use crate::versions::library::get_native_classifier;
use crate::versions::inheritance::load_version_json;
use crate::launch::classpath::{build_classpath, extract_natives, get_natives_dir};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;

// ==================== CONFIG COMMANDS ====================

#[tauri::command]
async fn config_get(state: State<'_, Arc<AppState>>) -> Result<Config> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
async fn config_set(state: State<'_, Arc<AppState>>, config: Config) -> Result<()> {
    config.save()?;
    let mut state_config = state.config.write().await;
    *state_config = config;
    Ok(())
}

#[tauri::command]
async fn config_open_data_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?)
}

// ==================== ACCOUNT COMMANDS ====================

#[tauri::command]
async fn accounts_list(state: State<'_, Arc<AppState>>) -> Result<Vec<OfflineAccount>> {
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
async fn accounts_create(
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
async fn accounts_update(
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

    let account_clone = account.clone();
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

    Ok(account_clone)
}

#[tauri::command]
async fn accounts_delete(
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
async fn accounts_generate_uuid(username: String) -> Result<String> {
    validate_username(&username)?;
    let uuid = generate_offline_uuid(&username);
    Ok(uuid.to_string())
}

// ==================== VERSION COMMANDS ====================

#[tauri::command]
async fn versions_fetch_manifest() -> Result<VersionManifest> {
    let client = reqwest::Client::new();
    fetch_version_manifest(&client).await
}

#[tauri::command]
async fn versions_list(
    state: State<'_, Arc<AppState>>,
    version_type: Option<String>,
) -> Result<Vec<VersionEntry>> {
    let manifest = versions_fetch_manifest().await?;
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
async fn versions_download(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    version_id: String,
) -> Result<()> {
    let client = reqwest::Client::new();
    let manifest = fetch_version_manifest(&client).await?;

    let version_entry = manifest.versions.iter()
        .find(|v| v.id == version_id)
        .ok_or(LauncherError::VersionNotFound(version_id.clone()))?;

    // Download version JSON
    let version_json: VersionJson = client.get(&version_entry.url)
        .send().await?
        .json().await?;

    // Save version JSON
    let version_dir = crate::app_dirs::get_version_dir(&version_id)?;
    tokio::fs::create_dir_all(&version_dir).await?;
    let json_path = version_dir.join(format!("{}.json", version_id));
    tokio::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?).await?;

    // Emit progress
    emit_download_progress(&app, DownloadProgressEvent {
        version_id: version_id.clone(),
        file_name: format!("{}.json", version_id),
        downloaded: 1,
        total: 1,
        stage: DownloadStage::VersionJson,
    })?;

    // TODO: Download client jar, libraries, assets, natives
    // This is a simplified version - full implementation in milestone (c)

    emit_download_complete(&app, DownloadCompleteEvent {
        version_id,
        success: true,
        error: None,
    })?;

    Ok(())
}

#[tauri::command]
async fn versions_get_version_json(version_id: String) -> Result<VersionJson> {
    let version_dir = crate::app_dirs::get_version_dir(&version_id)?;
    load_version_json(&version_id, &version_dir).await
}

// ==================== JAVA COMMANDS ====================

#[tauri::command]
async fn java_detect() -> Result<Vec<JavaInstallation>> {
    detect_system_java().await
}

#[tauri::command]
async fn java_list_runtimes(state: State<'_, Arc<AppState>>) -> Result<Vec<JavaSelection>> {
    state.java_manager.list_all().await
}

#[tauri::command]
async fn java_resolve(
    state: State<'_, Arc<AppState>>,
    required_major: u8,
    override_path: Option<PathBuf>,
) -> Result<JavaSelection> {
    state.java_manager.resolve_java(required_major, override_path.as_ref()).await
}

// ==================== INSTANCE COMMANDS ====================

#[tauri::command]
async fn instances_list(state: State<'_, Arc<AppState>>) -> Result<Vec<Instance>> {
    Ok(state.instance_storage.list().await)
}

#[tauri::command]
async fn instances_create(
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
async fn instances_update(
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
async fn instances_delete(
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
async fn instances_clone(
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
async fn instances_open_folder(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<()> {
    let uuid = Uuid::parse_str(&id)?;
    state.instance_storage.open_game_dir(uuid).await
}

// ==================== LAUNCH COMMANDS ====================

#[tauri::command]
async fn launch_start(
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

    // Resolve Java
    let required_java = instance.java.major_version.unwrap_or(21); // Default to 21
    let java_selection = state.java_manager.resolve_java(
        required_java,
        instance.java.path.as_ref(),
    ).await?;

    // Load version JSON
    let version_dir = crate::app_dirs::get_version_dir(&instance.version_id)?;
    let version_json = load_version_json(&instance.version_id, &version_dir).await?;

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
        &instance.effective_game_dir(),
        &assets_dir,
        &version_json.asset_index.id,
        &natives_dir,
        &classpath,
        "OfflineLauncher",
        env!("CARGO_PKG_VERSION"),
    )?;

    // Combine JVM args
    let mut jvm_args = launch_args.jvm_args;
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

    // Combine game args
    let mut game_args = launch_args.game_args;
    game_args.extend(
        instance.game_args
            .split_whitespace()
            .map(String::from)
    );

    // Launch game
    let _process = launch_game(
        app,
        instance_uuid,
        &java_selection.path,
        jvm_args,
        launch_args.main_class,
        game_args,
        &instance.effective_game_dir(),
    ).await?;

    // Update last played
    state.instance_storage.update({
        let mut inst = instance;
        inst.touch_last_played();
        inst
    }).await?;

    Ok(())
}

#[tauri::command]
async fn launch_kill(
    state: State<'_, Arc<AppState>>,
    instance_id: String,
) -> Result<()> {
    // In a real implementation, we'd track running processes
    // For now, this is a stub
    Ok(())
}

// ==================== LOADER COMMANDS ====================

#[tauri::command]
async fn loaders_fabric_versions() -> Result<Vec<crate::loaders::fabric::FabricVersion>> {
    let client = reqwest::Client::new();
    fetch_fabric_versions(&client).await
}

#[tauri::command]
async fn loaders_fabric_loader_versions() -> Result<Vec<crate::loaders::fabric::FabricLoaderVersion>> {
    let client = reqwest::Client::new();
    fetch_fabric_loader_versions(&client).await
}

#[tauri::command]
async fn loaders_quilt_versions() -> Result<Vec<crate::loaders::quilt::QuiltVersion>> {
    let client = reqwest::Client::new();
    fetch_quilt_versions(&client).await
}

#[tauri::command]
async fn loaders_quilt_loader_versions() -> Result<Vec<crate::loaders::quilt::QuiltLoaderVersion>> {
    let client = reqwest::Client::new();
    fetch_quilt_loader_versions(&client).await
}

// ==================== MAIN ====================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = initialize_app(&app_handle).await {
                    eprintln!("Failed to initialize app: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config_get,
            config_set,
            config_open_data_dir,
            accounts_list,
            accounts_create,
            accounts_update,
            accounts_delete,
            accounts_generate_uuid,
            versions_fetch_manifest,
            versions_list,
            versions_download,
            versions_get_version_json,
            java_detect,
            java_list_runtimes,
            java_resolve,
            instances_list,
            instances_create,
            instances_update,
            instances_delete,
            instances_clone,
            instances_open_folder,
            launch_start,
            launch_kill,
            loaders_fabric_versions,
            loaders_fabric_loader_versions,
            loaders_quilt_versions,
            loaders_quilt_loader_versions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}