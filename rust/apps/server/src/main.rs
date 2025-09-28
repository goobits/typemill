use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use cb_server::handlers::plugin_dispatcher::{AppState, PluginDispatcher};
use cb_server::systems::LspManager;
use cb_core::config::LspConfig;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::info;

#[derive(Parser)]
#[command(name = "codeflow-buddy")]
#[command(about = "Pure Rust MCP server bridging Language Server Protocol functionality")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server in stdio mode for Claude Code
    Start,
    /// Start WebSocket server
    Serve,
    /// Show status
    Status,
    /// Setup configuration
    Setup,
    /// Stop the running server
    Stop,
    /// Link to AI assistants
    Link,
    /// Remove AI from config
    Unlink,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            eprintln!("DEBUG: Starting MCP server in stdio mode");
            run_stdio_mode().await;
        }
        Commands::Serve => {
            eprintln!("DEBUG: Starting WebSocket server");
            run_websocket_server().await;
        }
        Commands::Status => {
            println!("Status: Running");
        }
        Commands::Setup => {
            println!("Setup: Not implemented");
        }
        Commands::Stop => {
            println!("Stop: Not implemented");
        }
        Commands::Link => {
            println!("Link: Not implemented");
        }
        Commands::Unlink => {
            println!("Unlink: Not implemented");
        }
    }
}

async fn run_stdio_mode() {
    eprintln!("DEBUG: Initializing stdio mode MCP server");
    eprintln!("DEBUG: Current working directory in run_stdio_mode: {:?}", std::env::current_dir());

    // Create AppState similar to the test implementation
    let app_state = create_app_state().await;

    let dispatcher = Arc::new(PluginDispatcher::new(app_state));
    eprintln!("DEBUG: About to call dispatcher.initialize()");
    dispatcher.initialize().await.expect("Failed to initialize dispatcher");
    eprintln!("DEBUG: Plugin dispatcher initialized successfully");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);

    eprintln!("DEBUG: Starting stdio message loop");
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                eprintln!("DEBUG: EOF received, exiting");
                break; // EOF
            }
            Ok(_) => {
                eprintln!("DEBUG: Received message: {}", line.trim());
                match serde_json::from_str(&line) {
                    Ok(mcp_message) => {
                        eprintln!("DEBUG: Parsed MCP message, dispatching");
                        match dispatcher.dispatch(mcp_message).await {
                            Ok(response) => {
                                let response_json = serde_json::to_string(&response).unwrap();
                                eprintln!("DEBUG: Sending response: {}", response_json);
                                if let Err(e) = stdout.write_all(response_json.as_bytes()).await {
                                    eprintln!("DEBUG: Error writing to stdout: {}", e);
                                    break;
                                }
                                if let Err(e) = stdout.write_all(b"\n").await {
                                    eprintln!("DEBUG: Error writing newline: {}", e);
                                    break;
                                }
                                if let Err(e) = stdout.flush().await {
                                    eprintln!("DEBUG: Error flushing stdout: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("DEBUG: Error dispatching message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("DEBUG: Failed to parse JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("DEBUG: Error reading from stdin: {}", e);
                break;
            }
        }
    }
    eprintln!("DEBUG: Stdio mode exiting");
}

async fn run_websocket_server() {
    // Create AppState similar to the test implementation
    let app_state = create_app_state().await;

    let dispatcher = Arc::new(PluginDispatcher::new(app_state));
    dispatcher.initialize().await.expect("Failed to initialize dispatcher");

    let app = Router::new().route("/ws", get(ws_handler)).with_state(dispatcher);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn create_app_state() -> Arc<AppState> {
    let lsp_config = LspConfig::default();
    let lsp_manager = Arc::new(LspManager::new(lsp_config));

    // Use current working directory as project root for production
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    eprintln!("DEBUG: Server project_root set to: {}", project_root.display());

    let file_service = Arc::new(cb_server::services::FileService::new(project_root.clone()));
    let lock_manager = Arc::new(cb_server::services::LockManager::new());
    let operation_queue = Arc::new(cb_server::services::OperationQueue::new(lock_manager.clone()));

    Arc::new(AppState {
        lsp: lsp_manager,
        file_service,
        project_root,
        lock_manager,
        operation_queue,
    })
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(dispatcher): State<Arc<PluginDispatcher>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, dispatcher))
}

async fn handle_socket(mut socket: WebSocket, dispatcher: Arc<PluginDispatcher>) {
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                let response = match serde_json::from_str(&text) {
                    Ok(mcp_message) => dispatcher.dispatch(mcp_message).await,
                    Err(e) => {
                        // Handle deserialization error
                        tracing::error!("Failed to deserialize message: {}", e);
                        continue;
                    }
                };

                match response {
                    Ok(response_message) => {
                        let response_text = serde_json::to_string(&response_message).unwrap();
                        if socket.send(Message::Text(response_text.into())).await.is_err() {
                            break; // client disconnected
                        }
                    }
                    Err(e) => {
                        // Handle dispatch error
                        tracing::error!("Error dispatching message: {}", e);
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
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }
}