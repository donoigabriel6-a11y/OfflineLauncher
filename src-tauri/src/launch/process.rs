//! Game process spawning and management

use crate::error::{LauncherError, Result};
use crate::events::{emit_launch_output, emit_launch_status, LaunchOutputEvent, LaunchStatusEvent, LaunchStatus, OutputStream};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tauri::AppHandle;
use uuid::Uuid;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Handle to a running game process
#[derive(Clone)]
pub struct LaunchHandle {
    pub instance_id: Uuid,
    child: Arc<Mutex<Option<Child>>>,
    pub pid: Option<u32>,
}

impl LaunchHandle {
    /// Create a new launch handle
    pub fn new(instance_id: Uuid) -> Self {
        Self {
            instance_id,
            child: Arc::new(Mutex::new(None)),
            pid: None,
        }
    }

    /// Set the child process
    pub async fn set_child(&mut self, child: Child) {
        self.pid = child.id();
        let mut guard = self.child.lock().await;
        *guard = Some(child);
    }

    /// Kill the game process
    pub async fn kill(&self) -> Result<()> {
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            child.kill().await
                .map_err(|e| LauncherError::Process(e.to_string()))?;
        }
        Ok(())
    }

    /// Wait for the process to exit and return exit code
    pub async fn wait(&self) -> Result<Option<i32>> {
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            let status = child.wait().await
                .map_err(|e| LauncherError::Process(e.to_string()))?;
            Ok(status.code())
        } else {
            Ok(None)
        }
    }

    /// Check if process is still running
    pub async fn is_running(&self) -> bool {
        let guard = self.child.lock().await;
        guard.is_some()
    }
}

/// Game process wrapper
pub struct GameProcess {
    pub handle: LaunchHandle,
    pub instance_id: Uuid,
}

impl GameProcess {
    /// Launch the game with the given arguments
    pub async fn launch(
        app: AppHandle,
        instance_id: Uuid,
        java_exe: &Path,
        jvm_args: Vec<String>,
        main_class: String,
        game_args: Vec<String>,
        working_dir: &Path,
    ) -> Result<Self> {
        let mut handle = LaunchHandle::new(instance_id);

        // Build command
        let mut cmd = Command::new(java_exe);
        cmd.args(&jvm_args)
            .arg(&main_class)
            .args(&game_args)
            .current_dir(working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // On Windows, prevent console window from appearing for child processes
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        tracing::info!("Launching game: {} {:?} {} {:?}",
            java_exe.display(), jvm_args, main_class, game_args);

        // Emit starting status
        emit_launch_status(&app, LaunchStatusEvent {
            instance_id: instance_id.to_string(),
            status: LaunchStatus::Starting,
            exit_code: None,
            pid: None,
        })?;

        // Spawn process
        let child = match cmd.spawn() {
            Ok(child) => child,
            Err(error) => {
                let _ = emit_launch_status(&app, LaunchStatusEvent {
                    instance_id: instance_id.to_string(),
                    status: LaunchStatus::Crashed,
                    exit_code: None,
                    pid: None,
                });
                return Err(LauncherError::Process(format!("Failed to spawn process: {}", error)));
            }
        };

        let pid = child.id();
        handle.set_child(child).await;

        // Emit running status
        emit_launch_status(&app, LaunchStatusEvent {
            instance_id: instance_id.to_string(),
            status: LaunchStatus::Running,
            exit_code: None,
            pid,
        })?;

        // Get stdout and stderr
        let stdout = handle.child.lock().await.as_mut().and_then(|c| c.stdout.take());
        let stderr = handle.child.lock().await.as_mut().and_then(|c| c.stderr.take());

        // Stream stdout
        if let Some(stdout) = stdout {
            let app_clone = app.clone();
            let instance_id_str = instance_id.to_string();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = emit_launch_output(&app_clone, LaunchOutputEvent {
                        instance_id: instance_id_str.clone(),
                        line,
                        stream: OutputStream::Stdout,
                    });
                }
            });
        }

        // Monitor the child without holding the lock between checks so the
        // Stop Game command can still acquire it and kill the process.
        let monitor_handle = handle.clone();
        let monitor_app = app.clone();
        let monitor_instance_id = instance_id.to_string();
        tokio::spawn(async move {
            loop {
                let exit_status = {
                    let mut child_guard = monitor_handle.child.lock().await;
                    match child_guard.as_mut() {
                        Some(child) => child.try_wait(),
                        None => return,
                    }
                };

                match exit_status {
                    Ok(Some(status)) => {
                        let exit_code = status.code();
                        let launch_status = if status.success() {
                            LaunchStatus::Exited
                        } else {
                            LaunchStatus::Crashed
                        };
                        monitor_handle.child.lock().await.take();
                        let _ = emit_launch_status(&monitor_app, LaunchStatusEvent {
                            instance_id: monitor_instance_id.clone(),
                            status: launch_status,
                            exit_code,
                            pid,
                        });
                        break;
                    }
                    Ok(None) => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
                    Err(error) => {
                        let _ = emit_launch_output(&monitor_app, LaunchOutputEvent {
                            instance_id: monitor_instance_id.clone(),
                            line: format!("Failed to monitor game process: {}", error),
                            stream: OutputStream::Stderr,
                        });
                        monitor_handle.child.lock().await.take();
                        let _ = emit_launch_status(&monitor_app, LaunchStatusEvent {
                            instance_id: monitor_instance_id.clone(),
                            status: LaunchStatus::Crashed,
                            exit_code: None,
                            pid,
                        });
                        break;
                    }
                }
            }
        });

        // Stream stderr
        if let Some(stderr) = stderr {
            let app_clone = app.clone();
            let instance_id_str = instance_id.to_string();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = emit_launch_output(&app_clone, LaunchOutputEvent {
                        instance_id: instance_id_str.clone(),
                        line,
                        stream: OutputStream::Stderr,
                    });
                }
            });
        }

        Ok(Self { handle, instance_id })
    }

    /// Get the launch handle
    pub fn handle(&self) -> &LaunchHandle {
        &self.handle
    }
}

/// Launch a game instance
pub async fn launch_game(
    app: AppHandle,
    instance_id: Uuid,
    java_exe: &Path,
    jvm_args: Vec<String>,
    main_class: String,
    game_args: Vec<String>,
    working_dir: &Path,
) -> Result<GameProcess> {
    GameProcess::launch(app, instance_id, java_exe, jvm_args, main_class, game_args, working_dir).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_launch_handle_creation() {
        let id = Uuid::new_v4();
        let handle = LaunchHandle::new(id);
        assert_eq!(handle.instance_id, id);
        assert!(handle.pid.is_none());
    }
}