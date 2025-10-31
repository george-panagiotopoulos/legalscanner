use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScanResult {
    pub id: i64,
    pub scan_id: String,
    pub file_path: String,
    pub result_type: String, // license, copyright
    pub license_name: Option<String>,
    pub license_spdx_id: Option<String>,
    pub copyright_statement: Option<String>,
    pub copyright_holders: Option<String>, // JSON array
    pub copyright_years: Option<String>,    // JSON array
    pub confidence: Option<f32>,
    pub raw_data: Option<String>, // Original scanner output (JSON)
}

impl ScanResult {
    pub async fn create_license(
        pool: &SqlitePool,
        scan_id: &str,
        file_path: &str,
        license_name: &str,
        license_spdx_id: Option<&str>,
        confidence: f32,
    ) -> Result<ScanResult, sqlx::Error> {
        sqlx::query_as::<_, ScanResult>(
            r#"
            INSERT INTO scan_results
            (scan_id, file_path, result_type, license_name, license_spdx_id, confidence)
            VALUES (?, ?, 'license', ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(scan_id)
        .bind(file_path)
        .bind(license_name)
        .bind(license_spdx_id)
        .bind(confidence)
        .fetch_one(pool)
        .await
    }

    pub async fn create_copyright(
        pool: &SqlitePool,
        scan_id: &str,
        file_path: &str,
        copyright_statement: &str,
        copyright_holders: &[String],
        copyright_years: &[String],
    ) -> Result<ScanResult, sqlx::Error> {
        let holders_json = serde_json::to_string(copyright_holders).unwrap_or_default();
        let years_json = serde_json::to_string(copyright_years).unwrap_or_default();

        sqlx::query_as::<_, ScanResult>(
            r#"
            INSERT INTO scan_results
            (scan_id, file_path, result_type, copyright_statement, copyright_holders, copyright_years)
            VALUES (?, ?, 'copyright', ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(scan_id)
        .bind(file_path)
        .bind(copyright_statement)
        .bind(holders_json)
        .bind(years_json)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_scan_id(
        pool: &SqlitePool,
        scan_id: &str,
    ) -> Result<Vec<ScanResult>, sqlx::Error> {
        sqlx::query_as::<_, ScanResult>(
            "SELECT * FROM scan_results WHERE scan_id = ? ORDER BY file_path, result_type",
        )
        .bind(scan_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_licenses_by_scan_id(
        pool: &SqlitePool,
        scan_id: &str,
    ) -> Result<Vec<ScanResult>, sqlx::Error> {
        sqlx::query_as::<_, ScanResult>(
            r#"
            SELECT * FROM scan_results
            WHERE scan_id = ? AND result_type = 'license'
            ORDER BY file_path
            "#,
        )
        .bind(scan_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_copyrights_by_scan_id(
        pool: &SqlitePool,
        scan_id: &str,
    ) -> Result<Vec<ScanResult>, sqlx::Error> {
        sqlx::query_as::<_, ScanResult>(
            r#"
            SELECT * FROM scan_results
            WHERE scan_id = ? AND result_type = 'copyright'
            ORDER BY file_path
            "#,
        )
        .bind(scan_id)
        .fetch_all(pool)
        .await
    }
}
