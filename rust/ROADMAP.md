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

#### 1. Structured Logging - ✅ COMPLETE
- [x] **Foundation**: Tracing framework integrated in cb-server ✅
- [x] **Production Code**: All production libraries use tracing ✅
  - Status: 100% done - Analysis revealed no work needed
  - Core libraries (cb-ast, cb-api, cb-core, cb-plugins, cb-transport): 0 println
  - Server library (cb-server/src): Uses tracing throughout
  - 216 println! calls are in CLI tools (user output) and tests (appropriate usage)
  - Only 1 eprintln! in server main.rs during logger init (correct - can't log before logger exists)
  - **Decision**: ✅ Complete - Production observability requirements met
  - Priority: **DONE**

#### 2. Error Handling - Remove .unwrap() from production code
- [ ] **Phase 1**: Production hot paths (services/, handlers/) - 8-10 hours
- [ ] **Phase 2**: CLI and startup code - 6-8 hours
- [ ] **Phase 3**: Keep .unwrap() in tests (tests are allowed to panic)
  - Status: 0% done, .unwrap() in 100+ files
  - Total estimated effort: 14-18 hours (phased approach)
  - **Decision**: 📋 Phase it - Break into chunks, critical paths first
  - Priority: **HIGH** for Phase 1, **MEDIUM** for Phase 2

#### 3. Dependency Cleanup - ✅ COMPLETE
- [x] Run `cargo tree --duplicates` to identify all duplicates
- [x] Align versions in Cargo.toml across workspace
- [x] Verify build and tests pass
  - Status: Done - Consolidated thiserror 2.0 and jsonwebtoken 10.0
  - Unified across cb-plugins, cb-mcp-proxy, cb-server, cb-transport, tests
  - Remaining duplicates are from external transitive dependencies (acceptable)
  - **Decision**: ✅ Complete - Core dependencies unified
  - Priority: **DONE**

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

#### 6. Benchmark Suite - ✅ COMPLETE
- [x] Delete `benchmark-harness/benches/config_benchmark.rs.disabled`
- [x] Document that benchmarks can be recreated later if needed
  - Status: Done - Removed 238 lines of stale code
  - API changed (ClientConfig::load_with_path doesn't exist), unmaintainable
  - **Decision**: ✅ Complete - Clutter removed, can recreate if needed
  - Priority: **DONE**

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

**✅ COMPLETED:**
1. ✅ Structured Logging - DONE (production code uses tracing, CLI println appropriate)
2. ✅ Dependency Cleanup - DONE (thiserror 2.0, jsonwebtoken 10.0 unified)
3. ✅ Benchmark Suite - DONE (stale code removed)

**HIGH Priority (Before 1.0):**
4. 📋 Error Handling Phase 1 - 8-10 hours (production hot paths)

**MEDIUM Priority (Consider for 1.0):**
5. ⚠️ VFS Feature-gating - 1-2 hours (defer decision, keep code)

**LOW Priority (Post-1.0):**
6. ⏸️ SWC Integration - 20-40 hours (optimization, not required)

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