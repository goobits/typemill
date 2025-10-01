# Proposal: Unify Configuration

## 1. Problem

Configuration for the Codebuddy project is currently scattered across multiple files and formats, including `.codebuddy/config.json`, `docker/config/*.json`, and potentially others. 

- **Complexity:** This fragmentation makes it difficult for new users to understand how to configure the system.
- **Inconsistency:** There is no single source of truth, leading to potential conflicts and management overhead.
- **Deployment Challenges:** Managing different configuration files for development, testing, and production environments is cumbersome.

## 2. Proposed Solution

We will consolidate all configuration into a single, unified file at the project root.

1.  **Adopt `codebuddy.toml`:** We will use a single `codebuddy.toml` file at the project root for all configuration. TOML is chosen for its superior readability for structured data.

2.  **Consolidate All Settings:** All settings from `lsp`, `server`, and environment-specific configurations (like those in `docker/`) will be migrated into clearly defined sections within `codebuddy.toml`.

3.  **Update Loading Logic:** The configuration loading mechanism in `cb-core` will be refactored to parse this single `codebuddy.toml` file.

4.  **Deprecate Old Files:** The legacy `.json` configuration files will be removed after their settings have been migrated.

**Example `codebuddy.toml` Structure:**

```toml
# Top-level server settings
[server]
host = "127.0.0.1"
port = 3040

# Centralized LSP server definitions
[lsp.servers.typescript]
extensions = ["ts", "tsx", "js", "jsx"]
command = ["typescript-language-server", "--stdio"]

[lsp.servers.python]
extensions = ["py"]
command = ["pylsp"]

# Environment-specific overrides
[environments.docker]
server.host = "0.0.0.0"

[environments.production]
server.host = "0.0.0.0"
# Other production-specific settings...
```

## 3. Benefits

- **Simplicity:** A single file makes configuration drastically easier to understand and manage.
- **Clarity:** Provides a clear, unambiguous source of truth for all settings.
- **Deployability:** Simplifies the process of managing configuration across different environments (development, Docker, production).
