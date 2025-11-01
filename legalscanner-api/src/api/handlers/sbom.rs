use crate::{
    db::models::{Scan, ScanResult},
    error::AppError,
    export::{spdx, SbomFormat},
    AppState,
};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Response, StatusCode},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SbomQueryParams {
    #[serde(default)]
    format: SbomFormat,
}

/// GET /api/v1/scans/:id/sbom - Export scan results as SPDX/SBOM
pub async fn get_scan_sbom(
    State(state): State<AppState>,
    Path(scan_id): Path<String>,
    Query(params): Query<SbomQueryParams>,
) -> Result<Response<Body>, AppError> {
    // Fetch scan from database
    let scan = Scan::find_by_id(&state.db, &scan_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scan not found: {}", scan_id)))?;

    // Verify scan is completed
    if scan.status != "completed" {
        return Err(AppError::Validation(format!(
            "Scan is not completed yet. Current status: {}",
            scan.status
        )));
    }

    // Fetch all scan results
    let results = ScanResult::find_by_scan_id(&state.db, &scan_id).await?;

    if results.is_empty() {
        return Err(AppError::NotFound("No scan results found".to_string()));
    }

    // Build SPDX document
    let spdx_doc = spdx::build_spdx_document(&scan, &results)?;

    // Serialize to requested format
    let (content, content_type, extension) = match params.format {
        SbomFormat::Json => {
            let json = serde_json::to_string_pretty(&spdx_doc)
                .map_err(|e| AppError::Internal(format!("Failed to serialize SPDX to JSON: {}", e)))?;
            (json, params.format.content_type(), params.format.file_extension())
        }
        SbomFormat::Yaml => {
            let yaml = serde_yaml::to_string(&spdx_doc)
                .map_err(|e| AppError::Internal(format!("Failed to serialize SPDX to YAML: {}", e)))?;
            (yaml, params.format.content_type(), params.format.file_extension())
        }
    };

    // Extract repository name for filename
    let repo_name = scan
        .git_url
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or("repository");

    let filename = format!("{}-sbom.spdx.{}", repo_name, extension);

    // Build response with proper headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(content))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}
