# Architecture Documentation

## Overview

The Rust MCP Server is a production-ready implementation that bridges the Model Context Protocol (MCP) with Language Server Protocol (LSP) functionality. The architecture follows a layered design with clear separation of concerns, enabling both WebSocket and stdio transports while providing comprehensive code intelligence tools.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Transport Layer                          │
├─────────────────────┬───────────────────────────────────────────┤
│   WebSocket Server  │              Stdio Server                 │
│   (Production)      │              (MCP Clients)                │
│   Port 3040         │              stdin/stdout                 │
└─────────────────────┴───────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                     MCP Dispatcher                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   Navigation    │  │  Intelligence   │  │   Filesystem    │  │
│  │     Tools       │  │     Tools       │  │     Tools       │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│  ┌─────────────────┐  ┌─────────────────┐                       │
│  │    Editing      │  │    Analysis     │                       │
│  │     Tools       │  │     Tools       │                       │
│  └─────────────────┘  └─────────────────┘                       │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      App State                                  │
│                                                                 │
│  ┌─────────────────┐              ┌─────────────────┐           │
│  │   LSP Manager   │◄────────────►│   AST Service   │           │
│  │                 │              │                 │           │
│  │ ┌─────────────┐ │              │ ┌─────────────┐ │           │
│  │ │TypeScript LS│ │              │ │   Parser    │ │           │
│  │ │Python LSP   │ │              │ │  Analyzer   │ │           │
│  │ │Rust Analyzer│ │              │ │Transformer  │ │           │
│  │ │   Clangd    │ │              │ └─────────────┘ │           │
│  │ │    Gopls    │ │              └─────────────────┘           │
│  │ └─────────────┘ │                                            │
│  └─────────────────┘                                            │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Subsystems                                    │
│                                                                 │
│  ┌─────────────────┐              ┌─────────────────┐           │
│  │ FUSE Filesystem │              │  Configuration  │           │
│  │                 │              │    Management   │           │
│  │ • Virtual FS    │              │                 │           │
│  │ • 1s TTL Cache  │              │ • JSON Config   │           │
│  │ • Inode Mgmt    │              │ • LSP Servers   │           │
│  │ • Background    │              │ • FUSE Options  │           │
│  │   Mounting      │              │ • Logging       │           │
│  └─────────────────┘              └─────────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

## Request Lifecycle

### MCP Request Flow (Stdio Transport)

1. **Request Reception**
   ```
   stdin → BufReader → JSON parsing → McpMessage::Request
   ```

2. **Dispatch Processing**
   ```
   McpDispatcher::dispatch() → Tool lookup → Handler execution
   ```

3. **Tool Execution**
   ```
   Tool handler → AppState services → LSP/AST operations
   ```

4. **Response Generation**
   ```
   Tool result → MCP response → JSON serialization → stdout
   ```

### WebSocket Request Flow

1. **Connection Management**
   ```
   WebSocket connection → Session creation → Initialize handshake
   ```

2. **Message Processing**
   ```
   WebSocket frame → JSON parsing → MCP dispatch → Response frame
   ```

3. **Session State**
   ```
   Connection pooling → Concurrent request handling → Cleanup
   ```

## Component Interactions

### MCP Dispatcher

The central orchestrator that:
- Registers all 21 MCP tools at startup
- Routes incoming tool calls to appropriate handlers
- Provides dependency injection for services (LSP, AST)
- Handles error conversion and response formatting

```rust
pub struct McpDispatcher {
    tools: HashMap<String, Box<dyn ToolHandler>>,
    app_state: Arc<AppState>,
}

impl McpDispatcher {
    pub async fn dispatch(&self, message: McpMessage) -> Result<McpMessage, ServerError> {
        match message {
            McpMessage::Request(req) => {
                let tool_name = extract_tool_name(&req)?;
                let handler = self.tools.get(&tool_name)?;
                let result = handler.execute(&req, &self.app_state).await?;
                Ok(McpMessage::Response(result))
            }
        }
    }
}
```

### App State

Provides shared services to all tool handlers:

```rust
pub struct AppState {
    pub lsp: Arc<LspManager>,
    // Future: AST service, cache, metrics
}
```

### LSP Manager

Manages multiple Language Server Protocol clients:

```rust
pub struct LspManager {
    clients: HashMap<String, Arc<LspClient>>,
    config: LspConfig,
}

impl LspManager {
    pub async fn get_client(&self, extension: &str) -> Result<Arc<LspClient>, CoreError> {
        // Find appropriate LSP server for file extension
        // Start server if not running
        // Return client handle
    }
}
```

### LSP Client

Individual LSP server process manager:

```rust
pub struct LspClient {
    process: Child,
    stdin: ChildStdin,
    stdout_receiver: Receiver<LspResponse>,
    request_id: AtomicU64,
}

impl LspClient {
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, CoreError> {
        // Generate unique request ID
        // Send JSON-RPC request to LSP server
        // Wait for correlated response
        // Handle timeouts and errors
    }
}
```

## Tool Handler Architecture

### Handler Trait

All tools implement a common interface:

```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(
        &self,
        request: &McpRequest,
        app_state: &AppState,
    ) -> Result<McpResponse, ServerError>;
}
```

### Tool Categories

1. **Navigation Tools**
   - Direct LSP integration
   - Symbol resolution
   - Cross-reference analysis

2. **Intelligence Tools**
   - Hover information
   - Code completions
   - Signature help
   - Diagnostics

3. **Editing Tools**
   - Symbol renaming
   - Code formatting
   - Code actions
   - Workspace edits

4. **Filesystem Tools**
   - Direct file system operations
   - No LSP dependency
   - Cross-platform path handling

5. **Analysis Tools**
   - cb-ast integration for import analysis
   - LSP-based dead code detection
   - System health monitoring

## FUSE Subsystem

### Filesystem Implementation

```rust
pub struct CodeflowFS {
    workspace_path: PathBuf,
    attr_cache: HashMap<u64, FileAttr>,
    next_inode: u64,
    inode_to_path: HashMap<u64, PathBuf>,
    path_to_inode: HashMap<PathBuf, u64>,
}

impl Filesystem for CodeflowFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // Resolve path from parent inode + name
        // Generate or retrieve inode
        // Return file attributes with TTL
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        // Look up path from inode
        // Get file metadata
        // Cache and return attributes
    }
}
```

### FUSE Integration

- **Background mounting**: Runs in dedicated thread to avoid blocking main server
- **TTL-based caching**: 1-second cache for metadata to balance performance and consistency
- **Read-only access**: Prevents accidental modifications through FUSE mount
- **Graceful failure**: Server continues operation even if FUSE mount fails

## Error Handling Strategy

### Error Types Hierarchy

```rust
CoreError → ServerError → Transport-specific errors
```

### Error Propagation

1. **LSP Errors**: Wrapped and propagated as JSON-RPC errors
2. **File System Errors**: Converted to appropriate HTTP status codes
3. **Protocol Errors**: Return proper MCP error responses
4. **Configuration Errors**: Fail fast during startup

### Error Recovery

- **LSP server crashes**: Graceful error responses, no automatic restart
- **File system errors**: Per-operation error handling
- **Network errors**: Connection-level retry logic
- **Parse errors**: Detailed error messages with context

## Configuration Management

### Configuration Sources

1. **Primary**: `.codebuddy/config.json` in working directory
2. **Fallback**: Embedded default configuration
3. **Validation**: Comprehensive validation with descriptive errors

### Configuration Schema

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 3040
  },
  "lsp": {
    "servers": [
      {
        "name": "typescript",
        "command": ["typescript-language-server", "--stdio"],
        "extensions": ["ts", "tsx", "js", "jsx"],
        "timeout": 30
      }
    ]
  },
  "fuse": {
    "enabled": true,
    "mount_point": "/tmp/codeflow-workspace"
  }
}
```

## Threading Model

### Async Runtime

- **Tokio runtime**: Single-threaded async executor for main server
- **Background threads**: FUSE mounting, LSP process management
- **Concurrent operations**: Multiple MCP requests handled concurrently

### Synchronization

- **Arc<T>**: Shared immutable data across threads
- **Mutex<T>**: Mutable shared state (minimized)
- **Channel communication**: LSP process communication

## Performance Characteristics

### Memory Usage

- **Baseline**: ~50MB for server core
- **LSP overhead**: Variable per language server
- **FUSE caching**: Bounded by TTL and working set size

### Response Times

- **File operations**: < 100ms typical
- **LSP operations**: < 5 seconds (LSP-dependent)
- **FUSE operations**: < 50ms (cached)

### Scalability

- **Concurrent connections**: WebSocket server supports multiple clients
- **LSP multiplexing**: Single LSP server handles multiple requests
- **Resource limits**: Configurable timeouts and request limits

## Security Considerations

### Input Validation

- **JSON schema validation**: All MCP requests validated
- **Path traversal prevention**: File operations restricted to workspace
- **Command injection prevention**: LSP commands from configuration only

### Process Isolation

- **LSP servers**: Run as separate processes
- **FUSE filesystem**: Read-only access
- **Network binding**: Localhost only by default

### Error Information

- **Error messages**: No sensitive information leaked
- **Stack traces**: Debug mode only
- **File paths**: Normalized and validated

## Testing Architecture

### Test Levels

1. **Unit Tests**: Individual component testing
2. **Contract Tests**: MCP protocol validation
3. **Integration Tests**: Cross-component interaction
4. **E2E Tests**: Full client-server scenarios

### Test Infrastructure

- **Mocking**: LSP services, file system operations
- **Test fixtures**: Realistic code samples
- **Contract validation**: JSON schema compliance
- **Performance testing**: Response time measurement

This architecture provides a robust, scalable foundation for bridging MCP and LSP protocols while maintaining excellent performance and reliability characteristics.