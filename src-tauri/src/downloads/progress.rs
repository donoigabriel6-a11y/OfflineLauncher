//! Download progress reporting

use serde::{Deserialize, Serialize};
use crate::events::DownloadStage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub version_id: String,
    pub file_name: String,
    pub downloaded: u64,
    pub total: u64,
    pub stage: DownloadStage,
    pub speed_bps: Option<u64>, // bytes per second
    pub eta_seconds: Option<u64>,
}

impl DownloadProgress {
    pub fn new(version_id: String, file_name: String, stage: DownloadStage) -> Self {
        Self {
            version_id,
            file_name,
            downloaded: 0,
            total: 0,
            stage,
            speed_bps: None,
            eta_seconds: None,
        }
    }

    pub fn percent(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.downloaded as f32 / self.total as f32) * 100.0
        }
    }

    pub fn update(&mut self, downloaded: u64, total: u64) {
        self.downloaded = downloaded;
        self.total = total;
    }
}