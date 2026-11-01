//! Java selection and management

use crate::app_dirs::get_java_runtimes_dir;
use crate::error::{LauncherError, Result};
use crate::java::detection::{detect_system_java, find_java_for_version, JavaInstallation};
use crate::java::runtime::{fetch_adoptium_runtimes, fetch_java_runtimes, JavaRuntime};
use reqwest::Client;
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
        Self {
            client: Client::new(),
        }
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
                        path: path.to_path_buf(),
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
                path: install.path.join("bin").join(crate::java::detection::java_executable_name()),
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

        // 4. Download from Mojang
        if let Ok(runtimes) = fetch_java_runtimes(&self.client).await {
            if let Some(runtime) = runtimes.iter().find(|r| r.major_version >= required_major) {
                let path = self.download_runtime(runtime).await?;
                return Ok(JavaSelection {
                    path,
                    version: runtime.version.clone(),
                    major_version: runtime.major_version,
                    source: JavaSource::Mojang,
                });
            }
        }

        // 5. Fallback to Adoptium
        if let Ok(runtimes) = fetch_adoptium_runtimes(&self.client, required_major).await {
            if let Some(runtime) = runtimes.first() {
                let path = self.download_runtime(runtime).await?;
                return Ok(JavaSelection {
                    path,
                    version: runtime.version.clone(),
                    major_version: runtime.major_version,
                    source: JavaSource::Adoptium,
                });
            }
        }

        Err(LauncherError::JavaNotFound(
            format!("No suitable Java {} found", required_major)
        ))
    }

    async fn find_local_runtime(&self, runtimes_dir: &PathBuf, required_major: u8) -> Result<JavaSelection> {
        let mut entries = fs::read_dir(runtimes_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let java_exe = entry.path().join("bin").join(crate::java::detection::java_executable_name());
            if java_exe.exists() {
                let install = crate::java::detection::parse_java_version(&java_exe).await?;
                if install.major_version >= required_major {
                    return Ok(JavaSelection {
                        path: java_exe,
                        version: install.version,
                        major_version: install.major_version,
                        source: JavaSource::Mojang, // Could be either
                    });
                }
            }
        }
        Err(LauncherError::JavaNotFound("No local runtime found".into()))
    }

    async fn download_runtime(&self, runtime: &JavaRuntime) -> Result<PathBuf> {
        let runtimes_dir = get_java_runtimes_dir()?;
        fs::create_dir_all(&runtimes_dir).await?;

        // Extract filename from URL or use version
        let filename = format!("java-{}-{}-{}", runtime.vendor, runtime.major_version, runtime.version);
        let dest_dir = runtimes_dir.join(&filename);
        let dest_exe = dest_dir.join("bin").join(crate::java::detection::java_executable_name());

        // Check if already downloaded
        if dest_exe.exists() {
            return Ok(dest_exe);
        }

        // Download and extract (simplified - real implementation would handle archives)
        // For now, this is a stub
        tracing::info!("Would download Java runtime from: {}", runtime.url);

        Err(LauncherError::JavaNotFound("Java download not fully implemented".into()))
    }

    /// List all available Java installations (system + downloaded)
    pub async fn list_all(&self) -> Result<Vec<JavaSelection>> {
        let mut selections = Vec::new();

        // System
        for install in detect_system_java().await? {
            selections.push(JavaSelection {
                path: install.path.join("bin").join(crate::java::detection::java_executable_name()),
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
                let java_exe = entry.path().join("bin").join(crate::java::detection::java_executable_name());
                if java_exe.exists() {
                    if let Ok(install) = crate::java::detection::parse_java_version(&java_exe).await {
                        selections.push(JavaSelection {
                            path: java_exe,
                            version: install.version,
                            major_version: install.major_version,
                            source: JavaSource::Mojang,
                        });
                    }
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