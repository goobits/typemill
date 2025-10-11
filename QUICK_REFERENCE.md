# 🚀 Codebuddy Quick Reference

This guide is for experienced developers who want to get productive with Codebuddy in under 15 minutes. It assumes you are familiar with AI assistants, LSP, and your command line.

**Current Public API**: 17 tools (see table below) | **Internal Tools**: 25 backend-only tools

---

## 1. Installation

**Recommended (macOS/Linux):**
```bash
curl -fsSL https://raw.githubusercontent.com/goobits/codebuddy/main/install.sh | bash
```

**Alternative (Cargo):**
```bash
cargo install codebuddy --locked
```

---

## 2. Core Commands

**First, configure your project:**
```bash
codebuddy setup    # Auto-detects languages and creates .codebuddy/config.json
```

**Then, manage the server:**
```bash
codebuddy start    # Start the MCP server for your AI assistant
codebuddy status   # Check server status and loaded languages
codebuddy stop     # Stop the server
```

**Execute a tool directly:**
```bash
codebuddy tool find_definition '{"file_path":"src/app.ts","line":10,"character":5}'
```

---

## 3. Configuration (`.codebuddy/config.json`)

`codebuddy setup` handles this for you. For manual tweaking:

```json
{
  "servers": [
    {
      "extensions": ["ts", "tsx", "js", "jsx"],
      "command": ["typescript-language-server", "--stdio"],
      "restartInterval": 30
    },
    {
      "extensions": ["rs"],
      "command": ["rust-analyzer"]
    }
  ]
}
```
- **`extensions`**: File types this server is responsible for.
- **`command`**: The command to start the LSP server.
- **`restartInterval`**: (Optional) Auto-restart interval in minutes to ensure stability.

---

## 4. Top 17 Public MCP Tools

These are the public-facing tools for AI agents and MCP clients. See `API_REFERENCE.md` for complete details.

### Navigation & Intelligence (8 tools)
| Tool | Description | Example |
|------|-------------|---------|
| `find_definition` | Go to the definition of a symbol | `{"file_path":"src/app.ts","line":10,"character":5}` |
| `find_references` | Find all references to a symbol | `{"file_path":"src/app.ts","line":10,"character":5}` |
| `find_implementations` | Find implementations of interface/abstract class | `{"file_path":"src/app.ts","line":10,"character":5}` |
| `find_type_definition` | Find underlying type definition | `{"file_path":"src/app.ts","line":10,"character":5}` |
| `search_workspace_symbols` | Search for symbols by name across workspace | `{"query":"MyComponent"}` |
| `get_symbol_info` | Get detailed symbol information | `{"file_path":"src/app.ts","line":10,"character":5}` |
| `get_diagnostics` | Get all errors and warnings for a file | `{"file_path":"src/app.ts"}` |
| `get_call_hierarchy` | Get call hierarchy (callers/callees) | `{"file_path":"src/app.ts","line":10,"character":5}` |

### Refactoring (7 tools - Unified API)
| Tool | Description | Example |
|------|-------------|---------|
| `rename.plan` | Generate plan to rename symbol/file/directory | `{"target":{"kind":"symbol","path":"src/app.ts","selector":{"position":{"line":10,"character":5}}},"new_name":"newName"}` |
| `extract.plan` | Generate plan to extract function/variable | `{"kind":"function","source":{"file_path":"src/app.ts","range":{"start":{"line":10,"character":0},"end":{"line":15,"character":1}},"name":"extracted"}}` |
| `inline.plan` | Generate plan to inline variable/function | `{"kind":"variable","target":{"file_path":"src/app.ts","position":{"line":10,"character":5}}}` |
| `move.plan` | Generate plan to move code between files | `{"kind":"symbol","source":{"file_path":"src/old.ts","position":{"line":10,"character":5}},"destination":{"file_path":"src/new.ts"}}` |
| `reorder.plan` | Generate plan to reorder parameters/imports | `{"kind":"imports","target":{"file_path":"src/app.ts"},"options":{"strategy":"alphabetical"}}` |
| `transform.plan` | Generate plan to transform code (e.g., to async) | `{"kind":"to_async","target":{"file_path":"src/app.ts","position":{"line":10,"character":5}}}` |
| `delete.plan` | Generate plan to delete unused code | `{"kind":"unused_imports","target":{"scope":"file","path":"src/app.ts"}}` |

### Workspace & System (2 tools)
| Tool | Description | Example |
|------|-------------|---------|
| `workspace.apply_edit` | Execute a refactoring plan | `{"plan":{...}}` |
| `health_check` | Get server health status | `{"include_details":true}` |

---

## 5. Internal Tools (Backend Only - 25 tools)

These tools are **not visible** in MCP `tools/list` but are used internally by workflows and the backend. AI agents should use the public API instead.

**Categories:**
- **Lifecycle (3)**: notify_file_opened, notify_file_saved, notify_file_closed
- **File Operations (4)**: create_file, delete_file, rename_file, rename_directory
- **File Utilities (3)**: read_file, write_file, list_files
- **Workspace Tools (4)**: move_directory, find_dead_code, update_dependencies, update_dependency
- **Legacy Analysis (4)**: find_unused_imports → `analyze.dead_code` (future), analyze_code → `analyze.quality` (future), analyze_project → `analyze.quality` (future), analyze_imports → `analyze.dependencies` (future)
- **Structure Analysis (1)**: get_document_symbols → `analyze.structure` (future)
- **Advanced Plumbing (2)**: execute_edits → replaced by `workspace.apply_edit`, execute_batch → `analyze.batch` (future)
- **Legacy Editing (1)**: rename_symbol_with_imports
- **Legacy Workspace (1)**: apply_workspace_edit
- **Intelligence (2)**: get_completions, get_signature_help

---

## 6. Key Links

- **[API_REFERENCE.md](API_REFERENCE.md)**: The complete, detailed reference for all tools.
- **[CONTRIBUTING.md](CONTRIBUTING.md)**: For developers who want to build from source or contribute.
- **[docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md)**: A deep dive into the system architecture.
