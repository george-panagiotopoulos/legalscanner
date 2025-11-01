/// Utility to backfill risk scores for existing scans
///
/// Usage: cargo run --bin backfill_risk_scores
use legalscanner_api::api::handlers::risk::calculate_risk_score;
use legalscanner_api::config::Config;
use legalscanner_api::db;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting risk score backfill");

    // Load config
    let config = Config::from_env()?;

    // Connect to database
    let pool = db::create_pool(&config.database_url).await?;

    // Run migrations to ensure schema is up to date
    info!("Running migrations");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Get all completed scans without risk scores
    let scans = sqlx::query!(
        r#"
        SELECT id, status
        FROM scans
        WHERE status = 'completed' AND risk_score IS NULL
        ORDER BY completed_at DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    info!("Found {} scans without risk scores", scans.len());

    let mut success_count = 0;
    let mut error_count = 0;

    for scan in scans {
        let scan_id = scan.id.as_ref().unwrap();
        info!("Calculating risk for scan {}", scan_id);

        match calculate_risk_score(&pool, scan_id).await {
            Ok(risk_assessment) => {
                info!(
                    "  Risk calculated: score={}, level={}",
                    risk_assessment.score, risk_assessment.level
                );

                // Serialize risk factors to JSON
                let risk_factors_json = serde_json::to_string(&risk_assessment.factors)
                    .unwrap_or_else(|_| "[]".to_string());

                // Update scan with risk assessment
                match sqlx::query!(
                    r#"
                    UPDATE scans
                    SET risk_score = ?,
                        risk_level = ?,
                        risk_factors = ?
                    WHERE id = ?
                    "#,
                    risk_assessment.score,
                    risk_assessment.level,
                    risk_factors_json,
                    scan_id
                )
                .execute(&pool)
                .await
                {
                    Ok(_) => {
                        info!("  Risk assessment stored successfully");
                        success_count += 1;
                    }
                    Err(e) => {
                        error!("  Failed to store risk assessment: {}", e);
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                error!("  Failed to calculate risk: {}", e);
                error_count += 1;
            }
        }
    }

    info!("Backfill complete:");
    info!("  Success: {}", success_count);
    info!("  Errors: {}", error_count);

    Ok(())
}
