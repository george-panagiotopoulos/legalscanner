use crate::scanner::traits::{CopyrightFinding, LicenseFinding, ScanResult};
use regex::Regex;
use std::collections::HashMap;

use super::client::{CopyrightResult, LicenseResult};

/// Parse Fossology license results into standardized format
pub fn parse_license_results(results: Vec<LicenseResult>) -> Vec<ScanResult> {
    let mut file_map: HashMap<String, ScanResult> = HashMap::new();

    for license_result in results {
        let entry = file_map
            .entry(license_result.file_path.clone())
            .or_insert_with(|| ScanResult {
                file_path: license_result.file_path.clone(),
                licenses: Vec::new(),
                copyrights: Vec::new(),
            });

        for finding in license_result.findings {
            entry.licenses.push(LicenseFinding {
                name: finding.license.clone(),
                spdx_id: finding.spdx_id.or_else(|| map_to_spdx(&finding.license)),
                confidence: finding.match_percentage / 100.0,
            });
        }
    }

    file_map.into_values().collect()
}

/// Parse Fossology copyright results and merge with existing scan results
pub fn merge_copyright_results(
    mut scan_results: Vec<ScanResult>,
    copyright_results: Vec<CopyrightResult>,
) -> Vec<ScanResult> {
    let mut file_map: HashMap<String, Vec<CopyrightFinding>> = HashMap::new();

    // Group copyrights by file
    for copyright_result in copyright_results {
        let copyrights = file_map
            .entry(copyright_result.file_path.clone())
            .or_insert_with(Vec::new);

        for finding in copyright_result.findings {
            if let Some(copyright) = parse_copyright_statement(&finding.content) {
                copyrights.push(copyright);
            }
        }
    }

    // Merge with existing scan results
    for scan_result in &mut scan_results {
        if let Some(copyrights) = file_map.remove(&scan_result.file_path) {
            scan_result.copyrights.extend(copyrights);
        }
    }

    // Add files that only have copyrights
    for (file_path, copyrights) in file_map {
        scan_results.push(ScanResult {
            file_path,
            licenses: Vec::new(),
            copyrights,
        });
    }

    scan_results
}

/// Map Fossology license names to SPDX identifiers
pub fn map_to_spdx(license_name: &str) -> Option<String> {
    let normalized = license_name.to_lowercase().replace(" ", "-");

    let spdx_map = [
        ("mit", "MIT"),
        ("apache-2.0", "Apache-2.0"),
        ("apache-license-2.0", "Apache-2.0"),
        ("gpl-2.0", "GPL-2.0-only"),
        ("gpl-3.0", "GPL-3.0-only"),
        ("lgpl-2.1", "LGPL-2.1-only"),
        ("lgpl-3.0", "LGPL-3.0-only"),
        ("bsd-2-clause", "BSD-2-Clause"),
        ("bsd-3-clause", "BSD-3-Clause"),
        ("mpl-2.0", "MPL-2.0"),
        ("isc", "ISC"),
        ("cc0-1.0", "CC0-1.0"),
        ("unlicense", "Unlicense"),
        ("artistic-2.0", "Artistic-2.0"),
        ("zlib", "Zlib"),
    ];

    for (pattern, spdx) in &spdx_map {
        if normalized.contains(pattern) {
            return Some(spdx.to_string());
        }
    }

    None
}

/// Parse a copyright statement to extract holders and years
pub fn parse_copyright_statement(statement: &str) -> Option<CopyrightFinding> {
    let statement = statement.trim();
    if statement.is_empty() {
        return None;
    }

    let holders = extract_copyright_holders(statement);
    let years = extract_copyright_years(statement);

    if holders.is_empty() && years.is_empty() {
        return None;
    }

    Some(CopyrightFinding {
        statement: statement.to_string(),
        holders,
        years,
    })
}

/// Extract copyright holders from a statement
pub fn extract_copyright_holders(statement: &str) -> Vec<String> {
    let mut holders = Vec::new();

    // Common patterns: "Copyright (c) 2025 John Doe", "Copyright 2025 by Company Inc."
    let patterns = [
        r"(?i)copyright\s*(?:\(c\))?\s*(?:\d{4}[-,\s]*)*\s*(?:by\s+)?(.+?)(?:\.|$)",
        r"(?i)©\s*(?:\d{4}[-,\s]*)*\s*(?:by\s+)?(.+?)(?:\.|$)",
        r"(?i)copr\.\s*(?:\d{4}[-,\s]*)*\s*(?:by\s+)?(.+?)(?:\.|$)",
    ];

    for pattern_str in &patterns {
        if let Ok(re) = Regex::new(pattern_str) {
            if let Some(captures) = re.captures(statement) {
                if let Some(holder) = captures.get(1) {
                    let holder_text = holder.as_str().trim();
                    if !holder_text.is_empty() && !holder_text.starts_with(char::is_numeric) {
                        holders.push(holder_text.to_string());
                        break;
                    }
                }
            }
        }
    }

    // Deduplicate
    holders.sort();
    holders.dedup();
    holders
}

/// Extract copyright years from a statement
pub fn extract_copyright_years(statement: &str) -> Vec<String> {
    let mut years = Vec::new();

    // Match 4-digit years
    if let Ok(re) = Regex::new(r"\b(19\d{2}|20\d{2})\b") {
        for captures in re.captures_iter(statement) {
            if let Some(year) = captures.get(1) {
                years.push(year.as_str().to_string());
            }
        }
    }

    // Deduplicate and sort
    years.sort();
    years.dedup();
    years
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_to_spdx() {
        assert_eq!(map_to_spdx("MIT License"), Some("MIT".to_string()));
        assert_eq!(
            map_to_spdx("Apache License 2.0"),
            Some("Apache-2.0".to_string())
        );
        assert_eq!(map_to_spdx("GPL-3.0"), Some("GPL-3.0-only".to_string()));
        assert_eq!(map_to_spdx("Unknown License"), None);
    }

    #[test]
    fn test_extract_copyright_holders() {
        let holders = extract_copyright_holders("Copyright (c) 2025 John Doe");
        assert_eq!(holders, vec!["John Doe"]);

        let holders = extract_copyright_holders("Copyright 2024-2025 by Acme Corporation.");
        assert_eq!(holders, vec!["Acme Corporation"]);

        let holders = extract_copyright_holders("© 2025 Tech Company Inc.");
        assert_eq!(holders, vec!["Tech Company Inc"]);
    }

    #[test]
    fn test_extract_copyright_years() {
        let years = extract_copyright_years("Copyright (c) 2025 John Doe");
        assert_eq!(years, vec!["2025"]);

        let years = extract_copyright_years("Copyright 2020-2025 Company");
        assert_eq!(years, vec!["2020", "2025"]);

        let years = extract_copyright_years("© 2023, 2024, 2025 Company");
        assert_eq!(years, vec!["2023", "2024", "2025"]);
    }

    #[test]
    fn test_parse_copyright_statement() {
        let result = parse_copyright_statement("Copyright (c) 2025 John Doe");
        assert!(result.is_some());

        let copyright = result.unwrap();
        assert_eq!(copyright.statement, "Copyright (c) 2025 John Doe");
        assert_eq!(copyright.holders, vec!["John Doe"]);
        assert_eq!(copyright.years, vec!["2025"]);
    }
}
