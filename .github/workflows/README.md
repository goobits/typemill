# GitHub Actions Workflows

This directory contains GitHub Actions workflows for automated building, testing, and releasing.

## Manual Trigger Workflows

All workflows are configured to run **on manual trigger only** to give you full control.

### Available Workflows

#### 1. **Build All Platforms** (`build-all-platforms.yml`)
**Primary workflow for multi-platform builds**

Builds optimized release binaries for all supported platforms:
- Linux x86_64
- Linux ARM64
- macOS Intel (x86_64)
- macOS Apple Silicon (ARM64)
- Windows x86_64

**How to run:**
1. Go to **Actions** tab in GitHub
2. Select **"Build All Platforms"** workflow
3. Click **"Run workflow"** button
4. Select branch and click **"Run workflow"**

**Output:** Pre-built binaries available as workflow artifacts (.tar.gz for Unix, .zip for Windows)

---

#### 2. **CI** (`ci.yml`)
Complete continuous integration checks:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Tests on Linux, macOS, Windows
- LSP server integration tests
- Build verification

**How to run:** Same as above, select **"CI"** workflow

---

#### 3. **Release** (`release.yml`)
**Automatic on version tags** (`v*.*.*`)

Creates GitHub releases with binaries for all platforms when you push a version tag:

```bash
git tag v0.8.1
git push origin v0.8.1  # Triggers release workflow automatically
```

Also publishes to crates.io (requires `CARGO_REGISTRY_TOKEN` secret).

---

#### 4. **Docker** (`docker.yml`)
Builds multi-platform Docker images (linux/amd64, linux/arm64):
- Manual trigger available
- Auto-builds on version tags

**How to run:** Select **"Docker"** workflow in Actions tab

---

#### 5. **Security Audit** (`security.yml`)
Runs security audits:
- `cargo audit` for known vulnerabilities
- `cargo deny` for license/supply chain checks

**How to run:** Select **"Security Audit"** workflow in Actions tab

---

## Quick Start: Building for Distribution

**Option 1: Build all platforms at once**
```
Actions → Build All Platforms → Run workflow
```
Download artifacts (binaries) from the completed workflow run.

**Option 2: Create a release**
```bash
git tag v1.0.0
git push origin v1.0.0
```
GitHub automatically builds and attaches binaries to the release.

---

## Setting Up Secrets (Optional)

For full functionality, add these secrets in **Settings → Secrets and variables → Actions**:

| Secret | Purpose | Required? |
|--------|---------|-----------|
| `CARGO_REGISTRY_TOKEN` | Publish to crates.io | Only for releases |
| `GITHUB_TOKEN` | Automatic (provided by GitHub) | Auto-configured |

---

## Customization

### Change trigger behavior

Edit the `on:` section in workflow files:

```yaml
# Current (manual only)
on:
  workflow_dispatch:

# Add automatic triggers
on:
  workflow_dispatch:
  push:
    branches: [ main ]
  pull_request:
```

### Add/remove platforms

Edit the `matrix.include` section in build workflows to add or remove platforms.

---

## Dependabot

Automated dependency updates configured in `.github/dependabot.yml`:
- Weekly Cargo dependency updates
- Weekly GitHub Actions updates
- Groups patch/minor updates to reduce PR noise
