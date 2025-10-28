# Mill Setup Guide for Teams

**Complete guide to setting up Mill for collaborative development**

---

## 🚀 Quick Start

### First-Time Setup

```bash
# 1. Run interactive setup
mill setup --interactive

# 2. Install detected LSP servers
# (setup will prompt you)

# 3. Verify installation
mill doctor

# 4. Test with a simple query
mill tool health_check '{}'
```

### Updating Existing Setup

```bash
# Re-run setup on existing config
mill setup --update

# Or combine with interactive mode
mill setup --update --interactive
```

---

## 📁 Configuration Strategies

### Portable Configuration (Recommended for Teams)

**Best for:** Teams, shared repositories, CI/CD

**Strategy:**
- ✅ Use relative paths for LSP commands (`typescript-language-server`)
- ✅ Use relative paths for `rootDir` (`web`, not `/home/user/project/web`)
- ✅ Commit `.typemill/config.json` to version control
- ✅ Document PATH requirements in project README

**Example config:**
```json
{
  "lsp": {
    "servers": [
      {
        "extensions": ["ts", "tsx", "js", "jsx"],
        "command": ["typescript-language-server", "--stdio"],
        "rootDir": "web"
      }
    ]
  }
}
```

**Team README should include:**
```markdown
## Setup Requirements

Ensure these LSP servers are installed and in your PATH:
- `typescript-language-server` - `npm install -g typescript-language-server`
- `rust-analyzer` - `rustup component add rust-analyzer`
```

---

### Local Configuration (Single Developer)

**Best for:** Personal projects, local experimentation

**Strategy:**
- ✅ Use absolute paths for LSP commands (`/usr/local/bin/typescript-language-server`)
- ✅ Use absolute paths for `rootDir` (`/home/user/project/web`)
- ✅ Add `.typemill/config.json` to `.gitignore`

**Example config:**
```json
{
  "lsp": {
    "servers": [
      {
        "extensions": ["ts", "tsx"],
        "command": ["/home/user/.nvm/versions/node/v20.0.0/bin/typescript-language-server", "--stdio"],
        "rootDir": "/home/user/projects/myapp/web"
      }
    ]
  }
}
```

---

## 🔧 Language-Specific Setup

### TypeScript / JavaScript

**Why `rootDir` matters:**
TypeScript LSP needs to find `node_modules/typescript` and `tsconfig.json`. The `rootDir` tells it where to look.

**Auto-detection:**
```bash
mill setup --update  # Detects TypeScript projects automatically
```

**Manual configuration:**
```json
{
  "extensions": ["ts", "tsx", "js", "jsx"],
  "command": ["typescript-language-server", "--stdio"],
  "rootDir": "web"  // ← Directory containing package.json or tsconfig.json
}
```

**Monorepos with multiple TypeScript projects:**
```json
{
  "lsp": {
    "servers": [
      {
        "extensions": ["ts"],
        "command": ["typescript-language-server", "--stdio"],
        "rootDir": "packages/frontend"
      },
      {
        "extensions": ["tsx"],
        "command": ["typescript-language-server", "--stdio"],
        "rootDir": "packages/backend"
      }
    ]
  }
}
```

---

### Rust

**Simple projects:**
```json
{
  "extensions": ["rs"],
  "command": ["rust-analyzer"],
  "rootDir": "."  // ← Directory containing Cargo.toml
}
```

**Workspaces:**
```json
{
  "extensions": ["rs"],
  "command": ["rust-analyzer"],
  "rootDir": "."  // ← Root workspace directory
}
```

Rust analyzer automatically discovers workspace members from `Cargo.toml`.

---

### Python

```json
{
  "extensions": ["py"],
  "command": ["pylsp"],
  "rootDir": "."
}
```

For virtual environments, ensure `pylsp` is installed in the venv and activated.

---

## 🛠️ Adding LSP Binaries to PATH

### macOS / Linux

**Option 1: Shell profile (bash/zsh)**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.nvm/versions/node/v20.0.0/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"

# Reload
source ~/.bashrc  # or source ~/.zshrc
```

**Option 2: System-wide (requires sudo)**
```bash
sudo ln -s /path/to/typescript-language-server /usr/local/bin/
```

---

### Windows

**Option 1: System Environment Variables**
1. Open "Edit system environment variables"
2. Click "Environment Variables"
3. Edit "Path" in User or System variables
4. Add LSP binary directories

**Option 2: PowerShell profile**
```powershell
# Add to $PROFILE
$env:PATH += ";C:\Users\YourName\AppData\Roaming\npm"
```

---

## 📋 Common Configurations

### TypeScript Monorepo
```json
{
  "lsp": {
    "servers": [
      {
        "extensions": ["ts", "tsx", "js", "jsx"],
        "command": ["typescript-language-server", "--stdio"],
        "rootDir": "."
      }
    ]
  }
}
```

### Rust Workspace
```json
{
  "lsp": {
    "servers": [
      {
        "extensions": ["rs"],
        "command": ["rust-analyzer"],
        "rootDir": ".",
        "restartInterval": 15
      }
    ]
  }
}
```

### Full-Stack (TypeScript + Rust + Python)
```json
{
  "lsp": {
    "servers": [
      {
        "extensions": ["ts", "tsx", "js", "jsx"],
        "command": ["typescript-language-server", "--stdio"],
        "rootDir": "web"
      },
      {
        "extensions": ["rs"],
        "command": ["rust-analyzer"],
        "rootDir": "."
      },
      {
        "extensions": ["py"],
        "command": ["pylsp"],
        "rootDir": "scripts"
      }
    ]
  }
}
```

---

## 🔍 Verifying Setup

```bash
# Check configuration and LSP availability
mill doctor

# Check server status
mill status

# Test with a simple tool call
mill tool health_check '{}'
```

---

## ❓ Troubleshooting

See **[troubleshooting.md](troubleshooting.md)** for common issues and solutions.

---

## 📚 Next Steps

- **[cheatsheet.md](cheatsheet.md)** - Quick command reference
- **[tools/README.md](tools/README.md)** - Browse all 28 tools
- **[tools/refactoring.md](tools/refactoring.md)** - Advanced refactoring patterns
