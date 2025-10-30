mod cli;
mod dispatcher_factory;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use mill_server::handlers::plugin_dispatcher::PluginDispatcher;
use mill_server::workspaces::WorkspaceManager;
use mill_transport::SessionInfo;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

fn warn_if_fuse_enabled() {
    if let Ok(config) = mill_config::config::AppConfig::load() {
        if config.fuse.is_some() {
            eprintln!("⚠️  FUSE configured - requires SYS_ADMIN capability");
            eprintln!("   Only use in trusted development environments");
            eprintln!("   Set \"fuse\": null in config for production");
        }
    }
}

#[tokio::main]
async fn main() {
    warn_if_fuse_enabled();
    cli::run().await;
}

/// Runs the application in stdio mode.
///
/// This mode is used when the application is run from the command line. It
/// reads messages from stdin and writes responses to stdout.
pub async fn run_stdio_mode() {
    debug!("Initializing stdio mode MCP server");
    debug!(
        "Current working directory in run_stdio_mode: {:?}",
        std::env::current_dir()
    );

    // Initialize dispatcher via factory
    let dispatcher = match dispatcher_factory::create_initialized_dispatcher().await {
        Ok(d) => d,
        Err(e) => {
            error!(error = %e, "Failed to initialize dispatcher");
            return;
        }
    };
    let session_info = SessionInfo::default();
    debug!("Plugin dispatcher initialized successfully");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);

    debug!("Starting stdio message loop");
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("EOF received, exiting");
                break; // EOF
            }
            Ok(_) => {
                let trimmed = line.trim();

                // Skip frame delimiter used by test harness
                if trimmed == "---FRAME---" {
                    debug!("Skipping frame delimiter");
                    continue;
                }

                // Skip empty lines
                if trimmed.is_empty() {
                    continue;
                }

                debug!(message = %trimmed, "Received message");

                // First parse as raw JSON to extract ID for error responses
                let raw_message: serde_json::Value = match serde_json::from_str(trimmed) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(error = %e, line = %trimmed, "Failed to parse JSON");
                        continue;
                    }
                };
                let request_id = raw_message.get("id").cloned();

                // Then parse as proper MCP message
                match serde_json::from_value(raw_message.clone()) {
                    Ok(mcp_message) => {
                        debug!("Parsed MCP message, dispatching");

                        match dispatcher.dispatch(mcp_message, &session_info).await {
                            Ok(response) => {
                                let response_json = match serde_json::to_string(&response) {
                                    Ok(json) => json,
                                    Err(e) => {
                                        error!(error = %e, "Failed to serialize response");
                                        continue;
                                    }
                                };
                                debug!(response = %response_json, "Sending response");
                                if let Err(e) = stdout.write_all(response_json.as_bytes()).await {
                                    error!(error = %e, "Error writing to stdout");
                                    break;
                                }
                                if let Err(e) = stdout.write_all(b"\n").await {
                                    error!(error = %e, "Error writing newline");
                                    break;
                                }
                                // Send frame delimiter for test harness compatibility
                                if let Err(e) = stdout.write_all(b"---FRAME---\n").await {
                                    error!(error = %e, "Error writing frame delimiter");
                                    break;
                                }
                                if let Err(e) = stdout.flush().await {
                                    error!(error = %e, "Error flushing stdout");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "Error dispatching message");

                                // Send JSON-RPC error response back to client
                                let error_response = json!({
                                    "jsonrpc": "2.0",
                                    "id": request_id,
                                    "error": {
                                        "code": -32603,
                                        "message": format!("{}", e)
                                    }
                                });

                                if let Ok(response_json) = serde_json::to_string(&error_response) {
                                    let _ = stdout.write_all(response_json.as_bytes()).await;
                                    let _ = stdout.write_all(b"\n").await;
                                    let _ = stdout.write_all(b"---FRAME---\n").await;
                                    let _ = stdout.flush().await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, line = %trimmed, "Failed to parse MCP message");

                        // Send JSON-RPC error response for parse errors
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "error": {
                                "code": -32700,
                                "message": format!("Parse error: {}", e)
                            }
                        });

                        if let Ok(response_json) = serde_json::to_string(&error_response) {
                            let _ = stdout.write_all(response_json.as_bytes()).await;
                            let _ = stdout.write_all(b"\n").await;
                            let _ = stdout.write_all(b"---FRAME---\n").await;
                            let _ = stdout.flush().await;
                        }
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Error reading from stdin");
                break;
            }
        }
    }
    debug!("Stdio mode exiting");
}

/// Runs the application in websocket mode.
///
/// This mode is used when the application is run as a server. It listens for
/// websocket connections on port 3000 and handles messages from clients.
pub async fn run_websocket_server() {
    run_websocket_server_with_port(3000).await;
}

/// Handler for the homepage
async fn homepage_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TypeMill MCP Server</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            line-height: 1.6;
            color: #333;
        }
        h1 { color: #2c3e50; }
        .status { color: #27ae60; font-weight: bold; }
        .endpoint {
            background: #f4f4f4;
            padding: 10px;
            margin: 10px 0;
            border-left: 4px solid #3498db;
            font-family: 'Courier New', monospace;
        }
        .section { margin: 30px 0; }
        a { color: #3498db; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>TypeMill MCP Server</h1>
    <p class="status">✓ Server is running</p>

    <div class="section">
        <h2>WebSocket Endpoint</h2>
        <div class="endpoint">ws://YOUR_HOST_IP:3040/ws</div>
        <p>Connect your MCP-compatible AI assistant to this endpoint.</p>
    </div>

    <div class="section">
        <h2>Admin API (Port 4040)</h2>
        <div class="endpoint">GET /health - Health check</div>
        <div class="endpoint">GET /workspaces - List workspaces</div>
        <div class="endpoint">POST /workspaces/register - Register workspace</div>
        <p><a href="http://localhost:4040/health">View Health Status</a></p>
    </div>

    <div class="section">
        <h2>Documentation</h2>
        <p>For complete API documentation and tools reference, see the project README.</p>
    </div>
</body>
</html>"#)
}

/// Runs the application in websocket mode on a specific port.
///
/// This mode is used when the application is run as a server. It listens for
/// websocket connections on the specified port and handles messages from
/// clients.
///
/// # Arguments
///
/// * `port` - The port to listen on.
pub async fn run_websocket_server_with_port(port: u16) {
    // Load configuration
    let config = match mill_config::config::AppConfig::load() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            return;
        }
    };

    // Enforce TLS for non-loopback hosts
    if !config.server.is_loopback_host() {
        if config.server.tls.is_none() {
            error!(
                host = %config.server.host,
                "TLS is required when binding to non-loopback addresses. \
                 Configure server.tls in config or bind to 127.0.0.1"
            );
            eprintln!(
                "❌ ERROR: TLS required for non-loopback host '{}'",
                config.server.host
            );
            eprintln!("   Either:");
            eprintln!("   1. Configure server.tls.cert_path and server.tls.key_path");
            eprintln!("   2. Or bind to 127.0.0.1 (loopback only)");
            return;
        }
        info!(host = %config.server.host, "TLS enabled for non-loopback host");
    } else if config.server.tls.is_none() {
        tracing::warn!(
            host = %config.server.host,
            "Server running without TLS on loopback address. \
             This is acceptable for development but consider enabling TLS for production."
        );
    }

    // Create workspace manager
    let workspace_manager = Arc::new(WorkspaceManager::new());

    // Initialize dispatcher via factory
    let dispatcher = match dispatcher_factory::create_initialized_dispatcher_with_workspace(
        workspace_manager.clone(),
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            error!(error = %e, "Failed to initialize dispatcher");
            return;
        }
    };

    // Start admin server on a separate port
    let admin_port = port + 1000; // Admin on port+1000
    let admin_config = config.clone();
    let admin_workspace_manager = workspace_manager.clone();
    tokio::spawn(async move {
        if let Err(e) =
            mill_transport::start_admin_server(admin_port, admin_config, admin_workspace_manager)
                .await
        {
            error!(
                error_category = "admin_server_error",
                error = %e,
                "Admin server failed"
            );
        }
    });

    let app = Router::new()
        .route("/", get(homepage_handler))
        .route("/ws", get(ws_handler))
        .with_state(dispatcher);

    let bind_addr = format!("{}:{}", config.server.host, port);
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!(bind_addr = %bind_addr, error = %e, "Failed to bind to address");
            return;
        }
    };

    let addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(e) => {
            error!(error = %e, "Failed to get local address");
            return;
        }
    };
    info!(addr = %addr, "Server listening");

    if let Err(e) = axum::serve(listener, app).await {
        error!(error = %e, "Server error");
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(dispatcher): State<Arc<PluginDispatcher>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, dispatcher))
}

async fn handle_socket(mut socket: WebSocket, dispatcher: Arc<PluginDispatcher>) {
    let session_info = SessionInfo::default();
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                let response = match serde_json::from_str(&text) {
                    Ok(mcp_message) => dispatcher.dispatch(mcp_message, &session_info).await,
                    Err(e) => {
                        // Handle deserialization error
                        error!(error = %e, "Failed to deserialize message");
                        continue;
                    }
                };

                match response {
                    Ok(response_message) => {
                        let response_text = match serde_json::to_string(&response_message) {
                            Ok(text) => text,
                            Err(e) => {
                                error!(error = %e, "Failed to serialize response");
                                continue;
                            }
                        };
                        if socket
                            .send(Message::Text(response_text.into()))
                            .await
                            .is_err()
                        {
                            break; // client disconnected
                        }
                    }
                    Err(e) => {
                        // Handle dispatch error
                        error!(error = %e, "Error dispatching message");
                    }
                }
            }
            Some(Ok(Message::Close(_))) | None => {
                break; // client disconnected
            }
            Some(Ok(_)) => {
                // Ignore other message types (binary, ping, pong)
            }
            Some(Err(e)) => {
                error!(error = %e, "WebSocket error");
                break;
            }
        }
    }
}
