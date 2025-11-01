use crate::scanner::traits::{EccFinding, ScanError, ScanResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semgrep JSON output structure
#[derive(Debug, Deserialize, Serialize)]
pub struct SemgrepOutput {
    pub results: Vec<SemgrepResult>,
    #[serde(default)]
    pub errors: Vec<SemgrepError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SemgrepResult {
    pub path: String,
    pub start: SemgrepLocation,
    pub end: SemgrepLocation,
    pub check_id: String,
    pub extra: SemgrepExtra,
    #[serde(default)]
    pub lines: Option<String>,  // The actual matched code
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SemgrepLocation {
    pub line: i32,
    pub col: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SemgrepExtra {
    pub message: String,
    pub severity: String, // "ERROR", "WARNING", "INFO"
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SemgrepError {
    pub message: String,
    #[serde(default)]
    pub path: Option<String>,
}

/// Parse Semgrep JSON output and convert to ScanResult format
pub fn parse_semgrep_output(json_output: &str) -> Result<Vec<ScanResult>, ScanError> {
    let semgrep_output: SemgrepOutput = serde_json::from_str(json_output)
        .map_err(|e| ScanError::ParseError(format!("Failed to parse Semgrep JSON: {}", e)))?;

    // Log any errors from Semgrep
    for error in &semgrep_output.errors {
        tracing::warn!(
            "Semgrep scan error: {} (path: {:?})",
            error.message,
            error.path
        );
    }

    // Group findings by file path
    let mut results_by_file: HashMap<String, Vec<EccFinding>> = HashMap::new();

    for result in semgrep_output.results {
        // Build detailed content message with matched code
        let content = if let Some(matched_code) = &result.lines {
            let matched_code = matched_code.trim();
            format!(
                "{}\n\nMatched code: `{}`",
                result.extra.message,
                matched_code
            )
        } else {
            result.extra.message.clone()
        };

        let finding = EccFinding {
            content,
            risk_severity: map_severity(&result.extra.severity),
            source: Some("semgrep".to_string()),
            line_number: Some(result.start.line),
            check_id: Some(result.check_id.clone()),
        };

        results_by_file
            .entry(result.path.clone())
            .or_insert_with(Vec::new)
            .push(finding);
    }

    // Convert to ScanResult format
    let mut scan_results = Vec::new();
    for (file_path, ecc_findings) in results_by_file {
        scan_results.push(ScanResult {
            file_path,
            licenses: Vec::new(),
            copyrights: Vec::new(),
            ecc_findings,
        });
    }

    tracing::info!("Parsed {} Semgrep findings", scan_results.len());

    Ok(scan_results)
}

/// Map Semgrep severity to risk severity
fn map_severity(semgrep_severity: &str) -> String {
    match semgrep_severity.to_uppercase().as_str() {
        "ERROR" => "high".to_string(),
        "WARNING" => "medium".to_string(),
        "INFO" => "low".to_string(),
        _ => {
            tracing::warn!("Unknown Semgrep severity: {}, defaulting to 'low'", semgrep_severity);
            "low".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_output() {
        let json = r#"{"results": [], "errors": []}"#;
        let results = parse_semgrep_output(json).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_single_finding() {
        let json = r#"{
            "results": [
                {
                    "path": "src/crypto.rs",
                    "start": {"line": 42, "col": 5},
                    "end": {"line": 42, "col": 25},
                    "check_id": "rust.crypto.aes-usage",
                    "extra": {
                        "message": "Detected AES encryption usage",
                        "severity": "ERROR",
                        "metadata": {}
                    }
                }
            ],
            "errors": []
        }"#;

        let results = parse_semgrep_output(json).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_path, "src/crypto.rs");
        assert_eq!(results[0].ecc_findings.len(), 1);

        let finding = &results[0].ecc_findings[0];
        assert_eq!(finding.content, "Detected AES encryption usage");
        assert_eq!(finding.risk_severity, "high");
        assert_eq!(finding.source, Some("semgrep".to_string()));
        assert_eq!(finding.line_number, Some(42));
        assert_eq!(finding.check_id, Some("rust.crypto.aes-usage".to_string()));
    }

    #[test]
    fn test_severity_mapping() {
        assert_eq!(map_severity("ERROR"), "high");
        assert_eq!(map_severity("WARNING"), "medium");
        assert_eq!(map_severity("INFO"), "low");
        assert_eq!(map_severity("unknown"), "low");
    }
}
