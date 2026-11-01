//! Version JSON model structures matching Mojang's format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Version manifest entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: VersionType,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "sha1")]
    pub sha1: Option<String>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u32,
}

/// Version type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

/// Full version manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

/// Version JSON (the main version file)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub id: String,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    pub libraries: Vec<Library>,
    pub arguments: Arguments,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    pub downloads: Downloads,
    #[serde(default)]
    pub logging: Option<Logging>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    #[serde(rename = "component")]
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
    #[serde(rename = "sha1")]
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub client: DownloadInfo,
    #[serde(rename = "server")]
    pub server: Option<DownloadInfo>,
    #[serde(rename = "serverMappings")]
    pub server_mappings: Option<DownloadInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadInfo {
    pub url: String,
    #[serde(rename = "sha1")]
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub rules: Vec<LibraryRule>,
    #[serde(rename = "extract", default)]
    pub extract: Option<ExtractRules>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadInfo>,
    #[serde(rename = "classifiers")]
    pub classifiers: Option<HashMap<String, DownloadInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractRules {
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRule {
    pub action: LibraryRuleAction,
    #[serde(default)]
    pub os: Option<LibraryRuleOs>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryRuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRuleOs {
    pub name: LibraryRuleOsName,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryRuleOsName {
    Windows,
    Osx,
    Linux,
    #[serde(rename = "windows-arm")]
    WindowsArm,
    #[serde(rename = "osx-arm")]
    OsxArm,
    #[serde(rename = "linux-arm")]
    LinuxArm,
    #[serde(rename = "linux-aarch64")]
    LinuxAarch64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Simple(String),
    Conditional(ConditionalArgument),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalArgument {
    pub rules: Vec<LibraryRule>,
    pub value: ArgumentValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    pub client: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingConfig {
    pub argument: String,
    #[serde(rename = "file")]
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingFile {
    pub id: String,
    #[serde(rename = "sha1")]
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

impl VersionJson {
    /// Get the effective Java major version
    pub fn java_major_version(&self) -> u8 {
        self.java_version.major_version
    }

    /// Check if this version uses modern arguments format
    pub fn has_modern_arguments(&self) -> bool {
        !self.arguments.game.is_empty() || !self.arguments.jvm.is_empty()
    }
}