use crate::{
    api::models::{CreateApiKeyRequest, CreateApiKeyResponse},
    db::models::ApiKey,
    error::AppError,
    utils::crypto,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

/// POST /api/v1/api-keys - Create a new API key
pub async fn create_api_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreateApiKeyResponse>), AppError> {
    // Generate a new API key
    let raw_key = crypto::generate_api_key();

    // Hash the key
    let key_hash = crypto::hash_api_key(&raw_key, &state.config.api_key_salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash API key: {}", e)))?;

    // Store in database
    let api_key = ApiKey::create(&state.db, payload.name, key_hash).await?;

    // Return the raw key (only time it will be shown)
    Ok((
        StatusCode::CREATED,
        Json(CreateApiKeyResponse {
            id: api_key.id,
            name: api_key.name,
            key: raw_key,
            created_at: api_key.created_at,
            message: "Save this key securely. It will not be shown again.".to_string(),
        }),
    ))
}

/// GET /api/v1/api-keys - List all API keys
pub async fn list_api_keys(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiKey>>, AppError> {
    let api_keys = ApiKey::list_all(&state.db).await?;
    Ok(Json(api_keys))
}

/// DELETE /api/v1/api-keys/:id - Delete an API key
pub async fn delete_api_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // Check if key exists
    let _ = ApiKey::find_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("API key {} not found", id)))?;

    ApiKey::delete(&state.db, &id).await?;

    Ok(StatusCode::NO_CONTENT)
}
