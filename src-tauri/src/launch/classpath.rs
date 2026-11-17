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
    // Parse library name: "group:artifact:version"
    let parts: Vec<&str> = library.name.split(':').collect();
    if parts.len() != 3 {
        return Err(LauncherError::Launch(format!(
            "Invalid library name format: {}",
            library.name
        )));
    }

    if let Some(path) = library.downloads.as_ref()
        .and_then(|downloads| downloads.artifact.as_ref())
        .and_then(|artifact| artifact.path.as_deref())
    {
        return library_download_path(libraries_dir, path);
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
        let classifier = crate::versions::library::get_native_classifier();
        let Some(classifier) = classifier else { continue };

        if let Some(natives_map) = &library.natives {
            // The `natives` map stores platform keys mapped to classifier names.
            // Match the classifier value (for example `natives-windows`).
            if let Some(native_classifier) = natives_map.values().find(|value| *value == &classifier) {
                let artifact_path = library_native_path(library, libraries_dir, native_classifier)?;
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
    let parts: Vec<&str> = library.name.split(':').collect();
    if parts.len() != 3 {
        return Err(LauncherError::Launch(format!(
            "Invalid library name format: {}",
            library.name
        )));
    }

    if let Some(path) = library.downloads.as_ref()
        .and_then(|downloads| downloads.classifiers.as_ref())
        .and_then(|classifiers| classifiers.get(classifier))
        .and_then(|download| download.path.as_deref())
    {
        return library_download_path(libraries_dir, path);
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
    use crate::versions::model::Library;
    use std::path::PathBuf;

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
