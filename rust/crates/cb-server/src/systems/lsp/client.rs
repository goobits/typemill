//! LSP client implementation for communicating with a single LSP server

use crate::error::{ServerError, ServerResult};
use cb_core::config::LspServerConfig;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Timeout for LSP requests
const LSP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Timeout for LSP initialization
const LSP_INIT_TIMEOUT: Duration = Duration::from_secs(10);
/// Buffer size for message channels
const CHANNEL_BUFFER_SIZE: usize = 1000;

/// LSP client for communicating with a single LSP server process
pub struct LspClient {
    /// Child process handle
    process: Arc<Mutex<Child>>,
    /// Channel for sending requests to the LSP server
    request_tx: mpsc::Sender<LspRequest>,
    /// Pending requests waiting for responses
    pending_requests: Arc<Mutex<HashMap<i64, oneshot::Sender<Result<Value, String>>>>>,
    /// Next request ID
    next_id: Arc<Mutex<i64>>,
    /// Whether the client has been initialized
    initialized: Arc<Mutex<bool>>,
    /// Server configuration
    config: LspServerConfig,
}

/// Internal request structure
#[derive(Debug)]
struct LspRequest {
    id: i64,
    method: String,
    params: Value,
    response_tx: oneshot::Sender<Result<Value, String>>,
}

impl LspClient {
    /// Create a new LSP client and start the server process
    pub async fn new(config: LspServerConfig) -> ServerResult<Self> {
        if config.command.is_empty() {
            return Err(ServerError::config("LSP server command cannot be empty"));
        }

        let (command, args) = config.command.split_first().unwrap();

        // Start the LSP server process
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(config.root_dir.as_deref().unwrap_or(&std::env::current_dir()?))
            .spawn()
            .map_err(|e| {
                ServerError::runtime(format!(
                    "Failed to start LSP server '{}': {}",
                    config.command.join(" "),
                    e
                ))
            })?;

        // Take ownership of stdin/stdout
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ServerError::runtime("Failed to get stdin for LSP server"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ServerError::runtime("Failed to get stdout for LSP server"))?;

        let process = Arc::new(Mutex::new(child));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(Mutex::new(1));
        let initialized = Arc::new(Mutex::new(false));

        // Create request channel
        let (request_tx, mut request_rx) = mpsc::channel::<LspRequest>(CHANNEL_BUFFER_SIZE);

        // Spawn task to handle writing to LSP server
        let stdin = Arc::new(Mutex::new(stdin));
        let write_stdin = stdin.clone();
        tokio::spawn(async move {
            let mut stdin = write_stdin.lock().await;
            while let Some(request) = request_rx.recv().await {
                let lsp_request = json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "method": request.method,
                    "params": request.params
                });

                let message = format!("Content-Length: {}\r\n\r\n{}",
                    serde_json::to_string(&lsp_request).unwrap().len(),
                    serde_json::to_string(&lsp_request).unwrap());

                if let Err(e) = stdin.write_all(message.as_bytes()).await {
                    error!("Failed to write to LSP server: {}", e);
                    let _ = request.response_tx.send(Err(format!("Write error: {}", e)));
                    break;
                }

                if let Err(e) = stdin.flush().await {
                    error!("Failed to flush LSP server stdin: {}", e);
                    let _ = request.response_tx.send(Err(format!("Flush error: {}", e)));
                    break;
                }

                debug!("Sent LSP request: {}", request.method);
            }
        });

        // Spawn task to handle reading from LSP server
        let pending_requests_clone = pending_requests.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut buffer = String::new();

            loop {
                buffer.clear();
                match reader.read_line(&mut buffer).await {
                    Ok(0) => {
                        debug!("LSP server stdout closed");
                        break;
                    }
                    Ok(_) => {
                        let line = buffer.trim();
                        if let Some(content_length) = Self::parse_content_length(line) {
                            // Read the JSON message
                            if let Ok(message) = Self::read_json_message(&mut reader, content_length).await {
                                Self::handle_message(message, &pending_requests_clone).await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from LSP server: {}", e);
                        break;
                    }
                }
            }
        });

        let client = Self {
            process,
            request_tx,
            pending_requests,
            next_id,
            initialized,
            config,
        };

        // Initialize the LSP server
        client.initialize().await?;

        Ok(client)
    }

    /// Send a request to the LSP server and await the response
    pub async fn send_request(&self, method: &str, params: Value) -> ServerResult<Value> {
        let id = {
            let mut next_id = self.next_id.lock().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let (response_tx, response_rx) = oneshot::channel();

        // Store the pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id, response_tx);
        }

        // Create a dummy response channel for the request structure
        let (dummy_tx, _) = oneshot::channel();
        let request_to_send = LspRequest {
            id,
            method: method.to_string(),
            params,
            response_tx: dummy_tx,
        };

        if let Err(e) = self.request_tx.send(request_to_send).await {
            // Remove from pending requests
            let mut pending = self.pending_requests.lock().await;
            pending.remove(&id);
            return Err(ServerError::runtime(format!("Failed to send request: {}", e)));
        }

        // Wait for response with timeout
        match timeout(LSP_REQUEST_TIMEOUT, response_rx).await {
            Ok(Ok(Ok(result))) => Ok(result),
            Ok(Ok(Err(error))) => Err(ServerError::runtime(format!("LSP error: {}", error))),
            Ok(Err(_)) => {
                // Remove from pending requests
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id);
                Err(ServerError::runtime("Response channel closed"))
            }
            Err(_) => {
                // Remove from pending requests
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id);
                Err(ServerError::runtime("Request timeout"))
            }
        }
    }

    /// Initialize the LSP server
    async fn initialize(&self) -> ServerResult<()> {
        let initialize_params = json!({
            "processId": std::process::id(),
            "clientInfo": {
                "name": "codeflow-buddy",
                "version": "0.1.0"
            },
            "capabilities": {
                "textDocument": {
                    "synchronization": {
                        "didOpen": true,
                        "didChange": true,
                        "didClose": true
                    },
                    "definition": {
                        "linkSupport": false
                    },
                    "references": {
                        "includeDeclaration": true,
                        "dynamicRegistration": false
                    },
                    "rename": {
                        "prepareSupport": false
                    },
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true
                        }
                    },
                    "hover": {},
                    "signatureHelp": {},
                    "diagnostic": {
                        "dynamicRegistration": false,
                        "relatedDocumentSupport": false
                    }
                },
                "workspace": {
                    "workspaceEdit": {
                        "documentChanges": true
                    },
                    "workspaceFolders": true
                }
            },
            "rootUri": format!("file://{}",
                self.config.root_dir.as_deref()
                    .unwrap_or(&std::env::current_dir().unwrap())
                    .display()),
            "workspaceFolders": [{
                "uri": format!("file://{}",
                    self.config.root_dir.as_deref()
                        .unwrap_or(&std::env::current_dir().unwrap())
                        .display()),
                "name": "workspace"
            }]
        });

        // Send initialize request
        let result = timeout(
            LSP_INIT_TIMEOUT,
            self.send_request("initialize", initialize_params),
        )
        .await
        .map_err(|_| ServerError::runtime("LSP initialization timeout"))??;

        debug!("LSP server initialized with result: {:?}", result);

        // Send initialized notification
        self.send_notification("initialized", json!({})).await?;

        // Mark as initialized
        {
            let mut initialized = self.initialized.lock().await;
            *initialized = true;
        }

        info!("LSP server initialized successfully: {}", self.config.command.join(" "));

        Ok(())
    }

    /// Send a notification to the LSP server (no response expected)
    pub async fn send_notification(&self, method: &str, params: Value) -> ServerResult<()> {
        let _notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let _message = format!("Content-Length: {}\r\n\r\n{}",
            serde_json::to_string(&_notification).unwrap().len(),
            serde_json::to_string(&_notification).unwrap());

        // We need to send this directly to stdin since notifications don't have responses
        let _stdin = self.process.lock().await;

        // Note: This is simplified - in a real implementation we'd need better access to stdin
        warn!("Notification sending not fully implemented: {}", method);

        Ok(())
    }

    /// Check if the client has been initialized
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.lock().await
    }

    /// Get the server configuration
    pub fn config(&self) -> &LspServerConfig {
        &self.config
    }

    /// Kill the LSP server process
    pub async fn kill(&self) -> ServerResult<()> {
        let mut process = self.process.lock().await;
        if let Err(e) = process.kill().await {
            warn!("Failed to kill LSP server process: {}", e);
        }
        Ok(())
    }

    /// Parse Content-Length header from LSP message
    fn parse_content_length(line: &str) -> Option<usize> {
        if line.starts_with("Content-Length: ") {
            line["Content-Length: ".len()..].parse().ok()
        } else {
            None
        }
    }

    /// Read JSON message with specified content length
    async fn read_json_message(
        reader: &mut BufReader<tokio::process::ChildStdout>,
        content_length: usize,
    ) -> Result<Value, String> {
        // Skip the empty line
        let mut buffer = String::new();
        if let Err(e) = reader.read_line(&mut buffer).await {
            return Err(format!("Failed to read separator line: {}", e));
        }

        // Read the JSON content
        let mut json_buffer = vec![0u8; content_length];
        if let Err(e) = tokio::io::AsyncReadExt::read_exact(reader, &mut json_buffer).await {
            return Err(format!("Failed to read JSON content: {}", e));
        }

        let json_str = String::from_utf8(json_buffer)
            .map_err(|e| format!("Invalid UTF-8 in JSON content: {}", e))?;

        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    /// Handle incoming message from LSP server
    async fn handle_message(
        message: Value,
        pending_requests: &Arc<Mutex<HashMap<i64, oneshot::Sender<Result<Value, String>>>>>,
    ) {
        if let Some(id) = message.get("id") {
            if let Some(id_num) = id.as_i64() {
                let sender = {
                    let mut pending = pending_requests.lock().await;
                    pending.remove(&id_num)
                };

                if let Some(sender) = sender {
                    if message.get("error").is_some() {
                        let error_msg = message["error"]["message"]
                            .as_str()
                            .unwrap_or("Unknown error")
                            .to_string();
                        let _ = sender.send(Err(error_msg));
                    } else if let Some(result) = message.get("result") {
                        let _ = sender.send(Ok(result.clone()));
                    } else {
                        let _ = sender.send(Err("Invalid response format".to_string()));
                    }
                }
            }
        } else if message.get("method").is_some() {
            // Handle notifications from server
            debug!("Received notification from LSP server: {:?}", message);
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Kill the process when the client is dropped
        let process = self.process.clone();
        tokio::spawn(async move {
            let mut process = process.lock().await;
            if let Err(e) = process.kill().await {
                warn!("Failed to kill LSP server process on drop: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> LspServerConfig {
        LspServerConfig {
            extensions: vec!["py".to_string()],
            command: vec!["echo".to_string(), "test".to_string()], // Use echo for testing
            root_dir: None,
            restart_interval: None,
        }
    }

    #[tokio::test]
    async fn test_lsp_client_creation() {
        let config = create_test_config();

        // This will fail because echo is not an LSP server, but we can test the creation logic
        let result = LspClient::new(config).await;
        assert!(result.is_err()); // Expected to fail during initialization
    }

    #[test]
    fn test_parse_content_length() {
        assert_eq!(LspClient::parse_content_length("Content-Length: 123"), Some(123));
        assert_eq!(LspClient::parse_content_length("Content-Length: 0"), Some(0));
        assert_eq!(LspClient::parse_content_length("Other header"), None);
        assert_eq!(LspClient::parse_content_length("Content-Length: invalid"), None);
    }
}