use legalscanner_api::AppState;
use legalscanner_api::config::Config;
use legalscanner_api::scanner::fossology::FossologyScanner;
use legalscanner_api::scanner::semgrep::SemgrepScanner;
use legalscanner_api::{api, db, git};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "legalscanner_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Initialize database
    let db_pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    db::run_migrations(&db_pool).await?;
    tracing::info!("Database migrations completed");

    // Initialize Fossology scanner
    let fossology_scanner = FossologyScanner::new(
        config.fossology_url.clone(),
        config.fossology_api_token.clone(),
    );
    tracing::info!("Fossology scanner initialized");

    // Initialize Semgrep scanner
    let semgrep_scanner = SemgrepScanner::new();
    tracing::info!("Semgrep scanner initialized");

    // Ensure workspace directory exists
    git::workspace::ensure_base_dir(&config.temp_workspace_dir).await?;
    tracing::info!("Workspace directory ready");

    // Build app state
    let app_state = AppState {
        db: db_pool,
        config: Arc::new(config.clone()),
        fossology_scanner: Arc::new(fossology_scanner),
        semgrep_scanner: Arc::new(semgrep_scanner),
    };

    // Build router
    let app = api::routes::create_router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port))
        .await?;
    tracing::info!("Server starting on port {}", config.server_port);

    axum::serve(listener, app).await?;

    Ok(())
}
