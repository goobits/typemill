# Quickstart

Get Codebuddy running in 2 minutes.

## Prerequisites

- Language server for your project (e.g., `typescript-language-server`, `rust-analyzer`)
- AI assistant with MCP support (Claude Desktop, etc.)

## Setup Steps

### 1. Install Codebuddy

**Option A: Install script (recommended)**
```bash
curl -fsSL https://raw.githubusercontent.com/goobits/codebuddy/main/install.sh | bash
```

**Option B: Build from source**
```bash
cargo install codebuddy --locked
```

### 2. Configure Your Project

Auto-detects languages and creates `.codebuddy/config.json`:
```bash
codebuddy setup
```

Manual config: see [examples/setup/mcp-config.json](../examples/setup/mcp-config.json)

### 3. Start the Server

```bash
codebuddy start
```

### 4. Connect Your AI Assistant

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

Full example: [examples/setup/mcp-config.json](../examples/setup/mcp-config.json)

### 5. Verify

```bash
codebuddy status
```

## First Tool Call

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

## Next Steps

- **[tools_catalog.md](tools_catalog.md)** - Complete list of 35 MCP tools
- **[api_reference.md](api_reference.md)** - Detailed API with parameters and returns

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
