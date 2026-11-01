//! Download manager with concurrency, resume, retry, and SHA1 verification

pub mod manager;
pub mod hash;
pub mod progress;

pub use manager::{DownloadManager, DownloadTask, DownloadOptions};
pub use hash::{verify_sha1, compute_sha1};
pub use progress::{DownloadProgress, DownloadStage};