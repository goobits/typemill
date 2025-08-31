# ⚙️ Configuration Reference

Complete guide to configuring cclsp for your development environment.

## 🚀 Configuration Methods

### 1. Zero Configuration (Default)
cclsp works immediately with TypeScript/JavaScript - no setup required.

```bash
# Just install and use
npm install -g cclsp
cclsp  # Starts with TypeScript support
```

### 2. Generate Configuration
Create a well-commented config tailored to your project.

```bash
# Generate config file with detected languages
cclsp init

# Interactive setup wizard
cclsp setup

# User-wide configuration
cclsp setup --user
```

### 3. Manual Configuration
Create `cclsp.json` manually for full control.

## 📋 Configuration File Location

cclsp looks for configuration in this order:

1. **Environment variable**: `CCLSP_CONFIG_PATH=/path/to/config.json`
2. **Project config**: `./cclsp.json` (current directory)
3. **User config**: `~/.config/claude/cclsp.json` (with `--user`)
4. **Default**: Built-in TypeScript configuration

## 🔧 Configuration Schema

### Basic Structure
```json
{
  "servers": [
    {
      "extensions": ["py", "pyi"],
      "command": ["pylsp"],
      "rootDir": ".",
      "restartInterval": 30,
      "initializationOptions": {}
    }
  ]
}
```

### Required Fields

**`extensions`** - Array of file extensions
```json
{
  "extensions": ["ts", "tsx", "js", "jsx"]
}
```

**`command`** - Command array to spawn LSP server
```json
{
  "command": ["npx", "--", "typescript-language-server", "--stdio"]
}
```

### Optional Fields

**`rootDir`** - Working directory for LSP server
```json
{
  "rootDir": ".",           // Current directory (default)
  "rootDir": "/absolute/path",
  "rootDir": "relative/path"
}
```

**`restartInterval`** - Auto-restart interval in minutes
```json
{
  "restartInterval": 30     // Restart every 30 minutes
}
```
- **Default**: No auto-restart (opt-in feature)
- **Recommended**: 30 minutes for Python (pylsp performance)
- **Minimum**: 1 minute

**`initializationOptions`** - LSP server initialization settings
```json
{
  "initializationOptions": {
    "settings": {
      "python": {
        "analysis": {
          "typeCheckingMode": "strict"
        }
      }
    }
  }
}
```

## 🌐 Multi-Language Configurations

### Full-Stack Development
```json
{
  "servers": [
    {
      "extensions": ["js", "ts", "jsx", "tsx"],
      "command": ["npx", "--", "typescript-language-server", "--stdio"],
      "initializationOptions": {
        "preferences": {
          "includeInlayParameterNameHints": "all"
        }
      }
    },
    {
      "extensions": ["py", "pyi"],
      "command": ["pylsp"],
      "restartInterval": 30,
      "initializationOptions": {
        "settings": {
          "pylsp": {
            "plugins": {
              "pylint": { "enabled": false },
              "pycodestyle": { "enabled": false },
              "pyflakes": { "enabled": false }
            }
          }
        }
      }
    },
    {
      "extensions": ["go"],
      "command": ["gopls"],
      "initializationOptions": {
        "usePlaceholders": true,
        "completeUnimported": true
      }
    }
  ]
}
```

### Minimal Configuration
```json
{
  "servers": [
    {
      "extensions": ["py"],
      "command": ["pylsp"]
    },
    {
      "extensions": ["rs"],  
      "command": ["rust-analyzer"]
    }
  ]
}
```

## 🛠️ Advanced Configuration

### Python with Custom Plugin Configuration
```json
{
  "servers": [
    {
      "extensions": ["py", "pyi"],
      "command": ["uvx", "--from", "python-lsp-server", "pylsp"],
      "rootDir": ".",
      "restartInterval": 30,
      "initializationOptions": {
        "settings": {
          "pylsp": {
            "configurationSources": ["pycodestyle"],
            "plugins": {
              "jedi_completion": {
                "enabled": true,
                "include_params": true
              },
              "jedi_definition": { "enabled": true },
              "jedi_hover": { "enabled": true },
              "jedi_references": { "enabled": true },
              "jedi_signature_help": { "enabled": true },
              "jedi_symbols": { "enabled": true },
              "pylint": {
                "enabled": false,
                "args": ["--generate-members"]
              },
              "pycodestyle": {
                "enabled": true,
                "ignore": ["E501", "W503"],
                "maxLineLength": 100
              },
              "pyflakes": { "enabled": true },
              "autopep8": { "enabled": false },
              "yapf": { "enabled": false },
              "rope_autoimport": { "enabled": false }
            }
          }
        }
      }
    }
  ]
}
```

### TypeScript with Strict Settings
```json
{
  "servers": [
    {
      "extensions": ["ts", "tsx"],
      "command": ["npx", "--", "typescript-language-server", "--stdio"],
      "initializationOptions": {
        "preferences": {
          "includeInlayParameterNameHints": "all",
          "includeInlayVariableTypeHints": true,
          "includeInlayFunctionParameterTypeHints": true,
          "includeInlayPropertyDeclarationTypeHints": true,
          "includeInlayFunctionLikeReturnTypeHints": true,
          "includeInlayEnumMemberValueHints": true
        },
        "suggest": {
          "includeCompletionsForModuleExports": true,
          "includeAutomaticOptionalChainCompletions": true
        }
      }
    }
  ]
}
```

### C++ with Clangd Customization
```json
{
  "servers": [
    {
      "extensions": ["cpp", "cc", "c", "h", "hpp"],
      "command": ["clangd", "--background-index", "--clang-tidy"],
      "initializationOptions": {
        "clangdFileStatus": true,
        "usePlaceholders": true,
        "completeUnimported": true,
        "semanticHighlighting": true
      }
    }
  ]
}
```

## 🔄 Auto-Restart Configuration

Auto-restart helps with long-running LSP servers that may degrade over time.

### When to Use Auto-Restart

**Recommended for:**
- **Python (pylsp)** - Known to slow down after several hours
- **Large TypeScript projects** - Memory usage can grow over time
- **Development environments** - Frequent config changes

**Not needed for:**
- **Go (gopls)** - Generally stable
- **Rust (rust-analyzer)** - Good memory management
- **Short-lived sessions** - Less than a few hours

### Configuration Examples

```json
{
  "servers": [
    {
      "extensions": ["py"],
      "command": ["pylsp"],
      "restartInterval": 30      // Every 30 minutes
    },
    {
      "extensions": ["ts"],
      "command": ["typescript-language-server", "--stdio"],
      "restartInterval": 120     // Every 2 hours
    },
    {
      "extensions": ["rs"],
      "command": ["rust-analyzer"]
      // No restartInterval - rust-analyzer is stable
    }
  ]
}
```

## 📂 Environment-Specific Configuration

### Development vs Production

**Development** (frequent changes, debugging):
```json
{
  "servers": [
    {
      "extensions": ["py"],
      "command": ["pylsp"],
      "restartInterval": 15,
      "initializationOptions": {
        "settings": {
          "pylsp": {
            "plugins": {
              "pylint": { "enabled": true },
              "pycodestyle": { "enabled": true }
            }
          }
        }
      }
    }
  ]
}
```

**Production/CI** (performance-focused):
```json
{
  "servers": [
    {
      "extensions": ["py"],
      "command": ["pylsp"],
      "initializationOptions": {
        "settings": {
          "pylsp": {
            "plugins": {
              "pylint": { "enabled": false },
              "pycodestyle": { "enabled": false }
            }
          }
        }
      }
    }
  ]
}
```

## 🔗 MCP Client Configuration

### Claude Code Integration
```json
{
  "mcpServers": {
    "cclsp": {
      "command": "cclsp",
      "env": {
        "CCLSP_CONFIG_PATH": "/absolute/path/to/cclsp.json"
      }
    }
  }
}
```

### Multiple Configurations
```json
{
  "mcpServers": {
    "cclsp-python": {
      "command": "cclsp",  
      "env": {
        "CCLSP_CONFIG_PATH": "/path/to/python-only-config.json"
      }
    },
    "cclsp-web": {
      "command": "cclsp",
      "env": {
        "CCLSP_CONFIG_PATH": "/path/to/web-config.json" 
      }
    }
  }
}
```

## ✅ Configuration Validation

### Verify Your Configuration

```bash
# Test configuration loading
cclsp --env CCLSP_CONFIG_PATH=/path/to/cclsp.json

# Interactive validation
cclsp setup --validate-only

# Check server availability
which pylsp typescript-language-server gopls
```

### Common Configuration Errors

**Invalid JSON syntax:**
```bash
# Check JSON validity
cat cclsp.json | python -m json.tool
```

**Missing language servers:**
```json
// Error: Command not found
{
  "extensions": ["py"],
  "command": ["pylsp"]  // Make sure pylsp is installed
}
```

**Incorrect paths:**
```json
// Error: File not found
{
  "extensions": ["java"],
  "command": ["/wrong/path/to/jdtls"]  // Verify path exists
}
```

## 🔧 Troubleshooting Configuration

### Debug Configuration Loading
```bash
# Enable debug output
export CCLSP_DEBUG=1
cclsp

# Check which config file is loaded
cclsp --debug-config
```

### Reset to Defaults
```bash
# Remove custom configuration
rm cclsp.json
rm ~/.config/claude/cclsp.json

# Use built-in TypeScript configuration
cclsp  # Will use defaults
```

### Performance Tuning
```json
{
  "servers": [
    {
      "extensions": ["py"],
      "command": ["pylsp"],
      "restartInterval": 20,     // Shorter interval for heavy use
      "initializationOptions": {
        "settings": {
          "pylsp": {
            "plugins": {
              "rope_autoimport": { "enabled": false },  // Disable heavy plugins
              "pylint": { "enabled": false }
            }
          }
        }
      }
    }
  ]
}
```

## 📋 Configuration Examples Repository

All configuration examples are available in the project repository:
- [examples/](../examples/) - Pre-configured setups for common use cases
- [examples/languages/](../examples/languages/) - Language-specific configurations
- [examples/frameworks/](../examples/frameworks/) - Framework-specific setups

Use these as starting points for your own configuration.