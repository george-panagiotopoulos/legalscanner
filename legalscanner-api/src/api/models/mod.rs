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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScanResultsResponse {
    pub scan_id: String,
    pub repository_url: String,
    pub scan_date: String,
    pub status: String,
    pub results: serde_json::Value,
}

// Risk Assessment models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub score: i32,                  // 0-100
    pub level: String,               // low, medium, high, critical
    pub factors: Vec<RiskFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub category: String,            // e.g., "copyleft_license", "missing_spdx"
    pub severity: String,            // low, medium, high, critical
    pub description: String,
    pub affected_count: i32,
    pub details: Vec<String>,        // file paths or license names
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
