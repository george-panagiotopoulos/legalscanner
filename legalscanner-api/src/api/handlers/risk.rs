use crate::api::models::{RiskAssessment, RiskFactor};
use crate::db::models::scan_result::ScanResult;
use crate::error::AppError;
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::{debug, info};

/// Calculate risk score for a completed scan
pub async fn calculate_risk_score(
    pool: &SqlitePool,
    scan_id: &str,
) -> Result<RiskAssessment, AppError> {
    info!("Calculating risk score for scan {}", scan_id);

    // Fetch all scan results for this scan
    let results = ScanResult::find_by_scan_id(pool, scan_id).await?;

    // Load risk config from database
    let risk_config = load_risk_config(pool).await?;

    let mut base_score = 0;
    let mut risk_factors: Vec<RiskFactor> = Vec::new();

    // Track license-related risks
    let mut license_results: Vec<&ScanResult> = results
        .iter()
        .filter(|r| r.result_type == "license")
        .collect();

    // 1. LICENSE TYPE RISK (max +40 points)
    let mut license_risk_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut copyleft_licenses: Vec<String> = Vec::new();
    let mut unknown_licenses: Vec<String> = Vec::new();

    for result in &license_results {
        if let Some(license_name) = &result.license_name {
            // Try to find matching risk weight from config
            if let Some(weight) = get_license_weight(&risk_config, license_name) {
                if weight > 0 {
                    let entry = license_risk_map
                        .entry(license_name.to_string())
                        .or_insert_with(Vec::new);
                    if !entry.contains(&result.file_path) {
                        entry.push(result.file_path.clone());
                    }

                    // Categorize for risk factors
                    if is_copyleft(license_name) {
                        if !copyleft_licenses.contains(&license_name.to_string()) {
                            copyleft_licenses.push(license_name.to_string());
                        }
                    } else if is_unknown_or_proprietary(license_name) {
                        if !unknown_licenses.contains(&license_name.to_string()) {
                            unknown_licenses.push(license_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Add copyleft risk factor
    if !copyleft_licenses.is_empty() {
        let mut affected_count = 0;
        let mut details: Vec<String> = Vec::new();
        for license in &copyleft_licenses {
            if let Some(files) = license_risk_map.get(license) {
                affected_count += files.len() as i32;
                details.push(format!("{} ({} files)", license, files.len()));
                // Add weight for each unique copyleft license type
                if let Some(weight) = get_license_weight(&risk_config, license) {
                    base_score += weight;
                }
            }
        }

        risk_factors.push(RiskFactor {
            category: "copyleft_license".to_string(),
            severity: "high".to_string(),
            description: "Strong copyleft licenses detected - may require releasing derivative works under same license".to_string(),
            affected_count,
            details,
        });
    }

    // Add unknown/proprietary risk factor
    if !unknown_licenses.is_empty() {
        let mut affected_count = 0;
        let mut details: Vec<String> = Vec::new();
        for license in &unknown_licenses {
            if let Some(files) = license_risk_map.get(license) {
                affected_count += files.len() as i32;
                details.push(format!("{} ({} files)", license, files.len()));
                // Add weight for each unique unknown license
                if let Some(weight) = get_license_weight(&risk_config, license) {
                    base_score += weight;
                }
            }
        }

        risk_factors.push(RiskFactor {
            category: "unknown_license".to_string(),
            severity: "medium".to_string(),
            description: "Unknown or proprietary licenses detected - unclear usage rights".to_string(),
            affected_count,
            details,
        });
    }

    // 2. MISSING SPDX IDs (max +2 per file)
    let missing_spdx: Vec<&ScanResult> = license_results
        .iter()
        .filter(|r| {
            r.license_name.is_some()
                && (r.license_spdx_id.is_none() || r.license_spdx_id.as_ref().unwrap().is_empty())
        })
        .copied()
        .collect();

    if !missing_spdx.is_empty() {
        let count = missing_spdx.len() as i32;
        let points = count * 2;
        base_score += points;

        let mut license_counts: HashMap<String, i32> = HashMap::new();
        for result in &missing_spdx {
            if let Some(license_name) = &result.license_name {
                *license_counts.entry(license_name.clone()).or_insert(0) += 1;
            }
        }

        let mut details: Vec<String> = license_counts
            .iter()
            .map(|(name, count)| format!("{} ({} files)", name, count))
            .collect();
        details.sort();

        risk_factors.push(RiskFactor {
            category: "missing_spdx_id".to_string(),
            severity: "medium".to_string(),
            description: "Licenses without SPDX identifiers - ambiguous or non-standard licenses".to_string(),
            affected_count: count,
            details,
        });
    }

    // 3. LOW CONFIDENCE DETECTIONS (max +15 per finding for confidence < 0.5)
    let low_confidence: Vec<&ScanResult> = license_results
        .iter()
        .filter(|r| r.confidence.is_some() && r.confidence.unwrap() < 0.7)
        .copied()
        .collect();

    if !low_confidence.is_empty() {
        let mut points = 0;
        let mut critical_count = 0;
        let mut medium_count = 0;
        let mut details: Vec<String> = Vec::new();

        for result in &low_confidence {
            let confidence = result.confidence.unwrap();
            if confidence < 0.5 {
                points += 15;
                critical_count += 1;
                details.push(format!(
                    "{} ({}% confidence)",
                    result.license_name.as_ref().unwrap_or(&"Unknown".to_string()),
                    (confidence * 100.0) as i32
                ));
            } else if confidence < 0.7 {
                points += 8;
                medium_count += 1;
            }
        }

        base_score += points;

        let severity = if critical_count > 0 { "high" } else { "medium" };

        risk_factors.push(RiskFactor {
            category: "low_confidence".to_string(),
            severity: severity.to_string(),
            description: "Low confidence license detections - may require manual review".to_string(),
            affected_count: (critical_count + medium_count) as i32,
            details,
        });
    }

    // 4. ECC FINDINGS (max +30 points)
    let ecc_results: Vec<&ScanResult> = results
        .iter()
        .filter(|r| r.result_type == "ecc")
        .collect();

    if !ecc_results.is_empty() {
        let mut ecc_points = 0;
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        let mut critical_details: Vec<String> = Vec::new();
        let mut high_details: Vec<String> = Vec::new();
        let mut medium_details: Vec<String> = Vec::new();
        let mut low_details: Vec<String> = Vec::new();

        for result in &ecc_results {
            let severity = result.risk_severity.as_deref().unwrap_or("medium");
            match severity {
                "critical" => {
                    ecc_points += 20;
                    critical_count += 1;
                    critical_details.push(result.file_path.clone());
                }
                "high" => {
                    ecc_points += 12;
                    high_count += 1;
                    high_details.push(result.file_path.clone());
                }
                "medium" => {
                    ecc_points += 6;
                    medium_count += 1;
                    medium_details.push(result.file_path.clone());
                }
                _ => {
                    ecc_points += 2;
                    low_count += 1;
                    low_details.push(result.file_path.clone());
                }
            }
        }

        base_score += ecc_points;

        // Add risk factor for critical/high ECC findings
        if critical_count > 0 || high_count > 0 {
            let mut details: Vec<String> = Vec::new();
            if critical_count > 0 {
                details.push(format!("Critical: {} findings", critical_count));
                details.extend(critical_details.iter().take(3).cloned());
            }
            if high_count > 0 {
                details.push(format!("High: {} findings", high_count));
                details.extend(high_details.iter().take(3).cloned());
            }

            risk_factors.push(RiskFactor {
                category: "ecc_critical_high".to_string(),
                severity: if critical_count > 0 { "critical" } else { "high" }.to_string(),
                description: "Critical or high-severity export control findings - may require compliance review".to_string(),
                affected_count: (critical_count + high_count) as i32,
                details,
            });
        }

        // Add risk factor for medium/low ECC findings
        if medium_count > 0 || low_count > 0 {
            let mut details: Vec<String> = Vec::new();
            if medium_count > 0 {
                details.push(format!("Medium: {} findings", medium_count));
            }
            if low_count > 0 {
                details.push(format!("Low: {} findings", low_count));
            }

            risk_factors.push(RiskFactor {
                category: "ecc_medium_low".to_string(),
                severity: "medium".to_string(),
                description: "Export control findings detected - review for compliance requirements".to_string(),
                affected_count: (medium_count + low_count) as i32,
                details,
            });
        }
    }

    // 5. LICENSE DIVERSITY (max +10 points)
    let unique_licenses: std::collections::HashSet<String> = license_results
        .iter()
        .filter_map(|r| r.license_name.clone())
        .collect();

    let license_count = unique_licenses.len();
    let diversity_points = if license_count > 15 {
        10
    } else if license_count >= 10 {
        6
    } else if license_count >= 5 {
        3
    } else {
        0
    };

    if diversity_points > 0 {
        base_score += diversity_points;
        risk_factors.push(RiskFactor {
            category: "license_diversity".to_string(),
            severity: "low".to_string(),
            description: format!(
                "High license diversity ({} unique licenses) - may indicate compatibility issues",
                license_count
            ),
            affected_count: license_count as i32,
            details: unique_licenses.into_iter().take(10).collect(),
        });
    }

    // Calculate final score (cap at 100)
    let final_score = std::cmp::min(base_score, 100);

    // Determine risk level based on score
    let risk_level = match final_score {
        0..=25 => "low",
        26..=50 => "medium",
        51..=75 => "high",
        _ => "critical",
    };

    debug!(
        "Risk calculation complete for scan {}: score={}, level={}",
        scan_id, final_score, risk_level
    );

    Ok(RiskAssessment {
        score: final_score,
        level: risk_level.to_string(),
        factors: risk_factors,
    })
}

/// Load risk configuration from database
async fn load_risk_config(pool: &SqlitePool) -> Result<Vec<(String, i32)>, AppError> {
    #[derive(sqlx::FromRow)]
    struct RiskConfigRow {
        license_pattern: String,
        risk_weight: i32,
    }

    let records = sqlx::query_as::<_, RiskConfigRow>(
        "SELECT license_pattern, risk_weight FROM risk_config ORDER BY risk_weight DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r| (r.license_pattern, r.risk_weight))
        .collect())
}

/// Get risk weight for a license using pattern matching
fn get_license_weight(config: &[(String, i32)], license_name: &str) -> Option<i32> {
    for (pattern, weight) in config {
        if pattern.ends_with('%') {
            // Prefix match
            let prefix = &pattern[..pattern.len() - 1];
            if license_name.starts_with(prefix) {
                return Some(*weight);
            }
        } else if pattern.starts_with('%') {
            // Suffix match
            let suffix = &pattern[1..];
            if license_name.ends_with(suffix) {
                return Some(*weight);
            }
        } else if pattern.contains('%') {
            // Contains match
            let parts: Vec<&str> = pattern.split('%').collect();
            if parts.len() == 2 && license_name.starts_with(parts[0]) && license_name.ends_with(parts[1]) {
                return Some(*weight);
            }
        } else if pattern == license_name {
            // Exact match
            return Some(*weight);
        }
    }
    None
}

/// Check if license is copyleft
fn is_copyleft(license_name: &str) -> bool {
    let copyleft_patterns = [
        "GPL", "AGPL", "LGPL", "MPL", "EPL", "CDDL", "CPL", "Sleepycat",
    ];
    copyleft_patterns
        .iter()
        .any(|pattern| license_name.contains(pattern))
}

/// Check if license is unknown or proprietary
fn is_unknown_or_proprietary(license_name: &str) -> bool {
    let unknown_patterns = [
        "No_license_found",
        "Unknown",
        "Proprietary",
        "Commercial",
        "See-file",
        "possibility",
    ];
    unknown_patterns
        .iter()
        .any(|pattern| license_name.contains(pattern))
}
