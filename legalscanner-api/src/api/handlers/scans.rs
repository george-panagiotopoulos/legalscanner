use crate::{
    api::models::{CreateScanRequest, ScanResponse, ScanResultsResponse},
    db::models::{Scan, ScanResult},
    error::AppError,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

/// POST /api/v1/scans - Create a new scan
pub async fn create_scan(
    State(state): State<AppState>,
    Json(payload): Json<CreateScanRequest>,
) -> Result<(StatusCode, Json<ScanResponse>), AppError> {
    // Validate Git URL
    if payload.git_url.is_empty() {
        return Err(AppError::Validation("Git URL cannot be empty".to_string()));
    }

    // Validate Git URL format
    crate::git::validate_git_url(&payload.git_url)
        .map_err(|e| AppError::Validation(e))?;

    // Create scan in database
    let scan = Scan::create(&state.db, payload.git_url.clone(), payload.git_token, None).await?;

    // Spawn background task to execute the scan
    let scan_id = scan.id.clone();
    let state_clone = state.clone();

    tokio::spawn(async move {
        super::scan_job::execute_scan_job(scan_id, state_clone).await;
    });

    // Return immediately with pending status
    Ok((
        StatusCode::CREATED,
        Json(ScanResponse {
            scan_id: scan.id,
            status: scan.status,
            created_at: scan.created_at,
            git_url: scan.git_url,
            fossology_status: scan.fossology_status,
            semgrep_status: scan.semgrep_status,
        }),
    ))
}

/// GET /api/v1/scans - List all scans
pub async fn list_scans(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScanResponse>>, AppError> {
    let scans = Scan::list_all(&state.db, 100).await?;

    let responses: Vec<ScanResponse> = scans
        .into_iter()
        .map(|scan| ScanResponse {
            scan_id: scan.id,
            status: scan.status,
            created_at: scan.created_at,
            git_url: scan.git_url,
            fossology_status: scan.fossology_status,
            semgrep_status: scan.semgrep_status,
        })
        .collect();

    Ok(Json(responses))
}

/// GET /api/v1/scans/:id - Get scan details
pub async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let scan = Scan::find_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scan {} not found", id)))?;

    let summary = Scan::get_summary(&state.db, &id).await.ok();

    Ok(Json(serde_json::json!({
        "id": scan.id,
        "git_url": scan.git_url,
        "status": scan.status,
        "error_message": scan.error_message,
        "created_at": scan.created_at,
        "started_at": scan.started_at,
        "completed_at": scan.completed_at,
        "fossology_status": scan.fossology_status,
        "semgrep_status": scan.semgrep_status,
        "fossology_error": scan.fossology_error,
        "semgrep_error": scan.semgrep_error,
        "summary": summary
    })))
}

/// DELETE /api/v1/scans/:id - Delete a scan
pub async fn delete_scan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // Check if scan exists
    let _ = Scan::find_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scan {} not found", id)))?;

    Scan::delete(&state.db, &id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/scans - Delete all scans
pub async fn delete_all_scans(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted_count = Scan::delete_all(&state.db).await?;

    Ok(Json(serde_json::json!({
        "deleted": deleted_count
    })))
}

/// GET /api/v1/scans/:id/results - Get scan results
pub async fn get_scan_results(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ScanResultsResponse>, AppError> {
    // Check if scan exists
    let scan = Scan::find_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scan {} not found", id)))?;

    // Get all results
    let results = ScanResult::find_by_scan_id(&state.db, &id).await?;

    // Separate licenses, copyrights, and ECC findings
    let mut licenses = Vec::new();
    let mut copyrights = Vec::new();
    let mut ecc_findings = Vec::new();

    for result in results {
        if result.result_type == "license" {
            licenses.push(serde_json::json!({
                "file_path": result.file_path,
                "license": result.license_name,
                "spdx_id": result.license_spdx_id,
                "confidence": result.confidence
            }));
        } else if result.result_type == "copyright" {
            let holders: Vec<String> = result
                .copyright_holders
                .and_then(|h| serde_json::from_str(&h).ok())
                .unwrap_or_default();
            let years: Vec<String> = result
                .copyright_years
                .and_then(|y| serde_json::from_str(&y).ok())
                .unwrap_or_default();

            copyrights.push(serde_json::json!({
                "file_path": result.file_path,
                "statement": result.copyright_statement,
                "holders": holders,
                "years": years
            }));
        } else if result.result_type == "ecc" {
            ecc_findings.push(serde_json::json!({
                "file_path": result.file_path,
                "content": result.raw_data,
                "risk_severity": result.risk_severity,
                "source": result.ecc_source,
                "line_number": result.ecc_line_number,
                "check_id": result.ecc_check_id
            }));
        }
    }

    Ok(Json(ScanResultsResponse {
        scan_id: scan.id.clone(),
        repository_url: scan.git_url,
        scan_date: scan.created_at,
        status: scan.status,
        results: serde_json::json!({
            "licenses": licenses,
            "copyrights": copyrights,
            "ecc_findings": ecc_findings
        }),
    }))
}
