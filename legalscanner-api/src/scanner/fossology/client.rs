use base64::{engine::general_purpose, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::scanner::traits::ScanError;

#[derive(Clone)]
pub struct FossologyClient {
    base_url: String,
    api_token: String,
    username: String,
    password: String,
    client: Client,
}

#[derive(Debug, Serialize)]
pub struct UploadRequest {
    pub folder_id: i32,
    pub upload_description: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    pub code: i32,
    pub message: i32,  // This is actually the upload_id
    #[serde(rename = "type")]
    pub response_type: String,
}

#[derive(Debug, Serialize)]
pub struct JobRequest {
    pub upload_id: i32,
    pub folder_id: i32,
    pub analysis: AnalysisSpec,
}

#[derive(Debug, Serialize)]
pub struct AnalysisSpec {
    pub bucket: bool,
    pub copyright_email_author: bool,
    pub ecc: bool,
    pub keyword: bool,
    pub mime: bool,
    pub monk: bool,
    pub nomos: bool,
    pub ojo: bool,
    pub package: bool,
}

impl Default for AnalysisSpec {
    fn default() -> Self {
        Self {
            bucket: true,
            copyright_email_author: true,
            ecc: true,  // Enable ECC scanning for export control detection
            keyword: false,
            mime: true,
            monk: true,
            nomos: true,
            ojo: true,
            package: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct JobResponse {
    pub code: i32,
    pub message: i32,  // This is actually the job_id
    #[serde(rename = "type")]
    pub response_type: String,
}

#[derive(Debug, Deserialize)]
pub struct JobStatus {
    pub id: i32,
    pub name: String,
    pub status: String,
    pub eta: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct LicenseResult {
    pub file_path: String,
    pub findings: Vec<LicenseFinding>,
}

#[derive(Debug, Deserialize)]
pub struct LicenseFinding {
    pub license: String,
    pub spdx_id: Option<String>,
    pub match_percentage: f32,
}

#[derive(Debug, Deserialize)]
pub struct CopyrightResult {
    pub file_path: String,
    pub findings: Vec<CopyrightFinding>,
}

#[derive(Debug, Deserialize)]
pub struct CopyrightFinding {
    pub content: String,
    #[serde(rename = "type")]
    pub finding_type: String,
}

// Fossology API response structures
#[derive(Debug, Deserialize)]
pub struct FossologyLicenseResponse {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub findings: Option<FossologyFindings>,
}

#[derive(Debug, Deserialize)]
pub struct FossologyFindings {
    pub scanner: Option<Vec<String>>,
    pub conclusion: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FossologyLicenseMatch {
    #[serde(rename = "shortName", alias = "license")]
    pub short_name: String,
    #[serde(rename = "spdxId", alias = "spdx_id")]
    pub spdx_id: Option<String>,
    pub percentage: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct FossologyCopyrightResponse {
    #[serde(rename = "filePath")]
    pub file_path: Vec<String>,  // Fossology returns array of file paths
    pub copyright: String,  // Simple copyright statement string
}

#[derive(Debug, Deserialize)]
pub struct UploadDetails {
    pub id: i32,
    #[serde(rename = "folderid")]
    pub folder_id: i32,
    #[serde(rename = "foldername")]
    pub folder_name: String,
    #[serde(rename = "uploadname")]
    pub upload_name: String,
    pub hash: Option<UploadHash>,
}

#[derive(Debug, Deserialize)]
pub struct UploadHash {
    pub sha1: String,
    pub md5: String,
    pub sha256: String,
    pub size: i64,
}

impl FossologyClient {
    pub fn new(base_url: String, api_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap();

        // Use basic auth with default Fossology credentials
        let username = "fossy".to_string();
        let password = "fossy".to_string();

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_token,
            username,
            password,
            client,
        }
    }

    fn auth_header(&self) -> String {
        // Try API token first, fall back to basic auth
        if !self.api_token.is_empty() && self.api_token != "your_token_here" {
            format!("Bearer {}", self.api_token)
        } else {
            // Use basic auth
            let credentials = format!("{}:{}", self.username, self.password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
            format!("Basic {}", encoded)
        }
    }

    /// Health check - verify Fossology is reachable
    pub async fn health_check(&self) -> Result<(), ScanError> {
        let url = format!("{}/repo/api/v1/version", self.base_url);

        tracing::info!("Checking Fossology health at {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header())
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Fossology is healthy");
            Ok(())
        } else {
            Err(ScanError::Unavailable(format!(
                "Fossology health check failed: {}",
                response.status()
            )))
        }
    }

    /// Upload a file or directory to Fossology
    pub async fn upload_from_path(
        &self,
        path: &Path,
        folder_id: i32,
        description: &str,
    ) -> Result<i32, ScanError> {
        tracing::info!("Uploading {:?} to Fossology folder {}", path, folder_id);

        // Create a tar.gz archive of the path
        let archive_path = self.create_archive(path).await?;

        let url = format!("{}/repo/api/v1/uploads", self.base_url);

        // Read the archive file
        let archive_bytes = tokio::fs::read(&archive_path).await.map_err(|e| {
            ScanError::Failed(format!("Failed to read archive: {}", e))
        })?;

        // Create multipart form
        let form = reqwest::multipart::Form::new()
            .text("uploadDescription", description.to_string())
            .part(
                "fileInput",
                reqwest::multipart::Part::bytes(archive_bytes)
                    .file_name("repository.tar.gz")
                    .mime_str("application/gzip")
                    .unwrap(),
            );

        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header())
            .header("folderId", folder_id.to_string())
            .header("uploadType", "file")
            .multipart(form)
            .send()
            .await?;

        // Clean up the archive
        tokio::fs::remove_file(&archive_path).await.ok();

        if response.status().is_success() {
            let upload_response: UploadResponse = response.json().await?;
            tracing::info!("Upload successful, ID: {}", upload_response.message);
            Ok(upload_response.message)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(ScanError::Failed(format!(
                "Upload failed: {}",
                error_text
            )))
        }
    }

    /// Wait for upload to be ready (fully extracted and indexed by Fossology)
    pub async fn wait_for_upload_ready(&self, upload_id: i32) -> Result<(), ScanError> {
        tracing::info!("Waiting for upload {} to be ready", upload_id);

        let url = format!("{}/repo/api/v1/uploads/{}", self.base_url, upload_id);

        // Exponential backoff: 1s, 2s, 4s, 8s, 15s, then 30s intervals
        // Continue polling for up to 5 minutes total
        let max_total_wait = Duration::from_secs(300); // 5 minutes
        let start = std::time::Instant::now();
        let mut attempt = 0;

        loop {
            attempt += 1;

            // Check if we've exceeded total timeout
            if start.elapsed() > max_total_wait {
                return Err(ScanError::Failed(format!(
                    "Upload {} readiness timeout after {} seconds ({} attempts)",
                    upload_id,
                    start.elapsed().as_secs(),
                    attempt - 1
                )));
            }

            // Check upload status
            let response = self
                .client
                .get(&url)
                .header("Authorization", &self.auth_header())
                .send()
                .await?;

            if response.status().is_success() {
                match response.json::<UploadDetails>().await {
                    Ok(details) if details.hash.is_some() => {
                        tracing::info!(
                            "Upload {} is ready after {} seconds (attempt {})",
                            upload_id,
                            start.elapsed().as_secs(),
                            attempt
                        );
                        return Ok(());
                    }
                    Ok(_) => {
                        tracing::debug!(
                            "Upload {} not yet ready, hash field missing (attempt {})",
                            upload_id,
                            attempt
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse upload details (attempt {}): {}",
                            attempt,
                            e
                        );
                    }
                }
            } else if response.status().as_u16() == 503 {
                // 503 is normal - Fossology returns this while processing
                tracing::debug!(
                    "Upload {} still processing (attempt {}, elapsed {}s)",
                    upload_id,
                    attempt,
                    start.elapsed().as_secs()
                );
            } else {
                tracing::warn!(
                    "Upload {} status check failed: {} (attempt {})",
                    upload_id,
                    response.status(),
                    attempt
                );
            }

            // Calculate delay: exponential backoff up to 30s, then stay at 30s
            let delay_secs = match attempt {
                1 => 1,
                2 => 2,
                3 => 4,
                4 => 8,
                5 => 15,
                _ => 30, // Stay at 30s intervals after initial backoff
            };

            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
        }
    }

    /// Create a scan job for an upload
    pub async fn create_job(
        &self,
        upload_id: i32,
        folder_id: i32,
    ) -> Result<i32, ScanError> {
        tracing::info!("Creating scan job for upload {}", upload_id);

        let url = format!("{}/repo/api/v1/jobs", self.base_url);

        let analysis_spec = AnalysisSpec::default();

        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header())
            .header("uploadId", upload_id.to_string())
            .header("folderId", folder_id.to_string())
            .json(&serde_json::json!({ "analysis": analysis_spec }))
            .send()
            .await?;

        if response.status().is_success() {
            let job_response: JobResponse = response.json().await?;
            tracing::info!("Job created successfully, ID: {}", job_response.message);
            Ok(job_response.message)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(ScanError::Failed(format!(
                "Job creation failed: {}",
                error_text
            )))
        }
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: i32) -> Result<JobStatus, ScanError> {
        let url = format!("{}/repo/api/v1/jobs/{}", self.base_url, job_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header())
            .send()
            .await?;

        if response.status().is_success() {
            let status: JobStatus = response.json().await?;
            Ok(status)
        } else {
            Err(ScanError::Failed(format!(
                "Failed to get job status: {}",
                response.status()
            )))
        }
    }

    /// Wait for a job to complete
    pub async fn wait_for_job_completion(&self, job_id: i32) -> Result<(), ScanError> {
        tracing::info!("Waiting for job {} to complete", job_id);

        let max_attempts = 120; // 10 minutes with 5-second intervals
        let mut attempts = 0;
        let mut consecutive_errors = 0;
        let max_consecutive_errors = 3;

        loop {
            attempts += 1;
            if attempts > max_attempts {
                return Err(ScanError::Failed("Job timeout".to_string()));
            }

            // Try to get job status with retry logic for transient errors
            match self.get_job_status(job_id).await {
                Ok(status) => {
                    // Reset error counter on success
                    consecutive_errors = 0;

                    tracing::debug!("Job {} status: {}", job_id, status.status);

                    match status.status.as_str() {
                        "Completed" => {
                            tracing::info!("Job {} completed successfully", job_id);
                            return Ok(());
                        }
                        "Failed" => {
                            return Err(ScanError::Failed(format!("Job {} failed", job_id)));
                        }
                        _ => {
                            // Still running, wait and try again
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    tracing::warn!(
                        "Failed to get job {} status (attempt {}, consecutive errors: {}): {}",
                        job_id,
                        attempts,
                        consecutive_errors,
                        e
                    );

                    if consecutive_errors >= max_consecutive_errors {
                        return Err(ScanError::Failed(format!(
                            "Job {} status check failed after {} consecutive errors: {}",
                            job_id, consecutive_errors, e
                        )));
                    }

                    // Wait a bit before retrying
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Get license results for an upload
    pub async fn get_licenses(&self, upload_id: i32) -> Result<Vec<LicenseResult>, ScanError> {
        tracing::info!("Fetching license results for upload {}", upload_id);

        let url = format!("{}/repo/api/v1/uploads/{}/licenses", self.base_url, upload_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .query(&[
                ("agent", "nomos,monk,ojo"),
                ("containers", "true"),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            tracing::debug!("License response: {}", text);

            // Try to parse as array of FossologyLicenseResponse
            let fossology_responses: Vec<FossologyLicenseResponse> =
                serde_json::from_str(&text).map_err(|e| {
                    tracing::error!("Failed to parse license response: {}", e);
                    ScanError::ParseError(format!("Failed to parse license response: {}", e))
                })?;

            // Convert to LicenseResult format
            let results: Vec<LicenseResult> = fossology_responses
                .into_iter()
                .filter_map(|foss_resp| {
                    let findings_opt = foss_resp.findings?;

                    let mut all_findings = Vec::new();

                    // Collect scanner findings
                    if let Some(scanner_licenses) = findings_opt.scanner {
                        for license_name in scanner_licenses {
                            // Skip "No_license_found" placeholder
                            if license_name == "No_license_found" {
                                continue;
                            }

                            all_findings.push(LicenseFinding {
                                license: license_name.clone(),
                                spdx_id: None, // Fossology only returns license names
                                match_percentage: 100.0, // Default confidence
                            });
                        }
                    }

                    // Collect conclusion findings
                    if let Some(conclusion_licenses) = findings_opt.conclusion {
                        for license_name in conclusion_licenses {
                            // Skip "No_license_found" placeholder
                            if license_name == "No_license_found" {
                                continue;
                            }

                            all_findings.push(LicenseFinding {
                                license: license_name.clone(),
                                spdx_id: None, // Fossology only returns license names
                                match_percentage: 100.0, // Default confidence
                            });
                        }
                    }

                    if all_findings.is_empty() {
                        None
                    } else {
                        Some(LicenseResult {
                            file_path: foss_resp.file_path,
                            findings: all_findings,
                        })
                    }
                })
                .collect();

            tracing::info!("Parsed {} license results", results.len());
            Ok(results)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Failed to get license results: {} - {}", status, error_text);
            Err(ScanError::Failed(format!(
                "Failed to get license results: {} - {}",
                status, error_text
            )))
        }
    }

    /// Get copyright results for an upload
    pub async fn get_copyrights(&self, upload_id: i32) -> Result<Vec<CopyrightResult>, ScanError> {
        tracing::info!("Fetching copyright results for upload {}", upload_id);

        let url = format!("{}/repo/api/v1/uploads/{}/copyrights", self.base_url, upload_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header())
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            tracing::debug!("Copyright response: {}", text);

            // Try to parse as array of FossologyCopyrightResponse
            let fossology_responses: Vec<FossologyCopyrightResponse> =
                serde_json::from_str(&text).map_err(|e| {
                    tracing::error!("Failed to parse copyright response: {}", e);
                    ScanError::ParseError(format!("Failed to parse copyright response: {}", e))
                })?;

            // Convert to CopyrightResult format
            // Fossology returns: [{"copyright": "...", "filePath": ["path1", "path2"]}]
            // We need to flatten this into one CopyrightResult per file path
            let mut results: Vec<CopyrightResult> = Vec::new();

            for foss_resp in fossology_responses {
                // Skip empty copyrights
                if foss_resp.copyright.is_empty() {
                    continue;
                }

                // Skip copyrights with binary/non-printable characters
                if !is_printable_text(&foss_resp.copyright) {
                    tracing::debug!("Skipping copyright with binary data from: {:?}", foss_resp.file_path);
                    continue;
                }

                // Create a CopyrightResult for each file path
                for file_path in foss_resp.file_path {
                    results.push(CopyrightResult {
                        file_path,
                        findings: vec![CopyrightFinding {
                            content: foss_resp.copyright.clone(),
                            finding_type: "copyright".to_string(),
                        }],
                    });
                }
            }

            tracing::info!("Parsed {} copyright results", results.len());
            Ok(results)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Failed to get copyright results: {} - {}", status, error_text);
            Err(ScanError::Failed(format!(
                "Failed to get copyright results: {} - {}",
                status, error_text
            )))
        }
    }

    /// Create a tar.gz archive of a directory
    async fn create_archive(&self, path: &Path) -> Result<std::path::PathBuf, ScanError> {
        use std::process::Command;

        let archive_name = format!("{}.tar.gz", uuid::Uuid::new_v4());
        let archive_path = std::env::temp_dir().join(&archive_name);

        // Use tar command to create archive
        let output = Command::new("tar")
            .arg("-czf")
            .arg(&archive_path)
            .arg("-C")
            .arg(path.parent().unwrap_or(path))
            .arg(path.file_name().unwrap_or(path.as_os_str()))
            .output()
            .map_err(|e| ScanError::Failed(format!("Failed to create archive: {}", e)))?;

        if !output.status.success() {
            return Err(ScanError::Failed(format!(
                "tar command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(archive_path)
    }
}

/// Check if a string contains only printable text (no binary data)
fn is_printable_text(text: &str) -> bool {
    // Allow printable ASCII, common whitespace, and valid UTF-8 characters
    // Reject control characters except tab, newline, and carriage return
    text.chars().all(|c| {
        c == '\t' || c == '\n' || c == '\r' || (!c.is_control() && c.is_ascii()) || (!c.is_ascii() && c.is_alphanumeric())
    })
}
