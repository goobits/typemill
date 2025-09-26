//! cb-server main binary

use cb_core::AppConfig;
use cb_server::{bootstrap, ServerOptions};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Codeflow Buddy Server");

    // Load configuration
    let config = AppConfig::load()?;

    // Create server options
    let options = ServerOptions::from_config(config).with_debug(true);

    // Bootstrap the server
    let handle = bootstrap(options).await?;

    // Start the server
    handle.start().await?;

    // In a real implementation, we'd wait for shutdown signals
    // For now, just run briefly and then shutdown
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Shutdown gracefully
    handle.shutdown().await?;

    tracing::info!("Server stopped");
    Ok(())
}