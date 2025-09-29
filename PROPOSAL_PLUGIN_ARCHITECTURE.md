# Plugin Architecture Documentation

## Overview

The Codeflow Buddy MCP server now uses a **plugin-based architecture** that eliminates all hard-coded mappings and provides clean separation between protocol handling and language-specific functionality.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   MCP Client Request                         │
│                  (e.g., "find_definition")                   │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                    PluginDispatcher                          │
│              (Routes based on file extension)                │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                  LspAdapterPlugin                            │
│         (Translates MCP → LSP: "textDocument/definition")    │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                   DirectLspAdapter                           │
│            (Bypasses old manager, goes direct to LSP)        │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                      LspClient                               │
│              (Direct communication with LSP server)          │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. **PluginDispatcher** (`handlers/plugin_dispatcher.rs`)
- Central routing component
- Manages plugin lifecycle
- Routes requests to appropriate language plugin
- Contains `AppState` for service access

### 2. **Language Plugins** (`cb-plugins` crate)
- **TypeScript Plugin**: Handles `.ts`, `.tsx`, `.js`, `.jsx` files
- **Python Plugin**: Handles `.py`, `.pyi` files
- **Go Plugin**: Handles `.go` files
- **Rust Plugin**: Handles `.rs` files

### 3. **DirectLspAdapter** (`handlers/plugin_dispatcher.rs`)
- Bypasses old LSP manager completely
- Creates direct connections to LSP servers
- No hard-coded method mappings
- Manages LSP client lifecycle per language

### 4. **LspClient** (`systems/lsp/client.rs`)
- Low-level LSP protocol communication
- Handles request/response correlation
- Process management for LSP servers

## What Was Removed

### ❌ **Deleted Components**
1. **`mcp_dispatcher.rs`** - Old monolithic dispatcher with 27 hard-coded mappings
2. **`mcp_tools/` directory** - 24+ files with 5000+ lines of obsolete handlers
3. **Hard-coded mappings** in `lsp/manager.rs` lines 101-116
4. **`register_all_tools`** function - Tools now register via plugins

### ❌ **Eliminated Anti-patterns**
- No more hard-coded MCP→LSP method mappings
- No more hard-coded operation type categorization
- No more scattered tool registration
- No more tight coupling between MCP and LSP protocols

## How It Works

### Request Flow

1. **MCP Client** sends request (e.g., `find_definition` for a TypeScript file)
2. **PluginDispatcher** receives request and determines file type
3. **TypeScript Plugin** is selected based on file extension
4. **LspAdapterPlugin** translates MCP method to LSP method (`textDocument/definition`)
5. **DirectLspAdapter** gets/creates TypeScript LSP client
6. **LspClient** sends request directly to TypeScript Language Server
7. Response flows back through the same path

### Key Innovation: DirectLspAdapter

The `DirectLspAdapter` is the crucial component that makes this architecture work:

```rust
// Old flow (REMOVED):
MCP → McpDispatcher → manager.mcp_to_lsp_request() → LspClient

// New flow (ACTIVE):
MCP → Plugin → DirectLspAdapter → LspClient
```

By bypassing the old manager's hard-coded mappings, we achieve:
- **Dynamic routing** based on plugin capabilities
- **Protocol abstraction** - plugins handle translation
- **Clean separation** - no mixing of concerns
- **Easy extensibility** - add language = add plugin

## Adding a New Language

To add support for a new language:

1. Create a new plugin implementing `LanguagePlugin` trait
2. Register it in `PluginDispatcher::initialize()`
3. Configure LSP server in `.codebuddy/config.json`

Example:
```rust
let ruby_adapter = Arc::new(DirectLspAdapter::new(
    lsp_config.clone(),
    vec!["rb".to_string()],
    "ruby-lsp-direct".to_string(),
));
let ruby_plugin = Arc::new(LspAdapterPlugin::ruby(ruby_adapter));
plugin_manager.register_plugin("ruby", ruby_plugin).await?;
```

## Configuration

LSP servers are configured in `.codebuddy/config.json`:

```json
{
  "servers": [
    {
      "extensions": ["ts", "tsx", "js", "jsx"],
      "command": ["typescript-language-server", "--stdio"]
    },
    {
      "extensions": ["py"],
      "command": ["pylsp"]
    }
  ]
}
```

## Benefits of Plugin Architecture

### ✅ **Maintainability**
- Clear separation of concerns
- Each language plugin is independent
- No cross-contamination of language-specific logic

### ✅ **Extensibility**
- Adding new languages doesn't touch core code
- Plugins can have language-specific optimizations
- Custom capabilities per language

### ✅ **Performance**
- Direct path to LSP servers (no re-translation)
- Lazy loading of language servers
- Efficient request routing

### ✅ **Testing**
- Each component is independently testable
- Mock plugins for testing
- Clear interfaces and contracts

## Migration Timeline

1. **Phase 1**: Enabled plugin system, created DirectLspAdapter
2. **Phase 2**: Removed old McpDispatcher and hard-coded mappings
3. **Phase 2.5**: Deleted obsolete mcp_tools directory
4. **Current**: 100% plugin-based architecture

## Future Enhancements

- **Plugin marketplace** - Download and install community plugins
- **Hot reload** - Update plugins without restart
- **Custom protocols** - Support beyond LSP (tree-sitter, etc.)
- **Plugin composition** - Combine multiple plugins for polyglot files

## Testing

Run tests with:
```bash
cargo test --lib
```

All 164 tests pass with the new architecture.

## Conclusion

The plugin architecture provides a clean, maintainable, and extensible foundation for the Codeflow Buddy MCP server. By eliminating hard-coded mappings and embracing plugin-based design, we've created a system that can easily grow to support any language or protocol.