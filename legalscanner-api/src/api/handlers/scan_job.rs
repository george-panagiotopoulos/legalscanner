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

        // 3. Run scanner
        tracing::info!("Starting scan");
        let scan_results = state.scanner.scan(&workspace_path).await?;
        tracing::info!("Scan completed with {} results", scan_results.len());

        // 4. Store results in database
        tracing::info!("Storing results in database");
        store_scan_results(&state.db, &scan_id, scan_results).await?;
        tracing::info!("Results stored successfully");

        // 5. Update status to completed
        Scan::update_status(&state.db, &scan_id, "completed", None).await?;
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
    }

    Ok(())
}
