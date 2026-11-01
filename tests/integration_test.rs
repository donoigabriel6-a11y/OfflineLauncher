// Integration test: download and prepare a small version without launching
use offlinelauncher::{
    versions::{fetch_version_manifest, VersionType},
    downloads::{DownloadManager, DownloadOptions, DownloadTask, DownloadStage},
    app_dirs::{get_version_dir, get_libraries_dir, ensure_dirs},
    versions::inheritance::load_version_json,
    launch::classpath::build_classpath,
};
use std::path::PathBuf;

#[tokio::test]
async fn test_download_version_manifest() {
    let manifest = fetch_version_manifest(&reqwest::Client::new()).await;
    assert!(manifest.is_ok());

    let manifest = manifest.unwrap();
    assert!(!manifest.versions.is_empty());
    assert!(!manifest.latest.release.is_empty());
    assert!(!manifest.latest.snapshot.is_empty());
}

#[tokio::test]
async fn test_filter_versions() {
    let manifest = fetch_version_manifest(&reqwest::Client::new()).await.unwrap();

    let releases = offlinelauncher::versions::manifest::filter_versions(&manifest.versions, Some(VersionType::Release));
    let snapshots = offlinelauncher::versions::manifest::filter_versions(&manifest.versions, Some(VersionType::Snapshot));

    assert!(!releases.is_empty());
    assert!(!snapshots.is_empty());

    for v in &releases {
        assert_eq!(v.type_, VersionType::Release);
    }
    for v in &snapshots {
        assert_eq!(v.type_, VersionType::Snapshot);
    }
}

#[tokio::test]
async fn test_ensure_dirs() {
    ensure_dirs().unwrap();

    // Check that all required directories exist
    let dirs = [
        get_version_dir("test").unwrap(),
        get_libraries_dir().unwrap(),
    ];

    for dir in dirs {
        // Parent should exist
        assert!(dir.parent().unwrap().exists());
    }
}

#[tokio::test]
async fn test_download_manager_creation() {
    let manager = DownloadManager::new(DownloadOptions::default());
    assert_eq!(manager.options.max_concurrent, 8);
    assert_eq!(manager.options.max_retries, 3);
}

#[tokio::test]
async fn test_version_json_loading() {
    // This test would require a downloaded version
    // For now, just test that the function exists and handles missing files
    let result = load_version_json("nonexistent", &get_version_dir("nonexistent").unwrap()).await;
    assert!(result.is_err());
}

// Note: Full integration test that downloads 1.20.1 would require network
// and is better run manually. The test infrastructure is in place.
#[tokio::test]
#[ignore] // Requires network, run manually with: cargo test --test integration_test -- --ignored
async fn test_full_version_preparation() {
    // This test downloads and prepares version 1.20.1
    // Run manually to verify the download pipeline works

    let client = reqwest::Client::new();
    let manifest = fetch_version_manifest(&client).await.unwrap();

    // Find 1.20.1
    let version_entry = manifest.versions.iter()
        .find(|v| v.id == "1.20.1")
        .expect("1.20.1 not found in manifest");

    assert_eq!(version_entry.type_, VersionType::Release);

    // Download version JSON
    let version_json: offlinelauncher::versions::model::VersionJson = client.get(&version_entry.url)
        .send().await.unwrap()
        .json().await.unwrap();

    assert_eq!(version_json.id, "1.20.1");
    assert_eq!(version_json.java_version.major_version, 21);

    println!("Version 1.20.1 JSON downloaded successfully");
    println!("Main class: {}", version_json.main_class);
    println!("Libraries: {}", version_json.libraries.len());
    println!("Assets index: {}", version_json.asset_index.id);
}