// Unit tests for library rule evaluation
use offlinelauncher::versions::library::{evaluate_library_rules, get_native_classifier};
use offlinelauncher::versions::model::{LibraryRule, LibraryRuleAction, LibraryRuleOs, LibraryRuleOsName};

#[test]
fn test_empty_rules_allow() {
    assert!(evaluate_library_rules(&[]).unwrap());
}

#[test]
fn test_allow_rule_no_os() {
    let rules = vec![LibraryRule {
        action: LibraryRuleAction::Allow,
        os: None,
        features: None,
    }];
    assert!(evaluate_library_rules(&rules).unwrap());
}

#[test]
fn test_disallow_rule_no_os() {
    let rules = vec![LibraryRule {
        action: LibraryRuleAction::Disallow,
        os: None,
        features: None,
    }];
    assert!(!evaluate_library_rules(&rules).unwrap());
}

#[test]
fn test_os_specific_allow() {
    let current_os = current_os_name();
    let rules = vec![LibraryRule {
        action: LibraryRuleAction::Allow,
        os: Some(LibraryRuleOs {
            name: current_os,
            version: None,
            arch: None,
        }),
        features: None,
    }];
    assert!(evaluate_library_rules(&rules).unwrap());
}

#[test]
fn test_os_specific_disallow() {
    let current_os = current_os_name();
    let other_os = match current_os {
        LibraryRuleOsName::Windows => LibraryRuleOsName::Linux,
        _ => LibraryRuleOsName::Windows,
    };
    let rules = vec![LibraryRule {
        action: LibraryRuleAction::Disallow,
        os: Some(LibraryRuleOs {
            name: other_os,
            version: None,
            arch: None,
        }),
        features: None,
    }];
    // Should be allowed since rule doesn't apply to current OS
    assert!(evaluate_library_rules(&rules).unwrap());
}

#[test]
fn test_multiple_rules_last_wins() {
    let current_os = current_os_name();
    let rules = vec![
        LibraryRule {
            action: LibraryRuleAction::Allow,
            os: None,
            features: None,
        },
        LibraryRule {
            action: LibraryRuleAction::Disallow,
            os: Some(LibraryRuleOs {
                name: current_os,
                version: None,
                arch: None,
            }),
            features: None,
        },
    ];
    // Last matching rule wins - disallow for current OS
    assert!(!evaluate_library_rules(&rules).unwrap());
}

#[test]
fn test_native_classifier() {
    let classifier = get_native_classifier();
    assert!(classifier.is_some());

    #[cfg(target_os = "windows")]
    {
        assert!(classifier.unwrap().contains("windows"));
    }
    #[cfg(target_os = "macos")]
    {
        assert!(classifier.unwrap().contains("macos") || classifier.unwrap().contains("osx"));
    }
    #[cfg(target_os = "linux")]
    {
        assert!(classifier.unwrap().contains("linux"));
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
        LibraryRuleOsName::Linux
    }
}