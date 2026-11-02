//! Download manager with concurrency control, retry, and progress reporting

use crate::downloads::progress::{DownloadProgress, DownloadStage};
use crate::downloads::hash::{verify_sha1_or_error, compute_sha1};
use crate::error::{LauncherError, Result};
use crate::events::emit_download_progress;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::time::Duration;
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub max_concurrent: usize,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub timeout: Duration,
    pub verify_hash: bool,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            max_concurrent: 8,
            max_retries: 3,
            retry_delay: Duration::from_secs(2),
            timeout: Duration::from_secs(300),
            verify_hash: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub destination: PathBuf,
    pub expected_sha1: Option<String>,
    pub file_name: String,
    pub stage: DownloadStage,
    pub version_id: String,
}

pub struct DownloadManager {
    client: Client,
    options: DownloadOptions,
    semaphore: Arc<Semaphore>,
}

impl DownloadManager {
    pub fn new(options: DownloadOptions) -> Self {
        let client = Client::builder()
            .timeout(options.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            semaphore: Arc::new(Semaphore::new(options.max_concurrent)),
            options,
        }
    }

    /// Download a single file with resume support and retries
    pub async fn download_file(
        &self,
        task: DownloadTask,
        app: Option<&AppHandle>,
    ) -> Result<PathBuf> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| LauncherError::Download("Semaphore closed".into()))?;

        // Ensure parent directory exists
        if let Some(parent) = task.destination.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| LauncherError::Io(e))?;
        }

        let mut last_error = None;

        for attempt in 0..=self.options.max_retries {
            if attempt > 0 {
                tokio::time::sleep(self.options.retry_delay).await;
            }

            match self.download_once(&task, app).await {
                Ok(path) => return Ok(path),
                Err(e) => {
                    last_error = Some(e);
                    tracing::warn!("Download attempt {} failed for {}: {:?}", attempt + 1, task.file_name, last_error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| LauncherError::Download("Max retries exceeded".into())))
    }

    async fn download_once(
        &self,
        task: &DownloadTask,
        app: Option<&AppHandle>,
    ) -> Result<PathBuf> {
        // Check if file already exists and is valid
        if task.destination.exists() {
            if let Some(expected) = &task.expected_sha1 {
                if self.options.verify_hash {
                    match verify_sha1_or_error(&task.destination, expected, &task.file_name).await {
                        Ok(_) => {
                            // File exists and is valid
                            self.emit_progress(app, task, task.destination.metadata().await?.len(), task.destination.metadata().await?.len()).await;
                            return Ok(task.destination.clone());
                        }
                        Err(_) => {
                            // Hash mismatch, remove and re-download
                            tokio::fs::remove_file(&task.destination).await.ok();
                        }
                    }
                }
            } else {
                // No hash to verify, assume it's complete
                return Ok(task.destination.clone());
            }
        }

        // Start download with resume support
        let mut resume_from = 0u64;
        let temp_path = task.destination.with_extension("part");

        if temp_path.exists() {
            resume_from = temp_path.metadata().await?.len();
        }

        let mut request = self.client.get(&task.url);
        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let response = request.send().await
            .map_err(|e| LauncherError::Http(e))?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(LauncherError::Download(
                format!("HTTP {}: {}", response.status(), task.url)
            ));
        }

        // Resume only when the server honored the Range request. Some servers
        // ignore Range and return 200 with the entire file; appending that body
        // would corrupt the partial download.
        let resumed = resume_from > 0
            && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
        if !resumed {
            resume_from = 0;
        }

        let total_size = response.content_length().unwrap_or(0);
        let total = if resumed { total_size + resume_from } else { total_size };

        let mut options = OpenOptions::new();
        options.create(true).write(true);
        if resumed {
            options.append(true);
        } else {
            options.truncate(true);
        }
        let mut file = options.open(&temp_path).await
            .map_err(|e| LauncherError::Io(e))?;

        let mut downloaded = resume_from;
        let mut stream = response.bytes_stream();
        let start_time = Instant::now();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LauncherError::Http(e))?;
            file.write_all(&chunk).await
                .map_err(|e| LauncherError::Io(e))?;
            downloaded += chunk.len() as u64;

            // Emit progress every 100ms or so
            if start_time.elapsed().as_millis() > 100 {
                self.emit_progress(app, task, downloaded, total).await;
            }
        }

        file.flush().await.map_err(|e| LauncherError::Io(e))?;
        drop(file);

        // Verify hash if provided
        if let Some(expected) = &task.expected_sha1 {
            if self.options.verify_hash {
                verify_sha1_or_error(&temp_path, expected, &task.file_name).await?;
            }
        }

        // Move temp file to final destination
        tokio::fs::rename(&temp_path, &task.destination).await
            .map_err(|e| LauncherError::Io(e))?;

        // Final progress
        self.emit_progress(app, task, total, total).await;

        Ok(task.destination.clone())
    }

    async fn emit_progress(
        &self,
        app: Option<&AppHandle>,
        task: &DownloadTask,
        downloaded: u64,
        total: u64,
    ) {
        if let Some(app) = app {
            let mut progress = DownloadProgress::new(
                task.version_id.clone(),
                task.file_name.clone(),
                task.stage.clone(),
            );
            progress.update(downloaded, total);
            let _ = emit_download_progress(app, crate::events::DownloadProgressEvent {
                version_id: progress.version_id,
                file_name: progress.file_name,
                downloaded: progress.downloaded,
                total: progress.total,
                stage: progress.stage,
            });
        }
    }

    /// Download multiple files concurrently
    pub async fn download_all(
        &self,
        tasks: Vec<DownloadTask>,
        app: Option<&AppHandle>,
    ) -> Result<Vec<PathBuf>> {
        let mut handles = Vec::new();

        for task in tasks {
            let manager = self.clone();
            let app_clone = app.cloned();
            handles.push(tokio::spawn(async move {
                manager.download_file(task, app_clone.as_ref()).await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(path)) => results.push(path),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(LauncherError::Download(format!("Task panicked: {}", e))),
            }
        }

        Ok(results)
    }
}

impl Clone for DownloadManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            options: self.options.clone(),
            semaphore: self.semaphore.clone(),
        }
    }
}