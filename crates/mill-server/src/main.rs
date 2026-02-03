//! mill-server main binary

use clap::{Args, Parser, Subcommand};
use mill_config::AppConfig;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "mill-server")]
#[command(about = "TypeMill Server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP server on stdio
    Start,
    /// Start WebSocket server (default)
    Serve,
    /// Generate authentication token
    GenerateToken(GenerateTokenArgs),
}

#[derive(Args)]
struct GenerateTokenArgs {
    /// User ID for the token
    #[arg(long, default_value = "admin")]
    user_id: String,

    /// Optional Project ID
    #[arg(long)]
    project_id: Option<String>,

    /// Token expiry in seconds (optional, defaults to config)
    #[arg(long)]
    expiry_seconds: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments first
    let cli = Cli::parse();

    // Load configuration
    let config = Arc::new(AppConfig::load()?);

    // Initialize tracing based on configuration
    mill_config::logging::initialize(&config);

    // Handle token generation early to avoid starting server components
    if let Some(Commands::GenerateToken(args)) = &cli.command {
        let auth_config = config.server.auth.as_ref().ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Authentication is not configured in mill.toml or environment",
            )) as Box<dyn std::error::Error>
        })?;

        let expiry_seconds = args.expiry_seconds.unwrap_or(auth_config.jwt_expiry_seconds);

        let token = mill_auth::generate_token(
            &auth_config.jwt_secret,
            expiry_seconds,
            &auth_config.jwt_issuer,
            &auth_config.jwt_audience,
            args.project_id.clone(),
            Some(args.user_id.clone()),
        )?;

        println!("{}", token);
        return Ok(());
    }

    tracing::info!("Starting TypeMill Server");

    // Create workspace manager for tracking connected containers
    let workspace_manager = Arc::new(mill_workspaces::WorkspaceManager::new());

    // Build plugin registry using the application-layer bundle
    let all_plugins = mill_plugin_bundle::all_plugins();
    tracing::info!(
        discovered_plugins_count = all_plugins.len(),
        "Discovered language plugins from bundle"
    );
    let mut plugin_registry = mill_plugin_api::PluginDiscovery::new();
    for plugin in all_plugins {
        plugin_registry.register(plugin);
    }
    let plugin_registry = Arc::new(plugin_registry);

    // Create dispatcher using shared library function (reduces duplication)
    let dispatcher = mill_server::create_dispatcher_with_workspace(
        config.clone(),
        workspace_manager,
        plugin_registry,
    )
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Execute based on command
    match cli.command {
        Some(Commands::Start) => {
            // Start stdio MCP server
            tracing::info!("Starting stdio MCP server");
            if let Err(e) = mill_transport::start_stdio_server(dispatcher).await {
                tracing::error!(
                    error_category = "transport_error",
                    error = %e,
                    "Failed to start stdio server"
                );
                return Err(e);
            }
        }
        Some(Commands::GenerateToken(_)) => unreachable!("Handled early"),
        Some(Commands::Serve) | None => {
            // Start admin server on a separate port
            let admin_port = config.server.port + 1000; // Admin on port+1000
            let admin_config = config.clone();
            let admin_workspace_manager =
                Arc::new(mill_server::workspaces::WorkspaceManager::new());
            tokio::spawn(async move {
                if let Err(e) = mill_transport::start_admin_server(
                    admin_port,
                    admin_config,
                    admin_workspace_manager,
                )
                .await
                {
                    tracing::error!(
                        error_category = "admin_server_error",
                        error = %e,
                        admin_port = admin_port,
                        "Failed to start admin server"
                    );
                }
            });

            // Start WebSocket server (default)
            tracing::info!(
                "Starting WebSocket server on {}:{}",
                config.server.host,
                config.server.port
            );
            tracing::info!("Admin endpoints available on 127.0.0.1:{}", admin_port);

            if let Err(e) = mill_transport::start_ws_server(config, dispatcher).await {
                tracing::error!(
                    error_category = "transport_error",
                    error = %e,
                    "Failed to start WebSocket server"
                );
                return Err(e.into());
            }
        }
    }

    tracing::info!("Server stopped");
    Ok(())
}
