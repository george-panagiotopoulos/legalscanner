use std::path::{Path, PathBuf};
use tokio::fs;

/// Manages temporary workspace for scan operations
pub struct Workspace {
    base_dir: PathBuf,
    scan_id: String,
}

impl Workspace {
    pub fn new(base_dir: PathBuf, scan_id: String) -> Self {
        Self { base_dir, scan_id }
    }

    /// Create the workspace directory
    pub async fn create(&self) -> Result<PathBuf, std::io::Error> {
        let workspace_path = self.path();
        fs::create_dir_all(&workspace_path).await?;
        tracing::debug!("Created workspace at {:?}", workspace_path);
        Ok(workspace_path)
    }

    /// Get the workspace path
    pub fn path(&self) -> PathBuf {
        self.base_dir.join(&self.scan_id)
    }

    /// Check if workspace exists
    pub async fn exists(&self) -> bool {
        self.path().exists()
    }

    /// Clean up the workspace
    pub async fn cleanup(&self) -> Result<(), std::io::Error> {
        let workspace_path = self.path();
        if workspace_path.exists() {
            tracing::debug!("Cleaning up workspace at {:?}", workspace_path);
            fs::remove_dir_all(workspace_path).await?;
            tracing::debug!("Workspace cleaned up successfully");
        }
        Ok(())
    }
}

/// Ensure base workspace directory exists
pub async fn ensure_base_dir(base_dir: &Path) -> Result<(), std::io::Error> {
    if !base_dir.exists() {
        fs::create_dir_all(base_dir).await?;
        tracing::info!("Created base workspace directory at {:?}", base_dir);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_workspace_creation_and_cleanup() {
        let temp_dir = tempdir().unwrap();
        let workspace = Workspace::new(
            temp_dir.path().to_path_buf(),
            "test-scan-123".to_string(),
        );

        // Create workspace
        let path = workspace.create().await.unwrap();
        assert!(path.exists());

        // Cleanup
        workspace.cleanup().await.unwrap();
        assert!(!path.exists());
    }
}
