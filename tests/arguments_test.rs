// Unit tests for argument parsing and placeholder substitution
use offlinelauncher::launch::arguments::{substitute_placeholders, LaunchArguments};
use offlinelauncher::accounts::OfflineAccount;
use offlinelauncher::versions::model::{VersionJson, JavaVersion, AssetIndex, Downloads, DownloadInfo, Arguments, Argument, ArgumentValue, LibraryRule, LibraryRuleAction};
use std::collections::HashMap;
use std::path::PathBuf;

fn create_test_version() -> VersionJson {
    VersionJson {
        id: "1.20.1".to_string(),
        inherits_from: None,
        java_version: JavaVersion {
            component: "java-runtime-alpha".to_string(),
            major_version: 21,
        },
        main_class: "net.minecraft.client.main.Main".to_string(),
        asset_index: AssetIndex {
            id: "1.20".to_string(),
            url: "".to_string(),
            sha1: "".to_string(),
            size: 0,
            total_size: 0,
        },
        assets: "1.20".to_string(),
        libraries: vec![],
        arguments: Arguments {
            game: vec![
                Argument::Simple("--username".to_string()),
                Argument::Simple("${auth_player_name}".to_string()),
                Argument::Simple("--uuid".to_string()),
                Argument::Simple("${auth_uuid}".to_string()),
                Argument::Simple("--accessToken".to_string()),
                Argument::Simple("${auth_access_token}".to_string()),
            ],
            jvm: vec![
                Argument::Simple("-Xmx${max_memory}".to_string()),
                Argument::Simple("-Djava.library.path=${natives_directory}".to_string()),
            ],
        },
        minecraft_arguments: None,
        downloads: Downloads {
            client: DownloadInfo { url: "".to_string(), sha1: "".to_string(), size: 0 },
            server: None,
            server_mappings: None,
        },
        logging: None,
    }
}

fn create_test_account() -> OfflineAccount {
    OfflineAccount::new("TestPlayer".into()).unwrap()
}

#[test]
fn test_substitute_placeholders_basic() {
    let mut placeholders = HashMap::new();
    placeholders.insert("name".to_string(), "Steve".to_string());
    placeholders.insert("uuid".to_string(), "12345678-1234-1234-1234-123456789012".to_string());

    let result = substitute_placeholders("--username ${name} --uuid ${uuid}", &placeholders);
    assert_eq!(result, "--username Steve --uuid 12345678-1234-1234-1234-123456789012");
}

#[test]
fn test_substitute_placeholders_missing() {
    let placeholders = HashMap::new();
    let result = substitute_placeholders("--username ${missing}", &placeholders);
    assert_eq!(result, "--username ${missing}"); // Unchanged if not found
}

#[test]
fn test_substitute_placeholders_multiple_same() {
    let mut placeholders = HashMap::new();
    placeholders.insert("x".to_string(), "y".to_string());

    let result = substitute_placeholders("${x} ${x} ${x}", &placeholders);
    assert_eq!(result, "y y y");
}

#[test]
fn test_launch_arguments_from_version() {
    let version = create_test_version();
    let account = create_test_account();

    let args = LaunchArguments::from_version(
        &version,
        &account,
        &PathBuf::from("/game"),
        &PathBuf::from("/assets"),
        "1.20",
        &PathBuf::from("/natives"),
        "/classpath",
        "OfflineLauncher",
        "0.1.0",
    ).unwrap();

    // Check JVM args
    assert!(args.jvm_args.iter().any(|a| a.contains("-Djava.library.path=/natives")));

    // Check game args
    assert!(args.game_args.iter().any(|a| a == "--username"));
    assert!(args.game_args.iter().any(|a| a == "TestPlayer"));
    assert!(args.game_args.iter().any(|a| a == "--uuid"));
    assert!(args.game_args.iter().any(|a| a == account.uuid().to_string().replace('-', "")));
    assert!(args.game_args.iter().any(|a| a == "--accessToken"));
    assert!(args.game_args.iter().any(|a| a == "0"));

    assert_eq!(args.main_class, "net.minecraft.client.main.Main");
}

#[test]
fn test_conditional_arguments() {
    use offlinelauncher::versions::model::{Arguments, Argument, ArgumentValue, LibraryRule, LibraryRuleAction, LibraryRuleOs, LibraryRuleOsName};

    let version = VersionJson {
        id: "test".to_string(),
        inherits_from: None,
        java_version: JavaVersion { component: "".to_string(), major_version: 21 },
        main_class: "Main".to_string(),
        asset_index: AssetIndex { id: "".to_string(), url: "".to_string(), sha1: "".to_string(), size: 0, total_size: 0 },
        assets: "".to_string(),
        libraries: vec![],
        arguments: Arguments {
            game: vec![
                Argument::Conditional(offlinelauncher::versions::model::ConditionalArgument {
                    rules: vec![LibraryRule {
                        action: LibraryRuleAction::Allow,
                        os: Some(LibraryRuleOs { name: LibraryRuleOsName::Windows, version: None, arch: None }),
                        features: None,
                    }],
                    value: ArgumentValue::Single("--windows-only".to_string()),
                }),
            ],
            jvm: vec![],
        },
        minecraft_arguments: None,
        downloads: Downloads { client: DownloadInfo { url: "".to_string(), sha1: "".to_string(), size: 0 }, server: None, server_mappings: None },
        logging: None,
    };

    let account = create_test_account();

    #[cfg(target_os = "windows")]
    {
        let args = LaunchArguments::from_version(&version, &account, &PathBuf::from("."), &PathBuf::from("."), "", &PathBuf::from("."), "", "", "").unwrap();
        assert!(args.game_args.contains(&"--windows-only".to_string()));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let args = LaunchArguments::from_version(&version, &account, &PathBuf::from("."), &PathBuf::from("."), "", &PathBuf::from("."), "", "", "").unwrap();
        assert!(!args.game_args.contains(&"--windows-only".to_string()));
    }
}