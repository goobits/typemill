# 🔍 Troubleshooting Guide

Solutions for common issues and debugging techniques.

## 🚨 Known Issues

### Python LSP Server (pylsp) Performance Degradation

**Problem**: The Python Language Server (pylsp) may become slow or unresponsive after extended use (several hours), affecting symbol resolution and code navigation.

**Symptoms**:
- Slow or missing "go to definition" results for Python files
- Delayed or incomplete symbol references
- General responsiveness issues with Python code analysis

**Solutions**:

1. **Use auto-restart feature** (recommended):
   ```json
   {
     "servers": [
       {
         "extensions": ["py", "pyi"],
         "command": ["pylsp"],
         "restartInterval": 30
       }
     ]
   }
   ```

2. **Manual restart when needed**:
   ```bash
   # Restart Python server specifically
   restart_server(extensions=["py"])
   
   # Restart all servers
   restart_server()
   ```

3. **Alternative Python servers**:
   ```bash
   # Try pyright instead of pylsp
   npm install -g pyright
   ```
   ```json
   {
     "servers": [
       {
         "extensions": ["py"],
         "command": ["pyright-langserver", "--stdio"]
       }
     ]
   }
   ```

## 🔧 Common Configuration Issues

### LSP Server Not Starting

**Error**: "Language server for [extension] is not available"

**Causes & Solutions**:

1. **Server not installed**:
   ```bash
   # Check if server is available
   which typescript-language-server
   which pylsp
   which gopls
   
   # Install missing servers
   npm install -g typescript-language-server
   pip install python-lsp-server
   go install golang.org/x/tools/gopls@latest
   ```

2. **Server not in PATH**:
   ```bash
   # Add to PATH (add to ~/.bashrc or ~/.zshrc)
   export PATH="$PATH:~/.npm-global/bin"
   export PATH="$PATH:$(go env GOPATH)/bin" 
   export PATH="$PATH:~/.cargo/bin"
   ```

3. **Incorrect command in configuration**:
   ```json
   {
     // Wrong
     "command": ["typescript-language-server"],
     
     // Correct
     "command": ["npx", "--", "typescript-language-server", "--stdio"]
   }
   ```

### Configuration Not Loading

**Problem**: codebuddy uses only default TypeScript configuration

**Solutions**:

1. **Check configuration file location**:
   ```bash
   # Verify file exists and has correct name
   ls -la codebuddy.json
   ls -la ~/.config/claude/codebuddy.json
   ```

2. **Verify CODEBUDDY_CONFIG_PATH**:
   ```bash
   # Check environment variable
   echo $CODEBUDDY_CONFIG_PATH
   
   # Use absolute path
   export CODEBUDDY_CONFIG_PATH="/absolute/path/to/codebuddy.json"
   ```

3. **Validate JSON syntax**:
   ```bash
   # Check JSON validity
   cat codebuddy.json | python -m json.tool
   
   # Or use jq
   jq . codebuddy.json
   ```

4. **Debug configuration loading**:
   ```bash
   # Enable debug output
   export CODEBUDDY_DEBUG=1
   codebuddy
   ```

### Symbol Not Found Errors

**Problem**: "Go to definition" returns no results or "No symbols found"

**Causes & Solutions**:

1. **File not saved or indexed**:
   - Ensure file is saved
   - Wait a few seconds for LSP server to index project
   - Large projects may take longer to index

2. **Wrong file extension mapping**:
   ```json
   {
     "servers": [
       {
         // Make sure all relevant extensions are included
         "extensions": ["py", "pyi", "pyw"],  // Add .pyi for Python
         "command": ["pylsp"]
       }
     ]
   }
   ```

3. **LSP server doesn't support the file type**:
   ```bash
   # Check server capabilities
   codebuddy --debug-capabilities
   ```

4. **Project structure issues**:
   - Ensure files are in project root or have proper imports
   - Check if language server needs workspace configuration (tsconfig.json, pyproject.toml, etc.)

## 🐛 Debugging Techniques

### Enable Debug Output

```bash
# Enable verbose logging
export CODEBUDDY_DEBUG=1
export CODEBUDDY_TRACE=1
codebuddy

# Debug specific components
export CODEBUDDY_DEBUG_LSP=1      # LSP communication
export CODEBUDDY_DEBUG_MCP=1      # MCP protocol
export CODEBUDDY_DEBUG_TOOLS=1    # Tool execution
```

### Check Server Status

```bash
# List running LSP servers
ps aux | grep -E "(typescript-language-server|pylsp|gopls|rust-analyzer)"

# Check server logs (if available)
tail -f ~/.cache/codebuddy/logs/server.log
```

### Manual LSP Server Testing

```bash
# Test TypeScript server directly
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":"file:///path/to/project"}}' | npx typescript-language-server --stdio

# Test Python server directly  
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":"file:///path/to/project"}}' | pylsp
```

### Verify Tool Functionality

```bash
# Test individual MCP tools
codebuddy --test-tool find_definition --file test.py --symbol main
codebuddy --test-tool get_diagnostics --file test.py
codebuddy --test-tool restart_server --extensions py
```

## 🔄 Recovery Procedures

### Reset All Servers

```bash
# Stop all LSP servers and clear cache
pkill -f "typescript-language-server"
pkill -f "pylsp"
pkill -f "gopls" 
pkill -f "rust-analyzer"

# Clear any cached state
rm -rf ~/.cache/codebuddy/

# Restart codebuddy
codebuddy
```

### Reset Configuration

```bash
# Back up current config
cp codebuddy.json codebuddy.json.backup

# Generate fresh configuration
codebuddy init --overwrite

# Or use interactive setup
codebuddy setup --reset
```

### Manual Server Restart

Using MCP tools:
```bash
# Restart specific language servers
restart_server(extensions=["ts", "tsx"])     # TypeScript
restart_server(extensions=["py"])            # Python  
restart_server(extensions=["rs"])            # Rust
restart_server()                             # All servers
```

## 🚀 Performance Issues

### Slow Symbol Resolution

**Causes & Solutions**:

1. **Large project indexing**:
   - Add .gitignore patterns to exclude unnecessary files
   - Use project-specific exclusions in LSP server config

2. **Memory usage**:
   ```bash
   # Monitor memory usage
   ps aux | grep -E "(typescript-language-server|pylsp)" | awk '{print $2, $4, $11}'
   
   # Restart high-memory servers
   restart_server(extensions=["ts"])
   ```

3. **Enable auto-restart for problematic servers**:
   ```json
   {
     "servers": [
       {
         "extensions": ["py"],
         "command": ["pylsp"],
         "restartInterval": 20  // Restart every 20 minutes
       }
     ]
   }
   ```

### High CPU Usage

```bash
# Identify resource-heavy servers
top -p $(pgrep -d, -f "language-server|pylsp|gopls")

# Disable expensive features
{
  "initializationOptions": {
    "settings": {
      "pylsp": {
        "plugins": {
          "rope_autoimport": { "enabled": false },
          "pylint": { "enabled": false }
        }
      }
    }
  }
}
```

## 🔐 Permission Issues

### NPM Global Package Permissions

```bash
# Fix npm permissions
npm config set prefix ~/.npm-global
echo 'export PATH="$PATH:~/.npm-global/bin"' >> ~/.bashrc
source ~/.bashrc

# Install packages globally
npm install -g typescript-language-server
```

### Python Package Permissions

```bash
# Install with --user flag
pip install --user python-lsp-server

# Or use pipx for isolated installs
pipx install python-lsp-server
```

### File System Permissions

```bash
# Fix config directory permissions
mkdir -p ~/.config/claude
chmod 755 ~/.config/claude
chmod 644 ~/.config/claude/codebuddy.json
```

## 📱 Platform-Specific Issues

### Windows

```powershell
# Use full paths in configuration
{
  "servers": [
    {
      "extensions": ["py"],
      "command": ["C:\\Python39\\Scripts\\pylsp.exe"]
    }
  ]
}

# Check PATH
echo $env:PATH | Select-String python
```

### macOS

```bash
# Fix command line tools
xcode-select --install

# Homebrew PATH issues
echo 'export PATH="/opt/homebrew/bin:$PATH"' >> ~/.zshrc

# Python PATH on macOS
export PATH="$PATH:$(python3 -m site --user-base)/bin"
```

### Linux

```bash
# Missing build dependencies
sudo apt-get install build-essential python3-dev

# SELinux issues (if applicable)
setsebool -P allow_execheap 1
```

## 🔄 Version Compatibility

### Check Versions

```bash
# Check codebuddy version
codebuddy --version

# Check language server versions
typescript-language-server --version
pylsp --version
gopls version
rust-analyzer --version
```

### Update Everything

```bash
# Update codebuddy
npm install -g @goobits/codebuddy@latest

# Update language servers
npm update -g typescript-language-server
pip install --upgrade python-lsp-server
go install golang.org/x/tools/gopls@latest
rustup update
```

## 🆘 Getting Help

### Before Reporting Issues

1. **Enable debug logging**: `export CODEBUDDY_DEBUG=1`
2. **Check configuration**: `codebuddy --debug-config`
3. **Test individual servers**: Manual LSP server testing (see above)
4. **Check system resources**: Memory, CPU usage
5. **Try minimal configuration**: Test with single language server

### Report Issues

Include this information:

- **codebuddy version**: `codebuddy --version`
- **Operating system**: `uname -a` (Linux/macOS) or Windows version
- **Node.js version**: `node --version`
- **Language server versions**: Output of version commands
- **Configuration file**: Contents of `codebuddy.json`
- **Debug output**: With `CODEBUDDY_DEBUG=1`
- **Steps to reproduce**: Exact sequence of actions

### Community Support

- **GitHub Issues**: [https://github.com/ktnyt/codebuddy/issues](https://github.com/ktnyt/codebuddy/issues)
- **GitHub Discussions**: [https://github.com/ktnyt/codebuddy/discussions](https://github.com/ktnyt/codebuddy/discussions)
- **Stack Overflow**: Tag your question with `codebuddy` and `language-server-protocol`

### Emergency Recovery

If codebuddy becomes completely unresponsive:

```bash
# Nuclear option: Kill everything and start fresh
pkill -f codebuddy
pkill -f "language-server"
pkill -f pylsp
pkill -f gopls
pkill -f rust-analyzer
rm -rf ~/.cache/codebuddy/
rm codebuddy.json

# Reinstall and reconfigure
npm uninstall -g @goobits/codebuddy
npm install -g @goobits/codebuddy@latest
codebuddy setup
```