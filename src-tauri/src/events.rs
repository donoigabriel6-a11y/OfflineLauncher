use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Download progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressEvent {
    pub version_id: String,
    pub file_name: String,
    pub downloaded: u64,
    pub total: u64,
    pub stage: DownloadStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStage {
    VersionJson,
    ClientJar,
    Libraries,
    AssetsIndex,
    Assets,
    Natives,
    Complete,
}

/// Download complete event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadCompleteEvent {
    pub version_id: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Launch output event (stdout/stderr)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchOutputEvent {
    pub instance_id: String,
    pub line: String,
    pub stream: OutputStream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Launch status event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchStatusEvent {
    pub instance_id: String,
    pub status: LaunchStatus,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchStatus {
    Starting,
    Running,
    Exited,
    Crashed,
    Killed,
}

/// Account changed event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountChangedEvent {
    pub action: AccountAction,
    pub account: AccountInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountAction {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub account_type: String,
}

/// Instance changed event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceChangedEvent {
    pub action: InstanceAction,
    pub instance: InstanceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstanceAction {
    Created,
    Updated,
    Deleted,
    Cloned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    pub id: String,
    pub name: String,
    pub version_id: String,
    pub loader: Option<String>,
}

/// Emit a download progress event
pub fn emit_download_progress(app: &AppHandle, event: DownloadProgressEvent) -> Result<(), tauri::Error> {
    app.emit("download_progress", &event)
}

/// Emit a download complete event
pub fn emit_download_complete(app: &AppHandle, event: DownloadCompleteEvent) -> Result<(), tauri::Error> {
    app.emit("download_complete", &event)
}

/// Emit launch output
pub fn emit_launch_output(app: &AppHandle, event: LaunchOutputEvent) -> Result<(), tauri::Error> {
    app.emit("launch_output", &event)
}

/// Emit launch status
pub fn emit_launch_status(app: &AppHandle, event: LaunchStatusEvent) -> Result<(), tauri::Error> {
    app.emit("launch_status", &event)
}

/// Emit account changed
pub fn emit_account_changed(app: &AppHandle, event: AccountChangedEvent) -> Result<(), tauri::Error> {
    app.emit("accounts_changed", &event)
}

/// Emit instance changed
pub fn emit_instance_changed(app: &AppHandle, event: InstanceChangedEvent) -> Result<(), tauri::Error> {
    app.emit("instances_changed", &event)
}