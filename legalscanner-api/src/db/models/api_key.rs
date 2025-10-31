use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub key_hash: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub is_active: bool,
}

impl ApiKey {
    pub async fn create(
        pool: &SqlitePool,
        name: String,
        key_hash: String,
    ) -> Result<ApiKey, sqlx::Error> {
        let id = Uuid::new_v4().to_string();

        sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (id, name, key_hash)
            VALUES (?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&name)
        .bind(&key_hash)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_hash(
        pool: &SqlitePool,
        key_hash: &str,
    ) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = ? AND is_active = 1",
        )
        .bind(key_hash)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    pub async fn update_last_used(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE api_keys SET last_used_at = datetime('now') WHERE id = ?",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn deactivate(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET is_active = 0 WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM api_keys WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
