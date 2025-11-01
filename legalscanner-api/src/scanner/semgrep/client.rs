use crate::scanner::traits::{ScanError, ScanResult, Scanner};
use crate::scanner::semgrep::parser::parse_semgrep_output;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub struct SemgrepScanner {
    pub container_name: String,
    pub timeout: Duration,
}

impl SemgrepScanner {
    pub fn new() -> Self {
        Self {
            container_name: "legalscanner-semgrep".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes default
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Execute Semgrep scan in Docker container
    async fn execute_scan(&self, repo_path: &Path) -> Result<String, ScanError> {
        let repo_path_str = repo_path
            .to_str()
            .ok_or_else(|| ScanError::Failed("Invalid repository path".to_string()))?;

        tracing::info!("Executing Semgrep scan on {}", repo_path_str);

        // Run Semgrep via docker exec
        // We mount ./tmp -> /scans in the semgrep container
        // API uses /app/tmp/scans/SCAN_ID, so we need /scans/scans/SCAN_ID in the container
        let scan_path = format!("/scans/scans/{}",
            repo_path.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| ScanError::Failed("Could not determine repo folder name".to_string()))?
        );

        let output = Command::new("docker")
            .args(&[
                "exec",
                &self.container_name,
                "semgrep",
                "--config", "/semgrep-rules/ecc-crypto-detection.yaml",  // Use custom ECC/crypto export control rules
                "--json",
                "--no-git-ignore",  // Scan all files
                "--max-memory", "2000",  // Limit memory usage
                &scan_path,
            ])
            .output()
            .map_err(|e| ScanError::Failed(format!("Failed to execute Semgrep: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Semgrep failed with status {}: {}", output.status, stderr);

            // Check if it's just warnings/info, not a complete failure
            if !output.stdout.is_empty() {
                tracing::warn!("Semgrep had errors but produced output, continuing");
            } else {
                return Err(ScanError::Failed(format!(
                    "Semgrep scan failed: {}",
                    stderr
                )));
            }
        }

        let json_output = String::from_utf8(output.stdout)
            .map_err(|e| ScanError::ParseError(format!("Invalid UTF-8 in Semgrep output: {}", e)))?;

        Ok(json_output)
    }
}

impl Default for SemgrepScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scanner for SemgrepScanner {
    fn name(&self) -> &str {
        "semgrep"
    }

    async fn scan(&self, repo_path: &Path) -> Result<Vec<ScanResult>, ScanError> {
        tracing::info!("Starting Semgrep scan for {:?}", repo_path);

        // Execute Semgrep scan
        let json_output = self.execute_scan(repo_path).await?;

        // Parse output
        let results = parse_semgrep_output(&json_output)?;

        tracing::info!("Semgrep scan completed, found {} files with findings", results.len());

        Ok(results)
    }

    async fn health_check(&self) -> Result<(), ScanError> {
        tracing::debug!("Checking Semgrep container health");

        let output = Command::new("docker")
            .args(&[
                "exec",
                &self.container_name,
                "semgrep",
                "--version",
            ])
            .output()
            .map_err(|e| ScanError::Unavailable(format!("Failed to check Semgrep version: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScanError::Unavailable(format!(
                "Semgrep is not available: {}",
                stderr
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        tracing::info!("Semgrep is available, version: {}", version.trim());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = SemgrepScanner::new();
        assert_eq!(scanner.name(), "semgrep");
        assert_eq!(scanner.container_name, "legalscanner-semgrep");
    }

    #[test]
    fn test_scanner_with_timeout() {
        let scanner = SemgrepScanner::new().with_timeout(Duration::from_secs(60));
        assert_eq!(scanner.timeout, Duration::from_secs(60));
    }
}
