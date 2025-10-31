use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Scanner error: {0}")]
    Scanner(String),

    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AppError::Git(ref e) => {
                tracing::error!("Git error: {}", e);
                (StatusCode::BAD_REQUEST, "Git operation failed")
            }
            AppError::Scanner(ref msg) => {
                tracing::error!("Scanner error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Scanner error")
            }
            AppError::Docker(ref msg) => {
                tracing::error!("Docker error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Docker error")
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string(),
        }));

        (status, body).into_response()
    }
}
