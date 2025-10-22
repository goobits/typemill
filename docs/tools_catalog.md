# MCP Tools Catalog

Fast lookup table for all Codebuddy MCP tools.

**Format:** Tool name → Parameters → Returns (no examples)
**Detailed docs:** [api_reference.md](api_reference.md)

---

**Public Tools:** 35 MCP tools (visible to AI agents)
**Internal Tools:** 20 backend-only tools (see [Internal Tools](#internal-tools-backend-only) below)

---

## Navigation & Intelligence (8 tools)

| Tool | Description | Required Parameters | Returns |
|------|-------------|---------------------|---------|
| `find_definition` | Find the definition of a symbol at a position | `file_path`, `line`, `character` | Definition locations with ranges |
| `find_references` | Find all references to a symbol | `file_path`, `line`, `character`, `symbol_name` | Array of reference locations |
| `find_implementations` | Find implementations of an interface/abstract class | `file_path`, `line`, `character` | Implementation locations |
| `find_type_definition` | Find the underlying type definition | `file_path`, `line`, `character` | Type definition locations |
| `search_symbols` | Search for symbols across the workspace | `query` | Array of matching symbols with locations |
| `get_symbol_info` | Get detailed symbol information | `file_path`, `line`, `character` | Symbol details with documentation |
| `get_diagnostics` | Get diagnostics (errors, warnings, hints) | `file_path` | Array of diagnostics with severity |
| `get_call_hierarchy` | Get call hierarchy for a symbol | `file_path`, `line`, `character` | Call hierarchy with callers/callees |

---

## Editing & Refactoring (15 tools)

### Plan Operations (7 tools - dry-run, preview only)

| Tool | Description | Required Parameters | Returns |
|------|-------------|---------------------|---------|
| `rename.plan` | **Plan** rename refactoring | `target`, `new_name`, `options` | Refactoring plan (not applied) |
| `extract.plan` | **Plan** extract function/variable | `kind`, `source`, `options` | Refactoring plan (not applied) |
| `inline.plan` | **Plan** inline variable refactoring | `kind`, `target`, `options` | Refactoring plan (not applied) |
| `move.plan` | **Plan** move symbol refactoring | `kind`, `source`, `destination` | Refactoring plan (not applied) |
| `reorder.plan` | **Plan** reorder parameters/imports | `kind`, `target`, `options` | Refactoring plan (not applied) |
| `transform.plan` | **Plan** code transformation | `kind`, `target`, `options` | Refactoring plan (not applied) |
| `delete.plan` | **Plan** delete code/imports | `kind`, `target`, `options` | Refactoring plan (not applied) |

### Quick Operations (7 tools - one-step plan+execute)

| Tool | Description | Required Parameters | Returns |
|------|-------------|---------------------|---------|
| `rename` | **Execute** rename (one-step) | Same as `rename.plan` | Applied changes |
| `extract` | **Execute** extract (one-step) | Same as `extract.plan` | Applied changes |
| `inline` | **Execute** inline (one-step) | Same as `inline.plan` | Applied changes |
| `move` | **Execute** move (one-step) | Same as `move.plan` | Applied changes |
| `reorder` | **Execute** reorder (one-step) | Same as `reorder.plan` | Applied changes |
| `transform` | **Execute** transform (one-step) | Same as `transform.plan` | Applied changes |
| `delete` | **Execute** delete (one-step) | Same as `delete.plan` | Applied changes |

### Apply Tool (1 tool)

| Tool | Description | Required Parameters | Returns |
|------|-------------|---------------------|---------|
| `workspace.apply_edit` | **Apply** a refactoring plan from *.plan tools | `plan` (from *.plan result), `options` | Applied changes with summary |

---

## Unified Analysis API (8 tools)

| Tool | Description | Required Parameters | Returns |
|------|-------------|---------------------|---------|
| `analyze.quality` | Code quality analysis (complexity, smells, maintainability, readability) | `kind`, `scope`, `options` | AnalysisResult with findings |
| `analyze.dead_code` | Unused code detection (imports, symbols, parameters, variables, types, unreachable) | `kind`, `scope`, `options` | AnalysisResult with findings |
| `analyze.dependencies` | Dependency analysis (imports, graph, circular, coupling, cohesion, depth) | `kind`, `scope`, `options` | AnalysisResult with findings |
| `analyze.structure` | Code structure (symbols, hierarchy, interfaces, inheritance, modules) | `kind`, `scope`, `options` | AnalysisResult with findings |
| `analyze.documentation` | Documentation quality (coverage, quality, style, examples, todos) | `kind`, `scope`, `options` | AnalysisResult with findings |
| `analyze.tests` | Test analysis (coverage, quality, assertions, organization) | `kind`, `scope`, `options` | AnalysisResult with findings |
| `analyze.batch` | Multi-file batch analysis with optimized AST caching | `queries`, `options` | BatchAnalysisResult |
| `analyze.module_dependencies` | Rust module dependency analysis for crate extraction | `target`, `options` | Module dependencies (external, workspace, std) |

---

## Workspace Operations (3 tools)

| Tool | Description | Required Parameters | Returns |
|------|-------------|---------------------|---------|
| `workspace.create_package` | Create a new package in the workspace | `package_path`, `package_type`, `options` | Created files, package info |
| `workspace.extract_dependencies` | Extract dependencies from a module for crate extraction | `source_path`, `target_package`, `options` | Dependency manifest, workspace updates |
| `workspace.update_members` | Update workspace member list in manifest | `action`, `member_path`, `options` | Updated manifest, member list |

---

## System & Health (1 tool)

| Tool | Description | Required Parameters | Returns |
|------|-------------|---------------------|---------|
| `health_check` | Get comprehensive server health and statistics | None | Status, uptime, LSP servers, memory |

---

## Quick Notes

### Unified Refactoring API (Two Patterns)

**Two-Step Pattern (Recommended for Safety):**
1. **`*.plan` tools** (e.g., `rename.plan`, `extract.plan`, `inline.plan`, `move.plan`) - Generate refactoring plan (dry-run, never writes to filesystem)
2. **`workspace.apply_edit`** - Apply the plan to make actual changes

**One-Step Pattern (Quick Operations):**
- **Quick tools** (e.g., `rename`, `extract`, `inline`, `move`) - Combine plan + execute in one call
- Same parameters as `.plan` versions, but automatically applies changes
- Less safe (no preview), but convenient for small, trusted operations

**Benefits of Two-Step:**
- Preview all changes before applying
- Safe by design (*.plan commands are always dry-run)
- Consistent pattern across all refactorings
- Can use `dry_run: true` in `workspace.apply_edit` for final preview

### Common Optional Parameters
- **`dry_run`**: Preview changes without applying (many editing/file tools, workspace.apply_edit)
- **`workspace_id`**: Execute in remote workspace (read_file, write_file)
- **`include_declaration`**: Include definition in results (find_references)

### Indexing Conventions
- **Lines**: 1-indexed in user-facing APIs, 0-indexed in LSP protocol
- **Characters**: Always 0-indexed

### Language Support
LSP-based tools depend on configured language servers. Native tools (file ops, AST-based) support:
- TypeScript/JavaScript (SWC parser)
- Python (native AST)
- Go (tree-sitter-go)
- Rust (syn crate)
- Java (tree-sitter-java)
- Swift (tree-sitter-swift)
- C# (tree-sitter-c-sharp)

**AST Refactoring Support:**
- ✅ Full: TypeScript/JavaScript, Python, Rust, Go, Java, Swift
- ⚠️ Partial: C# (extract.plan works, inline.plan has known issues)

---

## Internal Tools (Backend Only)

Not visible in MCP `tools/list`. Used by backend workflows. AI agents should use public API instead.

| Category | Tools | Count |
|----------|-------|-------|
| **Lifecycle** | notify_file_opened, notify_file_saved, notify_file_closed | 3 |
| **File Operations** | create_file, delete_file, rename_file, rename_directory | 4 |
| **File Utilities** | read_file, write_file, list_files | 3 |
| **Workspace Tools** | move_directory, update_dependencies, update_dependency | 3 |
| **Structure Analysis** | get_document_symbols (replaced by `analyze.structure`) | 1 |
| **Advanced Plumbing** | execute_edits (replaced by `workspace.apply_edit`), execute_batch | 2 |
| **Legacy Editing** | rename_symbol_with_imports | 1 |
| **Legacy Workspace** | apply_workspace_edit | 1 |
| **Intelligence** | get_completions, get_signature_help | 2 |

**Total:** 20 internal tools

**Note:** Legacy analysis tools removed (Proposal 45). All analysis via unified `analyze.*` API.

---

**Detailed docs:** [api_reference.md](api_reference.md)
