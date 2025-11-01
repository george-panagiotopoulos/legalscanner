use crate::db::models::scan::Scan;
use crate::db::models::scan_result::ScanResult;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// SPDX 2.3 Document
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxDocument {
    pub spdx_version: String,
    pub data_license: String,
    #[serde(rename = "SPDXID")]
    pub spdxid: String,
    pub name: String,
    pub document_namespace: String,
    pub creation_info: CreationInfo,
    pub packages: Vec<Package>,
    pub files: Vec<File>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreationInfo {
    pub created: String,
    pub creators: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_list_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    #[serde(rename = "SPDXID")]
    pub spdxid: String,
    pub name: String,
    pub download_location: String,
    pub files_analyzed: bool,
    pub license_concluded: String,
    pub license_declared: String,
    pub copyright_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[serde(rename = "SPDXID")]
    pub spdxid: String,
    pub file_name: String,
    pub license_concluded: String,
    pub license_info_in_files: Vec<String>,
    pub copyright_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    #[serde(rename = "spdxElementId")]
    pub spdx_element_id: String,
    pub relationship_type: String,
    #[serde(rename = "relatedSpdxElement")]
    pub related_spdx_element: String,
}

/// Build an SPDX 2.3 document from scan data
pub fn build_spdx_document(scan: &Scan, results: &[ScanResult]) -> Result<SpdxDocument, AppError> {
    let repo_name = extract_repo_name(&scan.git_url);
    let namespace = format!("https://legalscanner.io/spdx/{}", scan.id);

    let creation_info = CreationInfo {
        created: scan.completed_at.clone().unwrap_or_else(|| scan.created_at.clone()),
        creators: vec!["Tool: LegalScanner-1.0".to_string()],
        license_list_version: Some("3.22".to_string()),
    };

    let package = build_package(scan, &repo_name, results);
    let files = build_files(results);
    let relationships = build_relationships(&files);

    Ok(SpdxDocument {
        spdx_version: "SPDX-2.3".to_string(),
        data_license: "CC0-1.0".to_string(),
        spdxid: "SPDXRef-DOCUMENT".to_string(),
        name: format!("Legal Scanner Report - {}", repo_name),
        document_namespace: namespace,
        creation_info,
        packages: vec![package],
        files,
        relationships,
    })
}

fn extract_repo_name(git_url: &str) -> String {
    git_url
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or("unknown-repo")
        .to_string()
}

fn build_package(scan: &Scan, repo_name: &str, results: &[ScanResult]) -> Package {
    let concluded_license = determine_concluded_license(results);
    let copyright_summary = extract_copyright_summary(results);

    let summary = format!(
        "Repository scanned for legal compliance. \
         Found {} license findings, {} copyright statements, and {} security findings.",
        results.iter().filter(|r| r.result_type == "license").count(),
        results.iter().filter(|r| r.result_type == "copyright").count(),
        results.iter().filter(|r| r.result_type == "ecc").count()
    );

    Package {
        spdxid: "SPDXRef-Package".to_string(),
        name: repo_name.to_string(),
        download_location: scan.git_url.clone(),
        files_analyzed: true,
        license_concluded: concluded_license.clone(),
        license_declared: concluded_license,
        copyright_text: copyright_summary,
        summary: Some(summary),
    }
}

fn determine_concluded_license(results: &[ScanResult]) -> String {
    let licenses: Vec<String> = results
        .iter()
        .filter(|r| r.result_type == "license")
        .filter_map(|r| {
            r.license_spdx_id
                .clone()
                .or_else(|| r.license_name.clone())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if licenses.is_empty() {
        "NOASSERTION".to_string()
    } else if licenses.len() == 1 {
        licenses[0].clone()
    } else {
        format!("({})", licenses.join(" AND "))
    }
}

fn extract_copyright_summary(results: &[ScanResult]) -> String {
    let copyright_statements: Vec<String> = results
        .iter()
        .filter(|r| r.result_type == "copyright")
        .filter_map(|r| r.copyright_statement.clone())
        .collect();

    if copyright_statements.is_empty() {
        "NOASSERTION".to_string()
    } else {
        copyright_statements.join("\n")
    }
}

fn build_files(results: &[ScanResult]) -> Vec<File> {
    let mut files_map: HashMap<String, Vec<&ScanResult>> = HashMap::new();
    for result in results {
        files_map
            .entry(result.file_path.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    files_map
        .into_iter()
        .enumerate()
        .map(|(idx, (file_path, file_results))| {
            build_file(&file_path, file_results, idx + 1)
        })
        .collect()
}

fn build_file(file_path: &str, results: Vec<&ScanResult>, index: usize) -> File {
    let spdx_id = format!("SPDXRef-File-{}", index);

    let licenses: Vec<String> = results
        .iter()
        .filter(|r| r.result_type == "license")
        .filter_map(|r| {
            r.license_spdx_id
                .clone()
                .or_else(|| r.license_name.clone())
        })
        .collect();

    let license_concluded = if licenses.is_empty() {
        "NOASSERTION".to_string()
    } else if licenses.len() == 1 {
        licenses[0].clone()
    } else {
        format!("({})", licenses.join(" AND "))
    };

    let copyright_text = results
        .iter()
        .filter(|r| r.result_type == "copyright")
        .filter_map(|r| r.copyright_statement.clone())
        .collect::<Vec<String>>()
        .join("\n");

    let copyright = if copyright_text.is_empty() {
        "NOASSERTION".to_string()
    } else {
        copyright_text
    };

    // Add ECC findings as comments
    let ecc_findings: Vec<String> = results
        .iter()
        .filter(|r| r.result_type == "ecc")
        .map(|r| {
            format!(
                "ECC: {} (Severity: {}, Line: {})",
                r.ecc_source.as_ref().unwrap_or(&"Unknown".to_string()),
                r.risk_severity.as_ref().unwrap_or(&"unknown".to_string()),
                r.ecc_line_number.unwrap_or(0)
            )
        })
        .collect();

    let comment = if !ecc_findings.is_empty() {
        Some(ecc_findings.join("; "))
    } else {
        None
    };

    File {
        spdxid: spdx_id,
        file_name: file_path.to_string(),
        license_concluded: license_concluded.clone(),
        license_info_in_files: if licenses.is_empty() {
            vec!["NOASSERTION".to_string()]
        } else {
            licenses
        },
        copyright_text: copyright,
        comment,
    }
}

fn build_relationships(files: &[File]) -> Vec<Relationship> {
    files
        .iter()
        .map(|file| Relationship {
            spdx_element_id: "SPDXRef-Package".to_string(),
            relationship_type: "CONTAINS".to_string(),
            related_spdx_element: file.spdxid.clone(),
        })
        .collect()
}
