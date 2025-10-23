# Codebuddy

**Pure Rust MCP server bridging Language Server Protocol (LSP) to AI coding assistants**

Codebuddy provides 35 comprehensive MCP tools for code navigation, refactoring, code intelligence, and batch operations. Built on Rust for performance and safety.

## Quick Start

Get Codebuddy running in 2 minutes.

### Prerequisites

- Language server for your project (e.g., `typescript-language-server`, `rust-analyzer`)
- AI assistant with MCP support (Claude Desktop, etc.)

### Installation

**Option A: Install script (recommended)**
```bash
curl -fsSL https://raw.githubusercontent.com/goobits/codebuddy/main/install.sh | bash
```

**Option B: Build from source**
```bash
cargo install codebuddy --locked
```

### Setup

Auto-detect languages and create `.codebuddy/config.json`:
```bash
codebuddy setup
```

### Start Server

```bash
codebuddy start
```

### Connect Your AI Assistant

Add to your MCP configuration:
```json
{
  "mcpServers": {
    "codebuddy": {
      "command": "codebuddy",
      "args": ["start"]
    }
  }
}
```

Full example: [examples/setup/mcp-config.json](examples/setup/mcp-config.json)

### Verify

```bash
codebuddy status
```

## First Tool Calls

Ask your AI assistant:
- "Find the definition of `main` in src/main.rs"
- "Show me all references to the `Config` type"
- "Rename the function `oldName` to `newName`"

## Common CLI Commands

### File Operations

```bash
# Move/rename a file
codebuddy tool rename --target file:src/old.rs --new-name src/new.rs

# Move a directory
codebuddy tool rename --target directory:old-dir --new-name new-dir
```

### Code Operations

```bash
# Move code symbol (function/class) between files
codebuddy tool move --source src/app.rs:10:5 --destination src/utils.rs

# Extract function
codebuddy tool extract --kind function --source src/app.rs:10:5 --name handleLogin
```

**Important:**
- Use `rename` for **file/directory operations**
- Use `move` for **code symbol operations** (requires line:char position)

If you use the wrong tool, Codebuddy will provide a helpful error with the correct command.

## Available Tools (35 total)

**Navigation & Intelligence (8 tools)**
- `find_definition`, `find_references`, `search_symbols`
- `find_implementations`, `find_type_definition`, `get_symbol_info`
- `get_diagnostics`, `get_call_hierarchy`

**Editing & Refactoring (15 tools)**
- **Plan Operations**: `rename.plan`, `extract.plan`, `inline.plan`, `move.plan`, `reorder.plan`, `transform.plan`, `delete.plan`
- **Quick Operations**: `rename`, `extract`, `inline`, `move`, `reorder`, `transform`, `delete`
- **Apply**: `workspace.apply_edit`

**Analysis (8 tools)**
- `analyze.quality`, `analyze.dead_code`, `analyze.dependencies`
- `analyze.structure`, `analyze.documentation`, `analyze.tests`
- `analyze.batch`, `analyze.module_dependencies`

**Workspace (3 tools)**
- `workspace.create_package`, `workspace.extract_dependencies`, `workspace.update_members`

**System (1 tool)**
- `health_check`

## Documentation

- **[docs/tools_catalog.md](docs/tools_catalog.md)** - Fast lookup of all 35 tools
- **[docs/api_reference.md](docs/api_reference.md)** - Complete API with parameters and examples
- **[contributing.md](contributing.md)** - Development guide
- **[docs/architecture/overview.md](docs/architecture/overview.md)** - System architecture
- **[docs/operations/docker_deployment.md](docs/operations/docker_deployment.md)** - Docker deployment

## Language Support

| Language | Extensions | LSP Server | Refactoring |
|----------|-----------|------------|-------------|
| TypeScript/JavaScript | ts, tsx, js, jsx | typescript-language-server | Full ✅ |
| Rust | rs | rust-analyzer | Full ✅ |

*Additional languages (Python, Go, Java, Swift, C#) available in git tag `pre-language-reduction`*

## Troubleshooting

**Server won't start:**
- Check `codebuddy status` for LSP server availability
- Verify language servers are installed and in PATH
- Check `.codebuddy/config.json` for correct command paths

**Tools not working:**
- Ensure file extensions match config (e.g., `.rs` → `rust-analyzer`)
- Check MCP connection with AI assistant
- Review server logs for errors

**Performance issues:**
- Enable cache (disabled by default for development)
- Adjust `restartInterval` in config (recommended: 10-30 minutes)
- Check system resources (LSP servers can be memory-intensive)

## Features

**Safe Refactoring**
- Two-step plan → apply pattern for all refactorings
- Dry-run mode previews changes before applying
- Atomic operations with automatic rollback on failure

**Comprehensive Coverage**
- Automatic import updates for file renames
- Cross-file reference tracking
- Rust-specific crate consolidation support
- Batch operations for bulk changes

**Production Ready**
- Pure Rust implementation for performance
- WebSocket server with JWT authentication
- Multi-tenant workspace isolation
- Structured logging for observability
- Docker deployment support

## Contributing

See [contributing.md](contributing.md) for development setup, testing, and PR workflow.

## License

See [LICENSE](LICENSE) for details.

## Links

- **Issues:** [GitHub Issues](https://github.com/goobits/codebuddy/issues)
- **Discussions:** [GitHub Discussions](https://github.com/goobits/codebuddy/discussions)
- **Security:** security@goobits.com (private disclosure)
