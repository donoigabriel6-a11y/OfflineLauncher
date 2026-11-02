use thiserror::Error;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("SHA1 verification failed for {file}: expected {expected}, got {actual}")]
    HashMismatch { file: String, expected: String, actual: String },

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Java not found: {0}")]
    JavaNotFound(String),

    #[error("Java version mismatch: required {required}, found {found}")]
    JavaVersionMismatch { required: u8, found: u8 },

    #[error("Download error: {0}")]
    Download(String),

    #[error("Launch error: {0}")]
    Launch(String),

    #[error("Library rule evaluation error: {0}")]
    LibraryRule(String),

    #[error("Inheritance resolution error: {0}")]
    Inheritance(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Mod loader error: {0}")]
    ModLoader(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Other error: {0}")]
    Other(String),
}

impl serde::Serialize for LauncherError { fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: serde::Serializer { serializer.serialize_str(&self.to_string()) } }

pub type Result<T> = std::result::Result<T, LauncherError>;