// Library exports for legalscanner-api
// This allows binaries to import modules from the main crate

pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod git;
pub mod scanner;
pub mod utils;

pub use error::AppError;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: Arc<config::Config>,
    pub fossology_scanner: Arc<dyn scanner::Scanner>,
    pub semgrep_scanner: Arc<dyn scanner::Scanner>,
}
