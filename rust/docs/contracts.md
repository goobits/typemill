# Crate API Contracts

## Overview
This document defines the public API contracts for each crate in the Rust workspace. These contracts ensure compatibility and proper integration between crates.

## cb-core

### Purpose
Foundation crate providing shared types, configuration, and error handling.

### Public API

```rust
// Configuration
pub struct AppConfig {
    pub server: ServerConfig,
    pub lsp: LspConfig,
    pub fuse: Option<FuseConfig>,
    pub logging: LoggingConfig,
    pub cache: CacheConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, CoreError>;
    pub fn validate(&self) -> Result<(), CoreError>;
}

// Error Handling
pub enum CoreError {
    Config { message: String },
    Io { source: std::io::Error },
    Serialization { source: serde_json::Error },
    Internal { message: String },
    Validation { message: String },
}

impl CoreError {
    pub fn config(msg: impl Into<String>) -> Self;
    pub fn internal(msg: impl Into<String>) -> Self;
    pub fn validation(msg: impl Into<String>) -> Self;
}

// MCP Models
pub enum McpMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
}

pub struct McpRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub struct ToolCall {
    pub name: String,
    pub arguments: Option<Value>,
}

// LSP Models
pub struct LspRequest {
    pub id: MessageId,
    pub method: String,
    pub params: Option<Value>,
}

pub struct LspResponse {
    pub id: MessageId,
    pub result: Option<Value>,
    pub error: Option<LspError>,
}

// Intent Models
pub struct IntentSpec {
    pub name: String,
    pub arguments: serde_json::Value,
    pub metadata: Option<IntentMetadata>,
}

pub struct IntentMetadata {
    pub source: String,
    pub correlation_id: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub priority: Option<u8>,
    pub context: HashMap<String, serde_json::Value>,
}
```

### Dependencies
- serde: Serialization
- serde_json: JSON handling
- thiserror: Error derivation
- chrono: Timestamps

## cb-ast

### Purpose
AST parsing, analysis, and transformation for code intelligence.

### Public API

```rust
// Parser
pub fn build_import_graph(source: &str, path: &Path) -> Result<ImportGraph, AstError>;

// Analyzer
pub struct ImportGraph {
    pub source_file: String,
    pub imports: Vec<ImportInfo>,
    pub importers: Vec<String>,
    pub metadata: ImportGraphMetadata,
}

// Note: Higher-level analysis methods are provided via the DependencyGraph struct
pub struct DependencyGraph {
    pub fn get_importers(&self, file_path: &str) -> Vec<String>;
    pub fn get_imports(&self, file_path: &str) -> Vec<String>;
    pub fn has_dependency_path(&self, from: &str, to: &str) -> bool;
}

pub fn build_dependency_graph(import_graphs: &[ImportGraph]) -> DependencyGraph;

// Transformer
pub fn plan_refactor(intent: &IntentSpec, source: &str) -> Result<EditPlan, AstError>;

pub struct EditPlan {
    pub file_path: String,
    pub edits: Vec<TextEdit>,
    pub metadata: RefactorMetadata,
}

impl EditPlan {
    pub fn apply(&self, source: &str) -> Result<String, AstError>;
    pub fn preview(&self) -> Vec<EditPreview>;
}

// Error Handling
pub enum AstError {
    ParseError(String),
    AnalysisError(String),
    TransformError(String),
    Core(CoreError),
}
```

### Dependencies
- cb-core: Core types
- swc_ecma_parser: TypeScript/JavaScript parsing
- swc_ecma_ast: AST types
- petgraph: Graph algorithms
- regex: Pattern matching

## cb-server

### Purpose
MCP server implementation with tool handlers and transport layers.

### Public API

```rust
// Bootstrap
pub async fn bootstrap(options: ServerOptions) -> Result<ServerHandle, ServerError>;

pub struct ServerOptions {
    pub config: AppConfig,
    pub debug: bool,
}

pub struct ServerHandle {
    pub async fn start(&self) -> Result<(), ServerError>;
    pub async fn shutdown(self) -> Result<(), ServerError>;
}

// MCP Dispatcher
pub struct McpDispatcher {
    pub fn new() -> Self;
    pub fn register_tool<F>(&mut self, name: String, handler: F);
    pub async fn dispatch(&self, message: McpMessage) -> Result<McpMessage, ServerError>;
}

// Tool Registration
pub fn register_all_tools(dispatcher: &mut McpDispatcher);

// Service Traits
#[async_trait]
pub trait AstService {
    async fn analyze(&self, source: &str, path: &Path) -> Result<ImportGraph, ServerError>;
    async fn refactor(&self, intent: &IntentSpec, source: &str) -> Result<EditPlan, ServerError>;
}

#[async_trait]
pub trait LspService {
    async fn request(&self, method: &str, params: Value) -> Result<Value, ServerError>;
    async fn get_server(&self, file_ext: &str) -> Result<LspServerHandle, ServerError>;
}

// Error Handling
pub enum ServerError {
    Config { message: String },
    Bootstrap { message: String },
    Runtime { message: String },
    InvalidRequest(String),
    Unsupported(String),
    Core(CoreError),
}
```

### Dependencies
- cb-core: Core types
- cb-ast: AST operations
- tokio: Async runtime
- async-trait: Async traits
- serde_json: JSON handling

## cb-client

### Purpose
CLI client for interacting with the MCP server.

### Public API

```rust
// Entry Point
pub async fn run_cli() -> Result<(), ClientError>;

// Session Management
pub struct SessionReport {
    pub duration: Duration,
    pub requests: u32,
    pub errors: u32,
}

// Commands
pub enum Command {
    Status,
    Connect { server: String },
    Request { tool: String, args: Value },
}

impl Command {
    pub async fn execute(&self, client: &Client) -> Result<CommandResult, ClientError>;
}

// Client
pub struct Client {
    pub fn new(config: ClientConfig) -> Self;
    pub async fn connect(&mut self, endpoint: &str) -> Result<(), ClientError>;
    pub async fn request(&mut self, msg: McpMessage) -> Result<McpMessage, ClientError>;
}

// Error Handling
pub enum ClientError {
    Connection(String),
    Request(String),
    Response(String),
    Core(CoreError),
}
```

### Dependencies
- cb-core: Core types
- clap: CLI parsing
- tokio: Async runtime
- serde_json: JSON handling

## tests

### Purpose
Integration testing and mocks for all crates.

### Public API

```rust
// Test Helpers
pub fn create_test_config() -> AppConfig;
pub fn create_mock_dispatcher() -> McpDispatcher;
pub fn create_test_ast() -> &'static str;

// Mocks
pub struct MockLspService;
pub struct MockAstService;
pub struct MockFileSystem;

// E2E Testing
pub async fn run_e2e_suite() -> TestResults;
pub async fn validate_tool(name: &str, args: Value) -> Result<(), TestError>;
```

### Dependencies
- All crates as dev-dependencies
- tokio-test: Async testing
- tempfile: Temporary files
- serde_json: Test fixtures

## Contract Guarantees

### Semantic Versioning
All crates follow semantic versioning:
- Breaking changes increment major version
- New features increment minor version
- Bug fixes increment patch version

### Backward Compatibility
- Public APIs are stable within major versions
- Deprecated items have at least one minor version warning
- Migration guides provided for breaking changes

### Error Handling
- All errors implement std::error::Error
- Errors are convertible to CoreError
- Error messages are descriptive and actionable

### Thread Safety
- All public types are Send + Sync where appropriate
- Async functions are cancellation-safe
- No global mutable state

### Performance Contracts
- bootstrap(): < 500ms
- dispatch(): < 10ms average
- Memory usage: < 50MB baseline

## Integration Points

### TypeScript Compatibility
- MCP protocol matches TypeScript implementation
- JSON serialization is compatible
- Tool names and parameters match exactly

### LSP Protocol
- Follows LSP 3.17 specification
- Content-Length header handling
- JSON-RPC 2.0 message format

### File System
- UTF-8 encoding for all text files
- Cross-platform path handling
- Atomic file operations where possible

## Testing Requirements

### Unit Tests
- Each public function has tests
- Error conditions are tested
- Edge cases are covered

### Integration Tests
- Cross-crate interactions tested
- E2E scenarios validated
- Performance benchmarks included

### Compatibility Tests
- TypeScript test suite can run against Rust
- Protocol compatibility verified
- Tool output matches TypeScript