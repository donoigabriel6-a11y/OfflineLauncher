//! Java selection and management

use crate::app_dirs::get_java_runtimes_dir;
use crate::error::{LauncherError, Result};
use crate::java::detection::{detect_system_java, find_java_for_version};
use crate::java::runtime::{fetch_adoptium_runtimes, JavaRuntime};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaSelection {
    pub path: PathBuf,
    pub version: String,
    pub major_version: u8,
    pub source: JavaSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JavaSource {
    System,
    Mojang,
    Adoptium,
    Custom,
}

pub struct JavaManager {
    client: Client,
}

impl JavaManager {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    /// Get the best Java for a given major version requirement
    pub async fn resolve_java(
        &self,
        required_major: u8,
        override_path: Option<&PathBuf>,
    ) -> Result<JavaSelection> {
        // 1. Check override path
        if let Some(path) = override_path {
            if path.exists() {
                let install = crate::java::detection::parse_java_version(path).await?;
                if install.major_version >= required_major {
                    return Ok(JavaSelection {
                        path: install.executable_path.clone(),
                        version: install.version,
                        major_version: install.major_version,
                        source: JavaSource::Custom,
                    });
                }
            }
        }

        // 2. Check system installations
        let system_java = detect_system_java().await?;
        if let Some(install) = find_java_for_version(&system_java, required_major) {
            return Ok(JavaSelection {
                path: install.executable_path.clone(),
                version: install.version.clone(),
                major_version: install.major_version,
                source: JavaSource::System,
            });
        }

        // 3. Check downloaded runtimes
        let runtimes_dir = get_java_runtimes_dir()?;
        if runtimes_dir.exists() {
            if let Ok(install) = self.find_local_runtime(&runtimes_dir, required_major).await {
                return Ok(install);
            }
        }

        // 4. Download a compatible Adoptium JDK when no local Java is available.
        let runtimes = fetch_adoptium_runtimes(&self.client, required_major).await?;
        if let Some(runtime) = runtimes.first() {
            let path = self.download_runtime(runtime).await?;
            return Ok(JavaSelection {
                path,
                version: runtime.version.clone(),
                major_version: runtime.major_version,
                source: JavaSource::Adoptium,
            });
        }

        Err(LauncherError::JavaNotFound(format!(
            "No suitable Java {} found",
            required_major
        )))
    }

    async fn find_local_runtime(
        &self,
        runtimes_dir: &PathBuf,
        required_major: u8,
    ) -> Result<JavaSelection> {
        let mut entries = fs::read_dir(runtimes_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let Ok(java_exe) = find_java_executable(&entry.path()) else { continue };
            if let Ok(install) = crate::java::detection::parse_java_version(&java_exe).await {
                if install.major_version >= required_major {
                    return Ok(JavaSelection {
                        path: java_exe,
                        version: install.version,
                        major_version: install.major_version,
                        source: JavaSource::Adoptium,
                    });
                }
            }
        }
        Err(LauncherError::JavaNotFound("No local runtime found".into()))
    }

    async fn download_runtime(&self, runtime: &JavaRuntime) -> Result<PathBuf> {
        let runtimes_dir = get_java_runtimes_dir()?;
        fs::create_dir_all(&runtimes_dir).await?;

        let filename = format!(
            "java-{}-{}-{}",
            runtime.vendor, runtime.major_version, runtime.version
        );
        let dest_dir = runtimes_dir.join(&filename);
        if let Ok(java_exe) = find_java_executable(&dest_dir) {
            return Ok(java_exe);
        }

        if dest_dir.exists() {
            fs::remove_dir_all(&dest_dir).await?;
        }

        tracing::info!("Downloading Java runtime from: {}", runtime.url);
        let response = self
            .client
            .get(&runtime.url)
            .send()
            .await
            .map_err(LauncherError::Http)?;
        if !response.status().is_success() {
            return Err(LauncherError::Download(format!(
                "Java runtime download failed: HTTP {}",
                response.status()
            )));
        }

        let archive_bytes = response.bytes().await.map_err(LauncherError::Http)?;
        if runtime.size > 0 && archive_bytes.len() as u64 != runtime.size {
            return Err(LauncherError::Download(format!(
                "Java runtime archive size mismatch: expected {}, got {}",
                runtime.size,
                archive_bytes.len()
            )));
        }

        let archive_extension = if cfg!(target_os = "windows") {
            "zip"
        } else {
            "tar.gz"
        };
        let archive_path = runtimes_dir.join(format!("{}.{}", filename, archive_extension));
        fs::write(&archive_path, &archive_bytes).await?;

        let archive_for_extract = archive_path.clone();
        let destination_for_extract = dest_dir.clone();
        let extraction = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            std::fs::create_dir_all(&destination_for_extract)?;
            let archive_file = std::fs::File::open(&archive_for_extract)?;

            #[cfg(target_os = "windows")]
            {
                let mut archive = zip::ZipArchive::new(archive_file)
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
                archive
                    .extract(&destination_for_extract)
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            }

            #[cfg(not(target_os = "windows"))]
            {
                let decoder = flate2::read::GzDecoder::new(archive_file);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(&destination_for_extract)?;
            }

            Ok(())
        })
        .await
        .map_err(|error| {
            LauncherError::Download(format!("Java runtime extraction task failed: {}", error))
        })?;

        if let Err(error) = extraction {
            let _ = fs::remove_file(&archive_path).await;
            return Err(LauncherError::Io(error));
        }
        fs::remove_file(&archive_path).await?;

        find_java_executable(&dest_dir)
    }

    /// List all available Java installations (system + downloaded)
    pub async fn list_all(&self) -> Result<Vec<JavaSelection>> {
        let mut selections = Vec::new();

        // System
        for install in detect_system_java().await? {
            selections.push(JavaSelection {
                path: install.executable_path.clone(),
                version: install.version,
                major_version: install.major_version,
                source: JavaSource::System,
            });
        }

        // Downloaded
        let runtimes_dir = get_java_runtimes_dir()?;
        if runtimes_dir.exists() {
            let mut entries = fs::read_dir(&runtimes_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let Ok(java_exe) = find_java_executable(&entry.path()) else { continue };
                if let Ok(install) = crate::java::detection::parse_java_version(&java_exe).await {
                    selections.push(JavaSelection {
                        path: install.executable_path.clone(),
                        version: install.version,
                        major_version: install.major_version,
                        source: JavaSource::Adoptium,
                    });
                }
            }
        }

        selections.sort_by(|a, b| b.major_version.cmp(&a.major_version));
        Ok(selections)
    }
}

impl Default for JavaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Find the Java executable inside an extracted JDK archive. Adoptium archives
/// contain a versioned top-level directory, so it is not always at root/bin.
fn find_java_executable(root: &std::path::Path) -> Result<PathBuf> {
    let executable_name = crate::java::detection::java_executable_name();
    for entry in walkdir::WalkDir::new(root).follow_links(true) {
        let entry = entry.map_err(|error| {
            LauncherError::Io(std::io::Error::new(std::io::ErrorKind::Other, error))
        })?;
        if entry.file_name().to_string_lossy() == executable_name && entry.path().is_file() {
            return Ok(entry.path().to_path_buf());
        }
    }

    Err(LauncherError::JavaNotFound(format!(
        "Java executable not found in {}",
        root.display()
    )))
}
