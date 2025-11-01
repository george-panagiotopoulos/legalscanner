use git2::{Repository, RemoteCallbacks, FetchOptions, build::RepoBuilder};
use std::path::Path;

/// Clone a Git repository to a destination path
/// Supports both public and private repositories
/// Accepts optional token parameter, falls back to GIT_TOKEN environment variable
pub async fn clone_repository(url: &str, destination: &Path, token: Option<&str>) -> Result<(), git2::Error> {
    // Validate URL first
    validate_git_url(url).map_err(|e| git2::Error::from_str(&e))?;

    // Use tokio::task::spawn_blocking for blocking git2 operations
    let url = url.to_string();
    let destination = destination.to_path_buf();
    let token = token.map(|t| t.to_string());

    tokio::task::spawn_blocking(move || {
        tracing::info!("Cloning repository {} to {:?}", url, destination);

        // Use provided token or fall back to environment variable
        let git_token = token.or_else(|| std::env::var("GIT_TOKEN").ok());

        if let Some(token) = git_token {
            tracing::info!("Using authentication token for git clone");

            // Setup authentication callbacks
            // For GitHub PATs, use the token as username with empty password
            // This is the correct authentication method for HTTPS GitHub clones with PAT
            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                tracing::debug!("Git credentials callback invoked");
                git2::Cred::userpass_plaintext(&token, "")
            });

            // Setup fetch options with callbacks
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            // Clone with authentication
            let mut builder = RepoBuilder::new();
            builder.fetch_options(fetch_options);
            builder.clone(&url, &destination)?;
        } else {
            tracing::info!("No GIT_TOKEN found, attempting public clone");
            // For public repositories, use simple clone
            Repository::clone(&url, &destination)?;
        }

        tracing::info!("Repository cloned successfully");
        Ok(())
    })
    .await
    .map_err(|e| git2::Error::from_str(&e.to_string()))?
}

/// Validate a Git URL format
pub fn validate_git_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("Git URL cannot be empty".to_string());
    }

    // Check for common Git URL patterns
    let valid_prefixes = ["http://", "https://", "git://", "ssh://", "git@"];

    let is_valid = valid_prefixes.iter().any(|prefix| url.starts_with(prefix));

    if !is_valid {
        return Err(format!(
            "Invalid Git URL format. Must start with one of: {}",
            valid_prefixes.join(", ")
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_git_url() {
        assert!(validate_git_url("https://github.com/user/repo.git").is_ok());
        assert!(validate_git_url("http://github.com/user/repo.git").is_ok());
        assert!(validate_git_url("git://github.com/user/repo.git").is_ok());
        assert!(validate_git_url("git@github.com:user/repo.git").is_ok());
        assert!(validate_git_url("").is_err());
        assert!(validate_git_url("not-a-git-url").is_err());
    }
}
