//! OfflineLauncher - Cross-platform Minecraft Java Edition launcher
//!
//! A Tauri 2 + React + TypeScript application with first-class offline account support.

#![allow(clippy::too_many_arguments)]

pub mod error;
pub mod app_dirs;
pub mod config;
pub mod events;

pub mod accounts;
pub mod versions;
pub mod downloads;
pub mod java;
pub mod instances;
pub mod launch;
pub mod loaders;

use tauri::{Manager, State};
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export commonly used types
pub use error::{LauncherError, Result};
pub use config::Config;
pub use accounts::{Account, AccountStorage, OfflineAccount, AccountType, UserType, validate_username, generate_offline_uuid};
pub use versions::{VersionManifest, VersionEntry, VersionJson, VersionType, fetch_version_manifest};
pub use downloads::{DownloadManager, DownloadOptions, DownloadTask, DownloadProgress, DownloadStage};
pub use java::{JavaManager, JavaInstallation, JavaSelection, JavaSource, detect_system_java};
pub use instances::{InstanceStorage, Instance, ModLoader, MemorySettings, JavaSettings, Resolution};
pub use launch::{LaunchArguments, GameProcess, LaunchHandle, launch_game};
pub use loaders::{fetch_fabric_versions, fetch_fabric_loader_versions, install_fabric, fetch_quilt_versions, fetch_quilt_loader_versions, install_quilt};

/// Application state shared across commands
pub struct AppState {
    pub config: RwLock<Config>,
    pub account_storage: AccountStorage,
    pub instance_storage: InstanceStorage,
    pub download_manager: DownloadManager,
    pub java_manager: JavaManager,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: RwLock::new(config),
            account_storage: AccountStorage::new(),
            instance_storage: InstanceStorage::new(),
            download_manager: DownloadManager::new(DownloadOptions::default()),
            java_manager: JavaManager::new(),
        }
    }
}

/// Initialize the application
pub async fn initialize_app(app: &tauri::AppHandle) -> Result<()> {
    // Ensure directories exist
    crate::app_dirs::ensure_dirs()?;

    // Load config
    let config = Config::load()?;

    // Create app state
    let state = AppState::new(config);

    // Load accounts
    state.account_storage.load().await?;

    // Load instances
    state.instance_storage.load().await?;

    // Store state
    app.manage(Arc::new(state));

    Ok(())
}