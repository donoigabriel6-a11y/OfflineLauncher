//! Launch argument parsing and placeholder substitution

use crate::accounts::model::Account;
use crate::error::{LauncherError, Result};
use crate::versions::model::{VersionJson, Argument, ArgumentValue};
use std::collections::HashMap;
use std::path::Path;

/// Build complete launch arguments from version JSON and account info
pub struct LaunchArguments {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub main_class: String,
}

impl LaunchArguments {
    /// Build arguments from version JSON
    pub fn from_version(
        version: &VersionJson,
        account: &dyn Account,
        game_dir: &Path,
        assets_dir: &Path,
        libraries_dir: &Path,
        assets_index: &str,
        natives_dir: &Path,
        classpath: &str,
        launcher_name: &str,
        launcher_version: &str,
    ) -> Result<Self> {
        let placeholders = build_placeholders(
            account,
            game_dir,
            assets_dir,
            libraries_dir,
            assets_index,
            natives_dir,
            classpath,
            &version.id,
            launcher_name,
            launcher_version,
        );

        let (jvm_args, game_args) = if version.has_modern_arguments() {
            parse_modern_arguments(&version.arguments, &placeholders)?
        } else {
            parse_legacy_arguments(version.minecraft_arguments.as_deref(), &placeholders)?
        };

        Ok(Self {
            jvm_args,
            game_args,
            main_class: version.main_class.clone(),
        })
    }
}

/// Build all placeholder values
fn build_placeholders(
    account: &dyn Account,
    game_dir: &Path,
    assets_dir: &Path,
    libraries_dir: &Path,
    assets_index: &str,
    natives_dir: &Path,
    classpath: &str,
    version_name: &str,
    launcher_name: &str,
    launcher_version: &str,
) -> HashMap<String, String> {
    let mut map = HashMap::new();

    map.insert("auth_player_name".to_string(), account.username().to_string());
    map.insert("auth_uuid".to_string(), account.uuid().to_string().replace('-', ""));
    map.insert("auth_access_token".to_string(), account.access_token().to_string());
    map.insert("user_type".to_string(), format!("{:?}", account.user_type()).to_lowercase());
    map.insert("version_name".to_string(), version_name.to_string());
    map.insert("game_directory".to_string(), game_dir.to_string_lossy().to_string());
    map.insert("assets_root".to_string(), assets_dir.to_string_lossy().to_string());
    map.insert("library_directory".to_string(), libraries_dir.to_string_lossy().to_string());
    map.insert("classpath_separator".to_string(), if cfg!(target_os = "windows") { ";".to_string() } else { ":".to_string() });
    map.insert("assets_index_name".to_string(), assets_index.to_string());
    map.insert("natives_directory".to_string(), natives_dir.to_string_lossy().to_string());
    map.insert("classpath".to_string(), classpath.to_string());
    map.insert("launcher_name".to_string(), launcher_name.to_string());
    map.insert("launcher_version".to_string(), launcher_version.to_string());

    // Additional placeholders used in newer versions
    map.insert("auth_session".to_string(), account.access_token().to_string());
    map.insert("clientid".to_string(), "0".to_string());
    map.insert("auth_xuid".to_string(), "0".to_string());
    map.insert("user_properties".to_string(), "{}".to_string()); // Empty JSON for offline

    // Quick play placeholders (used in modern version metadata)
    // Set to empty string so they get substituted and don't end up as literal arguments
    map.insert("quickPlayPath".to_string(), "".to_string());
    map.insert("quickPlaySingleplayer".to_string(), "".to_string());
    map.insert("quickPlayMultiplayer".to_string(), "".to_string());
    map.insert("quickPlayRealms".to_string(), "".to_string());

    map
}

/// Parse modern arguments format (with rules)
fn parse_modern_arguments(
    arguments: &crate::versions::model::Arguments,
    placeholders: &HashMap<String, String>,
) -> Result<(Vec<String>, Vec<String>)> {
    // New Mojang metadata separates default user JVM options from the
    // platform-conditional JVM options. Preserve both groups in launch order.
    let mut jvm_args = parse_argument_list(&arguments.default_user_jvm, placeholders)?;
    jvm_args.extend(parse_argument_list(&arguments.jvm, placeholders)?);
    let game_args = parse_argument_list(&arguments.game, placeholders)?;
    Ok((jvm_args, game_args))
}

/// Parse legacy minecraftArguments string
fn parse_legacy_arguments(
    minecraft_args: Option<&str>,
    placeholders: &HashMap<String, String>,
) -> Result<(Vec<String>, Vec<String>)> {
    let args = minecraft_args.unwrap_or("");
    // Tokenize before substitution so paths containing spaces remain one argument.
    let parsed = shell_words::split(args)
        .map_err(|e| LauncherError::Launch(format!("Failed to parse legacy arguments: {}", e)))?
        .into_iter()
        .map(|argument| substitute_placeholders(&argument, placeholders))
        .collect();

    // For legacy, all args are game args; JVM args are empty (handled separately)
    Ok((Vec::new(), parsed))
}

/// Parse a list of arguments (simple or conditional)
fn parse_argument_list(
    args: &[Argument],
    placeholders: &HashMap<String, String>,
) -> Result<Vec<String>> {
    let mut result = Vec::new();

    for arg in args {
        match arg {
            Argument::Simple(s) => {
                // Each modern manifest entry is already one process argument.
                result.push(substitute_placeholders(s, placeholders));
            }
            Argument::Conditional(cond) => {
                // Evaluate rules
                // An omitted rules array means the argument is unconditional.
                if cond.rules.is_empty() || evaluate_argument_rules(&cond.rules)? {
                    match &cond.value {
                        ArgumentValue::Single(s) => {
                            result.push(substitute_placeholders(s, placeholders));
                        }
                        ArgumentValue::Multiple(values) => {
                            result.extend(values.iter().map(|value| substitute_placeholders(value, placeholders)));
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Evaluate Mojang argument rules without coupling argument deserialization to
/// the stricter library-rule schema. Mojang can add OS labels and fields over time.
fn evaluate_argument_rules(rules: &[serde_json::Value]) -> Result<bool> {
    if rules.is_empty() {
        return Ok(false);
    }

    let current_os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        std::env::consts::OS
    };
    let current_arch = std::env::consts::ARCH;
    let mut allowed = false;

    for rule in rules {
        let Some(action) = rule.get("action").and_then(serde_json::Value::as_str) else {
            continue;
        };

        let os_matches = match rule.get("os") {
            None | Some(serde_json::Value::Null) => true,
            Some(os) => {
                let name_matches = os.get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(|name| name == current_os)
                    .unwrap_or(true);
                let arch_matches = os.get("arch")
                    .and_then(serde_json::Value::as_str)
                    .map(|arch| {
                        let normalized = match arch {
                            "x86_64" | "amd64" | "x64" => "x86_64",
                            "aarch64" | "arm64" => "aarch64",
                            "x86" | "i386" | "i686" => "x86",
                            other => other,
                        };
                        let actual = match current_arch {
                            "x86_64" => "x86_64",
                            "aarch64" => "aarch64",
                            "x86" => "x86",
                            other => other,
                        };
                        normalized == actual
                    })
                    .unwrap_or(true);
                name_matches && arch_matches
            }
        };

        if os_matches {
            match action {
                "allow" => allowed = true,
                "disallow" => allowed = false,
                _ => {}
            }
        }
    }

    Ok(allowed)
}

/// Substitute all ${placeholder} occurrences in a string
pub fn substitute_placeholders(template: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    for (key, value) in placeholders {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}

/// Deduplicate GC flags - keep only the LAST GC flag overall
/// to avoid "Multiple garbage collectors selected" error
/// Java only allows ONE garbage collector to be selected
pub fn deduplicate_gc_flags(args: &mut Vec<String>) {
    // GC flags that conflict with each other (mutually exclusive)
    let gc_flags = [
        "-XX:+UseG1GC",
        "-XX:+UseZGC",
        "-XX:+UseShenandoahGC",
        "-XX:+UseParallelGC",
        "-XX:+UseSerialGC",
        "-XX:+UseConcMarkSweepGC",
    ];

    // Find the index of the LAST GC flag in the entire argument list
    let mut last_gc_index: Option<usize> = None;

    for (i, arg) in args.iter().enumerate() {
        for gc in &gc_flags {
            if arg.starts_with(gc) {
                last_gc_index = Some(i);
                break;
            }
        }
    }

    // If we found at least one GC flag, remove ALL others
    if let Some(keep_index) = last_gc_index {
        let mut indices_to_remove = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if i == keep_index {
                continue;
            }
            for gc in &gc_flags {
                if arg.starts_with(gc) {
                    indices_to_remove.push(i);
                    break;
                }
            }
        }

        // Remove duplicate GC flags (from end to start to preserve indices)
        for &idx in indices_to_remove.iter().rev() {
            args.remove(idx);
        }
    }
}

/// Deduplicate game arguments that conflict with each other
/// Currently handles: quick play options (--quickPlayPath, --quickPlaySingleplayer, --quickPlayMultiplayer, --quickPlayRealms)
pub fn deduplicate_game_args(args: &mut Vec<String>) {
    // Quick play options that conflict with each other
    let quick_play_options = [
        "--quickPlayPath",
        "--quickPlaySingleplayer",
        "--quickPlayMultiplayer",
        "--quickPlayRealms",
    ];

    // First pass: remove quick play flags that have empty string as their next argument (value)
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        for qp in &quick_play_options {
            if arg == *qp {
                // Check if next argument is an empty string (placeholder was substituted with "")
                if i + 1 < args.len() && args[i + 1].is_empty() {
                    // Remove both the flag and its empty value
                    args.remove(i + 1); // Remove value first
                    args.remove(i);     // Then remove flag
                    if i > 0 { i -= 1; } // Adjust index
                    continue;
                }
            }
        }
        i += 1;
    }

    // Find the index of the LAST quick play option
    let mut last_qp_index: Option<usize> = None;

    for (i, arg) in args.iter().enumerate() {
        for qp in &quick_play_options {
            if arg == *qp {
                last_qp_index = Some(i);
                break;
            }
        }
    }

    // If we found at least one quick play option, remove ALL others
    if let Some(keep_index) = last_qp_index {
        let mut indices_to_remove = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if i == keep_index {
                continue;
            }
            for qp in &quick_play_options {
                if arg == *qp {
                    indices_to_remove.push(i);
                    break;
                }
            }
        }

        // Remove duplicate quick play options (from end to start to preserve indices)
        for &idx in indices_to_remove.iter().rev() {
            args.remove(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_modern_arguments_keep_paths_with_spaces_as_single_values() {
        let mut placeholders = HashMap::new();
        placeholders.insert("game_directory".to_string(), "C:/Users/Test User/.minecraft".to_string());
        let arguments = crate::versions::model::Arguments {
            game: vec![
                Argument::Simple("--gameDir".to_string()),
                Argument::Simple("${game_directory}".to_string()),
            ],
            jvm: vec![],
            default_user_jvm: vec![],
        };

        let (_, game_args) = parse_modern_arguments(&arguments, &placeholders).unwrap();
        assert_eq!(game_args, vec!["--gameDir".to_string(), "C:/Users/Test User/.minecraft".to_string()]);
    }

    #[test]
    fn test_mojang_default_user_jvm_arguments_without_rules_deserialize() {
        let json = r#"{
            "default-user-jvm": [
                {"value": ["-Xms2G", "-Xmx4G"]},
                {"rules": [{"action": "allow", "os": {"name": "windows"}}], "value": ["-Dexample=true"]}
            ],
            "game": [],
            "jvm": []
        }"#;
        let arguments: crate::versions::model::Arguments = serde_json::from_str(json).unwrap();
        let mut placeholders = HashMap::new();
        let (jvm, game) = parse_modern_arguments(&arguments, &placeholders).unwrap();
        assert_eq!(jvm, vec!["-Xms2G", "-Xmx4G"]);
        assert!(game.is_empty());
    }

    #[test]
    fn test_legacy_arguments_keep_paths_with_spaces_as_single_values() {
        let mut placeholders = HashMap::new();
        placeholders.insert("game_directory".to_string(), "C:/Users/Test User/.minecraft".to_string());

        let (_, game_args) = parse_legacy_arguments(
            Some("--gameDir ${game_directory} --username Steve"),
            &placeholders,
        ).unwrap();

        assert_eq!(game_args, vec!["--gameDir".to_string(), "C:/Users/Test User/.minecraft".to_string(), "--username".to_string(), "Steve".to_string()]);
    }

    #[test]
    fn test_substitute_placeholders() {
        let mut placeholders = HashMap::new();
        placeholders.insert("test".to_string(), "value".to_string());
        placeholders.insert("name".to_string(), "Steve".to_string());

        let result = substitute_placeholders("${test} and ${name}", &placeholders);
        assert_eq!(result, "value and Steve");
    }

    #[test]
    fn test_substitute_missing_placeholder() {
        let placeholders = HashMap::new();
        let result = substitute_placeholders("${missing}", &placeholders);
        assert_eq!(result, "${missing}"); // Unchanged if not found
    }

    #[test]
    fn test_substitute_multiple_same() {
        let mut placeholders = HashMap::new();
        placeholders.insert("x".to_string(), "y".to_string());

        let result = substitute_placeholders("${x} ${x} ${x}", &placeholders);
        assert_eq!(result, "y y y");
    }

    #[test]
    fn test_deduplicate_gc_flags_keeps_last() {
        let mut args = vec![
            "-XX:+UseG1GC".to_string(),
            "-Xms512m".to_string(),
            "-XX:+UseZGC".to_string(),
            "-Xmx2G".to_string(),
        ];
        deduplicate_gc_flags(&mut args);
        // Should keep ONLY the last GC flag (UseZGC) and remove all others
        assert_eq!(args, vec!["-Xms512m".to_string(), "-XX:+UseZGC".to_string(), "-Xmx2G".to_string()]);
    }

    #[test]
    fn test_deduplicate_gc_flags_multiple_duplicates() {
        let mut args = vec![
            "-XX:+UseG1GC".to_string(),
            "-XX:+UseZGC".to_string(),
            "-XX:+UseG1GC".to_string(),
            "-XX:+UseShenandoahGC".to_string(),
        ];
        deduplicate_gc_flags(&mut args);
        // Should keep ONLY the very last GC flag (UseShenandoahGC)
        assert_eq!(args, vec!["-XX:+UseShenandoahGC".to_string()]);
    }

    #[test]
    fn test_deduplicate_gc_flags_no_duplicates() {
        let mut args = vec![
            "-XX:+UseG1GC".to_string(),
            "-Xms512m".to_string(),
            "-Xmx2G".to_string(),
        ];
        deduplicate_gc_flags(&mut args);
        // Should remain unchanged (only one GC flag)
        assert_eq!(args, vec!["-XX:+UseG1GC".to_string(), "-Xms512m".to_string(), "-Xmx2G".to_string()]);
    }

    #[test]
    fn test_deduplicate_gc_flags_no_gc_flags() {
        let mut args = vec![
            "-Xms512m".to_string(),
            "-Xmx2G".to_string(),
        ];
        deduplicate_gc_flags(&mut args);
        // Should remain unchanged
        assert_eq!(args, vec!["-Xms512m".to_string(), "-Xmx2G".to_string()]);
    }

    #[test]
    fn test_deduplicate_game_args_keeps_last_quick_play() {
        let mut args = vec![
            "--quickPlaySingleplayer".to_string(),
            "--username".to_string(),
            "Steve".to_string(),
            "--quickPlayPath".to_string(),
            "/some/path".to_string(),
        ];
        deduplicate_game_args(&mut args);
        // Should keep only the last quick play option (--quickPlayPath)
        assert_eq!(args, vec!["--username".to_string(), "Steve".to_string(), "--quickPlayPath".to_string(), "/some/path".to_string()]);
    }

    #[test]
    fn test_deduplicate_game_args_multiple_quick_play() {
        let mut args = vec![
            "--quickPlaySingleplayer".to_string(),
            "--quickPlayMultiplayer".to_string(),
            "--quickPlayRealms".to_string(),
        ];
        deduplicate_game_args(&mut args);
        // Should keep only the very last quick play option (--quickPlayRealms)
        assert_eq!(args, vec!["--quickPlayRealms".to_string()]);
    }

    #[test]
    fn test_deduplicate_game_args_no_duplicates() {
        let mut args = vec![
            "--username".to_string(),
            "Steve".to_string(),
            "--quickPlaySingleplayer".to_string(),
        ];
        deduplicate_game_args(&mut args);
        // Should remain unchanged
        assert_eq!(args, vec!["--username".to_string(), "Steve".to_string(), "--quickPlaySingleplayer".to_string()]);
    }

    #[test]
    fn test_deduplicate_game_args_no_quick_play() {
        let mut args = vec![
            "--username".to_string(),
            "Steve".to_string(),
            "--gameDir".to_string(),
            "/path".to_string(),
        ];
        deduplicate_game_args(&mut args);
        // Should remain unchanged
        assert_eq!(args, vec!["--username".to_string(), "Steve".to_string(), "--gameDir".to_string(), "/path".to_string()]);
    }
}