//! OfflineLauncher - Cross-platform Minecraft Java Edition launcher
//!
//! A Tauri 2 + React + TypeScript application with first-class offline account support.

#![allow(clippy::too_many_arguments)]

pub mod app_dirs;
pub mod config;
pub mod error;
pub mod events;

pub mod accounts;
pub mod content;
pub mod downloads;
pub mod instances;
pub mod java;
pub mod launch;
pub mod loaders;
pub mod versions;

mod commands;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

// Re-export commonly used types
pub use accounts::{
    generate_offline_uuid, validate_username, Account, AccountStorage, AccountType, OfflineAccount,
    UserType,
};
pub use config::Config;
pub use content::commands::ContentState;
pub use content::storage::ContentStorage;
pub use downloads::{
    DownloadManager, DownloadOptions, DownloadProgress, DownloadStage, DownloadTask,
};
pub use error::{LauncherError, Result};
pub use instances::{
    Instance, InstanceStorage, JavaSettings, MemorySettings, ModLoader, Resolution,
};
pub use java::{detect_system_java, JavaInstallation, JavaManager, JavaSelection, JavaSource};
pub use launch::{
    extract_natives, get_natives_dir, launch_game, library_artifact_path, library_native_path,
    GameProcess, LaunchArguments, LaunchHandle,
};
pub use loaders::{
    fetch_fabric_loader_versions, fetch_fabric_versions, fetch_quilt_loader_versions,
    fetch_quilt_versions, install_fabric, install_quilt,
};
pub use versions::{
    fetch_version_manifest, AssetIndexObject, VersionEntry, VersionJson, VersionManifest,
    VersionType,
};

/// Application state shared across commands
pub struct AppState {
    pub config: RwLock<Config>,
    pub account_storage: AccountStorage,
    pub instance_storage: InstanceStorage,
    pub download_manager: DownloadManager,
    pub java_manager: JavaManager,
    pub http_client: reqwest::Client,
    pub content_state: ContentState,
    pub running_processes: Mutex<HashMap<Uuid, LaunchHandle>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config: RwLock::new(config),
            account_storage: AccountStorage::new(),
            instance_storage: InstanceStorage::new(),
            download_manager: DownloadManager::new(DownloadOptions::default()),
            java_manager: JavaManager::new(),
            http_client,
            content_state: ContentState {
                storage: ContentStorage::new().expect("content storage should initialize"),
            },
            running_processes: Mutex::new(HashMap::new()),
        }
    }
}

/// Initialize the application state synchronously
pub fn initialize_app(app: &tauri::AppHandle) -> Result<()> {
    // Ensure directories exist
    crate::app_dirs::ensure_dirs()?;

    // Load config
    let config = Config::load()?;

    // Create app state
    let state = AppState::new(config);

    // Load accounts
    tauri::async_runtime::block_on(state.account_storage.load())?;

    // Load instances
    tauri::async_runtime::block_on(state.instance_storage.load())?;

    // Load content storage (installed content, curseforge key)
    tauri::async_runtime::block_on(state.content_state.storage.load())?;

    // Store state
    app.manage(Arc::new(state));

    Ok(())
}

/// Run the Tauri application
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            initialize_app(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config_get,
            commands::config_set,
            commands::config_open_data_dir,
            commands::accounts_list,
            commands::accounts_create,
            commands::accounts_update,
            commands::accounts_delete,
            commands::accounts_generate_uuid,
            commands::versions_fetch_manifest,
            commands::versions_list,
            commands::versions_download,
            commands::versions_is_installed,
            commands::versions_get_version_json,
            commands::java_detect,
            commands::java_list_runtimes,
            commands::java_resolve,
            commands::instances_list,
            commands::instances_create,
            commands::instances_update,
            commands::instances_delete,
            commands::instances_clone,
            commands::instances_open_folder,
            commands::launch_start,
            commands::launch_kill,
            commands::loaders_fabric_versions,
            commands::loaders_fabric_loader_versions,
            commands::loaders_quilt_versions,
            commands::loaders_quilt_loader_versions,
            content::commands::content_search,
            content::commands::content_versions,
            content::commands::content_install,
            content::commands::content_list_installed,
            content::commands::content_uninstall,
            content::commands::curseforge_key_status,
            content::commands::curseforge_key_set,
            content::commands::curseforge_key_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
