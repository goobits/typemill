# Proposal 17: CI/CD Automated Path 🤖

## Status
**Proposed** - Onboarding test path for automated environments

## Overview
Test if Codebuddy can be installed and used in fully automated CI/CD pipelines with ZERO human interaction.

## Persona: GitHub Actions Runner / CI Bot

**Profile:**
- 🤖 Automated CI environment (GitHub Actions, GitLab CI, Jenkins, etc.)
- 🚫 No TTY (non-interactive terminal)
- 🚫 Cannot answer prompts or make decisions
- 🚫 No persistent state between runs
- ✅ Can run shell scripts and Docker containers
- ✅ Can cache dependencies
- ✅ Needs deterministic, repeatable builds

**Starting Point:**
- Fresh Ubuntu/Debian VM or Docker container
- No tools installed (must install everything)
- Environment variables available
- Network access available
- Runs as non-root user (best practice)

## Goal
Fully automated install + usage with ZERO human interaction. All operations must be scriptable and idempotent.

## Test Phases

### Phase 1: Non-Interactive Installation (5 min)
**Goal:** Install Codebuddy without any prompts

**Tasks:**
- [ ] Install Rust (if not in base image)
- [ ] Install Codebuddy via install script or cargo
- [ ] Verify installation
- [ ] No prompts, confirmations, or user input

**Success Metrics:**
- Installs successfully in fresh environment
- No interactive prompts (all flags provided)
- Exit code 0 on success
- Binary available in PATH

**Implementation Approaches:**

**Option A: Install Script with Auto-Yes**
```bash
curl -fsSL https://raw.githubusercontent.com/goobits/codebuddy/main/install.sh | bash -s -- --non-interactive
```

**Option B: Cargo Install**
```bash
cargo install codebuddy --locked
```

**Option C: Pre-built Binary**
```bash
curl -L -o codebuddy https://github.com/goobits/codebuddy/releases/latest/download/codebuddy-linux-x86_64
chmod +x codebuddy
sudo mv codebuddy /usr/local/bin/
```

**Common Friction Points:**
- Install script requires user confirmation
- Rust installation needs PATH updates (requires shell restart)
- Permission errors (trying to write to /usr/local/bin without sudo)
- No flag to skip confirmations

### Phase 2: Automated Configuration (3 min)
**Goal:** Generate config without interactive setup wizard

**Tasks:**
- [ ] Create `.codebuddy/config.json` programmatically
- [ ] Install required language servers non-interactively
- [ ] Validate config
- [ ] No prompts or wizards

**Success Metrics:**
- Config created without human input
- Valid JSON generated
- Language servers installed
- Exit codes correct

**Implementation Approaches:**

**Option A: Environment Variable Config**
```bash
export CODEBUDDY_LANGUAGES="typescript,rust"
codebuddy setup --auto --no-prompt
```

**Option B: Template Config**
```bash
cat > .codebuddy/config.json <<'EOF'
{
  "servers": [
    {
      "extensions": ["ts", "tsx", "js", "jsx"],
      "command": ["typescript-language-server", "--stdio"]
    }
  ]
}
EOF
```

**Option C: Config from File**
```bash
codebuddy setup --config-file ci-config.json
```

**Common Friction Points:**
- `codebuddy setup` requires interactive prompts
- No way to pass config via environment variables
- No validation of programmatically-created config
- Unclear which language servers to install

### Phase 3: GitHub Actions Workflow (10 min)
**Goal:** Complete workflow that installs, configures, and uses Codebuddy

**Tasks:**
- [ ] Create `.github/workflows/codebuddy-ci.yml`
- [ ] Install dependencies
- [ ] Configure Codebuddy
- [ ] Run MCP tools via CLI
- [ ] Cache for performance
- [ ] Report results

**Success Metrics:**
- Workflow runs without manual approval
- Completes in < 10 minutes
- Succeeds on every run (deterministic)
- Proper caching (subsequent runs < 5 min)

**Example Workflow:**
```yaml
name: Codebuddy CI

on: [push, pull_request]

jobs:
  test-codebuddy:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: ~/.cargo
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install Codebuddy
        run: |
          cargo install codebuddy --locked
          codebuddy --version

      - name: Install Language Servers
        run: |
          npm install -g typescript-language-server typescript
          rustup component add rust-analyzer

      - name: Configure Codebuddy
        run: |
          mkdir -p .codebuddy
          cat > .codebuddy/config.json <<'EOF'
          {
            "servers": [
              {
                "extensions": ["ts", "tsx", "js", "jsx"],
                "command": ["typescript-language-server", "--stdio"]
              },
              {
                "extensions": ["rs"],
                "command": ["rust-analyzer"]
              }
            ]
          }
          EOF

      - name: Run Codebuddy Tools
        run: |
          # Example: Find all TypeScript symbols
          codebuddy tool search_workspace_symbols --query "function"

          # Example: Get diagnostics for all files
          codebuddy tool get_diagnostics --file-path "src/main.ts"

      - name: Verify Results
        run: |
          codebuddy status
          codebuddy doctor
```

**Common Friction Points:**
- No CLI interface for MCP tools (only stdio server)
- Cannot run tools in one-shot mode
- Unclear how to use non-interactively
- Poor error messages in CI context

### Phase 4: Docker CI Workflow (10 min)
**Goal:** Run in Docker container (simulates GitLab CI, Jenkins)

**Tasks:**
- [ ] Create `Dockerfile.ci`
- [ ] Multi-stage build for efficiency
- [ ] Install and configure Codebuddy
- [ ] Run tools and validate
- [ ] Keep image size reasonable (< 500MB)

**Success Metrics:**
- Docker build succeeds
- Image size acceptable
- Tools work in container
- Can run as non-root user

**Example Dockerfile:**
```dockerfile
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Codebuddy
RUN cargo install codebuddy --locked

FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install language servers
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g typescript-language-server typescript

# Copy Codebuddy from builder
COPY --from=builder /usr/local/cargo/bin/codebuddy /usr/local/bin/

# Create non-root user
RUN useradd -m -s /bin/bash codebuddy
USER codebuddy
WORKDIR /workspace

# Default config
COPY ci-config.json /workspace/.codebuddy/config.json

CMD ["codebuddy", "start"]
```

**Common Friction Points:**
- Large image size
- Missing runtime dependencies
- Permission issues (root vs non-root)
- No official Docker image provided

### Phase 5: Non-Interactive Tool Execution (5 min)
**Goal:** Run MCP tools from CLI without starting server

**Tasks:**
- [ ] Execute tools in one-shot mode
- [ ] Parse JSON output in scripts
- [ ] Handle errors properly
- [ ] Chain multiple tool calls

**Success Metrics:**
- Tools run without starting long-lived server
- JSON output is parseable
- Exit codes reflect success/failure
- Can pipe output to other tools

**Example Commands:**
```bash
# Find definition
codebuddy tool find_definition \
  --file-path src/main.ts \
  --line 10 \
  --character 5 \
  --format json

# Find references
codebuddy tool find_references \
  --file-path src/main.ts \
  --line 10 \
  --character 5 \
  --include-declaration \
  --format json

# Rename symbol (dry run in CI)
codebuddy tool rename_symbol \
  --file-path src/main.ts \
  --line 10 \
  --character 5 \
  --new-name "newFunctionName" \
  --dry-run true \
  --format json
```

**Common Friction Points:**
- No one-shot CLI mode (only stdio server)
- Cannot run tools without MCP client
- Poor JSON output for scripting
- No dry-run flag for safety

### Phase 6: Caching and Performance (5 min)
**Goal:** Optimize for repeated CI runs

**Tasks:**
- [ ] Cache cargo installation
- [ ] Cache language servers
- [ ] Cache Codebuddy config
- [ ] Measure first run vs cached run

**Success Metrics:**
- First run: < 10 min
- Cached run: < 3 min
- Cache hit rate > 90%
- Deterministic results

**Caching Strategy:**
```yaml
# GitHub Actions
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/codebuddy
      ~/.cargo/registry
      node_modules
      .codebuddy/cache
    key: codebuddy-${{ runner.os }}-${{ hashFiles('**/package-lock.json', '**/Cargo.lock') }}
```

**Common Friction Points:**
- No cache directory documented
- Unclear what to cache
- Cache invalidation issues
- No performance benchmarks

## Success Criteria

### Platform Support
Must work on **at least 2 of these platforms:**
- ✅ GitHub Actions (Ubuntu runner)
- ✅ GitLab CI (Docker-based)
- ✅ Docker (for any CI system)
- ⚠️ Jenkins (if time permits)

### Time Limits
- **First run (no cache):** < 10 minutes
- **Cached run:** < 3 minutes
- **Tool execution:** < 5 seconds per tool

### Automation Requirements
- ✅ No interactive prompts
- ✅ All flags/env vars documented
- ✅ Exit codes correct (0 = success, non-zero = error)
- ✅ JSON output for parsing
- ✅ Deterministic (same input = same output)

### Key Questions
1. Can it install with zero human intervention?
2. Are all prompts disableable via flags?
3. Does it work in Docker/containerized environments?
4. Are errors clear and actionable in CI logs?
5. Is caching strategy documented?

## Documentation Gaps to Identify

This test should reveal:
- [ ] Missing non-interactive installation flags
- [ ] No CI/CD examples in README
- [ ] Unclear how to run tools from CLI (non-server mode)
- [ ] Missing Docker image or Dockerfile
- [ ] No environment variable reference
- [ ] Poor error messages in headless environments
- [ ] Missing exit code documentation
- [ ] No caching recommendations

## Test Execution Plan

### Test Environments
1. **GitHub Actions** (free tier, Ubuntu 22.04)
2. **Docker** (local, simulates GitLab CI)
3. **Fresh VM** (DigitalOcean droplet, simulate Jenkins)

### Test Protocol
1. **Create workflows** from scratch (no copy-paste from docs)
2. **Run multiple times** to test caching
3. **Inject failures** to test error handling
4. **Measure performance** at each step
5. **Document friction** in real-time

### Data Collection
- Time per phase (fresh vs cached)
- Cache hit rate
- Number of interactive prompts encountered
- Error messages that blocked automation
- Workarounds needed

## Deliverables

1. **CI/CD Guide:** Step-by-step for GitHub Actions, GitLab CI, Docker
2. **Example Workflows:** Copy-paste ready templates
3. **Dockerfile:** Official image for CI use
4. **Environment Variable Reference:** All non-interactive flags
5. **Troubleshooting Guide:** CI-specific issues
6. **Performance Benchmarks:** Expected timing for CI runs

## Timeline
- Create workflows: 2 days
- Test on 3 platforms: 3 days
- Document findings: 1 day
- Write CI guide: 2 days
- Create Docker image: 1 day

**Total:** ~1.5 weeks

## Proposed Features for CI/CD

### Environment Variables
```bash
CODEBUDDY_NON_INTERACTIVE=true    # No prompts
CODEBUDDY_CONFIG_FILE=path        # Config from file
CODEBUDDY_LANGUAGES=ts,rust       # Auto-detect skip
CODEBUDDY_LOG_FORMAT=json         # Machine-readable logs
```

### CLI Flags
```bash
codebuddy setup --non-interactive --languages typescript,rust
codebuddy tool <name> --format json --exit-on-error
codebuddy start --stdio --log-level error
```

### Docker Image
```bash
docker run --rm -v $(pwd):/workspace ghcr.io/goobits/codebuddy:latest \
  tool find_definition --file-path src/main.ts --line 10 --character 5
```

## Agent Assignment
**Agent 3:** Run automation test, verify CI works on ≥2 platforms, create CI/CD documentation and templates
