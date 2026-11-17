//! Classpath construction for Minecraft launch

use crate::versions::model::{VersionJson, Library};
use crate::error::{LauncherError, Result};
use std::path::{Path, PathBuf};
use std::io::Cursor;
use zip::ZipArchive;

/// Build the complete classpath for launching Minecraft
pub fn build_classpath(
    version: &VersionJson,
    libraries_dir: &Path,
    version_dir: &Path,
) -> Result<String> {
    let mut classpath_entries = Vec::new();

    // Add libraries
    for library in &version.libraries {
        // Check if library applies to current platform (default allow for libraries)
        if !crate::versions::library::evaluate_library_rules(&library.rules, true).unwrap_or(true) {
            continue;
        }

        // Determine artifact path
        let artifact_path = library_artifact_path(library, libraries_dir)?;
        if artifact_path.exists() {
            classpath_entries.push(artifact_path.to_string_lossy().to_string());
        }
    }

    // Add client jar
    let client_jar = version_dir.join(format!("{}.jar", version.id));
    if client_jar.exists() {
        classpath_entries.push(client_jar.to_string_lossy().to_string());
    } else {
        return Err(LauncherError::Launch(format!(
            "Client jar not found: {}",
            client_jar.display()
        )));
    }

    // Join with platform-specific separator
    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
    Ok(classpath_entries.join(separator))
}

/// Get the path to a library artifact
pub fn library_artifact_path(library: &Library, libraries_dir: &Path) -> Result<PathBuf> {
    // Prefer the exact Maven-relative path supplied by Mojang.
    if let Some(path) = library.downloads.as_ref()
        .and_then(|downloads| downloads.artifact.as_ref())
        .and_then(|artifact| artifact.path.as_deref())
    {
        return library_download_path(libraries_dir, path);
    }

    // Fall back to Maven coordinates when the manifest omits `path`.
    let parts: Vec<&str> = library.name.split(':').collect();
    if parts.len() != 3 {
        return Err(LauncherError::Launch(format!(
            "Invalid library name format: {}",
            library.name
        )));
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let group_path = group.replace('.', "/");

    let filename = format!("{}-{}.jar", artifact, version);
    Ok(libraries_dir
        .join(group_path)
        .join(artifact)
        .join(version)
        .join(filename))
}

/// Get the natives directory for a version
pub fn get_natives_dir(version_dir: &Path, version_id: &str) -> PathBuf {
    version_dir.join(format!("{}-natives", version_id))
}

/// Extract natives from library jars
pub async fn extract_natives(
    version: &VersionJson,
    libraries_dir: &Path,
    natives_dir: &Path,
) -> Result<()> {
    use tokio::fs;

    fs::create_dir_all(natives_dir).await?;

    for library in &version.libraries {
        // Check if library applies to current platform (default allow for libraries)
        if !crate::versions::library::evaluate_library_rules(&library.rules, true).unwrap_or(true) {
            continue;
        }

        // Check if library has natives for current platform
        let Some(natives_map) = &library.natives else { continue };
        let Some(classifiers) = library.downloads.as_ref().and_then(|downloads| downloads.classifiers.as_ref()) else {
            continue;
        };

        // Mojang maps the current platform key to a classifier name. Older
        // manifests may use `${arch}` in that name (32/64).
        #[cfg(target_os = "windows")]
        let platform_keys: &[&str] = if cfg!(target_arch = "aarch64") {
            &["windows-arm64", "windows"]
        } else { &["windows"] };
        #[cfg(target_os = "macos")]
        let platform_keys: &[&str] = if cfg!(target_arch = "aarch64") {
            &["osx-arm64", "osx"]
        } else { &["osx"] };
        #[cfg(target_os = "linux")]
        let platform_keys: &[&str] = if cfg!(target_arch = "aarch64") {
            &["linux-aarch64", "linux"]
        } else if cfg!(target_arch = "arm") {
            &["linux-arm", "linux"]
        } else { &["linux"] };
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let platform_keys: &[&str] = &[];

        let arch = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
        let native_classifier = platform_keys.iter()
            .find_map(|key| natives_map.get(*key))
            .map(|classifier| classifier.replace("${arch}", arch));

        if let Some(native_classifier) = native_classifier {
            if classifiers.contains_key(&native_classifier) {
                let artifact_path = library_native_path(library, libraries_dir, &native_classifier)?;
                if artifact_path.exists() {
                    extract_native_jar(&artifact_path, natives_dir, library.extract.as_ref()).await?;
                }
            }
        }
    }

    Ok(())
}

fn library_download_path(libraries_dir: &Path, relative_path: &str) -> Result<PathBuf> {
    use std::path::Component;

    let path = Path::new(relative_path);
    if relative_path.is_empty()
        || relative_path.contains('\\')
        || relative_path.contains(':')
        || path.components().any(|component| matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ))
    {
        return Err(LauncherError::Launch(format!("Invalid library download path: {}", relative_path)));
    }

    Ok(libraries_dir.join(path))
}

pub fn library_native_path(
    library: &Library,
    libraries_dir: &Path,
    classifier: &str,
) -> Result<PathBuf> {
    if let Some(path) = library.downloads.as_ref()
        .and_then(|downloads| downloads.classifiers.as_ref())
        .and_then(|classifiers| classifiers.get(classifier))
        .and_then(|download| download.path.as_deref())
    {
        return library_download_path(libraries_dir, path);
    }

    let parts: Vec<&str> = library.name.split(':').collect();
    if parts.len() != 3 {
        return Err(LauncherError::Launch(format!(
            "Invalid library name format: {}",
            library.name
        )));
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let group_path = group.replace('.', "/");

    let filename = format!("{}-{}-{}.jar", artifact, version, classifier);
    Ok(libraries_dir
        .join(group_path)
        .join(artifact)
        .join(version)
        .join(filename))
}

async fn extract_native_jar(
    jar_path: &Path,
    natives_dir: &Path,
    extract_rules: Option<&crate::versions::model::ExtractRules>,
) -> Result<()> {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    let mut file = File::open(jar_path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let cursor = Cursor::new(buffer);
    let mut archive = ZipArchive::new(cursor)?;

    let exclude_patterns = extract_rules
    .map(|r| r.exclude.as_slice())
    .unwrap_or(&[]);


    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name()?.into_owned();

        // Skip excluded files
        let excluded = exclude_patterns.iter().any(|pattern| {
            // Simple glob matching
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len()-1];
                name.starts_with(prefix)
            } else {
                name == *pattern
            }
        });

        if excluded {
            continue;
        }

        // Only extract native libraries (dll, so, dylib)
        if name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".dylib") {
            let dest_path = natives_dir.join(Path::new(&name).file_name().unwrap());
            let mut dest_file = std::fs::File::create(&dest_path)?;
            std::io::copy(&mut file, &mut dest_file)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versions::model::{DownloadInfo, Library, LibraryDownloads};
    use std::path::PathBuf;

    #[test]
    fn test_library_artifact_path_uses_manifest_path() {
        let library = Library {
            name: "com.example:test-lib:1.0.0".to_string(),
            downloads: Some(LibraryDownloads {
                artifact: Some(DownloadInfo {
                    url: "https://example.invalid/test.jar".to_string(),
                    sha1: "0000000000000000000000000000000000000000".to_string(),
                    size: 1,
                    path: Some("custom/repository/test-lib.jar".to_string()),
                }),
                classifiers: None,
            }),
            natives: None,
            rules: vec![],
            extract: None,
            url: None,
        };

        let path = library_artifact_path(&library, Path::new("/libraries")).unwrap();
        assert_eq!(path, PathBuf::from("/libraries/custom/repository/test-lib.jar"));
    }

    #[test]
    fn test_library_artifact_path_rejects_parent_traversal() {
        let library = Library {
            name: "com.example:test-lib:1.0.0".to_string(),
            downloads: Some(LibraryDownloads {
                artifact: Some(DownloadInfo {
                    url: "https://example.invalid/test.jar".to_string(),
                    sha1: "0000000000000000000000000000000000000000".to_string(),
                    size: 1,
                    path: Some("../outside/test-lib.jar".to_string()),
                }),
                classifiers: None,
            }),
            natives: None,
            rules: vec![],
            extract: None,
            url: None,
        };

        assert!(library_artifact_path(&library, Path::new("/libraries")).is_err());
    }

    #[test]
    fn test_library_artifact_path() {
        let library = Library {
            name: "com.example:test-lib:1.0.0".to_string(),
            downloads: None,
            natives: None,
            rules: vec![],
            extract: None,
            url: None,
        };

        let path = library_artifact_path(&library, Path::new("/libraries")).unwrap();
        assert_eq!(
            path,
            PathBuf::from("/libraries/com/example/test-lib/1.0.0/test-lib-1.0.0.jar")
        );
    }
}
