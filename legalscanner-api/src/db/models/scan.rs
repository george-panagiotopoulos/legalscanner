use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Scan {
    pub id: String,
    pub git_url: String,
    pub status: String, // pending, in_progress, completed, failed
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_by_key_id: Option<String>,
    // git_token is stored but not serialized for security
    #[serde(skip_serializing)]
    pub git_token: Option<String>,
    // Individual scanner status tracking
    pub fossology_status: String,
    pub semgrep_status: String,
    pub fossology_started_at: Option<String>,
    pub fossology_completed_at: Option<String>,
    pub semgrep_started_at: Option<String>,
    pub semgrep_completed_at: Option<String>,
    pub fossology_error: Option<String>,
    pub semgrep_error: Option<String>,
}

impl Scan {
    pub async fn create(
        pool: &SqlitePool,
        git_url: String,
        git_token: Option<String>,
        created_by_key_id: Option<String>,
    ) -> Result<Scan, sqlx::Error> {
        let id = Uuid::new_v4().to_string();

        sqlx::query_as::<_, Scan>(
            r#"
            INSERT INTO scans (id, git_url, git_token, status, created_by_key_id)
            VALUES (?, ?, ?, 'pending', ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&git_url)
        .bind(&git_token)
        .bind(created_by_key_id)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Scan>, sqlx::Error> {
        sqlx::query_as::<_, Scan>("SELECT * FROM scans WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn list_all(pool: &SqlitePool, limit: i64) -> Result<Vec<Scan>, sqlx::Error> {
        sqlx::query_as::<_, Scan>("SELECT * FROM scans ORDER BY created_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(pool)
            .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: &str,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE scans
            SET status = ?,
                error_message = ?,
                started_at = CASE
                    WHEN status = 'pending' AND ? = 'in_progress'
                    THEN datetime('now')
                    ELSE started_at
                END,
                completed_at = CASE
                    WHEN ? IN ('completed', 'failed')
                    THEN datetime('now')
                    ELSE completed_at
                END
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(error_message)
        .bind(status)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update Fossology scanner status
    pub async fn update_fossology_status(
        pool: &SqlitePool,
        id: &str,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE scans
            SET fossology_status = ?,
                fossology_error = ?,
                fossology_started_at = CASE
                    WHEN fossology_status = 'pending' AND ? = 'in_progress'
                    THEN datetime('now')
                    ELSE fossology_started_at
                END,
                fossology_completed_at = CASE
                    WHEN ? IN ('completed', 'failed')
                    THEN datetime('now')
                    ELSE fossology_completed_at
                END
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(error_message)
        .bind(status)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update Semgrep scanner status
    pub async fn update_semgrep_status(
        pool: &SqlitePool,
        id: &str,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE scans
            SET semgrep_status = ?,
                semgrep_error = ?,
                semgrep_started_at = CASE
                    WHEN semgrep_status = 'pending' AND ? = 'in_progress'
                    THEN datetime('now')
                    ELSE semgrep_started_at
                END,
                semgrep_completed_at = CASE
                    WHEN ? IN ('completed', 'failed')
                    THEN datetime('now')
                    ELSE semgrep_completed_at
                END
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(error_message)
        .bind(status)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update overall scan status based on individual scanner statuses
    pub async fn update_overall_status(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE scans
            SET status = CASE
                -- If both completed, overall is completed
                WHEN fossology_status = 'completed' AND semgrep_status = 'completed' THEN 'completed'
                -- If either failed, overall is failed
                WHEN fossology_status = 'failed' OR semgrep_status = 'failed' THEN 'failed'
                -- If at least one is in progress, overall is in progress
                WHEN fossology_status = 'in_progress' OR semgrep_status = 'in_progress' THEN 'in_progress'
                -- Otherwise pending
                ELSE 'pending'
            END,
            completed_at = CASE
                WHEN fossology_status = 'completed' AND semgrep_status = 'completed'
                THEN datetime('now')
                ELSE completed_at
            END
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM scans WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete_all(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scans")
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_summary(pool: &SqlitePool, scan_id: &str) -> Result<ScanSummary, sqlx::Error> {
        let summary = sqlx::query_as::<_, ScanSummary>(
            r#"
            SELECT
                COUNT(DISTINCT CASE WHEN result_type = 'license' THEN file_path END) as files_with_licenses,
                COUNT(DISTINCT CASE WHEN result_type = 'copyright' THEN file_path END) as files_with_copyrights,
                COUNT(DISTINCT CASE WHEN result_type = 'license' THEN license_name END) as unique_licenses,
                COUNT(DISTINCT CASE WHEN result_type = 'copyright' THEN copyright_statement END) as unique_copyrights,
                COUNT(DISTINCT file_path) as total_files
            FROM scan_results
            WHERE scan_id = ?
            "#,
        )
        .bind(scan_id)
        .fetch_one(pool)
        .await?;

        Ok(summary)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ScanSummary {
    pub total_files: i64,
    pub files_with_licenses: i64,
    pub files_with_copyrights: i64,
    pub unique_licenses: i64,
    pub unique_copyrights: i64,
}
