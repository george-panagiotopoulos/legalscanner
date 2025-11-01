use crate::{
    db::models::{Scan, ScanResult as DbScanResult},
    git::{clone_repository, workspace::Workspace},
    scanner::Scanner,
    AppState,
};
use sqlx::SqlitePool;
use std::sync::Arc;

/// Execute a complete scan job in the background
pub async fn execute_scan_job(scan_id: String, state: AppState) {
    tracing::info!("Starting background scan job for scan {}", scan_id);

    // Fetch scan to get git_url and git_token
    let scan = match Scan::find_by_id(&state.db, &scan_id).await {
        Ok(Some(scan)) => scan,
        Ok(None) => {
            tracing::error!("Scan {} not found", scan_id);
            return;
        }
        Err(e) => {
            tracing::error!("Failed to fetch scan: {}", e);
            return;
        }
    };

    // Update status to in_progress
    if let Err(e) = Scan::update_status(&state.db, &scan_id, "in_progress", None).await {
        tracing::error!("Failed to update scan status: {}", e);
        return;
    }

    // Execute the scan
    if let Err(e) = execute_scan_internal(scan_id.clone(), scan.git_url, scan.git_token, state.clone()).await {
        tracing::error!("Scan job failed: {}", e);

        // Update status to failed
        let _ = Scan::update_status(&state.db, &scan_id, "failed", Some(e.to_string())).await;
    }

    tracing::info!("Scan job completed for scan {}", scan_id);
}

async fn execute_scan_internal(
    scan_id: String,
    git_url: String,
    git_token: Option<String>,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Create workspace
    let workspace = Workspace::new(state.config.temp_workspace_dir.clone(), scan_id.clone());
    let workspace_path = workspace.create().await?;
    tracing::info!("Workspace created at {:?}", workspace_path);

    // Ensure cleanup happens
    let cleanup_result = async {
        // 2. Clone repository
        tracing::info!("Cloning repository: {}", git_url);
        clone_repository(&git_url, &workspace_path, git_token.as_deref()).await?;
        tracing::info!("Repository cloned successfully");

        // 3. Run both scanners in parallel
        tracing::info!("Starting Fossology and Semgrep scans in parallel");

        // Mark both scanners as in progress
        let _ = Scan::update_fossology_status(&state.db, &scan_id, "in_progress", None).await;
        let _ = Scan::update_semgrep_status(&state.db, &scan_id, "in_progress", None).await;
        let _ = Scan::update_overall_status(&state.db, &scan_id).await;

        // Clone state for parallel execution
        let fossology_state = state.clone();
        let semgrep_state = state.clone();
        let fossology_scan_id = scan_id.clone();
        let semgrep_scan_id = scan_id.clone();
        let fossology_path = workspace_path.clone();
        let semgrep_path = workspace_path.clone();

        // Run scanners in parallel
        let (fossology_result, semgrep_result) = tokio::join!(
            async {
                let result = fossology_state.fossology_scanner.scan(&fossology_path).await;
                match &result {
                    Ok(results) => {
                        tracing::info!("Fossology scan completed with {} results", results.len());
                        let _ = Scan::update_fossology_status(&fossology_state.db, &fossology_scan_id, "completed", None).await;
                    }
                    Err(e) => {
                        tracing::error!("Fossology scan failed: {}", e);
                        let _ = Scan::update_fossology_status(&fossology_state.db, &fossology_scan_id, "failed", Some(e.to_string())).await;
                    }
                }
                let _ = Scan::update_overall_status(&fossology_state.db, &fossology_scan_id).await;
                result
            },
            async {
                let result = semgrep_state.semgrep_scanner.scan(&semgrep_path).await;
                match &result {
                    Ok(results) => {
                        tracing::info!("Semgrep scan completed with {} results", results.len());
                        let _ = Scan::update_semgrep_status(&semgrep_state.db, &semgrep_scan_id, "completed", None).await;
                    }
                    Err(e) => {
                        tracing::error!("Semgrep scan failed: {}", e);
                        let _ = Scan::update_semgrep_status(&semgrep_state.db, &semgrep_scan_id, "failed", Some(e.to_string())).await;
                    }
                }
                let _ = Scan::update_overall_status(&semgrep_state.db, &semgrep_scan_id).await;
                result
            }
        );

        // Get results (fail if either scanner failed)
        let mut scan_results = fossology_result?;
        let semgrep_results = semgrep_result?;

        tracing::info!("Parallel scans completed: {} Fossology results, {} Semgrep results",
            scan_results.len(), semgrep_results.len());

        // 4. Merge Semgrep results into Fossology results
        merge_scan_results(&mut scan_results, semgrep_results);
        tracing::info!("Merged results, total files: {}", scan_results.len());

        // 5. Store results in database
        tracing::info!("Storing results in database");
        store_scan_results(&state.db, &scan_id, scan_results).await?;
        tracing::info!("Results stored successfully");

        // 6. Update overall status to completed (should already be set by individual scanners)
        Scan::update_overall_status(&state.db, &scan_id).await?;
        tracing::info!("Scan status updated to completed");

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    }
    .await;

    // 6. Cleanup workspace
    tracing::info!("Cleaning up workspace");
    workspace.cleanup().await?;
    tracing::info!("Workspace cleaned up");

    cleanup_result
}

/// Merge Semgrep ECC results into Fossology results
/// This combines results from both scanners by file path
fn merge_scan_results(
    fossology_results: &mut Vec<crate::scanner::ScanResult>,
    semgrep_results: Vec<crate::scanner::ScanResult>,
) {
    use std::collections::HashMap;

    // Create a map of file paths to indices in fossology_results
    let mut file_index_map: HashMap<String, usize> = HashMap::new();
    for (idx, result) in fossology_results.iter().enumerate() {
        file_index_map.insert(result.file_path.clone(), idx);
    }

    // Separate Semgrep results into those to merge and those to add
    let mut results_to_add = Vec::new();

    for semgrep_result in semgrep_results {
        if let Some(&idx) = file_index_map.get(&semgrep_result.file_path) {
            // File already has Fossology results, merge ECC findings
            fossology_results[idx].ecc_findings.extend(semgrep_result.ecc_findings);
        } else {
            // File only has Semgrep results, queue for addition
            results_to_add.push(semgrep_result);
        }
    }

    // Add new results
    fossology_results.extend(results_to_add);
}

/// Store scan results in the database
async fn store_scan_results(
    pool: &SqlitePool,
    scan_id: &str,
    scan_results: Vec<crate::scanner::ScanResult>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for result in scan_results {
        // Store licenses
        for license in result.licenses {
            DbScanResult::create_license(
                pool,
                scan_id,
                &result.file_path,
                &license.name,
                license.spdx_id.as_deref(),
                license.confidence,
            )
            .await?;
        }

        // Store copyrights
        for copyright in result.copyrights {
            DbScanResult::create_copyright(
                pool,
                scan_id,
                &result.file_path,
                &copyright.statement,
                &copyright.holders,
                &copyright.years,
            )
            .await?;
        }

        // Store ECC findings
        for ecc_finding in result.ecc_findings {
            DbScanResult::create_ecc(
                pool,
                scan_id,
                &result.file_path,
                &ecc_finding.content,
                &ecc_finding.risk_severity,
                ecc_finding.source.as_deref(),
                ecc_finding.line_number,
                ecc_finding.check_id.as_deref(),
            )
            .await?;
        }
    }

    Ok(())
}
