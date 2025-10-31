pub mod models;

use sqlx::{sqlite::{SqlitePoolOptions, SqliteConnectOptions}, SqlitePool};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Extract file path from SQLite URL if needed
    let file_path = if database_url.starts_with("sqlite://") {
        database_url.strip_prefix("sqlite://").unwrap()
    } else {
        database_url
    };

    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Create connect options with create_if_missing enabled
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
}
