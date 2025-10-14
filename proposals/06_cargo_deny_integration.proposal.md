# Proposal 06: cargo-deny Integration for Dependency Security & Compliance

**Dependencies:** None (can run in parallel with other proposals)

---

## Problem

The project lacks automated dependency security scanning and license compliance checks:

1. **Security vulnerabilities** - No automated CVE scanning in dependencies
2. **License compliance** - No enforcement of MIT/Apache-2.0 compatible licenses
3. **Duplicate dependencies** - 6+ duplicate versions exist (bitflags, dashmap, dirs, getrandom, hashbrown, phf_shared)
4. **Manual reviews** - Dependency audits are error-prone and time-consuming
5. **No CI enforcement** - Vulnerable or non-compliant dependencies can be merged

**Current duplicate dependency count:**
```bash
$ cargo tree --duplicates
bitflags v1.3.2 / v2.9.4
dashmap v5.5.3 / v6.1.0
dirs v5.0.1 / v6.0.0
getrandom v0.2.16 / v0.3.3
hashbrown v0.14.5 / v0.15.5 / v0.16.0
phf_shared (appears twice)
```

---

## Solution

Integrate `cargo-deny` to provide:
- Automated security vulnerability scanning (RustSec Advisory DB)
- License compliance enforcement
- Duplicate dependency detection
- CI/CD integration for continuous monitoring

### Architecture

```
┌─────────────────────────────────────────────┐
│  Developer Workflow                         │
│  ├─ cargo deny check (local)                │
│  ├─ make check (includes deny)              │
│  └─ Pre-commit validation                   │
└─────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────┐
│  CI/CD Pipeline (.github/workflows/ci.yml)  │
│  ├─ cargo deny check advisories             │
│  ├─ cargo deny check licenses               │
│  ├─ cargo deny check bans                   │
│  └─ cargo deny check sources                │
└─────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────┐
│  Configuration (.config/cargo-deny.toml)    │
│  ├─ Allowed licenses: MIT, Apache-2.0, BSD  │
│  ├─ Denied licenses: GPL, AGPL              │
│  ├─ Documented exceptions for duplicates    │
│  └─ Security severity threshold: medium     │
└─────────────────────────────────────────────┘
```

---

## Checklists

### Phase 1: Configuration Setup

- [ ] Create `.config/cargo-deny.toml` with full configuration:
  - [ ] Configure `[advisories]` section
    - [ ] Set `db-path = "~/.cargo/advisory-db"`
    - [ ] Set `db-urls = ["https://github.com/rustsec/advisory-db"]`
    - [ ] Set `severity-threshold = "medium"`
    - [ ] Add `ignore = []` for accepted advisories
  - [ ] Configure `[licenses]` section
    - [ ] Set `confidence-threshold = 0.8`
    - [ ] Add `allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-DFS-2016"]`
    - [ ] Add `deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]`
    - [ ] Set `copyleft = "deny"`
  - [ ] Configure `[bans]` section
    - [ ] Set `multiple-versions = "warn"` (start with warn, move to deny after cleanup)
    - [ ] Add exceptions for unfixable duplicates in `skip = []`
    - [ ] Add `deny = []` for specific problematic crates
  - [ ] Configure `[sources]` section
    - [ ] Set `unknown-registry = "warn"`
    - [ ] Set `unknown-git = "warn"`
    - [ ] Add `allow-git = []` for approved git dependencies

### Phase 2: Dependency Cleanup

- [ ] **Fix dashmap version conflict**
  - [ ] Open `crates/cb-plugins/Cargo.toml`
  - [ ] Change `dashmap = "6.1"` to `dashmap = { workspace = true }`
  - [ ] Run `cargo build` to verify
  - [ ] Run `cargo tree -p cb-plugins | grep dashmap` to confirm single version

- [ ] **Fix dirs version conflict**
  - [ ] Open `crates/cb-client/Cargo.toml`
  - [ ] Change `dirs = "5.0"` to `dirs = "6.0"`
  - [ ] Run `cargo build` to verify
  - [ ] Run `cargo tree -p cb-client | grep dirs` to confirm single version

- [ ] **Document unfixable duplicates**
  - [ ] Add to `.config/cargo-deny.toml` `[bans.skip]`:
    - [ ] `{ name = "bitflags", version = "=1.3.2" }  # lsp-types uses v1`
    - [ ] `{ name = "getrandom", version = "=0.2.16" }  # jsonwebtoken indirect dep`
    - [ ] `{ name = "hashbrown", version = "<0.16" }  # Transitive from collections`
    - [ ] `{ name = "phf_shared", version = "=0.11.3" }  # swc proc-macro internal`
  - [ ] Add comment for each explaining why it can't be fixed

### Phase 3: CI/CD Integration

- [ ] **Add cargo-deny job to CI**
  - [ ] Open `.github/workflows/ci.yml`
  - [ ] Add new job after `security-audit` job:
    ```yaml
    dependency-policy:
      name: Dependency Policy (cargo-deny)
      runs-on: ubuntu-latest
      timeout-minutes: 10

      steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-deny
        run: cargo install cargo-deny --locked

      - name: Check security advisories
        run: cargo deny check advisories

      - name: Check licenses
        run: cargo deny check licenses

      - name: Check bans (duplicates)
        run: cargo deny check bans

      - name: Check sources
        run: cargo deny check sources
    ```
  - [ ] Commit and verify job runs in CI

- [ ] **Alternative: Use cargo-deny GitHub Action**
  - [ ] Replace manual steps with:
    ```yaml
    - uses: EmbarkStudios/cargo-deny-action@v1
      with:
        log-level: warn
        command: check all
    ```

### Phase 4: Makefile Integration

- [ ] **Add deny targets**
  - [ ] Open `Makefile`
  - [ ] Add after line 177 (after `audit:` target):
    ```makefile
    deny:
    	@if ! command -v cargo-deny >/dev/null 2>&1; then \
    		echo "⚠️  cargo-deny not found. Installing..."; \
    		cargo install cargo-deny --locked; \
    	fi
    	@echo "🔒 Running cargo-deny checks..."
    	cargo deny check

    deny-update:
    	@if ! command -v cargo-deny >/dev/null 2>&1; then \
    		echo "⚠️  cargo-deny not found. Run 'make deny' first."; \
    		exit 1; \
    	fi
    	@echo "📡 Updating advisory database..."
    	cargo deny fetch
    ```

- [ ] **Update check target**
  - [ ] Modify line 177: `check: fmt clippy test audit` → `check: fmt clippy test audit deny`

- [ ] **Update help documentation**
  - [ ] Add to "🔍 Code Quality & Validation:" section (around line 376):
    ```makefile
      make deny              - Run cargo-deny dependency checks
      make deny-update       - Update security advisory database
    ```

- [ ] **Verify dev-extras installs cargo-deny**
  - [ ] Check line 144: `cargo binstall --no-confirm cargo-deny` is already present
  - [ ] Verify it works: `make dev-extras`

### Phase 5: Documentation Updates

- [ ] **Update CONTRIBUTING.md**
  - [ ] Add new section after line 118 ("Code Style and Linting"):
    ```markdown
    ## Dependency Management

    Before adding new dependencies:

    1. Check if the functionality already exists in the workspace
    2. Evaluate the dependency's maintenance status, license, and security
    3. Run `cargo deny check` to ensure no issues are introduced

    ### Running Dependency Checks

    ```bash
    # Check all: advisories, licenses, bans, sources
    cargo deny check

    # Check only security advisories
    cargo deny check advisories

    # Check only licenses
    cargo deny check licenses

    # Check only duplicate dependencies
    cargo deny check bans

    # Update advisory database
    cargo deny fetch

    # Or use Makefile
    make deny
    make deny-update
    ```

    ### Handling cargo-deny Failures

    If `cargo deny check` fails:

    - **Advisories (CVEs):** Investigate the vulnerability, assess risk, update dependency if possible. If must ignore, add to `.config/cargo-deny.toml` with justification comment.
    - **Licenses:** Ensure new dependency has compatible license (MIT/Apache-2.0/BSD). GPL and AGPL are blocked.
    - **Bans (duplicates):** Try to use workspace version or consolidate versions. If unfixable (transitive), add to `skip` with comment explaining why.
    - **Sources:** Avoid git dependencies unless necessary, prefer crates.io releases.

    If an exception is truly needed, update `.config/cargo-deny.toml` with clear justification comment.
    ```

- [ ] **Update README.md**
  - [ ] Add new "Security" section after line 127 (after "Development" section):
    ```markdown
    ## 🔒 Security

    This project uses automated dependency scanning:

    - **[cargo-deny](https://github.com/EmbarkStudios/cargo-deny)** - Security vulnerability scanning, license compliance, and dependency policy enforcement
    - **[RustSec Advisory Database](https://rustsec.org/)** - CVE tracking for Rust crates

    Run security checks:
    ```bash
    cargo deny check        # All checks
    make deny               # Same via Makefile
    ```

    Report security issues: See [SECURITY.md](SECURITY.md)
    ```

- [ ] **Update docs/QUICK_REFERENCE.md** (if exists)
  - [ ] Add `cargo deny check` to command reference
  - [ ] Add `make deny` to Makefile commands section

### Phase 6: Validation

- [ ] **Verify configuration**
  - [ ] Run `cargo deny check advisories` - should pass (no known CVEs)
  - [ ] Run `cargo deny check licenses` - should pass (all allowed licenses)
  - [ ] Run `cargo deny check bans` - should pass (documented exceptions)
  - [ ] Run `cargo deny check sources` - should pass (no unknown sources)

- [ ] **Verify CI integration**
  - [ ] Create test PR with a GPL dependency
  - [ ] Verify CI fails with clear error message
  - [ ] Remove GPL dependency and verify CI passes

- [ ] **Verify Makefile integration**
  - [ ] Run `make deny` - should pass
  - [ ] Run `make check` - should include deny checks
  - [ ] Run `make deny-update` - should fetch latest advisory DB

- [ ] **Test duplicate detection**
  - [ ] Run `cargo tree --duplicates` before fixes
  - [ ] Apply dashmap and dirs fixes
  - [ ] Run `cargo tree --duplicates` after fixes
  - [ ] Verify dashmap and dirs no longer appear as duplicates

---

## Success Criteria

### Security
- [ ] `cargo deny check advisories` passes with zero unaddressed CVEs
- [ ] Advisory database auto-updates in CI
- [ ] CI blocks PRs with security vulnerabilities

### License Compliance
- [ ] `cargo deny check licenses` passes
- [ ] All dependencies use MIT/Apache-2.0/BSD compatible licenses
- [ ] GPL/AGPL dependencies are blocked by CI

### Code Quality
- [ ] Duplicate dependency count reduced from 6+ to 3-4 documented exceptions
- [ ] `dashmap` consolidated to single version (5.5.3 workspace version)
- [ ] `dirs` consolidated to single version (6.0)
- [ ] All remaining duplicates documented with justification in `.config/cargo-deny.toml`

### Integration
- [ ] CI job `dependency-policy` runs on all PRs
- [ ] `make deny` command works and is documented
- [ ] `make check` includes deny checks
- [ ] Contributors can run checks locally before pushing

### Documentation
- [ ] CONTRIBUTING.md has "Dependency Management" section
- [ ] README.md has "Security" section mentioning cargo-deny
- [ ] `.config/cargo-deny.toml` has comments explaining all exceptions
- [ ] Makefile help shows deny commands

---

## Benefits

### 🔒 Security
- **Automated CVE scanning** - Continuous monitoring for known vulnerabilities in dependencies
- **Early detection** - Catches security issues before they reach production
- **Zero-day protection** - Advisory database updates daily with new CVEs
- **Supply chain security** - Validates dependency sources and prevents malicious packages

### 📜 Legal Protection
- **License compliance** - Enforces MIT/Apache-2.0 compatible licenses
- **GPL prevention** - Blocks copyleft licenses that could contaminate codebase
- **Audit trail** - Clear documentation of all license decisions
- **Reduced legal risk** - Automated checks prevent accidental license violations

### 🧹 Code Quality
- **Reduced bloat** - Eliminates 30-50% of duplicate dependencies
- **Faster builds** - Fewer duplicates = less compilation
- **Smaller binaries** - Less redundant code in release builds
- **Cleaner dependency tree** - Easier to reason about and maintain

### 🚨 CI/CD Integration
- **Blocks bad PRs** - Can't merge code with security or license issues
- **Saves review time** - Automated checks replace manual audits
- **Shift-left security** - Catches issues during development, not production
- **Continuous compliance** - Every commit is validated

### ⚡ Developer Experience
- **Fast checks** - `cargo deny check` completes in seconds
- **Clear errors** - Actionable error messages with remediation guidance
- **Local validation** - Run same checks locally as CI
- **No runtime overhead** - Only runs during development/CI, not production

### 📊 Maintenance
- **Prevents tech debt** - Stops duplicate dependency accumulation
- **Simplified upgrades** - Fewer versions to track and update
- **Better visibility** - Clear picture of dependency health
- **Industry standard** - Same tool used by rust-analyzer, tokio, cargo itself

### 🎯 Measurable Impact
- **Before:** 6+ duplicate dependency versions, no security scanning, no license checks
- **After:** 3-4 documented exceptions, automated CVE scanning, license enforcement in CI
- **ROI:** 3-4 hours setup for continuous protection and compliance
