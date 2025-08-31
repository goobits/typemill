# 🔧 API Reference

Complete documentation for all 28 MCP tools provided by cclsp.

## Core Navigation & Analysis

### `find_definition`

Find the definition of a symbol by name and kind in a file. Returns definitions for all matching symbols.

**Parameters:**
- `file_path`: The path to the file
- `symbol_name`: The name of the symbol
- `symbol_kind`: The kind of symbol (function, class, variable, method, etc.) (optional)

**Example:**
```bash
# Find definition of _calculateAge function
{
  "tool": "find_definition",
  "arguments": {
    "file_path": "/path/to/file.ts",
    "symbol_name": "_calculateAge",
    "symbol_kind": "function"
  }
}
```

### `find_references`

Find all references to a symbol by name and kind in a file. Returns references for all matching symbols.

**Parameters:**
- `file_path`: The path to the file
- `symbol_name`: The name of the symbol
- `symbol_kind`: The kind of symbol (function, class, variable, method, etc.) (optional)
- `include_declaration`: Whether to include the declaration (optional, default: true)

**Example:**
```bash
# Find all references to TestProcessor class
{
  "tool": "find_references", 
  "arguments": {
    "file_path": "/path/to/file.ts",
    "symbol_name": "TestProcessor",
    "include_declaration": true
  }
}
```

## Code Modification

### `rename_symbol`

Rename a symbol by name and kind in a file. **This tool applies the rename to all affected files by default.** If multiple symbols match, returns candidate positions and suggests using rename_symbol_strict.

**Parameters:**
- `file_path`: The path to the file
- `symbol_name`: The name of the symbol
- `symbol_kind`: The kind of symbol (function, class, variable, method, etc.) (optional)
- `new_name`: The new name for the symbol
- `dry_run`: If true, only preview the changes without applying them (optional, default: false)

**Note:** When `dry_run` is false (default), the tool will:
- Apply the rename to all affected files
- Create backup files with `.bak` extension
- Return the list of modified files

**Examples:**
```bash
# Preview rename changes
{
  "tool": "rename_symbol",
  "arguments": {
    "file_path": "/path/to/file.ts", 
    "symbol_name": "getUserData",
    "new_name": "fetchUserProfile",
    "dry_run": true
  }
}

# Apply rename across codebase
{
  "tool": "rename_symbol",
  "arguments": {
    "file_path": "/path/to/file.ts",
    "symbol_name": "getUserData", 
    "new_name": "fetchUserProfile"
  }
}
```

### `rename_symbol_strict`

Rename a symbol at a specific position in a file. Use this when rename_symbol returns multiple candidates. **This tool applies the rename to all affected files by default.**

**Parameters:**
- `file_path`: The path to the file
- `line`: The line number (1-indexed)
- `character`: The character position in the line (1-indexed)
- `new_name`: The new name for the symbol
- `dry_run`: If true, only preview the changes without applying them (optional, default: false)

**Example:**
```bash
# Rename symbol at specific position
{
  "tool": "rename_symbol_strict",
  "arguments": {
    "file_path": "/path/to/file.ts",
    "line": 45,
    "character": 10,
    "new_name": "userData"
  }
}
```

### `format_document`

Format a document according to the language server's formatting rules.

**Parameters:**
- `file_path`: The path to the file to format
- `options`: Formatting options (optional)
- `dry_run`: If true, only preview changes without applying them (optional, default: false)

### `get_code_actions`

Get available code actions (fixes, refactors) for a specific range in a file.

**Parameters:**
- `file_path`: The path to the file
- `start_line`: Start line number (1-indexed)
- `start_character`: Start character position (0-indexed)  
- `end_line`: End line number (1-indexed)
- `end_character`: End character position (0-indexed)

### `apply_workspace_edit`

Apply workspace edits (file changes) across multiple files.

**Parameters:**
- `changes`: Record mapping file paths to arrays of text edits
- `validate_before_apply`: Whether to validate changes before applying (optional, default: false)

### `create_file`

Create a new file with specified content.

**Parameters:**
- `file_path`: Path where the new file should be created
- `content`: Content for the new file

### `delete_file`

Delete a file from the filesystem.

**Parameters:**
- `file_path`: Path to the file to delete
- `dry_run`: If true, only preview the action without executing (optional, default: false)

### `rename_file`

Rename a file and update all import statements that reference it.

**Parameters:**
- `old_path`: Current file path
- `new_path`: New file path
- `dry_run`: If true, only preview changes without applying them (optional, default: false)

## Code Intelligence

### `get_hover`

Get hover information (documentation, types, signatures) for a symbol at a specific position.

**Parameters:**
- `file_path`: The path to the file
- `line`: The line number (1-indexed)
- `character`: The character position in the line (0-indexed)

### `get_completions`

Get code completion suggestions at a specific position in a file.

**Parameters:**
- `file_path`: The path to the file
- `line`: The line number (1-indexed) 
- `character`: The character position in the line (0-indexed)

### `get_inlay_hints`

Get inlay hints (parameter names, type annotations) for a range in a file.

**Parameters:**
- `file_path`: The path to the file
- `start_line`: Start line number (1-indexed)
- `start_character`: Start character position (0-indexed)
- `end_line`: End line number (1-indexed)  
- `end_character`: End character position (0-indexed)

### `get_semantic_tokens`

Get semantic tokens (detailed syntax analysis) for enhanced code understanding.

**Parameters:**
- `file_path`: The path to the file

### `get_signature_help`

Get function signature help at a specific position in the code.

**Parameters:**
- `file_path`: The path to the file
- `line`: The line number (1-indexed)
- `character`: The character position in the line (0-indexed)

## Code Structure Analysis

### `prepare_call_hierarchy`

Prepare call hierarchy items for a symbol at a specific position.

**Parameters:**
- `file_path`: The path to the file
- `line`: The line number (1-indexed)
- `character`: The character position in the line (0-indexed)

### `get_call_hierarchy_incoming_calls`

Get incoming calls for a call hierarchy item.

**Parameters:**
- `item`: Call hierarchy item from prepare_call_hierarchy

### `get_call_hierarchy_outgoing_calls`

Get outgoing calls for a call hierarchy item.

**Parameters:**
- `item`: Call hierarchy item from prepare_call_hierarchy

### `prepare_type_hierarchy`

Prepare type hierarchy items for a symbol at a specific position.

**Parameters:**
- `file_path`: The path to the file
- `line`: The line number (1-indexed)
- `character`: The character position in the line (0-indexed)

### `get_type_hierarchy_supertypes`

Get supertypes (parent classes/interfaces) for a type hierarchy item.

**Parameters:**
- `item`: Type hierarchy item from prepare_type_hierarchy

### `get_type_hierarchy_subtypes`

Get subtypes (child implementations) for a type hierarchy item.

**Parameters:**
- `item`: Type hierarchy item from prepare_type_hierarchy

### `get_selection_range`

Get hierarchical selection ranges for smart code block selection.

**Parameters:**
- `file_path`: The path to the file
- `positions`: Array of positions with line (1-indexed) and character (0-indexed) properties

## Workspace Operations

### `get_document_symbols`

Get all symbols in a document for code outline and navigation.

**Parameters:**
- `file_path`: The path to the file

### `get_folding_ranges`

Get folding ranges for code collapse/expand functionality.

**Parameters:**
- `file_path`: The path to the file

### `get_document_links`

Get document links (URLs, imports, references) found in the file.

**Parameters:**
- `file_path`: The path to the file

### `search_workspace_symbols`

Search for symbols across the entire workspace.

**Parameters:**
- `query`: Search query string

## System Operations

### `get_diagnostics`

Get language diagnostics (errors, warnings, hints) for a file. Uses LSP textDocument/diagnostic to pull current diagnostics.

**Parameters:**
- `file_path`: The path to the file to get diagnostics for

**Example:**
```bash
# Check for errors and warnings
{
  "tool": "get_diagnostics",
  "arguments": {
    "file_path": "/path/to/file.ts"
  }
}
```

### `restart_server`

Manually restart LSP servers and retry any previously failed servers. Can restart servers for specific file extensions or all running servers.

**Parameters:**
- `extensions`: Array of file extensions to restart servers for (e.g., ["ts", "tsx"]). If not provided, all servers will be restarted (optional)

**Examples:**
```bash
# Restart TypeScript server
{
  "tool": "restart_server",
  "arguments": {
    "extensions": ["ts", "tsx"]
  }
}

# Restart all servers
{
  "tool": "restart_server",
  "arguments": {}
}
```

## Real-world Usage Examples

### Finding Function Definitions

When Claude needs to understand how a function works:

```
Claude: Let me find the definition of the `processRequest` function
> Using cclsp.find_definition with symbol_name="processRequest", symbol_kind="function"

Result: Found definition at src/handlers/request.ts:127:1
```

### Finding All References

When refactoring or understanding code impact:

```
Claude: I'll find all places where `CONFIG_PATH` is used
> Using cclsp.find_references with symbol_name="CONFIG_PATH"

Results: Found 5 references:
- src/config.ts:10:1 (declaration)
- src/index.ts:45:15
- src/utils/loader.ts:23:8
- tests/config.test.ts:15:10
- tests/config.test.ts:89:12
```

### Safe Renaming Across Codebase

Preview changes before applying:

```
Claude: Let me first preview what will be renamed
> Using cclsp.rename_symbol with symbol_name="getUserData", new_name="fetchUserProfile", dry_run=true

Result: [DRY RUN] Would rename getUserData (function) to "fetchUserProfile":
File: src/api/user.ts
  - Line 55, Column 10 to Line 55, Column 21: "fetchUserProfile"
File: src/services/auth.ts
  - Line 123, Column 15 to Line 123, Column 26: "fetchUserProfile"
... (12 files total)
```

Apply the rename:

```
Claude: I'll rename `getUserData` to `fetchUserProfile`
> Using cclsp.rename_symbol with symbol_name="getUserData", new_name="fetchUserProfile"

Result: Successfully renamed getUserData (function) to "fetchUserProfile".

Modified files:
- src/api/user.ts
- src/services/auth.ts
- src/components/UserProfile.tsx
... (12 files total)
```

### Handling Multiple Symbol Matches

When multiple symbols match:

```
Claude: I'll rename the `data` variable to `userData`
> Using cclsp.rename_symbol with symbol_name="data", new_name="userData"

Result: Multiple symbols found matching "data". Please use rename_symbol_strict with one of these positions:
- data (variable) at line 45, character 10
- data (parameter) at line 89, character 25
- data (property) at line 112, character 5

> Using cclsp.rename_symbol_strict with line=45, character=10, new_name="userData"

Result: Successfully renamed symbol at line 45, character 10 to "userData".

Modified files:
- src/utils/parser.ts
```

### Checking File Diagnostics

When analyzing code quality:

```
Claude: Let me check for any errors or warnings in this file
> Using cclsp.get_diagnostics

Results: Found 3 diagnostics:
- Error [TS2304]: Cannot find name 'undefinedVar' (Line 10, Column 5)
- Warning [no-unused-vars]: 'config' is defined but never used (Line 25, Column 10)
- Hint: Consider using const instead of let (Line 30, Column 1)
```

### Restarting LSP Servers

When LSP servers become unresponsive:

```
Claude: The TypeScript server seems unresponsive, let me restart it
> Using cclsp.restart_server with extensions ["ts", "tsx"]

Result: Successfully restarted 1 LSP server(s)
Restarted servers:
• typescript-language-server --stdio

Note: Any previously failed servers have been cleared and will be retried on next access.
```

Or restart all servers:

```
Claude: I'll restart all LSP servers to ensure they're working properly
> Using cclsp.restart_server

Result: Successfully restarted 2 LSP server(s)
Restarted servers:
• typescript-language-server --stdio (ts, tsx)
• pylsp (py)

Note: Any previously failed servers have been cleared and will be retried on next access.
```