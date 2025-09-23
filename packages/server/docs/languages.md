# 🌐 Language Setup Guide

Complete installation instructions for 15+ supported languages.

## 🚀 Quick Install Commands

### TypeScript/JavaScript
```bash
# Global installation (recommended)
npm install -g typescript-language-server typescript

# Alternative: npx usage (bundled with codebuddy by default)
npx typescript-language-server --stdio
```

### Python
```bash
# Full installation with all plugins
pip install "python-lsp-server[all]"

# Basic installation
pip install python-lsp-server

# Alternative: using uv
uv tool install python-lsp-server
uvx --from python-lsp-server pylsp
```

### Go
```bash
# Install gopls
go install golang.org/x/tools/gopls@latest

# Ensure GOPATH/bin is in PATH
export PATH=$PATH:$(go env GOPATH)/bin
```

### Rust
```bash
# Install rust-analyzer via rustup
rustup component add rust-analyzer
rustup component add rust-src

# Alternative: standalone binary
curl -L https://github.com/rust-lang/rust-analyzer/releases/latest/download/rust-analyzer-x86_64-unknown-linux-gnu.gz | gunzip -c - > ~/.local/bin/rust-analyzer
chmod +x ~/.local/bin/rust-analyzer
```

### C/C++
```bash
# Ubuntu/Debian
sudo apt install clangd

# macOS
brew install llvm
# Add to PATH: export PATH="/opt/homebrew/opt/llvm/bin:$PATH"

# Windows: Download from LLVM releases
# https://github.com/llvm/llvm-project/releases
```

### Ruby
```bash
# Install Solargraph
gem install solargraph

# For bundler projects
bundle add solargraph --group=development
```

### PHP
```bash
# Install Intelephense
npm install -g intelephense

# Alternative: PHP Language Server
composer global require phpactor/phpactor
```

### Java
```bash
# Eclipse JDT Language Server
# Download from: https://download.eclipse.org/jdtls/milestones/

# Alternative: via VS Code extension
code --install-extension redhat.java
```

### C#
```bash
# OmniSharp
dotnet tool install --global OmniSharp.Http

# Alternative: via VS Code extension  
code --install-extension ms-dotnettools.csharp
```

### Swift
```bash
# Install Xcode (macOS)
xcode-select --install

# Linux: Build from source
git clone https://github.com/apple/sourcekit-lsp.git
cd sourcekit-lsp
swift build -c release
```

## 📋 Configuration Examples

### TypeScript/JavaScript
```json
{
  "servers": [
    {
      "extensions": ["js", "ts", "jsx", "tsx"],
      "command": ["npx", "--", "typescript-language-server", "--stdio"],
      "rootDir": ".",
      "initializationOptions": {
        "preferences": {
          "includeInlayParameterNameHints": "all",
          "includeInlayVariableTypeHints": true
        }
      }
    }
  ]
}
```

### Python with Custom Settings
```json
{
  "servers": [
    {
      "extensions": ["py", "pyi"],
      "command": ["pylsp"],
      "rootDir": ".",
      "restartInterval": 30,
      "initializationOptions": {
        "settings": {
          "pylsp": {
            "plugins": {
              "jedi_completion": { "enabled": true },
              "jedi_definition": { "enabled": true },
              "jedi_hover": { "enabled": true },
              "jedi_references": { "enabled": true },
              "jedi_signature_help": { "enabled": true },
              "jedi_symbols": { "enabled": true },
              "pylint": { "enabled": false },
              "pycodestyle": { "enabled": false },
              "pyflakes": { "enabled": false }
            }
          }
        }
      }
    }
  ]
}
```

### Go
```json
{
  "servers": [
    {
      "extensions": ["go"],
      "command": ["gopls"],
      "rootDir": ".",
      "initializationOptions": {
        "usePlaceholders": true,
        "completeUnimported": true,
        "staticcheck": true
      }
    }
  ]
}
```

### Rust
```json
{
  "servers": [
    {
      "extensions": ["rs"],
      "command": ["rust-analyzer"],
      "rootDir": ".",
      "initializationOptions": {
        "cargo": {
          "buildScripts": {
            "enable": true
          }
        },
        "procMacro": {
          "enable": true
        }
      }
    }
  ]
}
```

### Multi-Language Configuration
```json
{
  "servers": [
    {
      "extensions": ["py", "pyi"],
      "command": ["pylsp"],
      "restartInterval": 30
    },
    {
      "extensions": ["js", "ts", "jsx", "tsx"],
      "command": ["npx", "--", "typescript-language-server", "--stdio"]
    },
    {
      "extensions": ["go"],
      "command": ["gopls"]
    },
    {
      "extensions": ["rs"],
      "command": ["rust-analyzer"]
    },
    {
      "extensions": ["c", "cpp", "cc", "h", "hpp"],
      "command": ["clangd"]
    },
    {
      "extensions": ["rb"],
      "command": ["solargraph", "stdio"]
    },
    {
      "extensions": ["php"],
      "command": ["intelephense", "--stdio"]
    }
  ]
}
```

## 🔧 Advanced Language Server Options

### Java (Eclipse JDT LS)
```json
{
  "servers": [
    {
      "extensions": ["java"],
      "command": [
        "java",
        "-Declipse.application=org.eclipse.jdt.ls.core.id1",
        "-Dosgi.bundles.defaultStartLevel=4",
        "-Declipse.product=org.eclipse.jdt.ls.core.product",
        "-jar",
        "/path/to/jdt-language-server/plugins/org.eclipse.equinox.launcher_*.jar",
        "-configuration",
        "/path/to/jdt-language-server/config_linux",
        "-data",
        "/path/to/workspace"
      ],
      "rootDir": "."
    }
  ]
}
```

### C# (OmniSharp)
```json
{
  "servers": [
    {
      "extensions": ["cs"],
      "command": ["omnisharp", "-lsp"],
      "rootDir": ".",
      "initializationOptions": {
        "FormattingOptions": {
          "EnableEditorConfigSupport": true,
          "OrganizeImports": true
        }
      }
    }
  ]
}
```

### Lua
```json
{
  "servers": [
    {
      "extensions": ["lua"],
      "command": ["lua-language-server"],
      "rootDir": ".",
      "initializationOptions": {
        "Lua": {
          "runtime": {
            "version": "LuaJIT"
          },
          "diagnostics": {
            "globals": ["vim"]
          }
        }
      }
    }
  ]
}
```

## 🛠️ Installation Verification

After installing language servers, verify they work:

```bash
# Test TypeScript server
echo 'const x: number = 42;' | npx typescript-language-server --stdio

# Test Python server
python -c "import sys; print('Python LSP ready')" && pylsp --help

# Test Go server
go version && gopls version

# Test Rust server  
rustc --version && rust-analyzer --version
```

## 🔍 Troubleshooting Installation

### Common Issues

**Command not found**
```bash
# Check if language server is in PATH
which typescript-language-server
which pylsp  
which gopls
which rust-analyzer

# Add to PATH if needed (add to ~/.bashrc or ~/.zshrc)
export PATH="$PATH:/path/to/language/server"
```

**Permission errors**
```bash
# Fix npm permissions
npm config set prefix ~/.npm-global
export PATH="$PATH:~/.npm-global/bin"

# Fix Python pip permissions
pip install --user python-lsp-server
```

**Version conflicts**
```bash
# Check versions
typescript-language-server --version
pylsp --version

# Update to latest
npm update -g typescript-language-server
pip install --upgrade python-lsp-server
```

### Performance Optimization

**Python LSP (pylsp) optimization:**
- Disable unused plugins in configuration
- Use `restartInterval: 30` for long-running sessions
- Consider using pyright instead: `npm install -g pyright`

**TypeScript performance:**
- Enable `skipLibCheck: true` in tsconfig.json
- Use project references for large codebases
- Exclude node_modules and build directories

## 📦 Language Server Ecosystem

### Alternative Servers

**Python:**
- **pylsp** (recommended) - Full-featured, plugin-based
- **pyright** - Fast, type-focused from Microsoft
- **jedi-language-server** - Lightweight, Jedi-based

**JavaScript/TypeScript:**
- **typescript-language-server** (recommended) - Official TypeScript support
- **vscode-langservers-extracted** - Extracted from VS Code

**C/C++:**
- **clangd** (recommended) - LLVM-based, fast
- **ccls** - Alternative with different features

### Community Servers

Many more languages have community LSP servers:

- **Kotlin** - kotlin-language-server
- **Scala** - metals
- **Haskell** - haskell-language-server
- **Elixir** - elixir-ls
- **Dart** - dart language server (built-in)
- **OCaml** - ocaml-lsp-server

Use `codebuddy setup` to see the full list with installation instructions.