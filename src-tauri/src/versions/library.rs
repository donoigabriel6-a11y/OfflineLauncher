//! Library rules evaluation for OS/arch filtering

use crate::versions::model::{LibraryRule, LibraryRuleAction, LibraryRuleOs, LibraryRuleOsName};
use crate::error::{LauncherError, Result};

/// Evaluate library rules for the current platform
pub fn evaluate_library_rules(rules: &[LibraryRule]) -> Result<bool> {
    if rules.is_empty() {
        return Ok(true); // No rules = allow
    }

    let current_os = current_os_name();
    let current_arch = current_arch();
    let current_version = current_os_version();

    let mut allowed = false;

    for rule in rules {
        let rule_applies = matches_rule(&rule, &current_os, &current_arch, &current_version)?;

        if rule_applies {
            match rule.action {
                LibraryRuleAction::Allow => allowed = true,
                LibraryRuleAction::Disallow => allowed = false,
            }
        }
    }

    Ok(allowed)
}

fn matches_rule(
    rule: &LibraryRule,
    current_os: &LibraryRuleOsName,
    current_arch: &str,
    current_version: &str,
) -> Result<bool> {
    // Check OS rule
    if let Some(os_rule) = &rule.os {
        if os_rule.name != *current_os {
            return Ok(false);
        }

        // Check OS version if specified
        if let Some(version) = &os_rule.version {
            if !version_match(version, current_version) {
                return Ok(false);
            }
        }

        // Check architecture if specified
        if let Some(arch) = &os_rule.arch {
            if !arch_match(arch, current_arch) {
                return Ok(false);
            }
        }
    }

    // Check features (not implemented yet, always true for now)
    if let Some(features) = &rule.features {
        // Feature rules would go here
        // For now, we ignore them
        let _ = features;
    }

    Ok(true)
}

fn version_match(pattern: &str, actual: &str) -> bool {
    // Simple prefix match for version
    actual.starts_with(pattern)
}

fn arch_match(pattern: &str, actual: &str) -> bool {
    // Normalize architecture names
    let norm_pattern = normalize_arch(pattern);
    let norm_actual = normalize_arch(actual);
    norm_pattern == norm_actual
}

fn normalize_arch(arch: &str) -> String {
    match arch.to_lowercase().as_str() {
        "x86_64" | "amd64" | "x64" => "x86_64".to_string(),
        "aarch64" | "arm64" => "aarch64".to_string(),
        "arm" | "armv7" | "armv7l" => "arm".to_string(),
        "i386" | "i686" | "x86" => "x86".to_string(),
        other => other.to_string(),
    }
}

fn current_os_name() -> LibraryRuleOsName {
    #[cfg(target_os = "windows")]
    {
        if cfg!(target_arch = "aarch64") {
            LibraryRuleOsName::WindowsArm
        } else {
            LibraryRuleOsName::Windows
        }
    }
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            LibraryRuleOsName::OsxArm
        } else {
            LibraryRuleOsName::Osx
        }
    }
    #[cfg(target_os = "linux")]
    {
        if cfg!(target_arch = "aarch64") {
            LibraryRuleOsName::LinuxAarch64
        } else if cfg!(target_arch = "arm") {
            LibraryRuleOsName::LinuxArm
        } else {
            LibraryRuleOsName::Linux
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        LibraryRuleOsName::Linux // Fallback
    }
}

fn current_arch() -> String {
    std::env::consts::ARCH.to_string()
}

fn current_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        // Get Windows version
        "10.0".to_string() // Simplified
    }
    #[cfg(target_os = "macos")]
    {
        // Get macOS version
        "13.0".to_string() // Simplified
    }
    #[cfg(target_os = "linux")]
    {
        // Get Linux kernel version
        "5.15".to_string() // Simplified
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

/// Get the native classifier for the current platform
pub fn get_native_classifier() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        if cfg!(target_arch = "aarch64") {
            Some("windows-arm64".to_string())
        } else {
            Some("natives-windows".to_string())
        }
    }
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            Some("natives-macos-arm64".to_string())
        } else {
            Some("natives-macos".to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        if cfg!(target_arch = "aarch64") {
            Some("natives-linux-aarch64".to_string())
        } else if cfg!(target_arch = "arm") {
            Some("natives-linux-arm".to_string())
        } else {
            Some("natives-linux".to_string())
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versions::model::{LibraryRule, LibraryRuleAction, LibraryRuleOs, LibraryRuleOsName};

    #[test]
    fn test_empty_rules_allow() {
        assert!(evaluate_library_rules(&[]).unwrap());
    }

    #[test]
    fn test_allow_rule() {
        let rules = vec![LibraryRule {
            action: LibraryRuleAction::Allow,
            os: None,
            features: None,
        }];
        assert!(evaluate_library_rules(&rules).unwrap());
    }

    #[test]
    fn test_disallow_rule() {
        let rules = vec![LibraryRule {
            action: LibraryRuleAction::Disallow,
            os: None,
            features: None,
        }];
        assert!(!evaluate_library_rules(&rules).unwrap());
    }

    #[test]
    fn test_os_specific_allow() {
        let current = current_os_name();
        let rules = vec![LibraryRule {
            action: LibraryRuleAction::Allow,
            os: Some(LibraryRuleOs {
                name: current,
                version: None,
                arch: None,
            }),
            features: None,
        }];
        assert!(evaluate_library_rules(&rules).unwrap());
    }

    #[test]
    fn test_os_specific_disallow() {
        let current = current_os_name();
        let other = match current {
            LibraryRuleOsName::Windows => LibraryRuleOsName::Linux,
            _ => LibraryRuleOsName::Windows,
        };
        let rules = vec![LibraryRule {
            action: LibraryRuleAction::Disallow,
            os: Some(LibraryRuleOs {
                name: other,
                version: None,
                arch: None,
            }),
            features: None,
        }];
        // Should be allowed since rule doesn't apply to current OS
        assert!(evaluate_library_rules(&rules).unwrap());
    }

    #[test]
    fn test_get_native_classifier() {
        let classifier = get_native_classifier();
        assert!(classifier.is_some());
        println!("Native classifier: {:?}", classifier);
    }
}