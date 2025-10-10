# Proposal 15: Complete Beginner Path 🆕

## Status
**Proposed** - Onboarding test path for complete beginners

## Overview
Test if someone with zero prerequisites can go from nothing → working Codebuddy MCP server in < 90 minutes using only the README.

## Persona: Junior Dev / Bootcamp Grad

**Profile:**
- Recent bootcamp graduate or junior developer
- Never used Rust before
- Never used Docker before
- Familiar with basic command line (cd, ls, etc.)
- Has used AI coding assistants (Claude, Cursor, etc.)
- No prior knowledge of MCP or LSP

**Starting Point:**
- ❌ No Rust installed
- ❌ No Docker installed
- ❌ No language servers installed
- ❌ No prior knowledge of MCP/LSP
- ✅ Has Node.js/npm (common for web devs)
- ✅ Basic terminal proficiency

## Goal
Complete beginner can install, configure, and use Codebuddy with their AI assistant in < 90 minutes without external help (only README + docs).

## Test Phases

### Phase 1: Discovery (5 min)
**Goal:** Can they understand what Codebuddy does and why they need it?

**Tasks:**
- [ ] Read project README header
- [ ] Understand "what can it do" section
- [ ] Identify which AI assistant they're using (Claude Code, Cursor, etc.)
- [ ] Decide if this tool is relevant to them

**Success Metrics:**
- Can explain in 1 sentence what Codebuddy does
- Knows which section to start with (end user vs developer)

### Phase 2: Prerequisites Installation (25 min)
**Goal:** Install Rust from scratch

**Tasks:**
- [ ] Find Rust installation instructions (external)
- [ ] Run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [ ] Restart terminal or source environment
- [ ] Verify with `rustc --version`
- [ ] Understand they DON'T need Docker for basic usage

**Success Metrics:**
- Rust installed and in PATH
- Can run `cargo --version`

**Common Friction Points:**
- PATH not updated (need to restart terminal)
- Confusion about which installation method to use
- Uncertainty about whether Docker is required

### Phase 3: Codebuddy Installation (10 min)
**Goal:** Install pre-built Codebuddy binary

**Tasks:**
- [ ] Run install script: `curl -fsSL https://raw.githubusercontent.com/goobits/codebuddy/main/install.sh | bash`
- [ ] Verify with `codebuddy --version`
- [ ] Understand the difference between install script vs cargo install vs building from source

**Success Metrics:**
- `codebuddy` command works
- Binary is in PATH

**Common Friction Points:**
- Install script fails (need fallback to `cargo install codebuddy`)
- Binary not in PATH
- Confusion about which installation method to use

### Phase 4: Language Server Setup (20 min)
**Goal:** Install TypeScript language server (most common for web devs)

**Tasks:**
- [ ] Identify they need a language server
- [ ] Find TypeScript LSP install command in README
- [ ] Run `npm install -g typescript-language-server typescript`
- [ ] Verify installation with `typescript-language-server --version`
- [ ] Understand why they need this (LSP bridge concept)

**Success Metrics:**
- TypeScript language server installed and accessible
- Can explain why language server is needed (in simple terms)

**Common Friction Points:**
- Don't understand what LSP is
- Don't know which language server they need
- Permission errors with global npm install

### Phase 5: Project Configuration (15 min)
**Goal:** Configure Codebuddy for their project

**Tasks:**
- [ ] Navigate to their project directory
- [ ] Run `codebuddy setup`
- [ ] Follow interactive prompts
- [ ] Verify config created: `cat .codebuddy/config.json`
- [ ] Check status: `codebuddy status`

**Success Metrics:**
- `.codebuddy/config.json` exists
- `codebuddy status` shows "Working" for TypeScript
- Understand what the config does

**Common Friction Points:**
- Run setup in wrong directory
- Don't understand file extension mapping
- Config validation errors

### Phase 6: MCP Integration (10 min)
**Goal:** Connect Codebuddy to their AI assistant

**Tasks:**
- [ ] Find MCP config file location for their assistant
  - Claude Code: `~/.config/claude/mcp_config.json`
  - Cursor: Settings → MCP Servers
- [ ] Add Codebuddy config snippet from README
- [ ] Restart AI assistant
- [ ] Verify connection (check available MCP tools)

**Success Metrics:**
- Codebuddy appears in MCP servers list
- Can see tools like `find_definition`, `find_references`

**Common Friction Points:**
- Can't find MCP config file
- JSON syntax errors in config
- Don't know how to restart assistant

### Phase 7: First Use (15 min)
**Goal:** Use Codebuddy features with AI assistant

**Tasks:**
- [ ] Ask AI to "find definition of [function name]"
- [ ] Ask AI to "find all references to [variable]"
- [ ] Ask AI to "rename [symbol] to [new_name]"
- [ ] Observe multi-file updates working

**Success Metrics:**
- At least 2 successful tool invocations
- Can see difference between with/without Codebuddy
- Understand value proposition

**Common Friction Points:**
- AI doesn't use Codebuddy tools automatically
- Results are unexpected
- Don't understand what's happening under the hood

### Phase 8: Troubleshooting (10 min)
**Goal:** Fix common issues using built-in diagnostics

**Tasks:**
- [ ] Run `codebuddy status` to check health
- [ ] Run `codebuddy doctor` if issues found
- [ ] Check logs: `RUST_LOG=debug codebuddy start`
- [ ] Find answer in troubleshooting section

**Success Metrics:**
- Can diagnose common issues
- Know where to look for help
- Can read error messages

## Success Criteria

### Time Limits
- **Target:** < 90 minutes total
- **Acceptable:** < 120 minutes
- **Poor:** > 120 minutes

### Completion Rate
- **Success:** Complete all 8 phases
- **Partial:** Complete phases 1-6 (installation + config)
- **Failure:** Cannot complete phase 3 (install fails)

### Satisfaction Score (1-10)
- **Excellent:** 8-10 (would recommend to others)
- **Good:** 6-7 (useful but rough edges)
- **Poor:** 1-5 (frustrated, wouldn't recommend)

### Key Questions
1. Could you complete installation without external help?
2. Did you understand what Codebuddy does after using it?
3. Could you see the difference it made to your AI assistant?
4. Would you continue using it?
5. Would you recommend it to a colleague?

## Documentation Gaps to Identify

This test should reveal:
- [ ] Missing or unclear installation steps
- [ ] Confusing terminology (MCP, LSP, etc.)
- [ ] Missing prerequisite checks
- [ ] Poor error messages
- [ ] Unclear troubleshooting steps
- [ ] Missing "what to expect" guidance
- [ ] Gaps between README and detailed docs

## Test Execution Plan

### Recruit 3 Test Users
- 1 bootcamp grad (< 6 months experience)
- 1 junior dev (6-18 months experience)
- 1 career switcher (new to dev, experienced in other field)

### Test Protocol
1. **Pre-test survey:** Experience level, installed tools, expectations
2. **Screen recording:** Capture full session
3. **Think-aloud protocol:** Narrate what they're doing/thinking
4. **Time tracking:** Log time spent per phase
5. **Post-test survey:** Satisfaction, friction points, suggestions
6. **Follow-up interview:** What worked, what didn't, what confused them

### Data Collection
- Time per phase
- Error messages encountered
- Documentation sections visited
- Questions asked
- Satisfaction ratings
- Direct quotes about friction

## Deliverables

1. **Test report:** Summary of findings across 3 users
2. **Friction log:** Every point of confusion or difficulty
3. **README improvements:** Specific edits to address gaps
4. **Quick start checklist:** Step-by-step for beginners
5. **Error message audit:** Which errors need better explanations

## Timeline
- Recruit testers: 3 days
- Run tests: 1 week (3 sessions)
- Analyze data: 2 days
- Write report: 2 days
- Implement fixes: 1 week

**Total:** ~2.5 weeks

## Agent Assignment
**Agent 1:** Run beginner test, document ALL friction points, create detailed improvement recommendations
