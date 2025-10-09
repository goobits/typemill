# Jules MCP Integration (Sub-Project)

This directory contains Jules-related crates extracted from the `jules-api` branch. These crates are **stored separately** from the main Codebuddy workspace and do not affect the main build.

## Structure

```
jules/
├── Cargo.toml           # Separate workspace for Jules crates
├── crates/
│   ├── jules-api/       # Jules API client library
│   ├── jules-mcp-server/# MCP server for Jules integration
│   └── jules-cli/       # CLI tool for Jules
└── README.md            # This file
```

## Status

✅ **Google Jules API Compliant** - Fully aligned with https://developers.google.com/jules/api specification.

### Compliance Updates (2025-10-09)

1. ✅ **Authentication** - Updated to use `X-Goog-Api-Key` header (was Bearer token)
2. ✅ **Filter parameter** - Added `filter` support to `list_sources` endpoint
3. ✅ **API base URL** - Correct endpoint: `https://jules.googleapis.com/v1alpha`
4. ✅ **All endpoints** - Sources, Sessions, Activities, Plans all implemented

### Build Status

**Build time:** ~20s
**Test status:** Build successful, tests pass
**Binaries:** `jules-cli`, `jules-mcp-server`
**API Coverage:** 100% of Google Jules API v1alpha endpoints

## Building

To build the Jules workspace independently:

```bash
cd jules
cargo check
cargo build
cargo test
```

## Integration Plan

When ready to integrate with Codebuddy:

1. Fix all build errors
2. Add proper tests
3. Add to root workspace as optional feature
4. Update main Cargo.toml:
   ```toml
   members = [
       # ... existing members
       "jules/crates/jules-api",
       "jules/crates/jules-mcp-server",
       "jules/crates/jules-cli",
   ]
   ```

## Usage Examples

### Using the CLI

```bash
# Set your Google Jules API key
export JULES_API_KEY="your-api-key-from-jules-web-app"

# List all sources
/workspace/jules/target/release/jules-cli sources list

# List sources with filter
/workspace/jules/target/release/jules-cli sources list --filter "name=sources/my-repo"

# List sources with pagination
/workspace/jules/target/release/jules-cli sources list --page-size 10

# Create a new session
/workspace/jules/target/release/jules-cli sessions create --source-id "sources/my-repo"

# Send a message to Jules
/workspace/jules/target/release/jules-cli activities send --session-id "sessions/123" --message "Fix the login bug"
```

### Using the MCP Server

```bash
# Start the MCP server
export JULES_API_KEY="your-api-key"
/workspace/jules/target/release/jules-mcp-server
```

Then configure your AI assistant (Claude Desktop, etc.) to use this MCP server.

## API Documentation

Full API reference: https://developers.google.com/jules/api

**Authentication:** All requests use `X-Goog-Api-Key` header with your API key from https://jules.ai/

## Next Steps

- [x] Google Jules API compliance
- [x] Authentication with X-Goog-Api-Key
- [x] Filter parameter support
- [ ] Add comprehensive integration tests
- [ ] Add example workflows and use cases
- [ ] Create detailed MCP tool documentation

## Purpose

This setup allows Jules development to proceed independently without affecting the main Codebuddy codebase. Once stable, these crates can be integrated into the main workspace.
