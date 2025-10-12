# Proposal: Rename Project to TypeMill

**Status**: Draft
**Author**: Project Team
**Date**: 2025-10-10
**Current Name**: `codebuddy` / `codebuddy` CLI
**Proposed Name**: `typemill` / `mill` CLI

---

## Executive Summary

This proposal outlines the rationale, scope, and implementation plan for renaming the project from `codebuddy` to `typemill`, with the CLI command changing from `codebuddy` to `mill`.

## Motivation

### Why "TypeMill"?

1. **Better Reflects Core Functionality**
   - The project is fundamentally a "mill" that processes and refines code through LSP servers
   - "Type" emphasizes the strong type-safety focus (LSP intelligence, static analysis, refactoring)
   - Metaphor: A mill processes raw materials into refined products; TypeMill processes code into better code

2. **Stronger Brand Identity**
   - "codebuddy" is generic and conflicts with existing tools/services
   - "typemill" is distinctive and memorable
   - Conveys professionalism and precision

3. **CLI Ergonomics**
   - `mill` is short, fast to type (4 characters vs 9)
   - Follows Unix tradition of concise commands (`git`, `grep`, `sed`, `make`)
   - Natural verb-like quality: "mill the code", "run the mill"

4. **Technical Alignment**
   - Emphasizes the "grinding/processing" nature of the tool
   - "Type" connects to type systems, TypeScript support, and static analysis
   - Better SEO and searchability in developer tools space

5. **Domain Assets**
   - Project owns both `typemill.org` and `typemill.com`
   - `.org` hosts the open-source mill CLI and documentation
   - `.com` reserved for future Design Evolve product (visual/commercial layer)
   - Complete brand protection and clear product positioning

## Scope of Changes

### 1. Crate and Package Names

**Rust Workspace:**
- `codebuddy` → `typemill`
- `cb-protocol` → `mill-protocol`
- `cb-server` → `mill-server`
- `cb-client` → `mill-client`
- `cb-lsp` → `mill-lsp`
- `cb-services` → `mill-services`
- `cb-ast` → `mill-ast`
- `cb-vfs` → `mill-vfs`
- `cb-plugins` → `mill-plugins`
- `cb-language-plugin` → `mill-language-plugin`
- `cb-*` → `mill-*` (all crates)

**Naming Convention:**
- Old: `cb-{component}` (e.g., `cb-services`)
- New: `mill-{component}` (e.g., `mill-services`)

### 2. CLI Commands

**Primary Command:**
```bash
# Old
codebuddy setup
codebuddy status
codebuddy start
codebuddy serve

# New
mill setup
mill status
mill start
mill serve
```

**All Subcommands:**
- `mill setup` - Smart setup with auto-detection
- `mill status` - Show current status
- `mill start` - Start MCP server
- `mill stop` - Stop MCP server
- `mill serve` - Start WebSocket server
- `mill link` - Link to AI assistants
- `mill unlink` - Remove AI from config
- `mill --version` - Show version

### 3. Configuration and Paths

**Configuration Directory:**
- `.codebuddy/` → `.typemill/`
- `.codebuddy/config.json` → `.typemill/config.json`

**Binary Path:**
- `target/release/codebuddy` → `target/release/mill`
- `/usr/local/bin/codebuddy` → `/usr/local/bin/mill`

### 4. Environment Variables

**Prefix Migration:**
- `CODEBUDDY__*` (multilevel config) → `TYPEMILL__*`
- `CODEBUDDY_*` (CLI/runtime helpers) → `TYPEMILL_*`

**Migration Strategy:**
- Maintain dual-read support for legacy variables for at least two release cycles
- Emit structured `warn!` logs when legacy variables are detected
- Provide a one-time migration helper (`mill env migrate`) to rewrite `.env`/shell exports
- Update docs and examples to prefer the new prefix while noting backward compatibility
- Coordinate updates across `cb-core` config loaders and `cb-client` CLI parsing (see `crates/cb-core/src/config.rs` and `crates/cb-client/src/client_config.rs`)

### 5. Documentation Updates

**Files to Update:**
- `README.md` - Project name, CLI examples, installation
- `CLAUDE.md` / `AGENTS.md` - All references to project name and CLI
- `API_REFERENCE.md` - Package names and examples
- `CONTRIBUTING.md` - Development workflow references
- All `docs/**/*.md` files
- `Cargo.toml` - Package metadata
- `package.json` (if exists) - NPM package name

**Examples in Documentation:**
```bash
# Old examples
cargo run --bin codebuddy
./target/release/codebuddy setup

# New examples
cargo run --bin mill
./target/release/mill setup
```

### 6. Code References

**Rust Code:**
- Module imports: `use codebuddy::*` → `use typemill::*`
- Binary targets in `Cargo.toml`
- Error messages and help text
- Log messages mentioning project name

**Configuration Examples:**
- JSON schema references
- Sample configurations
- Docker compose files

### 7. Infrastructure

**Docker:**
- Image names: `codebuddy:latest` → `typemill:latest`
- Container names in docker-compose
- Volume mount paths

**GitHub/CI:**
- Repository name (if applicable)
- GitHub Actions workflow references
- Release artifact names

**Homebrew/Package Managers:**
- Formula/package names
- Installation paths

## Implementation Checklist

### Preparation
- [ ] Backup and branch: `git checkout -b rename-to-typemill && git tag pre-typemill-rename`
- [ ] Run global search for all instances of `codebuddy`, `cb-*`, `.codebuddy`
- [ ] Document external dependencies (CI, deployment scripts, user guides)
- [ ] Identify breaking changes for users
- [ ] Inventory all `CODEBUDDY_*` and `CODEBUDDY__*` environment variables
- [ ] Draft migration guide for existing users
- [ ] Prepare changelog entry

### Cargo Workspace
- [ ] Rename all `crates/cb-*` directories to `crates/mill-*`
- [ ] Update `Cargo.toml` in each crate (name field)
- [ ] Update workspace root `Cargo.toml`
- [ ] Update all internal imports across crates

### Binary and CLI
- [ ] Rename binary target in root `Cargo.toml` to `mill`
- [ ] Update CLI help text and error messages
- [ ] Update clap command definitions

### Configuration
- [ ] Update config path logic to use `.typemill/`
- [ ] Add migration code to auto-detect and migrate `.codebuddy/` → `.typemill/`
- [ ] Update all config examples and schemas

### Environment Variables
- [ ] Extend config loaders to read both `CODEBUDDY*` and `TYPEMILL*`
- [ ] Emit structured warnings when legacy prefixes are used
- [ ] Implement `mill env migrate` helper
- [ ] Update acceptance tests for dual-prefix support

### Documentation
- [ ] Update `README.md`, `CLAUDE.md`, `AGENTS.md`
- [ ] Update `API_REFERENCE.md`, `CONTRIBUTING.md`
- [ ] Update all `docs/**/*.md` files
- [ ] Update architecture diagrams
- [ ] Update all code snippets in documentation
- [ ] Update integration test examples

### Infrastructure
- [ ] Update Dockerfiles and docker-compose.yml
- [ ] Update GitHub Actions workflows
- [ ] Update release scripts and artifact names
- [ ] Update package metadata (Homebrew if exists)

### Testing
- [ ] Build and test all CLI commands: `cargo build --release && ./target/release/mill setup`
- [ ] Run full integration tests: `cargo nextest run --workspace --all-features`
- [ ] Verify all links work
- [ ] Test upgrade path from `.codebuddy/` to `.typemill/`

### Release
- [ ] Create detailed CHANGELOG entry
- [ ] Write migration guide
- [ ] Update version number (major bump to 2.0.0)
- [ ] Merge to main and tag release
- [ ] Publish to crates.io (if applicable)

## Migration Path for Users

### Automatic Migration

The tool will automatically detect and migrate:
```bash
# On first run of new version
mill setup
# → Detects .codebuddy/ directory
# → Offers to migrate to .typemill/
# → Preserves all configuration
```

### Manual Migration

Users can manually migrate:
```bash
# Backup old config
cp -r .codebuddy .codebuddy.backup

# Rename directory
mv .codebuddy .typemill

# Update any custom scripts
sed -i 's/codebuddy/mill/g' scripts/*.sh
```

### Backward Compatibility

**Deprecation Period (Optional):**
- Keep `codebuddy` as symlink to `mill` for 2-3 releases
- Show deprecation warning when `codebuddy` command is used
- Remove symlink in major version bump (v3.0.0)

## Breaking Changes

### For End Users

1. **CLI Command Change**
   - All scripts using `codebuddy` must change to `mill`
   - Shell aliases and shortcuts need updating

2. **Configuration Directory**
   - `.codebuddy/` → `.typemill/`
   - Automatic migration provided

3. **Binary Name**
   - Installation paths change
   - System PATH may need adjustment

### For Developers/Contributors

1. **Import Paths**
   - All `use codebuddy::*` → `use typemill::*`
   - Crate dependencies updated

2. **Crate Names**
   - All `cb-*` → `mill-*`
   - Affects plugin development

3. **Repository Structure**
   - Directory names changed
   - Update local development setups

## Risks and Mitigations

### Risk 1: User Confusion
**Impact**: Medium
**Mitigation**:
- Clear migration guide
- Deprecation warnings
- Comprehensive changelog
- Consider keeping old binary name as alias temporarily

### Risk 2: SEO and Discoverability
**Impact**: Low
**Mitigation**:
- Redirect old documentation URLs
- Update all external references
- Maintain old repository name redirects

### Risk 3: Broken External Integrations
**Impact**: High
**Mitigation**:
- Survey known integrations before rename
- Provide migration timeline (not immediate)
- Maintain backward compatibility symlinks
- Update integration examples in documentation

### Risk 4: Build System Disruption
**Impact**: Medium
**Mitigation**:
- Comprehensive testing before merge
- Staged rollout (internal testing first)
- Clear rollback plan (git tag before rename)

## Success Criteria

- [ ] All tests pass with new names
- [ ] All documentation updated and accurate
- [ ] CLI commands work with `mill` prefix
- [ ] Migration path tested and documented
- [ ] No regression in functionality
- [ ] Docker builds succeed with new names
- [ ] Package registries updated (if applicable)
- [ ] Users can successfully migrate from old version

## Open Questions

1. **Version Bump Strategy**: Should this be v2.0.0 (major) or v1.x.0 (minor)?
   - Recommendation: **v2.0.0** (breaking change for CLI command)

2. **Deprecation Period**: How long should we maintain `codebuddy` symlink?
   - Recommendation: **2-3 releases** or **6 months**, whichever is longer

3. **Repository Name**: Should GitHub repository also be renamed?
   - Recommendation: **Yes**, with automatic redirect from old name

4. **NPM Package** (if applicable): Claim `typemill` package name?
   - Recommendation: **Reserve name early** to prevent squatting

5. **Domain Strategy**: How to launch the .org and .com sites?
   - Recommendation: **Launch typemill.org immediately** with CLI docs; point typemill.com to "coming soon"

## Alternatives Considered

### Alternative 1: Keep `codebuddy` name
**Pros**: No migration effort, no user disruption
**Cons**: Misses opportunity for better branding, name conflicts persist

### Alternative 2: Rename to something else
**Other names considered**:
- `codemason` - Building metaphor, but less precise
- `forgemill` - Emphasizes crafting, but verbose
- `lspmill` - Too technical, less approachable
- `refinemill` - Clear purpose, but redundant with "mill"

**Why TypeMill wins**: Best balance of brevity, meaning, and technical accuracy

### Alternative 3: Gradual rename (keep both names)
**Pros**: Easier migration path
**Cons**: Confusing documentation, technical debt, diluted brand

## Conclusion

Renaming to **TypeMill** with CLI command **mill** provides:
- ✅ Better brand identity and memorability
- ✅ Improved CLI ergonomics (shorter command)
- ✅ Clearer technical positioning
- ✅ Professional and distinctive naming
- ✅ Better SEO and discoverability

**Recommendation**: **Approve and proceed** with phased implementation plan.

---

## Next Steps

1. **Gather feedback** on this proposal from team and stakeholders
2. **Finalize timeline** based on project priorities
3. **Begin Phase 1** (Preparation) once approved
4. **Track progress** using project management tools
5. **Update this document** with decisions on open questions

## Appendix A: Search Patterns for Rename

```bash
# Find all references to codebuddy
rg "codebuddy" --type rust
rg "codebuddy" --type md
rg "cb-[a-z]+" --type rust
rg "\.codebuddy"

# Find cargo.toml files
fd Cargo.toml

# Find all documentation
fd -e md

# Find configuration examples
rg "\.codebuddy" -g "*.md" -g "*.json"
```

## Appendix B: Critical Files Checklist

- [ ] `Cargo.toml` (root workspace)
- [ ] `Cargo.toml` (all crate manifests)
- [ ] `README.md`
- [ ] `CLAUDE.md` / `AGENTS.md`
- [ ] `API_REFERENCE.md`
- [ ] `CONTRIBUTING.md`
- [ ] All `docs/**/*.md`
- [ ] `Dockerfile`
- [ ] `docker-compose.yml`
- [ ] `.github/workflows/*.yml`
- [ ] All Rust source files (imports)
- [ ] CLI help text and error messages
- [ ] Test files and examples
- [ ] Configuration schema files

## Appendix C: Communication Templates

### Migration Guide Template
```markdown
# Migrating from codebuddy to mill

Version 2.0.0 introduces a new name: **TypeMill** (CLI: `mill`)

## Quick Migration

1. Update CLI commands: `codebuddy` → `mill`
2. Configuration automatically migrates on first run
3. Update scripts and aliases

See full guide: [MIGRATION.md](MIGRATION.md)
```

### Changelog Entry Template
```markdown
## [2.0.0] - 2025-XX-XX

### BREAKING CHANGES
- **Project renamed to TypeMill** - CLI command is now `mill` instead of `codebuddy`
- Configuration directory changed from `.codebuddy/` to `.typemill/`
- All crate names updated from `cb-*` to `mill-*`

### Migration
- Run `mill setup` to automatically migrate configuration
- Update scripts to use `mill` command
- See [MIGRATION.md](MIGRATION.md) for detailed guide
```
