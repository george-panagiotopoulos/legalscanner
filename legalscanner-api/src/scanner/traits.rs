use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub file_path: String,
    pub licenses: Vec<LicenseFinding>,
    pub copyrights: Vec<CopyrightFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFinding {
    pub name: String,
    pub spdx_id: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyrightFinding {
    pub statement: String,
    pub holders: Vec<String>,
    pub years: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Scanner unavailable: {0}")]
    Unavailable(String),

    #[error("Scan failed: {0}")]
    Failed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[async_trait]
pub trait Scanner: Send + Sync {
    /// Returns the name/identifier of this scanner
    fn name(&self) -> &str;

    /// Scans a repository at the given path
    /// Returns a list of results for each file scanned
    async fn scan(&self, repo_path: &Path) -> Result<Vec<ScanResult>, ScanError>;

    /// Checks if the scanner is available and healthy
    async fn health_check(&self) -> Result<(), ScanError>;
}
