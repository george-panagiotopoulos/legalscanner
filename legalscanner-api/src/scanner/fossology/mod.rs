mod client;
mod parser;

pub use client::FossologyClient;

use crate::scanner::traits::{ScanError, ScanResult, Scanner};
use async_trait::async_trait;
use std::path::Path;

pub struct FossologyScanner {
    client: FossologyClient,
    folder_id: i32,
}

impl FossologyScanner {
    pub fn new(base_url: String, api_token: String) -> Self {
        Self {
            client: FossologyClient::new(base_url, api_token),
            folder_id: 1, // Default folder - TODO: make configurable
        }
    }

    pub fn new_with_folder(base_url: String, api_token: String, folder_id: i32) -> Self {
        Self {
            client: FossologyClient::new(base_url, api_token),
            folder_id,
        }
    }
}

#[async_trait]
impl Scanner for FossologyScanner {
    fn name(&self) -> &str {
        "fossology"
    }

    async fn scan(&self, repo_path: &Path) -> Result<Vec<ScanResult>, ScanError> {
        tracing::info!("Starting Fossology scan for {:?}", repo_path);

        // 1. Upload repository to Fossology
        let upload_id = self
            .client
            .upload_from_path(
                repo_path,
                self.folder_id,
                &format!("Repository scan: {}", repo_path.display()),
            )
            .await?;

        tracing::info!("Upload ID: {}", upload_id);

        // Wait for Fossology to fully process the upload
        // Polls upload status until extraction and indexing are complete
        self.client.wait_for_upload_ready(upload_id).await?;

        // 2. Create scan job
        let job_id = self.client.create_job(upload_id, self.folder_id).await?;

        tracing::info!("Job ID: {}", job_id);

        // 3. Wait for job completion
        self.client.wait_for_job_completion(job_id).await?;

        tracing::info!("Job completed, fetching results");

        // 4. Fetch license results
        let license_results = self.client.get_licenses(upload_id).await?;

        // 5. Parse license results
        let mut scan_results = parser::parse_license_results(license_results);

        // 6. Fetch and merge copyright results
        let copyright_results = self.client.get_copyrights(upload_id).await?;
        scan_results = parser::merge_copyright_results(scan_results, copyright_results);

        tracing::info!(
            "Scan complete, found results for {} files",
            scan_results.len()
        );

        Ok(scan_results)
    }

    async fn health_check(&self) -> Result<(), ScanError> {
        self.client.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = FossologyScanner::new(
            "http://localhost:8081".to_string(),
            "test-token".to_string(),
        );
        assert_eq!(scanner.name(), "fossology");
    }
}
