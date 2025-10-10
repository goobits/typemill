# Proposal 16: Quick Start Developer Path 🚀

## Status
**Proposed** - Onboarding test path for experienced developers

## Overview
Speed test: Can an experienced engineer go from zero to productive in < 15 minutes with minimal documentation?

## Persona: Senior Engineer

**Profile:**
- 5+ years professional development experience
- Daily Rust user OR has Rust already installed
- Docker proficient
- CLI power user (zsh/bash customization, aliases, etc.)
- Uses AI assistants regularly (Claude Code, Cursor, GitHub Copilot)
- Has worked with language servers (in IDE context)
- Values efficiency and "just works" tools

**Starting Point:**
- ✅ Rust installed and updated
- ✅ Docker running
- ✅ CLI proficient (knows how to debug PATH, env vars, etc.)
- ✅ At least 2 language servers already installed (for their daily work)
- ✅ Familiar with MCP concept (at least aware it exists)
- ❌ Never used Codebuddy before

## Goal
Zero to productive in < 15 minutes with MINIMAL docs reading. Test if tool is intuitive enough for experts.

## Test Phases

### Phase 1: Lightning Install (3 min)
**Goal:** Install and verify in under 3 minutes

**Tasks:**
- [ ] Skim README (30 seconds max)
- [ ] Choose installation method (install script vs cargo install)
- [ ] Run installation
- [ ] Verify: `codebuddy --version`

**Success Metrics:**
- Installation completes in < 3 min
- Zero troubleshooting needed
- Command immediately available in PATH

**Common Friction Points:**
- Install script vs cargo install decision unclear
- Binary not immediately in PATH
- Unclear which method is "best" for experts

**Expected Behavior:**
- "I just want `cargo install codebuddy` to work"
- Will NOT read detailed docs
- Expects sensible defaults

### Phase 2: Real-World Project (5 min)
**Goal:** Configure for actual production codebase

**Tasks:**
- [ ] Navigate to real work project (multi-language, large codebase)
- [ ] Run `codebuddy setup`
- [ ] Review auto-detected languages
- [ ] Accept defaults or customize
- [ ] Verify: `codebuddy status`

**Success Metrics:**
- Setup detects all project languages correctly
- Suggests LSP servers they already have installed
- Config generation is instant
- Status shows green for existing LSP servers

**Common Friction Points:**
- Auto-detection misses a language
- Suggests installing LSP they already have (detection failure)
- Unclear what "restart interval" does
- Don't know if they should customize or use defaults

**Expected Behavior:**
- "Just detect everything and use what I have"
- Will skip reading about each config option
- Expects smart defaults (30 min restart interval, etc.)

### Phase 3: MCP Integration (2 min)
**Goal:** Connect to AI assistant in < 2 minutes

**Tasks:**
- [ ] Find MCP config snippet in README
- [ ] Add to existing MCP config
- [ ] Restart assistant
- [ ] Verify tools available

**Success Metrics:**
- Copy-paste one snippet, it works
- No config errors
- Tools immediately available

**Common Friction Points:**
- MCP config location unclear for their specific assistant
- JSON merge errors (trailing commas, etc.)
- Don't know if they need to restart assistant

**Expected Behavior:**
- "Show me the one snippet I need"
- Will NOT read about MCP protocol details
- Expects config to just merge with existing setup

### Phase 4: Advanced Features (10 min)
**Goal:** Explore power-user features without reading docs

**Tasks:**
- [ ] Multi-file refactoring (rename symbol across workspace)
- [ ] Batch operations (format multiple files)
- [ ] Workspace-wide symbol search
- [ ] Check dry-run mode for safety
- [ ] Test with temporary VM/container (optional)

**Success Metrics:**
- Discovers advanced features through tool names/descriptions
- Understands dry-run without reading docs
- Can chain operations efficiently

**Common Friction Points:**
- Don't know what tools are available
- Unclear which tools are "safe" vs destructive
- Missing examples for advanced workflows

**Expected Behavior:**
- "Show me what tools exist, I'll figure out usage"
- Will experiment with dry-run first
- Wants JSON output for scripting

### Phase 5: Performance Check (2 min)
**Goal:** Verify it's fast enough for large codebases

**Tasks:**
- [ ] Run workspace-wide symbol search
- [ ] Find references in large file (>1000 lines)
- [ ] Check memory usage
- [ ] Test responsiveness on large rename

**Success Metrics:**
- Operations complete in < 5 seconds
- Memory usage reasonable (< 500MB)
- No lag or freezing

**Common Friction Points:**
- Slow on large codebases
- High memory usage
- Unclear how to optimize for performance

**Expected Behavior:**
- "If it's slow, I won't use it"
- Will check resource usage immediately
- Expects sub-second response times

### Phase 6: Configuration Customization (3 min)
**Goal:** Tweak config for advanced use case

**Tasks:**
- [ ] Find config file: `.codebuddy/config.json`
- [ ] Add custom LSP server (maybe gopls, rust-analyzer with special flags)
- [ ] Set restart interval
- [ ] Reload config (restart server or hot reload?)

**Success Metrics:**
- Config format is obvious (well-documented JSON)
- Can add custom servers without examples
- Config changes take effect immediately

**Common Friction Points:**
- Unclear how to reload config
- Missing schema/validation
- No examples for advanced server configs

**Expected Behavior:**
- "Config should be self-documenting"
- Will edit JSON directly (won't use setup wizard again)
- Expects hot reload or fast restart

## Success Criteria

### Time Limits
- **Excellent:** < 10 minutes (install → productive)
- **Good:** 10-15 minutes
- **Acceptable:** 15-20 minutes
- **Poor:** > 20 minutes (too slow for experts)

### Zero-Docs Test
**Can they complete phases 1-3 without reading ANY docs?**
- Only README headings/snippets allowed
- No deep-dive into architecture, MCP protocol, etc.

**Success:** Yes, completed with just README snippets
**Failure:** Had to read detailed docs to proceed

### Satisfaction Score (1-10)
- **9-10:** "This is my new default tool"
- **7-8:** "Useful, will keep using it"
- **5-6:** "Meh, might use occasionally"
- **1-4:** "Not worth the setup time"

### Key Questions
1. Would you install this again on your next project?
2. Is it faster than your current workflow?
3. Did anything surprise you (good or bad)?
4. What would make you uninstall it?
5. Would you recommend to your team?

## Advanced Use Cases to Test

### Multi-Language Monorepo
- TypeScript frontend + Rust backend
- Does setup detect both correctly?
- Do tools work across language boundaries?

### Custom LSP Configuration
- Rust-analyzer with special clippy flags
- TypeScript with monorepo plugin
- Can they configure without examples?

### CI/CD Integration (Preview)
- Can they see how to use in GitHub Actions?
- Is there a Docker image for CI?
- Does it work non-interactively?

### Team Rollout
- How would they share config with team?
- Can config be checked into repo?
- Is there a team/workspace mode?

## Documentation Gaps to Identify

This test should reveal:
- [ ] README too verbose (experts won't read walls of text)
- [ ] Missing "quick reference" card
- [ ] Unclear performance characteristics
- [ ] No advanced examples
- [ ] Missing config schema documentation
- [ ] No troubleshooting for experts (they debug differently)
- [ ] Unclear upgrade path
- [ ] Missing comparison to alternatives

## Test Execution Plan

### Recruit 3 Test Users
- 1 Rust expert (uses Rust daily)
- 1 TypeScript expert (large-scale frontend dev)
- 1 polyglot senior (uses 3+ languages daily)

### Test Protocol
1. **Minimal briefing:** "Here's the repo, install it and use it. Time starts now."
2. **Screen + audio recording:** Capture everything
3. **No help allowed:** Must succeed or fail independently
4. **Time tracking:** Stopwatch from `git clone` to "I'm productive"
5. **Post-test interview:** What was fast, what was slow, what was confusing

### Data Collection
- Time to productivity
- Docs sections visited (track with analytics or ask)
- Number of errors/restarts
- Tool usage patterns
- Feature discovery rate
- Direct quotes about experience

## Deliverables

1. **Speed test report:** Average time to productivity across 3 users
2. **Quick reference card:** One-page cheat sheet for experts
3. **README streamlining:** Cut verbose sections, add speed-focused path
4. **Advanced examples:** Real-world use cases for power users
5. **Performance benchmarks:** Document expected performance characteristics

## Timeline
- Recruit testers: 3 days
- Run tests: 3 days (1 hour per session + analysis)
- Create quick reference: 1 day
- Streamline README: 2 days
- Add advanced examples: 2 days

**Total:** ~2 weeks

## Success Metrics for README

After improvements, expert developers should:
- Install in < 3 min (from discovery to first command)
- Configure in < 5 min (setup + MCP integration)
- Productive in < 15 min (using advanced features)
- Satisfaction score ≥ 8/10

## Agent Assignment
**Agent 2:** Run quick-start test, measure speed, identify verbose docs sections, create quick reference materials
