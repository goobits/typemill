//! Client configuration utilities

use cb_core::AppConfig;
use crate::client_config::ClientConfig;
use crate::error::{ClientError, ClientResult};
use std::path::Path;

/// Load client configuration from default location
pub async fn load_client_config() -> ClientResult<ClientConfig> {
    ClientConfig::load().await
}

/// Load client configuration from specific path
pub async fn load_client_config_from_path<P: AsRef<Path>>(path: P) -> ClientResult<ClientConfig> {
    ClientConfig::load_from_path(path).await
}

/// Load server configuration (for backwards compatibility)
pub fn load_server_config() -> ClientResult<AppConfig> {
    AppConfig::load().map_err(|err| ClientError::config(format!("Failed to load server config: {}", err)))
}

/// Configuration merge utilities
pub mod merge {
    use super::*;

    /// Merge command line arguments with client configuration
    pub fn merge_with_args(
        mut config: ClientConfig,
        url: Option<String>,
        token: Option<String>,
        timeout: Option<u64>,
    ) -> ClientConfig {
        if let Some(url) = url {
            config.set_url(url);
        }
        if let Some(token) = token {
            config.set_token(token);
        }
        if let Some(timeout) = timeout {
            config.set_timeout_ms(timeout);
        }
        config
    }

    /// Create a client config with environment variable overrides
    pub async fn with_env_overrides() -> ClientResult<ClientConfig> {
        let mut config = ClientConfig::load().await?;

        // Override with environment variables if present
        if let Ok(url) = std::env::var("CODEFLOW_BUDDY_URL") {
            config.set_url(url);
        }

        if let Ok(token) = std::env::var("CODEFLOW_BUDDY_TOKEN") {
            config.set_token(token);
        }

        if let Ok(timeout_str) = std::env::var("CODEFLOW_BUDDY_TIMEOUT") {
            if let Ok(timeout) = timeout_str.parse::<u64>() {
                config.set_timeout_ms(timeout);
            }
        }

        Ok(config)
    }
}

/// Configuration validation utilities
pub mod validate {
    use super::*;

    /// Validate a complete client configuration
    pub fn client_config(config: &ClientConfig) -> ClientResult<()> {
        config.validate()
    }

    /// Check if configuration is sufficient for making requests
    pub fn is_complete(config: &ClientConfig) -> bool {
        config.is_complete()
    }

    /// Get missing configuration items
    pub fn missing_items(config: &ClientConfig) -> Vec<String> {
        let mut missing = Vec::new();

        if config.url.is_none() {
            missing.push("Server URL".to_string());
        }

        missing
    }

    /// Suggest configuration actions
    pub fn suggestions(config: &ClientConfig) -> Vec<String> {
        let mut suggestions = Vec::new();

        if config.url.is_none() {
            suggestions.push("Run 'codeflow-buddy setup' to configure server URL".to_string());
        }

        if !config.has_token() {
            suggestions.push("Consider adding an authentication token for secure access".to_string());
        }

        suggestions
    }
}

/// Configuration discovery utilities
pub mod discover {
    use super::*;
    use std::env;

    /// Discover configuration from various sources in priority order:
    /// 1. Command line arguments
    /// 2. Environment variables
    /// 3. Configuration file
    /// 4. Defaults
    pub async fn from_all_sources(
        config_path: Option<&str>,
        url_override: Option<String>,
        token_override: Option<String>,
        timeout_override: Option<u64>,
    ) -> ClientResult<ClientConfig> {
        // Start with file configuration
        let mut config = if let Some(path) = config_path {
            ClientConfig::load_from_path(path).await?
        } else {
            ClientConfig::load().await?
        };

        // Apply environment variables
        if let Ok(env_url) = env::var("CODEFLOW_BUDDY_URL") {
            if url_override.is_none() {
                config.set_url(env_url);
            }
        }

        if let Ok(env_token) = env::var("CODEFLOW_BUDDY_TOKEN") {
            if token_override.is_none() {
                config.set_token(env_token);
            }
        }

        if let Ok(env_timeout) = env::var("CODEFLOW_BUDDY_TIMEOUT") {
            if timeout_override.is_none() {
                if let Ok(timeout) = env_timeout.parse::<u64>() {
                    config.set_timeout_ms(timeout);
                }
            }
        }

        // Apply command line overrides (highest priority)
        if let Some(url) = url_override {
            config.set_url(url);
        }
        if let Some(token) = token_override {
            config.set_token(token);
        }
        if let Some(timeout) = timeout_override {
            config.set_timeout_ms(timeout);
        }

        config.validate()?;
        Ok(config)
    }

    /// Check if configuration exists in standard locations
    pub fn config_exists() -> bool {
        if let Ok(config_path) = ClientConfig::default_config_path() {
            config_path.exists()
        } else {
            false
        }
    }

    /// Get configuration status summary
    pub async fn status_summary() -> String {
        let config_exists = config_exists();
        let env_vars = check_env_vars();

        let mut summary = String::new();
        summary.push_str(&format!("Configuration file: {}\n", if config_exists { "Found" } else { "Not found" }));
        summary.push_str(&format!("Environment variables: {}\n", env_vars));

        if let Ok(config) = ClientConfig::load().await {
            summary.push_str(&format!("Current config: {}\n", config.summary()));
        }

        summary
    }

    /// Check which environment variables are set
    fn check_env_vars() -> String {
        let vars = [
            ("CODEFLOW_BUDDY_URL", env::var("CODEFLOW_BUDDY_URL").is_ok()),
            ("CODEFLOW_BUDDY_TOKEN", env::var("CODEFLOW_BUDDY_TOKEN").is_ok()),
            ("CODEFLOW_BUDDY_TIMEOUT", env::var("CODEFLOW_BUDDY_TIMEOUT").is_ok()),
        ];

        let set_vars: Vec<&str> = vars
            .iter()
            .filter_map(|(name, is_set)| if *is_set { Some(*name) } else { None })
            .collect();

        if set_vars.is_empty() {
            "None set".to_string()
        } else {
            format!("{} set", set_vars.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_loading() {
        // This test requires a valid config file, so we'll just test the error case
        let result = load_client_config_from_path("/nonexistent/path").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_with_args() {
        let mut config = ClientConfig::new();
        config.set_url("ws://localhost:3000".to_string());

        let merged = merge::merge_with_args(
            config,
            Some("ws://example.com:4000".to_string()),
            Some("test-token".to_string()),
            Some(60000),
        );

        assert_eq!(merged.url, Some("ws://example.com:4000".to_string()));
        assert_eq!(merged.token, Some("test-token".to_string()));
        assert_eq!(merged.timeout_ms, Some(60000));
    }

    #[test]
    fn test_validation() {
        let mut config = ClientConfig::new();

        // Valid config
        config.set_url("ws://localhost:3000".to_string());
        assert!(validate::client_config(&config).is_ok());

        // Missing URL is still valid (can be provided later)
        let empty_config = ClientConfig::new();
        assert!(validate::client_config(&empty_config).is_ok());
    }

    #[test]
    fn test_missing_items() {
        let config = ClientConfig::new();
        let missing = validate::missing_items(&config);
        assert!(missing.contains(&"Server URL".to_string()));
    }

    #[test]
    fn test_suggestions() {
        let config = ClientConfig::new();
        let suggestions = validate::suggestions(&config);
        assert!(!suggestions.is_empty());
    }
}