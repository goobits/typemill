# Proposal: Clean Plugin Architecture (Complete Rewrite)

**Status:** Ready for Implementation
**Target:** Complete replacement of existing MCP tool system
**No Legacy Maintenance:** Clean slate implementation

## Executive Summary

This proposal outlines a **complete rewrite** of the MCP tool system using a clean plugin architecture. The current implementation suffers from hard-coded mappings, spider web dependencies, and tight coupling between MCP tools and LSP protocol specifics. Instead of maintaining legacy code, we will build a completely new plugin-first system that is extensible, maintainable, and performant.

## Current Problems to Eliminate

### 🚫 Hard-coded Method Mappings (manager.rs:101-116)
```rust
// REMOVING THIS ENTIRELY
let method = match mcp_request.method.as_str() {
    "find_definition" => "textDocument/definition",
    "find_references" => "textDocument/references",
    // ... 8+ more hard-coded mappings
};
```

### 🚫 Hard-coded Tool Operations (mcp_dispatcher.rs:56-86)
```rust
// REMOVING THIS ENTIRELY
self.tool_operations.insert("find_definition".to_string(), OperationType::Read);
self.tool_operations.insert("rename_symbol".to_string(), OperationType::Refactor);
// ... 30+ lines of hard-coded categorization
```

### 🚫 Tight Coupling Problems
- MCP tools directly create LSP requests
- Handlers embedded with LSP protocol knowledge
- No abstraction between MCP interface and language backends
- Extension-based routing logic scattered across multiple files

## New Architecture: Plugin-First Design

### Core Philosophy
1. **Plugin-First:** Everything is a plugin, no built-in tool handlers
2. **Protocol Agnostic:** Plugins don't know about LSP/MCP specifics
3. **Capability-Based:** Dynamic discovery of what each plugin can do
4. **Zero Hard-coding:** All mappings and behaviors are declarative

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    MCP Interface                        │
│                   (HTTP/WebSocket)                      │
├─────────────────────────────────────────────────────────┤
│                  Plugin Router                         │
│              (Capability-based routing)                │
├─────────────────────────────────────────────────────────┤
│     Plugin Manager     │      Protocol Adapters       │
│  - Dynamic Loading     │   ┌─────────┬─────────────┐   │
│  - Lifecycle Mgmt      │   │   LSP   │    Tree-    │   │
│  - Hook System         │   │ Adapter │   sitter    │   │
│                        │   └─────────┴─────────────┘   │
├────────────────────────┴─────────────────────────────────┤
│                    Plugin Runtime                       │
│  ┌───────────┬──────────────┬──────────────┬─────────┐  │
│  │   Rust    │  TypeScript  │    Python    │   Go    │  │
│  │  Plugin   │    Plugin    │    Plugin    │ Plugin  │  │
│  └───────────┴──────────────┴──────────────┴─────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Plugin System Design

### 1. Core Plugin Trait

```rust
#[async_trait]
pub trait LanguagePlugin: Send + Sync + 'static {
    /// Plugin identification and metadata
    fn metadata(&self) -> PluginMetadata;

    /// File extensions this plugin handles
    fn file_patterns(&self) -> Vec<FilePattern>;

    /// Capabilities this plugin provides
    fn capabilities(&self) -> PluginCapabilities;

    /// Initialize plugin with configuration
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()>;

    /// Handle a protocol-agnostic request
    async fn handle_request(&self, request: PluginRequest) -> PluginResult<PluginResponse>;

    /// Plugin lifecycle hooks
    async fn on_file_opened(&self, ctx: &FileContext) -> PluginResult<()> { Ok(()) }
    async fn on_file_saved(&self, ctx: &FileContext) -> PluginResult<()> { Ok(()) }
    async fn on_file_closed(&self, ctx: &FileContext) -> PluginResult<()> { Ok(()) }

    /// Cleanup on shutdown
    async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }
}
```

### 2. Capability System

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginCapabilities {
    pub navigation: NavigationCapabilities,
    pub editing: EditingCapabilities,
    pub refactoring: RefactoringCapabilities,
    pub intelligence: IntelligenceCapabilities,
    pub formatting: FormattingCapabilities,
    pub diagnostics: DiagnosticsCapabilities,
    /// Plugin-specific custom capabilities
    pub custom: HashMap<String, CapabilityMetadata>,
}

#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterSchema>,
    pub return_type: ResponseSchema,
}
```

### 3. Protocol-Agnostic Requests

```rust
#[derive(Debug, Clone)]
pub struct PluginRequest {
    pub id: RequestId,
    pub capability: String,  // e.g., "navigation.find_definition"
    pub file_context: FileContext,
    pub parameters: RequestParameters,
}

#[derive(Debug, Clone)]
pub struct FileContext {
    pub file_path: PathBuf,
    pub content: Option<String>,
    pub position: Option<Position>,
    pub selection: Option<Range>,
}

#[derive(Debug, Clone)]
pub enum RequestParameters {
    Position(Position),
    Range(Range),
    Symbol(String),
    Query(String),
    Custom(HashMap<String, serde_json::Value>),
}
```

### 4. Plugin Manager

```rust
pub struct PluginManager {
    /// Registry of loaded plugins
    plugins: HashMap<PluginId, Box<dyn LanguagePlugin>>,
    /// Capability index for fast routing
    capability_index: CapabilityIndex,
    /// File pattern matcher
    pattern_matcher: PatternMatcher,
    /// Plugin configurations
    configs: HashMap<PluginId, PluginConfig>,
}

impl PluginManager {
    /// Route request to appropriate plugin based on capabilities
    pub async fn route_request(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
        let plugin_id = self.find_plugin_for_request(&request)?;
        let plugin = self.plugins.get(&plugin_id)
            .ok_or(PluginError::PluginNotFound(plugin_id))?;
        plugin.handle_request(request).await
    }

    /// Dynamic plugin loading
    pub async fn load_plugin(&mut self, plugin_spec: PluginSpec) -> PluginResult<PluginId>;

    /// Unload plugin and cleanup
    pub async fn unload_plugin(&mut self, plugin_id: PluginId) -> PluginResult<()>;
}
```

## Implementation Phases (No Legacy Support)

### Phase 1: Foundation (Week 1)
**Goal:** Build the core plugin system from scratch

- ✅ **New Plugin Trait System**
  - Define `LanguagePlugin` trait and all related types
  - Implement capability system with full metadata support
  - Create protocol-agnostic request/response types

- ✅ **Plugin Manager Infrastructure**
  - Build plugin registry with dynamic loading
  - Implement capability-based routing engine
  - Create pattern matching for file extensions

- ✅ **MCP Interface Layer**
  - Replace `mcp_dispatcher.rs` with new `PluginRouter`
  - Implement MCP → PluginRequest translation
  - Handle PluginResponse → MCP translation

### Phase 2: Core Language Plugins (Week 2)
**Goal:** Reimplement existing language support as plugins

- ✅ **LSP Adapter Plugin**
  - Generic LSP protocol adapter as a plugin
  - Configurable per language server
  - Replaces hard-coded `manager.rs` mappings

- ✅ **Built-in Language Plugins**
  - TypeScript/JavaScript plugin (replaces existing handlers)
  - Python plugin with language-specific optimizations
  - Go plugin with module awareness
  - Rust plugin with Cargo integration

- ✅ **Replace Legacy Handlers**
  - Remove all files in `handlers/mcp_tools/`
  - Remove `mcp_dispatcher.rs` entirely
  - Remove hard-coded mappings in `lsp/manager.rs`

### Phase 3: Advanced Features (Week 3)
**Goal:** Add features impossible with old architecture

- ✅ **Plugin Hooks System**
  - File lifecycle hooks (open/save/close)
  - Project lifecycle hooks (init/load/unload)
  - Cross-plugin communication channels

- ✅ **Enhanced Capabilities**
  - Language-specific custom capabilities
  - Multi-file refactoring support
  - Project-aware intelligence

- ✅ **Developer Experience**
  - Plugin development toolkit
  - Testing framework for plugins
  - Hot-reloading during development

## Plugin Examples

### TypeScript Plugin Implementation

```rust
pub struct TypeScriptPlugin {
    lsp_adapter: LspAdapterPlugin,
    tsconfig_manager: TsConfigManager,
    import_organizer: ImportOrganizer,
}

#[async_trait]
impl LanguagePlugin for TypeScriptPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "TypeScript Language Plugin".to_string(),
            version: semver::Version::parse("1.0.0").unwrap(),
            author: "Codeflow Buddy Team".to_string(),
            description: "Complete TypeScript/JavaScript language support".to_string(),
        }
    }

    fn file_patterns(&self) -> Vec<FilePattern> {
        vec![
            FilePattern::Extension("ts"),
            FilePattern::Extension("tsx"),
            FilePattern::Extension("js"),
            FilePattern::Extension("jsx"),
            FilePattern::Extension("mjs"),
            FilePattern::Extension("cjs"),
        ]
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            navigation: NavigationCapabilities {
                find_definition: true,
                find_references: true,
                find_implementations: true,
                find_type_definition: true,
                ..Default::default()
            },
            refactoring: RefactoringCapabilities {
                rename_symbol: true,
                extract_function: true,
                organize_imports: true,
                ..Default::default()
            },
            custom: hashmap! {
                "typescript.infer_types".to_string() => CapabilityMetadata {
                    name: "Type Inference".to_string(),
                    description: "Infer TypeScript types at cursor position".to_string(),
                    parameters: vec![ParameterSchema::Position],
                    return_type: ResponseSchema::TypeInfo,
                },
                "typescript.auto_import".to_string() => CapabilityMetadata {
                    name: "Auto Import".to_string(),
                    description: "Automatically add import statements".to_string(),
                    parameters: vec![ParameterSchema::Symbol],
                    return_type: ResponseSchema::TextEdit,
                },
            },
        }
    }

    async fn handle_request(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
        match request.capability.as_str() {
            // Standard capabilities delegated to LSP
            "navigation.find_definition" => {
                self.lsp_adapter.handle_request(request).await
            }

            // TypeScript-specific capabilities
            "typescript.infer_types" => {
                self.handle_type_inference(request).await
            }
            "typescript.auto_import" => {
                self.handle_auto_import(request).await
            }

            _ => Err(PluginError::CapabilityNotSupported(request.capability))
        }
    }
}
```

## Configuration System

```yaml
# .codebuddy/plugin-config.yaml
plugins:
  typescript:
    enabled: true
    lsp:
      command: ["typescript-language-server", "--stdio"]
      initialization_options:
        preferences:
          includeCompletionsForModuleExports: true
    custom:
      auto_import_enabled: true
      strict_type_checking: true

  python:
    enabled: true
    lsp:
      command: ["pylsp"]
    custom:
      virtual_env_detection: true
      type_stub_support: true

  rust:
    enabled: true
    lsp:
      command: ["rust-analyzer"]
    custom:
      cargo_integration: true
      macro_expansion: true
```

## Benefits of Clean Rewrite

### 🎯 **Zero Legacy Burden**
- No compatibility layers or migration paths
- Clean, modern Rust code throughout
- No hard-coded mappings anywhere

### 🚀 **Immediate Value**
- Adding new languages = dropping in a plugin
- Language-specific optimizations possible
- Plugin development independent of core

### 🔧 **Developer Experience**
- Clear plugin development model
- Comprehensive testing framework
- Hot-reload during development

### 📈 **Performance**
- Lazy plugin loading
- Capability-based routing (O(1) lookup)
- Zero overhead for unused languages

### 🛡️ **Maintainability**
- Single responsibility per plugin
- Clear boundaries and interfaces
- Independent plugin versioning

## Success Metrics

- ✅ **Zero hard-coded mappings** in entire codebase
- ✅ **Adding new language** requires zero core changes
- ✅ **Plugin development** completely independent
- ✅ **Performance** matches or exceeds current implementation
- ✅ **All existing functionality** available through plugins
- ✅ **Test coverage** > 90% for all plugin interfaces

## Risk Mitigation

### **Development Risk**
- **Mitigation:** Build incrementally, test each component
- **Fallback:** Current Rust implementation continues working until replacement complete

### **Performance Risk**
- **Mitigation:** Capability indexing, lazy loading, request batching
- **Validation:** Benchmark against current implementation

### **Plugin Ecosystem Risk**
- **Mitigation:** Provide comprehensive built-in plugins for all current languages
- **Documentation:** Complete plugin development guide and examples

## Conclusion

This clean rewrite approach eliminates all architectural debt while building a truly extensible system. By removing legacy constraints, we can create the plugin architecture the codebase needs for long-term success.

**No compromise. No legacy. Clean plugin architecture from day one.**