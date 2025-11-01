use serde::{Deserialize, Serialize};

// Scan models
#[derive(Debug, Deserialize)]
pub struct CreateScanRequest {
    pub git_url: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub git_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub scan_id: String,
    pub status: String,
    pub created_at: String,
    pub git_url: String,
    pub fossology_status: String,
    pub semgrep_status: String,
}

#[derive(Debug, Serialize)]
pub struct ScanResultsResponse {
    pub scan_id: String,
    pub repository_url: String,
    pub scan_date: String,
    pub status: String,
    pub results: serde_json::Value,
}

// API Key models
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key: String,
    pub created_at: String,
    pub message: String,
}
