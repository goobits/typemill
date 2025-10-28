# Workspace Tools: C

This document provides examples of how to use TypeMill's workspace tools with C projects.

## C Language Support

TypeMill supports C projects through the `mill-lang-c` plugin, which integrates with the `clangd` language server. This provides robust support for code navigation, analysis, and refactoring.

### Shared `clangd` Configuration

The C language plugin uses `clangd` for its core language intelligence. In the future, C++ support will also be provided via `clangd`, and both languages will share the same underlying language server configuration. This ensures a consistent experience for developers working in mixed C/C++ codebases.

## Examples

### `analyze.dead_code`

You can use the `analyze.dead_code` tool to find unused functions in your C projects.

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "tools/call",
  "params": {
    "name": "analyze.dead_code",
    "arguments": {
      "scope": {
        "include": ["**/*.c"]
      }
    }
  }
}
```

### `find_definition`

You can use `find_definition` to locate the definition of a function or variable.

**Request:**

```json
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "tools/call",
  "params": {
    "name": "find_definition",
    "arguments": {
      "file_path": "/path/to/your/project/main.c",
      "line": 10,
      "character": 5
    }
  }
}
```