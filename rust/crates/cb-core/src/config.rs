//! Configuration management for Codeflow Buddy

use crate::error::{CoreError, CoreResult};
use config::{Config, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// LSP configuration
    pub lsp: LspConfig,
    /// FUSE configuration (optional)
    pub fuse: Option<FuseConfig>,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Cache configuration
    pub cache: CacheConfig,
}

/// Server-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Maximum number of concurrent clients
    pub max_clients: Option<usize>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable TLS
    pub tls: Option<TlsConfig>,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_path: PathBuf,
    /// Path to private key file
    pub key_path: PathBuf,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    /// JWT secret for signing tokens
    pub jwt_secret: String,
    /// JWT expiry in seconds
    pub jwt_expiry_seconds: u64,
    /// JWT issuer
    pub jwt_issuer: String,
    /// JWT audience
    pub jwt_audience: String,
}

/// LSP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspConfig {
    /// List of LSP server configurations
    pub servers: Vec<LspServerConfig>,
    /// Default timeout for LSP requests in milliseconds
    pub default_timeout_ms: u64,
    /// Enable LSP server preloading
    pub enable_preload: bool,
}

/// Individual LSP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspServerConfig {
    /// File extensions this server handles
    pub extensions: Vec<String>,
    /// Command to run the LSP server
    pub command: Vec<String>,
    /// Working directory (optional)
    pub root_dir: Option<PathBuf>,
    /// Auto-restart interval in minutes (optional)
    pub restart_interval: Option<u64>,
}

/// FUSE filesystem configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuseConfig {
    /// Mount point for the FUSE filesystem
    pub mount_point: PathBuf,
    /// Enable read-only mode
    pub read_only: bool,
    /// Cache timeout in seconds
    pub cache_timeout_seconds: u64,
    /// Maximum file size to cache in bytes
    pub max_file_size_bytes: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Output format (json, pretty)
    pub format: String,
    /// Enable file logging
    pub file: Option<FileLoggingConfig>,
}

/// File logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLoggingConfig {
    /// Path to log file
    pub path: PathBuf,
    /// Maximum log file size in bytes
    pub max_size_bytes: u64,
    /// Number of log files to retain
    pub max_files: usize,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache size limit in bytes
    pub max_size_bytes: u64,
    /// Cache entry TTL in seconds
    pub ttl_seconds: u64,
    /// Enable persistent cache
    pub persistent: bool,
    /// Cache directory (for persistent cache)
    pub cache_dir: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            lsp: LspConfig::default(),
            fuse: None,
            logging: LoggingConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            max_clients: Some(10),
            timeout_ms: 30000,
            tls: None,
            auth: None,
        }
    }
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            servers: vec![
                LspServerConfig {
                    extensions: vec!["ts".to_string(), "tsx".to_string(), "js".to_string(), "jsx".to_string()],
                    command: vec!["typescript-language-server".to_string(), "--stdio".to_string()],
                    root_dir: None,
                    restart_interval: Some(10),
                },
                LspServerConfig {
                    extensions: vec!["py".to_string()],
                    command: vec!["pylsp".to_string()],
                    root_dir: None,
                    restart_interval: Some(5),
                },
                LspServerConfig {
                    extensions: vec!["go".to_string()],
                    command: vec!["gopls".to_string()],
                    root_dir: None,
                    restart_interval: Some(10),
                },
            ],
            default_timeout_ms: 5000,
            enable_preload: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file: None,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_bytes: 256 * 1024 * 1024, // 256 MB
            ttl_seconds: 3600, // 1 hour
            persistent: false,
            cache_dir: None,
        }
    }
}

impl AppConfig {
    /// Load configuration from environment and config files
    pub fn load() -> CoreResult<Self> {
        // Start with default configuration
        let mut app_config = AppConfig::default();

        // Load from configuration file if it exists
        let config_paths = [
            ".codebuddy/config.json",
            ".codebuddy/config.toml",
            "codebuddy.json", // Legacy support
            "codebuddy.toml", // Legacy support
        ];

        let mut config_builder = Config::builder();
        let mut file_found = false;

        for config_path in &config_paths {
            let path = std::path::Path::new(config_path);
            if path.exists() {
                let format = if config_path.ends_with(".json") {
                    FileFormat::Json
                } else {
                    FileFormat::Toml
                };
                config_builder = config_builder.add_source(File::from(path).format(format));
                file_found = true;
                break;
            }
        }

        // If a config file was found, merge it with defaults
        if file_found {
            // Override with environment variables
            config_builder = config_builder.add_source(
                Environment::with_prefix("CODEFLOW_BUDDY")
                    .separator("__")
                    .try_parsing(true),
            );

            let config = config_builder.build()?;

            // Merge file/env config into our default config
            if let Ok(server_config) = config.get::<ServerConfig>("server") {
                app_config.server = server_config;
            }
            if let Ok(lsp_config) = config.get::<LspConfig>("lsp") {
                app_config.lsp = lsp_config;
            }
            if let Ok(fuse_config) = config.get::<FuseConfig>("fuse") {
                app_config.fuse = Some(fuse_config);
            }
            if let Ok(logging_config) = config.get::<LoggingConfig>("logging") {
                app_config.logging = logging_config;
            }
            if let Ok(cache_config) = config.get::<CacheConfig>("cache") {
                app_config.cache = cache_config;
            }
        } else {
            // No file found, just check for environment overrides
            let config = Config::builder()
                .add_source(
                    Environment::with_prefix("CODEFLOW_BUDDY")
                        .separator("__")
                        .try_parsing(true),
                )
                .build();

            if let Ok(config) = config {
                // Apply environment overrides selectively
                if let Ok(port) = config.get::<u16>("server.port") {
                    app_config.server.port = port;
                }
                if let Ok(host) = config.get::<String>("server.host") {
                    app_config.server.host = host;
                }
                if let Ok(level) = config.get::<String>("logging.level") {
                    app_config.logging.level = level;
                }
                if let Ok(enabled) = config.get::<bool>("cache.enabled") {
                    app_config.cache.enabled = enabled;
                }
            }
        }

        // Validate configuration
        app_config.validate()?;

        Ok(app_config)
    }

    /// Validate the configuration
    fn validate(&self) -> CoreResult<()> {
        // Validate server config
        if self.server.port == 0 {
            return Err(CoreError::config("Server port cannot be 0"));
        }

        if self.server.timeout_ms == 0 {
            return Err(CoreError::config("Server timeout cannot be 0"));
        }

        // Validate LSP config
        if self.lsp.servers.is_empty() {
            return Err(CoreError::config("At least one LSP server must be configured"));
        }

        for server in &self.lsp.servers {
            if server.extensions.is_empty() {
                return Err(CoreError::config("LSP server must handle at least one extension"));
            }
            if server.command.is_empty() {
                return Err(CoreError::config("LSP server command cannot be empty"));
            }
        }

        // Validate logging config
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(CoreError::config(format!(
                "Invalid log level '{}', must be one of: {}",
                self.logging.level,
                valid_levels.join(", ")
            )));
        }

        let valid_formats = ["json", "pretty"];
        if !valid_formats.contains(&self.logging.format.as_str()) {
            return Err(CoreError::config(format!(
                "Invalid log format '{}', must be one of: {}",
                self.logging.format,
                valid_formats.join(", ")
            )));
        }

        // Validate cache config
        if self.cache.enabled && self.cache.max_size_bytes == 0 {
            return Err(CoreError::config("Cache max size cannot be 0 when cache is enabled"));
        }

        Ok(())
    }
}