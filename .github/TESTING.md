# Testing GitHub Actions Workflows Locally

Before pushing workflows to GitHub, you can test them locally using several methods.

## Method 1: Run CI Commands Locally (Recommended)

**Fastest and most reliable method** - runs the exact same commands as CI:

```bash
# Make the test script executable
chmod +x .github/workflows/test-locally.sh

# Run all CI checks
./.github/workflows/test-locally.sh
```

This tests:
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Build
- ✅ Tests
- ✅ Doc tests
- ✅ Release build
- ✅ Xtask checks

**Quick manual tests:**
```bash
# Format check
cargo fmt --all -- --check

# Lint check
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo nextest run --workspace

# Build release
cargo build --release
```

---

## Method 2: Test Multi-Platform Builds with Cross

Test cross-compilation for all platforms:

```bash
# Install cross (one-time setup)
cargo install cross

# Make the script executable
chmod +x .github/workflows/test-builds.sh

# Test builds for all platforms
./.github/workflows/test-builds.sh
```

**Note:** Requires Docker. This simulates what GitHub Actions does for multi-platform builds.

**Manual testing for specific platform:**
```bash
# Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu

# macOS Apple Silicon
cross build --release --target aarch64-apple-darwin

# Windows
cross build --release --target x86_64-pc-windows-gnu
```

---

## Method 3: Use `act` to Run Workflows Locally

**Most accurate** - runs actual GitHub Actions workflows in Docker:

### Install `act`

**macOS:**
```bash
brew install act
```

**Linux:**
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

**Windows (with Chocolatey):**
```bash
choco install act-cli
```

### Run workflows

```bash
# List all workflows
act -l

# Run CI workflow
act workflow_dispatch -W .github/workflows/ci.yml

# Run build workflow (takes a long time!)
act workflow_dispatch -W .github/workflows/build-all-platforms.yml

# Or use the helper script
chmod +x .github/workflows/test-with-act.sh
./.github/workflows/test-with-act.sh
```

**Limitations:**
- ⚠️ Slow (downloads large Docker images)
- ⚠️ Multi-platform builds may not work perfectly
- ⚠️ Some GitHub-specific features won't work (secrets, artifacts)

---

## Method 4: Test on GitHub (Safe Branch)

Most reliable way to test full workflows:

```bash
# Create a test branch
git checkout -b test-github-actions

# Commit the workflow files
git add .github/
git commit -m "test: Add GitHub Actions workflows"

# Push to GitHub
git push origin test-github-actions
```

Then on GitHub:
1. Go to **Actions** tab
2. Select a workflow (e.g., "Build All Platforms")
3. Click **"Run workflow"**
4. Select your test branch
5. Click **"Run workflow"**

**Advantages:**
- ✅ Tests the actual GitHub environment
- ✅ Tests all platforms accurately
- ✅ Tests artifact uploads/downloads
- ✅ Validates workflow syntax

**After testing:** Delete the test branch if everything works.

---

## Method 5: Validate Workflow Syntax

Check for syntax errors without running:

```bash
# Install actionlint
brew install actionlint  # macOS
# or
go install github.com/rhysd/actionlint/cmd/actionlint@latest

# Validate all workflows
actionlint .github/workflows/*.yml
```

Or use GitHub's online validator:
1. Create a new repository or use existing
2. Add workflow file
3. GitHub automatically validates on push
4. Check "Actions" tab for errors

---

## Recommended Testing Flow

**Before first push:**
```bash
# 1. Quick local tests (1-2 minutes)
./.github/workflows/test-locally.sh

# 2. Validate syntax
actionlint .github/workflows/*.yml

# 3. Test on GitHub (safe branch)
git checkout -b test-ci
git add .github/
git commit -m "test: GitHub Actions"
git push origin test-ci
# Then manually trigger workflow on GitHub
```

**For subsequent changes:**
```bash
# Quick validation
cargo fmt --all -- --check
cargo clippy --all-targets
cargo test

# Push to test branch and verify
```

---

## Troubleshooting

### Workflow doesn't appear in Actions tab
- Workflow files must be in `.github/workflows/`
- Files must end in `.yml` or `.yaml`
- Workflow must be on the default branch (or your current branch)
- Check syntax with `actionlint`

### Build fails on GitHub but works locally
- Check workflow uses same Rust version (`stable`)
- Check dependencies are in `Cargo.toml` (not just locally installed)
- Review logs in GitHub Actions run

### Cross-compilation fails
- Install `cross`: `cargo install cross`
- Ensure Docker is running
- Some targets may not support all crates

---

## What Each Method Tests

| Method | Speed | Accuracy | Platforms | Artifacts |
|--------|-------|----------|-----------|-----------|
| **Local commands** | ⚡ Fast | ✅ High | Current only | ❌ No |
| **Cross builds** | 🐢 Slow | ✅ High | All | ❌ No |
| **act** | 🐢 Very slow | ⚠️ Medium | Limited | ⚠️ Partial |
| **GitHub test branch** | 🐢 Slow | ✅✅ Perfect | All | ✅ Yes |
| **Syntax validator** | ⚡ Instant | ⚠️ Syntax only | N/A | N/A |

**Recommendation:** Use **local commands** for fast iteration, then **GitHub test branch** for final verification.
