//! Download manager with concurrency, resume, retry, and SHA1 verification

pub mod hash;
pub mod manager;
pub mod progress;

pub use hash::{compute_sha1, verify_sha1};
pub use manager::{DownloadManager, DownloadOptions, DownloadTask};
pub use progress::{DownloadProgress, DownloadStage};
