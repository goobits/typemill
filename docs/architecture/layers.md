# Architectural Layers & Dependency Model

This document defines the formal layered architecture for the Codebuddy project. These layers are programmatically enforced using `cargo-deny` to prevent architectural drift and maintain clean separation of concerns.

## Layer Hierarchy

The architecture follows a strict **unidirectional dependency flow**: higher layers may depend on lower layers, but never the reverse.

```
┌─────────────────────────────────────────────────────────────┐
│                    LAYER 5: APPLICATION                     │
│                      (apps/codebuddy)                       │
│                                                             │
│  Entry point, CLI parsing, bootstrap, process management   │
└──────────────────────────┬──────────────────────────────────┘
                           │ depends on ↓
┌─────────────────────────────────────────────────────────────┐
│                  LAYER 4: PRESENTATION                      │
│              (cb-transport, cb-handlers)                    │
│                                                             │
│  MCP routing, WebSocket/stdio, session mgmt, marshaling    │
└──────────────────────────┬──────────────────────────────────┘
                           │ depends on ↓
┌─────────────────────────────────────────────────────────────┐
│              LAYER 3: ORCHESTRATION & SERVER                │
│                      (cb-server)                            │
│                                                             │
│  Tool registry, plugin dispatch, app state, request loop   │
└──────────────────────────┬──────────────────────────────────┘
                           │ depends on ↓
┌─────────────────────────────────────────────────────────────┐
│                LAYER 2: BUSINESS LOGIC                      │
│          (cb-services, cb-ast, cb-plugins)                  │
│                                                             │
│  Refactoring, analysis, import mgmt, plugin system          │
└──────────────────────────┬──────────────────────────────────┘
                           │ depends on ↓
┌─────────────────────────────────────────────────────────────┐
│             LAYER 1: INFRASTRUCTURE & CORE                  │
│   (cb-lsp, cb-core, cb-types, cb-protocol, cb-client)       │
│                                                             │
│  LSP comm, config, logging, error types, protocol defs     │
└─────────────────────────────────────────────────────────────┘
                           │ depends on ↓
┌─────────────────────────────────────────────────────────────┐
│               LAYER 0: LANGUAGE PLUGINS                     │
│         (cb-lang-*, cb-lang-common, cb-plugin-api)          │
│                                                             │
│  Language-specific parsers, analyzers, plugin API           │
└─────────────────────────────────────────────────────────────┘
```

## Layer Definitions

### Layer 5: Application
**Crates**: `apps/codebuddy`

**Responsibilities**:
- Binary entry point (`main()`)
- CLI argument parsing
- Server bootstrap and initialization
- Process lifecycle management

**May depend on**: All lower layers

**Must NOT**:
- Contain business logic
- Implement protocol handlers
- Perform file I/O directly

---

### Layer 4: Presentation
**Crates**: `cb-transport`, `cb-handlers`

**Responsibilities**:
- MCP request routing and dispatch
- Transport protocol handling (WebSocket, stdio)
- Session management
- Request/response marshaling
- Tool handler registration

**May depend on**: Layers 0-3

**Must NOT**:
- Contain business logic (e.g., plan conversion, validation)
- Perform direct file I/O
- Implement LSP communication
- Parse or analyze code

---

### Layer 3: Orchestration & Server
**Crates**: `cb-server`

**Responsibilities**:
- Central orchestration and wiring
- Tool registry management
- Plugin dispatcher coordination
- AppState service container
- Main request processing loop

**May depend on**: Layers 0-2

**Must NOT**:
- Implement transport protocols
- Contain business logic
- Perform direct file operations

---

### Layer 2: Business Logic
**Crates**: `cb-services`, `cb-ast`, `cb-plugins`

**Responsibilities**:
- Refactoring operations (plan generation, execution)
- Code analysis and transformations
- Import and reference management
- Plugin system implementation
- Service trait implementations
- Plan conversion and validation

**May depend on**: Layers 0-1

**Must NOT**:
- Handle MCP protocol concerns
- Manage WebSocket/stdio connections
- Implement CLI parsing

**Key Services**:
- `AstService`: Code parsing and analysis
- `FileService`: File operations with locking
- `ReferenceUpdater`: Import/reference tracking
- `PluginManager`: Language plugin dispatch

---

### Layer 1: Infrastructure & Core
**Crates**: `cb-lsp`, `cb-core`, `cb-types`, `cb-protocol`, `cb-client`

**Responsibilities**:
- LSP server communication (`cb-lsp`)
- Configuration management (`cb-core`)
- Logging and error handling (`cb-core`)
- Core data types (`cb-types`)
- Protocol trait definitions (`cb-protocol`)
- CLI/WebSocket client (`cb-client`)

**May depend on**: Layer 0 only

**Must NOT**:
- Contain business logic
- Implement MCP tool handlers
- Parse or analyze code

**Special Note**: `cb-protocol` defines service traits (interfaces) that Layer 2 implements. This is the **dependency inversion boundary**.

---

### Layer 0: Language Plugins
**Crates**: `cb-lang-*`, `cb-lang-common`, `cb-plugin-api`, `cb-plugin-registry`

**Responsibilities**:
- Language-specific parsing and analysis
- Plugin API trait definitions (`cb-plugin-api`)
- Plugin registration system (`cb-plugin-registry`)
- Shared plugin utilities (`cb-lang-common`)
- Individual language plugins (`cb-lang-rust`, `cb-lang-typescript`, etc.)

**May depend on**: Standard library and external parsing crates only

**Must NOT**:
- Depend on any workspace crates outside Layer 0
- Implement MCP or LSP protocol handling
- Contain orchestration logic

**Exception**: Plugins may depend on `cb-types` for shared data structures (e.g., `Symbol`, `SourceLocation`).

---

## Dependency Rules

### Allowed Dependencies

| Layer | May Depend On | Rationale |
|-------|---------------|-----------|
| **5: Application** | 0, 1, 2, 3, 4 | Entry point needs access to all layers |
| **4: Presentation** | 0, 1, 2, 3 | Needs server orchestration and services |
| **3: Orchestration** | 0, 1, 2 | Wires together business logic and infrastructure |
| **2: Business Logic** | 0, 1 | Uses infrastructure and language plugins |
| **1: Infrastructure** | 0 | Uses plugin API for extensibility |
| **0: Language Plugins** | None (stdlib + external crates only) | Pure language-specific logic |

### Forbidden Dependencies (Enforced by cargo-deny)

These rules prevent architectural violations:

1. **No Upward Dependencies**: Lower layers MUST NOT depend on higher layers
   - `cb-core` → `cb-services` ❌
   - `cb-lsp` → `cb-server` ❌
   - `cb-types` → `cb-handlers` ❌

2. **No Cross-Layer Shortcuts**: Must follow layer hierarchy
   - `cb-handlers` → `cb-lsp` directly ❌ (should go through `cb-server` or service traits)
   - `apps/codebuddy` → `cb-ast` directly ❌ (should go through `cb-server`)

3. **No Business Logic in Presentation**: Presentation layer must delegate to services
   - `cb-handlers` must not implement plan conversion, validation, or file I/O

4. **No Infrastructure Logic in Business**: Business logic uses infrastructure through abstractions
   - `cb-services` should not directly spawn LSP servers (use `cb-lsp` abstractions)

## Special Cases

### Cross-Cutting Concerns

**Testing Utilities** (`cb-test-support`, `cb-bench`):
- May depend on any layer for testing purposes
- Exempt from layer restrictions (test-only code)

**Analysis Tools** (`analysis/*` crates):
- Separate workspace for tooling
- May analyze workspace structure but not participate in runtime architecture

### Dependency Inversion

The `cb-protocol` crate contains **trait definitions only**:
- Defines `AstService`, `LspService`, `MessageDispatcher`, `ToolHandler` traits
- Has no implementations (zero-cost abstraction boundary)
- Higher layers depend on these traits
- Lower layers implement these traits

This creates proper dependency inversion: both presentation (`cb-handlers`) and business logic (`cb-services`) depend on the protocol abstraction, not on each other.

## Validation

These rules are enforced by:

1. **`cargo-deny`**: Graph rules in `deny.toml` prevent forbidden dependencies
2. **CI Pipeline**: Fails builds that violate layer boundaries
3. **Code Review**: Manual review for architectural concerns
4. **Documentation**: This file serves as the source of truth

## Migration Path

During the workspace consolidation (Proposal 06), crates will be merged while preserving these layers:

- `cb-core` + `cb-types` + `cb-protocol` → `codebuddy-foundation` (Layer 1)
- `cb-plugins` + `cb-plugin-registry` → `codebuddy-plugin-system` (Layer 0/2 boundary)
- All crates renamed to `codebuddy-*` prefix

The layer definitions will remain the same, but enforcement will be stricter after consolidation.

## Benefits

1. **Prevents Architectural Drift**: Automated enforcement stops violations at build time
2. **Clear Mental Model**: Developers understand where code belongs
3. **Easier Testing**: Each layer can be tested in isolation
4. **Flexible Refactoring**: Layer boundaries define safe refactoring zones
5. **Documentation as Code**: `deny.toml` serves as executable architecture documentation

## References

- [SOC_LAYER_DIAGRAM.md](../../SOC_LAYER_DIAGRAM.md) - Current state analysis and violations
- [overview.md](./overview.md) - Detailed architecture documentation
- [Proposal 06](../../proposals/06_workspace_consolidation.proposal.md) - Consolidation plan
- `deny.toml` - Programmatic enforcement configuration
