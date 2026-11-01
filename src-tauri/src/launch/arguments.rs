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
    map.insert("assets_index_name".to_string(), assets_index.to_string());
    map.insert("natives_directory".to_string(), natives_dir.to_string_lossy().to_string());
    map.insert("classpath".to_string(), classpath.to_string());
    map.insert("launcher_name".to_string(), launcher_name.to_string());
    map.insert("launcher_version".to_string(), launcher_version.to_string());

    // Additional placeholders used in newer versions
    map.insert("auth_session".to_string(), account.access_token().to_string());
    map.insert("user_properties".to_string(), "{}".to_string()); // Empty JSON for offline

    map
}

/// Parse modern arguments format (with rules)
fn parse_modern_arguments(
    arguments: &crate::versions::model::Arguments,
    placeholders: &HashMap<String, String>,
) -> Result<(Vec<String>, Vec<String>)> {
    let jvm_args = parse_argument_list(&arguments.jvm, placeholders)?;
    let game_args = parse_argument_list(&arguments.game, placeholders)?;
    Ok((jvm_args, game_args))
}

/// Parse legacy minecraftArguments string
fn parse_legacy_arguments(
    minecraft_args: Option<&str>,
    placeholders: &HashMap<String, String>,
) -> Result<(Vec<String>, Vec<String>)> {
    let args = minecraft_args.unwrap_or("");
    let substituted = substitute_placeholders(args, placeholders);
    let parsed = shell_words::split(&substituted)
        .map_err(|e| LauncherError::Launch(format!("Failed to parse legacy arguments: {}", e)))?;

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
                let substituted = substitute_placeholders(s, placeholders);
                // Split by spaces but respect quotes
                let parts = shell_words::split(&substituted)
                    .map_err(|e| LauncherError::Launch(format!("Failed to parse argument: {}", e)))?;
                result.extend(parts);
            }
            Argument::Conditional(cond) => {
                // Evaluate rules
                if evaluate_argument_rules(&cond.rules)? {
                    match &cond.value {
                        ArgumentValue::Single(s) => {
                            let substituted = substitute_placeholders(s, placeholders);
                            let parts = shell_words::split(&substituted)
                                .map_err(|e| LauncherError::Launch(format!("Failed to parse conditional argument: {}", e)))?;
                            result.extend(parts);
                        }
                        ArgumentValue::Multiple(vec) => {
                            for s in vec {
                                let substituted = substitute_placeholders(s, placeholders);
                                let parts = shell_words::split(&substituted)
                                    .map_err(|e| LauncherError::Launch(format!("Failed to parse conditional argument: {}", e)))?;
                                result.extend(parts);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Evaluate argument rules (reuse library rules logic)
fn evaluate_argument_rules(rules: &[crate::versions::model::LibraryRule]) -> Result<bool> {
    crate::versions::library::evaluate_library_rules(rules)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
}