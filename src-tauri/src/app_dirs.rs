use dirs;
use std::path::{Path, PathBuf};
use crate::error::{LauncherError, Result};

/// Get the platform-correct application data directory for OfflineLauncher
pub fn get_app_data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| LauncherError::Config("Could not determine data directory".into()))?;
    Ok(base.join("OfflineLauncher"))
}

/// Get the platform-correct application config directory
pub fn get_app_config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| LauncherError::Config("Could not determine config directory".into()))?;
    Ok(base.join("OfflineLauncher"))
}

/// Get the platform-correct cache directory
pub fn get_app_cache_dir() -> Result<PathBuf> {
    let base = dirs::cache_dir()
        .ok_or_else(|| LauncherError::Config("Could not determine cache directory".into()))?;
    Ok(base.join("OfflineLauncher"))
}

/// Get the accounts storage file path
pub fn get_accounts_file() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("accounts.json"))
}

/// Get the instances storage directory
pub fn get_instances_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("instances"))
}

/// Get an instance's game directory
pub fn get_instance_game_dir(instance_id: &str) -> Result<PathBuf> {
    Ok(get_instances_dir()?.join(instance_id).join("game"))
}

/// Get the versions cache directory
pub fn get_versions_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("versions"))
}

/// Get a specific version's directory
pub fn get_version_dir(version_id: &str) -> Result<PathBuf> {
    Ok(get_versions_dir()?.join(version_id))
}

/// Get the assets directory
pub fn get_assets_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("assets"))
}

/// Get the assets index directory
pub fn get_assets_index_dir() -> Result<PathBuf> {
    Ok(get_assets_dir()?.join("indexes"))
}

/// Get the assets objects directory
pub fn get_assets_objects_dir() -> Result<PathBuf> {
    Ok(get_assets_dir()?.join("objects"))
}

/// Get the libraries directory
pub fn get_libraries_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("libraries"))
}

/// Get the Java runtimes directory
pub fn get_java_runtimes_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("runtimes"))
}

/// Get the logs directory
pub fn get_logs_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("logs"))
}

/// Get the crash reports directory
pub fn get_crash_reports_dir() -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join("crash-reports"))
}

/// Get the temporary directory for downloads
pub fn get_temp_dir() -> Result<PathBuf> {
    Ok(get_app_cache_dir()?.join("temp"))
}

/// Ensure all required directories exist
pub fn ensure_dirs() -> Result<()> {
    let dirs = [
        get_app_data_dir()?,
        get_app_config_dir()?,
        get_app_cache_dir()?,
        get_instances_dir()?,
        get_versions_dir()?,
        get_assets_dir()?,
        get_assets_index_dir()?,
        get_assets_objects_dir()?,
        get_libraries_dir()?,
        get_java_runtimes_dir()?,
        get_logs_dir()?,
        get_crash_reports_dir()?,
        get_temp_dir()?,
    ];

    for dir in dirs {
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| LauncherError::Io(e))?;
        }
    }
    Ok(())
}

/// Get a path relative to the app data dir
pub fn resolve_data_path<P: AsRef<Path>>(relative: P) -> Result<PathBuf> {
    Ok(get_app_data_dir()?.join(relative))
}

/// Get a path relative to the app config dir
pub fn resolve_config_path<P: AsRef<Path>>(relative: P) -> Result<PathBuf> {
    Ok(get_app_config_dir()?.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirs_exist() {
        // Just verify the functions return paths without error
        assert!(get_app_data_dir().is_ok());
        assert!(get_app_config_dir().is_ok());
        assert!(get_app_cache_dir().is_ok());
    }
}