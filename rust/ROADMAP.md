# CodeBuddy Roadmap

## Current Status: Pre-1.0 Development (v0.1.0)

CodeBuddy is in active development with core functionality working but no API stability guarantees.

---

## 🎯 Path to 1.0 Release

**Target:** Q2 2025

### Requirements for 1.0
- [ ] API stability commitment
- [ ] Complete documentation coverage
- [ ] Production deployments validated
- [ ] Performance benchmarks met
- [ ] Security audit completed
- [ ] All HIGH priority technical debt addressed

---

## 🚀 Planned Features & Technical Debt

### 🔥 HIGH Priority (Before 1.0)

#### 1. Structured Logging - Convert 216 println!/eprintln! calls
- [x] **Foundation**: Tracing framework integrated in cb-server ✅
- [ ] **Remaining Work**: Convert 216 direct console calls to tracing
  - Status: 20% done (framework exists, conversions needed)
  - Pattern: `println!("message")` → `info!("message")`
  - Estimated effort: 4-6 hours
  - **Decision**: ✅ Complete it - Required for production observability
  - Priority: **HIGH** - Do before 1.0

#### 2. Error Handling - Remove .unwrap() from production code
- [ ] **Phase 1**: Production hot paths (services/, handlers/) - 8-10 hours
- [ ] **Phase 2**: CLI and startup code - 6-8 hours
- [ ] **Phase 3**: Keep .unwrap() in tests (tests are allowed to panic)
  - Status: 0% done, .unwrap() in 100+ files
  - Total estimated effort: 14-18 hours (phased approach)
  - **Decision**: 📋 Phase it - Break into chunks, critical paths first
  - Priority: **HIGH** for Phase 1, **MEDIUM** for Phase 2

#### 3. Dependency Cleanup - Resolve duplicates
- [ ] Run `cargo tree --duplicates` to identify all duplicates
- [ ] Align versions in Cargo.toml across workspace
- [ ] Verify build and tests pass
  - Status: Multiple duplicates visible (base64 v0.21.7, etc.)
  - Estimated effort: 1-2 hours
  - **Decision**: ✅ Clean up - Quick win, reduces binary size
  - Priority: **MEDIUM** - Do before 1.0

### ⚠️ MEDIUM Priority (Consider for 1.0)

#### 4. VFS Feature - Feature-gate and defer
- [ ] Add `#[cfg(feature = "vfs")]` guards to cb-vfs crate
- [ ] Update Cargo.toml to make vfs an optional feature
- [ ] Document as experimental
  - Status: Partial implementation (466 lines), decision pending
  - Estimated effort: 1-2 hours to feature-gate
  - **Decision**: ⚠️ Feature-gate and defer - Docker volumes proposal eliminates immediate need
  - Priority: **LOW** - Not blocking 1.0, revisit in 3-6 months

### 📦 LOW Priority (Post-1.0)

#### 5. SWC Integration - Faster TypeScript/JavaScript parsing
- [ ] Integrate SWC for AST parsing
- [ ] Benchmark performance improvements
- [ ] Update existing TS/JS tools to use SWC
  - Status: Not implemented, blocked by network restrictions during initial dev
  - Estimated effort: 20-40 hours
  - **Decision**: ⏸️ Defer - Current AST parsing works, this is optimization
  - Priority: **LOW** - Post-1.0 optimization if performance becomes issue

#### 6. Benchmark Suite - Delete stale benchmarks
- [ ] Delete `benchmark-harness/benches/config_benchmark.rs.disabled`
- [ ] Document that benchmarks can be recreated later if needed
  - Status: Disabled, API changed (ClientConfig::load_with_path doesn't exist)
  - Estimated effort: 5 minutes to delete
  - **Decision**: ❌ Delete it - Zero value, API changed, not critical
  - Priority: **LOW** - Remove clutter

---

## 📅 Release Timeline

### Q4 2024 (Current)
- ✅ Core LSP integration
- ✅ MCP protocol support
- ✅ Plugin architecture
- 🔄 Technical debt reduction (in progress)

### Q1 2025
- Performance optimization
- Documentation improvements
- Security hardening
- Beta testing program

### Q2 2025
- API stabilization
- 1.0 Release candidate
- Production readiness validation
- **1.0 RELEASE**

### Post-1.0
- Follow semantic versioning (semver 2.0)
- Breaking changes only in major versions
- Regular security updates
- Community-driven feature development

---

## 🔧 Technical Debt Summary

See section above for detailed breakdown. Quick reference:

**HIGH Priority (Before 1.0):**
1. ✅ Structured Logging - 4-6 hours (foundation done, 216 calls to convert)
2. 📋 Error Handling Phase 1 - 8-10 hours (production hot paths)
3. ✅ Dependency Cleanup - 1-2 hours (quick win)

**MEDIUM Priority (Consider for 1.0):**
4. ⚠️ VFS Feature-gating - 1-2 hours (defer decision, keep code)

**LOW Priority (Post-1.0):**
5. ⏸️ SWC Integration - 20-40 hours (optimization, not required)
6. ❌ Benchmark Suite - 5 minutes (delete stale code)

---

## 📊 Version Strategy

### Pre-1.0 (Current: 0.1.0)
- Breaking changes allowed without notice
- No API stability guarantees
- Rapid iteration and experimentation
- Internal use and testing only

### Post-1.0
- **Major version** (X.0.0): Breaking changes
- **Minor version** (0.X.0): New features, backwards compatible
- **Patch version** (0.0.X): Bug fixes only

---

## 🤝 Contributing

Want to help shape CodeBuddy's future?

- Review open issues tagged with `roadmap`
- Discuss features in GitHub Discussions
- Submit PRs for planned features
- Help with documentation and testing

---

**Last Updated:** 2025-09-30