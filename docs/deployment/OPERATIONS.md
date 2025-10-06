# Operations Guide

This document provides comprehensive operational guidance for deploying, configuring, and maintaining the Rust MCP Server in production environments.

## Table of Contents

1. [Installation & Compilation](#installation--compilation)
2. [Configuration](#configuration)
3. [CLI Usage](#cli-usage)
4. [Running the Server](#running-the-server)
5. [Testing](#testing)
6. [Monitoring & Health Checks](#monitoring--health-checks)
7. [Troubleshooting](#troubleshooting)
8. [Maintenance](#maintenance)
9. [Security](#security)

## Installation & Compilation

### Prerequisites

- **Rust**: 1.70+ (Edition 2021)
- **System packages**: Required for FUSE support
  - Linux: `libfuse-dev` or `fuse3-dev`
  - macOS: `macfuse` or `osxfuse`
  - Windows: `WinFsp` (optional)

### Language Server Prerequisites

Install language servers for full functionality:

```bash
# TypeScript/JavaScript
npm install -g typescript-language-server typescript

# Python
pip install python-lsp-server

# Rust
rustup component add rust-analyzer

# Go (optional)
go install golang.org/x/tools/gopls@latest

# C/C++ (optional)
# Install clangd via your package manager
```

### Quick Installation

**Automated Script (Recommended):**
```bash
# One-command install (handles all dependencies)
curl -fsSL https://raw.githubusercontent.com/goobits/codebuddy/main/install.sh | bash
```

This script:
- Installs Rust toolchain if needed
- Clones repository
- Builds from source
- Installs to system location
- Configures PATH automatically

**Manual Compilation:**

```bash
# Clone repository
git clone https://github.com/goobits/codebuddy.git
cd codebuddy

# Build release binary
cargo build --release

# Binary location
./target/release/codebuddy
```

**Manual Installation:**

```bash
# Copy binary to system location
sudo cp target/release/codebuddy /usr/local/bin/

# Or use Makefile
make install  # Installs to ~/.local/bin
```

## Configuration

### Configuration File Location

The server loads configuration from `.codebuddy/config.json` in the current working directory.

### Minimal Configuration

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 3040
  },
  "lsp": {
    "servers": [
      {
        "name": "typescript",
        "command": ["typescript-language-server", "--stdio"],
        "extensions": ["ts", "tsx", "js", "jsx"],
        "timeout": 30
      }
    ]
  }
}
```

### Complete Configuration

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 3040,
    "max_connections": 100,
    "request_timeout": 30
  },
  "lsp": {
    "servers": [
      {
        "name": "typescript",
        "command": ["typescript-language-server", "--stdio"],
        "extensions": ["ts", "tsx", "js", "jsx"],
        "timeout": 30,
        "working_directory": ".",
        "environment": {
          "TSS_LOG": "-level verbose -file /tmp/tss.log"
        }
      },
      {
        "name": "python",
        "command": ["pylsp"],
        "extensions": ["py", "pyi"],
        "timeout": 30
      },
      {
        "name": "rust",
        "command": ["rust-analyzer"],
        "extensions": ["rs"],
        "timeout": 45
      }
    ]
  },
  "fuse": {
    "enabled": true,
    "mount_point": "/tmp/codeflow-workspace",
    "permissions": "755"
  },
  "logging": {
    "level": "info",
    "format": "json"
  }
}
```

### Configuration Validation

The server validates configuration on startup and provides detailed error messages:

```bash
# Test configuration
codebuddy --validate-config

# Example validation error
Error: Invalid LSP server configuration
  - Server 'typescript': Command 'typescript-language-server' not found in PATH
  - Server 'python': Extension 'py' is already handled by server 'python-alt'
```

## Running the Server

### WebSocket Mode (Production)

```bash
# Start WebSocket server on default port 3040
codebuddy serve

# Start with custom configuration
cd /path/to/project
codebuddy serve

# Custom host/port via config or CLI args (future)
codebuddy serve --host 0.0.0.0 --port 8080
```

### Stdio Mode (MCP Clients)

```bash
# Start stdio server for MCP protocol
codebuddy start

# Used by MCP clients like Claude Code
echo '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}' | codebuddy start
```

## CLI Usage

### Environment Variables

Configure the CLI using environment variables:

- `CODEBUDDY_URL`: WebSocket URL of the Codebuddy server (e.g., `ws://localhost:3000`)
- `CODEBUDDY_TOKEN`: Authentication token for the server

```bash
export CODEBUDDY_URL="ws://localhost:3000"
export CODEBUDDY_TOKEN="your-secret-token"
```

### Shell Completions

Generate auto-completion scripts for your shell:

**Bash:**
```bash
# Add to ~/.bashrc
source <(codebuddy completions bash)
```

**Zsh:**
```bash
# Add to ~/.zshrc
source <(codebuddy completions zsh)
```

**Fish:**
```fish
# Add to ~/.config/fish/config.fish
codebuddy completions fish | source
```

### Common Commands

#### `status`
Check client status and server connectivity:

```bash
# Basic status check
codebuddy status

# Verbose status with connection details
codebuddy status --verbose
```

#### `tool`
Execute MCP tools directly (useful for scripting):

```bash
# Read a file
codebuddy tool read_file '{"file_path":"/path/to/file.txt"}'

# List files recursively
codebuddy tool list_files '{"recursive":true}'

# Get hover information (raw JSON output)
codebuddy tool get_hover '{"file_path":"/src/index.ts","line":10,"character":5}' --format json
```

#### `connect`
Start interactive session with server:

```bash
codebuddy connect --url ws://custom-server:4000
```

### Process Management

#### Systemd Service (Linux)

Create `/etc/systemd/system/codebuddy.service`:

```ini
[Unit]
Description=Codeflow Buddy MCP Server
After=network.target

[Service]
Type=simple
User=codeflow
Group=codeflow
WorkingDirectory=/opt/codebuddy
ExecStart=/usr/local/bin/codebuddy serve
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl enable codebuddy
sudo systemctl start codebuddy

# Check status
sudo systemctl status codebuddy

# View logs
sudo journalctl -u codebuddy -f
```

#### Docker Deployment

```deployment/dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    nodejs npm \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g typescript-language-server typescript
RUN pip3 install python-lsp-server

COPY --from=builder /app/target/release/codebuddy /usr/local/bin/
COPY config.json /app/.codebuddy/config.json

WORKDIR /app
EXPOSE 3040
CMD ["codebuddy", "serve"]
```

```bash
# Build and run
deployment/docker build -t codebuddy .
deployment/docker run -p 3040:3040 -v /path/to/workspace:/workspace codebuddy
```

## Testing

### Unit Tests

```bash
# Run all unit tests
cargo test --workspace

# Run tests for specific crate
cargo test -p codebuddy

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_name --workspace
```

### Contract Tests

```bash
# Run contract tests (requires built binary)
cargo build --release
cargo test contract_tests --lib

# Expected output:
# test contract_tests::test_tools_list_contract ... ok
# test contract_tests::test_find_definition_contract ... ok
# test contract_tests::test_analyze_imports_contract ... ok
# ... (8 tests total)
```

### E2E Tests (TypeScript Test Runner)

```bash
# Ensure binary is built
cargo build --release

# Run E2E tests against Rust backend
TEST_RUST_BACKEND=true bun run test:e2e:rust

# Expected: TypeScript test client connects to Rust server
```

### Performance Testing

```bash
# Load testing with multiple concurrent requests
for i in {1..10}; do
  (echo '{"jsonrpc":"2.0","id":"'$i'","method":"tools/list","params":{}}' | codebuddy start) &
done
wait
```

## Monitoring & Health Checks

### Health Endpoints (WebSocket Mode)

```bash
# Basic health check
curl http://localhost:3040/health

# Expected response:
{
  "status": "healthy",
  "timestamp": "2023-12-07T10:30:00Z",
  "version": "0.1.0",
  "uptime_seconds": 3600
}

# Detailed health check
curl http://localhost:3040/health/detailed

# Response includes LSP server status:
{
  "status": "healthy",
  "lsp_servers": {
    "typescript": "running",
    "python": "error",
    "rust": "starting"
  },
  "fuse": {
    "mounted": true,
    "mount_point": "/tmp/codeflow-workspace"
  }
}
```

### MCP Health Tool

```bash
# Check health via MCP tool
echo '{"jsonrpc":"2.0","id":"health","method":"tools/call","params":{"name":"health_check","arguments":{}}}' | codebuddy start
```

### Log Analysis

```bash
# View structured logs
journalctl -u codebuddy -o json | jq '.MESSAGE'

# Monitor LSP server starts/failures
journalctl -u codebuddy | grep "LSP"

# Watch error patterns
journalctl -u codebuddy -f | grep "ERROR"
```

### Metrics Collection

For production monitoring, collect these metrics:

- **Request rate**: MCP requests per second
- **Response times**: 95th percentile latency
- **Error rates**: Error responses by tool type
- **LSP server health**: Start/restart frequency
- **Memory usage**: Process RSS and heap usage
- **Connection count**: Active WebSocket connections

## Troubleshooting

### Common Issues

#### 1. LSP Server Not Found

```
ERROR Failed to get LSP client for extension ts: Runtime error: Failed to start LSP server 'typescript-language-server --stdio': No such file or directory
```

**Solution:**
```bash
# Install TypeScript language server
npm install -g typescript-language-server typescript

# Verify installation
which typescript-language-server

# Update PATH in systemd service if needed
```

#### 2. FUSE Mount Failed

```
ERROR Failed to start FUSE mount: Permission denied
```

**Solutions:**
```bash
# Add user to fuse group (Linux)
sudo usermod -a -G fuse $USER

# Install FUSE development packages
# Ubuntu/Debian:
sudo apt-get install libfuse-dev
# CentOS/RHEL:
sudo yum install fuse-devel

# Verify FUSE module is loaded
lsmod | grep fuse
```

#### 3. WebSocket Connection Refused

```
Error: connection refused (os error 111)
```

**Solution:**
```bash
# Check if server is running
ps aux | grep codebuddy

# Check port binding
ss -tlnp | grep 3040

# Check firewall
sudo ufw status
sudo iptables -L

# Test local connection
curl http://localhost:3040/health
```

#### 4. High Memory Usage

**Investigation:**
```bash
# Check process memory
ps aux | grep codebuddy

# Monitor memory over time
while true; do
  ps -p $(pgrep codebuddy) -o pid,rss,vsz,pcpu,cmd
  sleep 5
done

# Check for memory leaks in LSP servers
pgrep -a "typescript-language-server\|pylsp\|rust-analyzer"
```

**Solutions:**
- Restart LSP servers periodically
- Reduce LSP server timeout values
- Monitor for LSP server memory leaks
- Consider LSP server alternatives

#### 5. Request Timeouts

```
ERROR Request timeout after 30 seconds
```

**Solutions:**
```bash
# Increase timeout in configuration
{
  "lsp": {
    "servers": [
      {
        "timeout": 60
      }
    ]
  }
}

# Check LSP server responsiveness
echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}' | timeout 10 typescript-language-server --stdio
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug codebuddy serve

# Enable LSP debug logging
export TSS_LOG="-level verbose -file /tmp/tss.log"
codebuddy serve

# Trace all system calls (Linux)
strace -e trace=network,file codebuddy serve
```

### Performance Analysis

```bash
# Profile CPU usage
perf record -g codebuddy serve
# ... run workload ...
perf report

# Memory profiling with Valgrind
valgrind --tool=massif codebuddy serve

# Analyze with heaptrack (if available)
heaptrack codebuddy serve
```

## Maintenance

### Regular Maintenance Tasks

#### Weekly

1. **Log rotation**: Ensure logs don't fill disk space
2. **Health monitoring**: Review error rates and response times
3. **LSP server updates**: Check for language server updates

#### Monthly

1. **Dependency updates**: Update Rust dependencies
2. **Security patches**: Apply system security updates
3. **Configuration review**: Validate configuration against best practices

#### Quarterly

1. **Performance review**: Analyze long-term performance trends
2. **Capacity planning**: Assess resource usage trends
3. **Backup procedures**: Test configuration backup/restore

### Updates

#### Updating the Server

```bash
# Build new version
git pull
cargo build --release

# Test new binary
./target/release/codebuddy --version

# Deploy (systemd)
sudo systemctl stop codebuddy
sudo cp target/release/codebuddy /usr/local/bin/
sudo systemctl start codebuddy
sudo systemctl status codebuddy
```

#### Updating Language Servers

```bash
# TypeScript
npm update -g typescript-language-server typescript

# Python
pip install --upgrade python-lsp-server

# Rust
rustup update

# Restart codebuddy to pick up changes
sudo systemctl restart codebuddy
```

### Backup Procedures

```bash
# Backup configuration
cp -r .codebuddy/ backup/config-$(date +%Y%m%d)/

# Backup logs (if using file logging)
tar czf logs-backup-$(date +%Y%m%d).tar.gz /var/log/codebuddy/

# Backup binary
cp /usr/local/bin/codebuddy backup/codebuddy-$(codebuddy --version)
```

## Security

### Network Security

```bash
# Bind to localhost only (default)
{
  "server": {
    "host": "127.0.0.1",
    "port": 3040
  }
}

# Use firewall for additional protection
sudo ufw allow from 127.0.0.1 to any port 3040
sudo ufw deny 3040
```

### Process Security

```bash
# Run as non-privileged user
sudo useradd -r -s /bin/false codeflow
sudo chown -R codeflow:codeflow /opt/codebuddy

# Restrict file permissions
chmod 640 .codebuddy/config.json
chmod 750 /usr/local/bin/codebuddy
```

### Input Validation

The server validates all inputs:
- JSON schema validation for MCP requests
- Path traversal prevention for file operations
- Command injection prevention (LSP commands from config only)

### Local Development Security Model

Codebuddy operates under a **local development trust model**:

- ✅ **No authentication required** - Assumes local user has legitimate access
- ✅ **OS-level permissions** - Respects filesystem permissions
- ✅ **Project-scoped operations** - All file paths anchored to project root
- ✅ **No network exposure** - stdio-based MCP protocol (no listening ports)

**Trust Boundary:** If the user's account is compromised, Codebuddy offers no additional attack surface beyond what the user already has access to (similar to git, cargo, npm).

### Validation Rollback Behavior ⚠️

**Config:** `.codebuddy/config.json`

```json
{
  "validation": {
    "enabled": true,
    "command": "cargo check",
    "on_failure": "Rollback"  // ⚠️ DANGER: Runs "git reset --hard HEAD"
  }
}
```

**⚠️ Warning:** `on_failure: "Rollback"` will **discard all uncommitted changes** if validation fails.

**Safe alternatives:**
- `"Report"` - Show errors but keep changes (recommended)
- `"Interactive"` - Prompt user to decide (requires manual action)

**Best practice:** Always commit work before risky operations when using Rollback mode.

### WebSocket Mode Security

**For Docker/CI/CD deployments:**

```json
{
  "websocket": {
    "enabled": true,
    "port": 3000
  },
  "auth": {
    "enabled": true,
    "jwt_secret": "your-secret-key-here"  // ⚠️ Use strong random value
  }
}
```

**⚠️ Important:** Only enable JWT auth for network-exposed instances. Not needed for local development.

**Without JWT auth (NOT recommended for production):**
```json
{
  "websocket": { "enabled": true },
  "auth": { "enabled": false }  // ⚠️ DANGER: Anyone can connect
}
```

### Dependency Security

Run monthly security audits:

```bash
# Check for known vulnerabilities
cargo audit

# Known issues in 1.0.0:
# - atty 0.2.14 (unmaintained, terminal detection, LOW severity)
# - paste 1.0.15 (unmaintained, compile-time only, LOW severity)
```

See [docs/security/AUDIT.md](../security/AUDIT.md) for latest audit report.

### Monitoring for Security Issues

```bash
# Monitor failed connections
journalctl -u codebuddy | grep "connection refused\|invalid request"

# Monitor suspicious file access
journalctl -u codebuddy | grep "permission denied\|access denied"

# Monitor LSP server crashes (potential DoS)
journalctl -u codebuddy | grep "LSP.*crashed\|LSP.*failed"
```

### Security Best Practices

1. **Principle of least privilege**: Run with minimal required permissions
2. **Network isolation**: Bind to localhost or use VPN/tunnels for remote access
3. **Input validation**: All user inputs are validated and sanitized
4. **Error handling**: No sensitive information leaked in error messages
5. **Process isolation**: LSP servers run as separate processes
6. **Regular updates**: Keep dependencies and language servers updated

This operations guide provides the foundation for successful deployment and maintenance of the Rust MCP Server in production environments.