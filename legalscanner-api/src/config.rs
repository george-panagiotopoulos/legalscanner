use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub fossology_url: String,
    pub fossology_api_token: String,
    pub temp_workspace_dir: PathBuf,
    pub server_port: u16,
    pub api_key_salt: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "./data/legalscanner.db".to_string()),
            fossology_url: std::env::var("FOSSOLOGY_URL")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            fossology_api_token: std::env::var("FOSSOLOGY_API_TOKEN")
                .unwrap_or_else(|_| "".to_string()),
            temp_workspace_dir: std::env::var("TEMP_WORKSPACE_DIR")
                .unwrap_or_else(|_| "/tmp/legalscanner".to_string())
                .into(),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            api_key_salt: std::env::var("API_KEY_SALT")
                .unwrap_or_else(|_| "default-salt-change-in-production".to_string()),
        })
    }
}
