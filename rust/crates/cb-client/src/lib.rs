//! cb-client: Codeflow Buddy client implementation

pub mod error;
pub mod config;

pub use error::{ClientError, ClientResult};
pub use config::*;

use cb_core::AppConfig;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "cb-client")]
#[command(about = "Codeflow Buddy Client", long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,
}

/// CLI subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Connect to server and run interactive session
    Connect {
        /// Server URL
        #[arg(short, long, default_value = "ws://localhost:3000")]
        url: String,

        /// Authentication token
        #[arg(short, long)]
        token: Option<String>,
    },

    /// Send a single request to server
    Request {
        /// Server URL
        #[arg(short, long, default_value = "ws://localhost:3000")]
        url: String,

        /// MCP method to call
        #[arg(short, long)]
        method: String,

        /// Request parameters (JSON)
        #[arg(short, long)]
        params: Option<String>,

        /// Authentication token
        #[arg(short, long)]
        token: Option<String>,
    },

    /// Show client status and configuration
    Status,
}

/// Session report summarizing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReport {
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub failure_count: usize,
    /// Session duration in milliseconds
    pub duration_ms: u64,
    /// Operations performed
    pub operations: Vec<String>,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Run the CLI application
pub async fn run_cli() -> ClientResult<()> {
    let args = CliArgs::parse();

    if args.debug {
        tracing_subscriber::fmt::init();
        tracing::info!("Debug mode enabled");
    }

    match args.command {
        Commands::Connect { url, token } => {
            connect_to_server(&url, token.as_deref()).await
        }
        Commands::Request { url, method, params, token } => {
            send_request(&url, &method, params.as_deref(), token.as_deref()).await
        }
        Commands::Status => {
            show_status().await
        }
    }
}

/// Connect to server and run interactive session
async fn connect_to_server(url: &str, _token: Option<&str>) -> ClientResult<()> {
    println!("Connecting to server at: {}", url);

    // In a real implementation, this would:
    // 1. Establish WebSocket connection
    // 2. Authenticate if token provided
    // 3. Enter interactive mode
    // 4. Handle user commands
    // 5. Maintain session state

    println!("Interactive session not yet implemented");
    Ok(())
}

/// Send a single request to the server
async fn send_request(
    url: &str,
    method: &str,
    params: Option<&str>,
    _token: Option<&str>,
) -> ClientResult<()> {
    println!("Sending request to: {}", url);
    println!("Method: {}", method);

    if let Some(params) = params {
        println!("Params: {}", params);
    }

    // In a real implementation, this would:
    // 1. Parse parameters as JSON
    // 2. Create MCP request message
    // 3. Send to server and wait for response
    // 4. Display formatted response

    println!("Single request not yet implemented");
    Ok(())
}

/// Show client status and configuration
async fn show_status() -> ClientResult<()> {
    println!("Codeflow Buddy Client Status");
    println!("============================");

    // Load configuration
    match AppConfig::load() {
        Ok(config) => {
            println!("✓ Configuration loaded successfully");
            println!("  Server: {}:{}", config.server.host, config.server.port);
            println!("  LSP servers: {}", config.lsp.servers.len());
            println!("  Cache enabled: {}", config.cache.enabled);

            if let Some(fuse_config) = &config.fuse {
                println!("  FUSE mount: {}", fuse_config.mount_point.display());
            }
        }
        Err(err) => {
            println!("✗ Configuration error: {}", err);
        }
    }

    println!();
    println!("Client version: 0.1.0");
    println!("Protocol version: 2024-11-05");

    Ok(())
}

impl Default for SessionReport {
    fn default() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            duration_ms: 0,
            operations: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl SessionReport {
    /// Create a new empty session report
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a successful operation
    pub fn add_success(&mut self, operation: impl Into<String>) {
        self.success_count += 1;
        self.operations.push(operation.into());
    }

    /// Add a failed operation
    pub fn add_failure(&mut self, operation: impl Into<String>, error: impl Into<String>) {
        self.failure_count += 1;
        self.operations.push(operation.into());
        self.errors.push(error.into());
    }

    /// Set session duration
    pub fn set_duration(&mut self, duration_ms: u64) {
        self.duration_ms = duration_ms;
    }

    /// Get total operation count
    pub fn total_operations(&self) -> usize {
        self.success_count + self.failure_count
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_operations() == 0 {
            100.0
        } else {
            (self.success_count as f64 / self.total_operations() as f64) * 100.0
        }
    }
}