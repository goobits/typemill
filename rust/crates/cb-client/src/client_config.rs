use crate::error::{ClientError, ClientResult};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// Client configuration for connecting to codeflow-buddy server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// WebSocket server URL (e.g., "ws://localhost:3000")
    pub url: Option<String>,
    /// JWT authentication token
    pub token: Option<String>,
    /// Request timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            url: None,
            token: None,
            timeout_ms: Some(30000), // 30 seconds default
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from file, falling back to defaults and environment variables
    pub async fn load() -> ClientResult<Self> {
        let config_path = Self::default_config_path()?;

        let mut config = if config_path.exists() {
            debug!("Loading config from {}", config_path.display());
            let content = fs::read_to_string(&config_path).await
                .map_err(|e| ClientError::ConfigError(format!("Failed to read config file: {}", e)))?;

            serde_json::from_str(&content)
                .map_err(|e| ClientError::ConfigError(format!("Failed to parse config file: {}", e)))?
        } else {
            debug!("No config file found, using defaults");
            Self::default()
        };

        // Override with environment variables if present
        if let Ok(url) = std::env::var("CODEFLOW_BUDDY_URL") {
            debug!("Using URL from environment variable");
            config.url = Some(url);
        }

        if let Ok(token) = std::env::var("CODEFLOW_BUDDY_TOKEN") {
            debug!("Using token from environment variable");
            config.token = Some(token);
        }

        if let Ok(timeout) = std::env::var("CODEFLOW_BUDDY_TIMEOUT") {
            match timeout.parse::<u64>() {
                Ok(timeout_ms) => {
                    debug!("Using timeout from environment variable: {}ms", timeout_ms);
                    config.timeout_ms = Some(timeout_ms);
                }
                Err(e) => {
                    warn!("Invalid timeout in environment variable: {}", e);
                }
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a specific file path
    pub async fn load_from_path<P: AsRef<Path>>(path: P) -> ClientResult<Self> {
        let path = path.as_ref();
        debug!("Loading config from {}", path.display());

        let content = fs::read_to_string(path).await
            .map_err(|e| ClientError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let config: Self = serde_json::from_str(&content)
            .map_err(|e| ClientError::ConfigError(format!("Failed to parse config file: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to the default config file
    pub async fn save(&self) -> ClientResult<()> {
        let config_path = Self::default_config_path()?;
        self.save_to_path(&config_path).await
    }

    /// Save configuration to a specific file path
    pub async fn save_to_path<P: AsRef<Path>>(&self, path: P) -> ClientResult<()> {
        let path = path.as_ref();

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| ClientError::ConfigError(format!("Failed to create config directory: {}", e)))?;
        }

        // Serialize config with pretty formatting
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ClientError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content).await
            .map_err(|e| ClientError::ConfigError(format!("Failed to write config file: {}", e)))?;

        info!("Configuration saved to {}", path.display());
        Ok(())
    }

    /// Get the default configuration file path
    pub fn default_config_path() -> ClientResult<PathBuf> {
        let home = home_dir()
            .ok_or_else(|| ClientError::ConfigError("Unable to determine home directory".to_string()))?;

        Ok(home.join(".codeflow-buddy").join("config.json"))
    }

    /// Get the configuration directory path
    pub fn config_dir() -> ClientResult<PathBuf> {
        let home = home_dir()
            .ok_or_else(|| ClientError::ConfigError("Unable to determine home directory".to_string()))?;

        Ok(home.join(".codeflow-buddy"))
    }

    /// Validate the configuration
    pub fn validate(&self) -> ClientResult<()> {
        // Validate URL format if provided
        if let Some(ref url) = self.url {
            if let Err(e) = url::Url::parse(url) {
                return Err(ClientError::ConfigError(format!("Invalid URL format: {}", e)));
            }
        }

        // Validate timeout is reasonable
        if let Some(timeout) = self.timeout_ms {
            if timeout == 0 {
                return Err(ClientError::ConfigError("Timeout cannot be zero".to_string()));
            }
            if timeout > 300_000 { // 5 minutes max
                return Err(ClientError::ConfigError("Timeout cannot exceed 5 minutes".to_string()));
            }
        }

        Ok(())
    }

    /// Get the URL, returning an error if not configured
    pub fn get_url(&self) -> ClientResult<&str> {
        self.url.as_deref()
            .ok_or_else(|| ClientError::ConfigError("No server URL configured".to_string()))
    }

    /// Get the timeout in milliseconds
    pub fn get_timeout_ms(&self) -> u64 {
        self.timeout_ms.unwrap_or(30000)
    }

    /// Check if authentication token is available
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    /// Get the authentication token
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Set the URL
    pub fn set_url(&mut self, url: String) {
        self.url = Some(url);
    }

    /// Set the token
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Set the timeout
    pub fn set_timeout_ms(&mut self, timeout_ms: u64) {
        self.timeout_ms = Some(timeout_ms);
    }

    /// Clear the token
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Create a config with command line overrides
    pub fn with_overrides(&self, url: Option<String>, token: Option<String>) -> Self {
        let mut config = self.clone();

        if let Some(url) = url {
            config.url = Some(url);
        }

        if let Some(token) = token {
            config.token = Some(token);
        }

        config
    }

    /// Check if the configuration appears to be complete for making requests
    pub fn is_complete(&self) -> bool {
        self.url.is_some()
    }

    /// Get a summary of the configuration for display
    pub fn summary(&self) -> String {
        let url = self.url.as_deref().unwrap_or("<not configured>");
        let token_status = if self.token.is_some() { "configured" } else { "not configured" };
        let timeout = self.get_timeout_ms();

        format!(
            "URL: {}\nToken: {}\nTimeout: {}ms",
            url, token_status, timeout
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let mut config = ClientConfig::new();
        config.set_url("ws://localhost:3000".to_string());
        config.set_token("test-token".to_string());
        config.set_timeout_ms(60000);

        // Save config
        config.save_to_path(&config_path).await.unwrap();

        // Load config
        let loaded_config = ClientConfig::load_from_path(&config_path).await.unwrap();

        assert_eq!(loaded_config.url, Some("ws://localhost:3000".to_string()));
        assert_eq!(loaded_config.token, Some("test-token".to_string()));
        assert_eq!(loaded_config.timeout_ms, Some(60000));
    }

    #[test]
    fn test_config_validation() {
        let mut config = ClientConfig::new();

        // Valid config
        assert!(config.validate().is_ok());

        // Invalid URL
        config.set_url("invalid-url".to_string());
        assert!(config.validate().is_err());

        // Valid URL
        config.set_url("ws://localhost:3000".to_string());
        assert!(config.validate().is_ok());

        // Invalid timeout
        config.set_timeout_ms(0);
        assert!(config.validate().is_err());

        // Timeout too large
        config.set_timeout_ms(400_000);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_overrides() {
        let mut base_config = ClientConfig::new();
        base_config.set_url("ws://localhost:3000".to_string());
        base_config.set_token("base-token".to_string());

        let overridden = base_config.with_overrides(
            Some("ws://example.com:4000".to_string()),
            Some("override-token".to_string())
        );

        assert_eq!(overridden.url, Some("ws://example.com:4000".to_string()));
        assert_eq!(overridden.token, Some("override-token".to_string()));

        // Original should be unchanged
        assert_eq!(base_config.url, Some("ws://localhost:3000".to_string()));
        assert_eq!(base_config.token, Some("base-token".to_string()));
    }
}