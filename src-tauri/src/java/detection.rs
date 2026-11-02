//! System Java detection

use crate::error::{LauncherError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// A detected Java installation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstallation {
    pub path: PathBuf,
    pub version: String,
    pub major_version: u8,
    pub vendor: String,
    pub is_jre: bool,
}

/// Detect all Java installations on the system
pub async fn detect_system_java() -> Result<Vec<JavaInstallation>> {
    let mut installations = Vec::new();

    // Check JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_exe = Path::new(&java_home).join("bin").join(java_executable_name());
        if java_exe.exists() {
            if let Ok(install) = parse_java_version(&java_exe).await {
                installations.push(install);
            }
        }
    }

    // Check PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let java_exe = dir.join(java_executable_name());
            if java_exe.exists() {
                if let Ok(install) = parse_java_version(&java_exe).await {
                    // Avoid duplicates
                    if !installations.iter().any(|i| i.path == install.path) {
                        installations.push(install);
                    }
                }
            }
        }
    }

    // Check common install directories
    for dir in common_java_dirs() {
        if let Ok(entries) = tokio::fs::read_dir(&dir).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                    let java_exe = entry.path().join("bin").join(java_executable_name());
                    if java_exe.exists() {
                        if let Ok(install) = parse_java_version(&java_exe).await {
                            if !installations.iter().any(|i| i.path == install.path) {
                                installations.push(install);
                            }
                        }
                    }
                }
            }
        }

    // Sort by version (newest first)
    installations.sort_by(|a, b| b.major_version.cmp(&a.major_version).then_with(|| b.version.cmp(&a.version)));

    Ok(installations)
}

pub(crate) fn java_executable_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "java.exe";
    #[cfg(not(target_os = "windows"))]
    return "java";
}

fn common_java_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "windows")]
    {
        dirs.push(PathBuf::from(r"C:\Program Files\Java"));
        dirs.push(PathBuf::from(r"C:\Program Files (x86)\Java"));
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            dirs.push(PathBuf::from(program_files).join("Java"));
        }
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            dirs.push(PathBuf::from(program_files_x86).join("Java"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
        dirs.push(PathBuf::from("/System/Library/Java/JavaVirtualMachines"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join("Library/Java/JavaVirtualMachines"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/lib/jvm"));
        dirs.push(PathBuf::from("/usr/java"));
        dirs.push(PathBuf::from("/opt/java"));
    }

    dirs
}

pub(crate) async fn parse_java_version(java_exe: &Path) -> Result<JavaInstallation> {
    let output = Command::new(java_exe)
        .arg("-version")
        .output()
        .await
        .map_err(|e| LauncherError::JavaNotFound(e.to_string()))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_output = format!("{}{}", stderr, stdout);

    // Parse version string like: openjdk version "21.0.1" 2023-10-17
    // or: java version "1.8.0_381"
    let version = extract_version(&version_output)
        .ok_or_else(|| LauncherError::JavaNotFound("Could not parse version".into()))?;

    let major = parse_major_version(&version)
        .ok_or_else(|| LauncherError::JavaNotFound("Could not parse major version".into()))?;

    let vendor = extract_vendor(&version_output);

    let path = java_exe.parent().unwrap().parent().unwrap().to_path_buf();

    Ok(JavaInstallation {
        path,
        version,
        major_version: major,
        vendor,
        is_jre: !java_exe.parent().unwrap().join("javac").exists(),
    })
}

fn extract_version(output: &str) -> Option<String> {
    // Try various version patterns
    for line in output.lines() {
        if line.contains("version") {
            // Pattern: version "X.Y.Z" or version "1.X.Y_Z"
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start+1..].find('"') {
                    return Some(line[start+1..start+1+end].to_string());
                }
            }
        }
    }
    None
}

fn parse_major_version(version: &str) -> Option<u8> {
    // Handle "1.8.0_381" -> 8, "11.0.1" -> 11, "21.0.1" -> 21
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        if parts[0] == "1" {
            parts[1].parse().ok()
        } else {
            parts[0].parse().ok()
        }
    } else {
        None
    }
}

fn extract_vendor(output: &str) -> String {
    let lower = output.to_lowercase();
    if lower.contains("oracle") { "Oracle".into() }
    else if lower.contains("openjdk") { "OpenJDK".into() }
    else if lower.contains("adoptium") || lower.contains("temurin") { "Eclipse Temurin".into() }
    else if lower.contains("azul") || lower.contains("zulu") { "Azul Zulu".into() }
    else if lower.contains("amazon") || lower.contains("corretto") { "Amazon Corretto".into() }
    else if lower.contains("microsoft") { "Microsoft".into() }
    else if lower.contains("ibm") { "IBM".into() }
    else { "Unknown".into() }
}

/// Find a Java installation matching the required major version
pub fn find_java_for_version(installations: &[JavaInstallation], required_major: u8) -> Option<&JavaInstallation> {
    // Prefer exact match
    installations.iter().find(|i| i.major_version == required_major)
        // Fall back to newer compatible version
        .or_else(|| installations.iter().find(|i| i.major_version > required_major))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_major_version() {
        assert_eq!(parse_major_version("1.8.0_381"), Some(8));
        assert_eq!(parse_major_version("11.0.1"), Some(11));
        assert_eq!(parse_major_version("17.0.5"), Some(17));
        assert_eq!(parse_major_version("21.0.1"), Some(21));
        assert_eq!(parse_major_version("1.7.0"), Some(7));
        assert_eq!(parse_major_version("9"), Some(9));
    }

    #[test]
    fn test_extract_vendor() {
        assert_eq!(extract_vendor("openjdk version \"21\""), "OpenJDK");
        assert_eq!(extract_vendor("java version \"1.8\" Oracle"), "Oracle");
        assert_eq!(extract_vendor("Eclipse Temurin 17"), "Eclipse Temurin");
        assert_eq!(extract_vendor("Azul Zulu 11"), "Azul Zulu");
        assert_eq!(extract_vendor("Amazon Corretto 11"), "Amazon Corretto");
    }
}