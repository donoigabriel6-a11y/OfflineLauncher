//! Version JSON inheritance resolution (inheritsFrom)

use crate::versions::model::VersionJson;
use crate::error::{LauncherError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Resolve version inheritance chain
///
/// Versions can inherit from another version via `inheritsFrom`.
/// This function recursively resolves the chain and merges the JSONs.
pub async fn resolve_inheritance(
    version_json: VersionJson,
    versions_dir: &Path,
    visited: &mut Vec<String>,
) -> Result<VersionJson> {
    // Detect circular inheritance
    if visited.contains(&version_json.id) {
        return Err(LauncherError::Inheritance(
            format!("Circular inheritance detected: {}", version_json.id)
        ));
    }

    visited.push(version_json.id.clone());

    // If no inheritance, return as-is
    let Some(parent_id) = version_json.inherits_from.clone() else {
        return Ok(version_json);
    };

    // Load parent version JSON
    let parent_path = versions_dir.join(&parent_id).join(format!("{}.json", parent_id));
    if !parent_path.exists() {
        return Err(LauncherError::Inheritance(
            format!("Parent version not found: {}", parent_id)
        ));
    }

    let parent_content = tokio::fs::read_to_string(&parent_path).await
        .map_err(|e| LauncherError::Io(e))?;

    let mut parent_json: VersionJson = serde_json::from_str(&parent_content)
        .map_err(|e| LauncherError::Serde(e))?;

    // Recursively resolve parent's inheritance
    parent_json = Box::pin(resolve_inheritance(parent_json, versions_dir, visited)).await?;

    // Merge child into parent (child overrides parent)
    let merged = merge_version_json(parent_json, version_json);

    Ok(merged)
}

/// Merge two version JSONs (child overrides parent)
fn merge_version_json(mut parent: VersionJson, child: VersionJson) -> VersionJson {
    // Child fields that should override parent
    if !child.id.is_empty() {
        parent.id = child.id;
    }
    if let Some(inherits) = child.inherits_from.clone() {
        parent.inherits_from = Some(inherits);
    }
    parent.java_version = child.java_version;
    if !child.main_class.is_empty() {
        parent.main_class = child.main_class;
    }
    parent.asset_index = child.asset_index;
    if !child.assets.is_empty() {
        parent.assets = child.assets;
    }

    // Merge libraries: child libraries replace parent libraries with same name
    parent.libraries = merge_libraries(parent.libraries, child.libraries);

    // Merge arguments
    parent.arguments = merge_arguments(parent.arguments, child.arguments);

    // Legacy arguments
    if child.minecraft_arguments.is_some() {
        parent.minecraft_arguments = child.minecraft_arguments;
    }

    // Downloads: child overrides
    parent.downloads = child.downloads;

    // Logging: child overrides
    parent.logging = child.logging.or(parent.logging);

    parent
}

/// Merge libraries: child libraries with same name replace parent libraries
fn merge_libraries(parent: Vec<crate::versions::model::Library>, child: Vec<crate::versions::model::Library>) -> Vec<crate::versions::model::Library> {
    let mut map: HashMap<String, crate::versions::model::Library> = parent.into_iter()
        .map(|lib| (lib.name.clone(), lib))
        .collect();

    for lib in child {
        map.insert(lib.name.clone(), lib);
    }

    map.into_values().collect()
}

/// Merge arguments: child arguments are appended to parent (for game) or prepended (for JVM)
fn merge_arguments(
    mut parent: crate::versions::model::Arguments,
    child: crate::versions::model::Arguments,
) -> crate::versions::model::Arguments {
    // For game arguments, child args are appended (they come after parent)
    parent.game.extend(child.game);

    // For JVM arguments, child args are prepended (they come before parent)
    let mut jvm = child.jvm;
    jvm.extend(parent.jvm);
    parent.jvm = jvm;

    parent
}

/// Load a version JSON with full inheritance resolution
pub async fn load_version_json(version_id: &str, versions_dir: &Path) -> Result<VersionJson> {
    let version_path = versions_dir.join(version_id).join(format!("{}.json", version_id));

    if !version_path.exists() {
        return Err(LauncherError::VersionNotFound(version_id.to_string()));
    }

    let content = tokio::fs::read_to_string(&version_path).await
        .map_err(|e| LauncherError::Io(e))?;

    let version_json: VersionJson = serde_json::from_str(&content)
        .map_err(|e| LauncherError::Serde(e))?;

    let mut visited = Vec::new();
    resolve_inheritance(version_json, versions_dir, &mut visited).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versions::model::{VersionJson, JavaVersion, AssetIndex, Downloads, DownloadInfo, Arguments};

    fn create_test_version(id: &str, inherits: Option<&str>, main_class: &str) -> VersionJson {
        VersionJson {
            id: id.to_string(),
            inherits_from: inherits.map(String::from),
            java_version: JavaVersion {
                component: "java-runtime-alpha".to_string(),
                major_version: 21,
            },
            main_class: main_class.to_string(),
            asset_index: AssetIndex {
                id: "1.20".to_string(),
                url: "".to_string(),
                sha1: "".to_string(),
                size: 0,
                total_size: 0,
            },
            assets: "1.20".to_string(),
            libraries: vec![],
            arguments: Arguments { game: vec![], jvm: vec![] },
            minecraft_arguments: None,
            downloads: Downloads {
                client: DownloadInfo { url: "".to_string(), sha1: "".to_string(), size: 0 },
                server: None,
                server_mappings: None,
            },
            logging: None,
        }
    }

    #[tokio::test]
    async fn test_no_inheritance() {
        let version = create_test_version("1.20.1", None, "net.minecraft.client.main.Main");
        let mut visited = Vec::new();
        let result = resolve_inheritance(version, Path::new("."), &mut visited).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "1.20.1");
    }
}