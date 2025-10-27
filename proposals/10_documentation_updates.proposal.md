# Proposal 10: Documentation Updates for Analysis Features

## Problem

Documentation doesn't reflect Proposal 00's actionable suggestions features:

1. **Outdated Examples**: Tool examples don't show suggestion output
2. **Missing Features**: Actionable suggestions not documented
3. **No Workflows**: Missing guidance on using suggestions effectively
4. **Tool Docs**: Individual tool docs don't explain suggestion generation
5. **Configuration**: No docs for suggestion configuration options

Users cannot discover or use new features effectively without updated documentation.

## Solution(s)

### 1. Update Tool Documentation

Update all analysis tool docs to show suggestion output:

**Before:**
```markdown
## analyze.quality

Returns code quality findings including complexity and code smells.
```

**After:**
```markdown
## analyze.quality

Returns code quality findings with actionable suggestions.

### Output Structure
- `findings`: Raw analysis findings
- `suggestions`: Actionable next steps with exact commands
- `summary`: Aggregated statistics

### Example
Shows findings with rename and extract suggestions.
```

### 2. Add Suggestion Documentation

Create `docs/features/actionable_suggestions.md`:

- What are actionable suggestions
- How they're generated
- Confidence levels explained
- Safety levels explained
- How to execute suggestions
- How to customize via config

### 3. Update Quick Start Guide

Add suggestions to getting started flow:

```markdown
1. Run analysis: `mill analyze quality src/main.rs`
2. Review suggestions in output
3. Execute high-confidence suggestions
4. Re-run analysis to verify improvements
```

### 4. Add Workflow Guides

Create workflow documentation:

- **Interactive Workflow**: Using suggestions during development
- **CI/CD Workflow**: Automated quality enforcement
- **Refactoring Workflow**: Using suggestions for large refactors
- **Team Workflow**: Sharing analysis configs

### 5. Update API Reference

Update MCP tool documentation:

```markdown
### analyze.quality

**Response includes:**
- `findings`: Array of quality issues
- `suggestions`: Array of actionable suggestions
  - `tool`: MCP tool name to execute
  - `arguments`: Exact parameters
  - `confidence`: 0.0-1.0 confidence score
  - `reason`: Human explanation
  - `safety_level`: safe | requires_review | requires_validation
```

### 6. Add Configuration Examples

Document configuration options:

```toml
# .typemill/analysis.toml
[suggestions]
min_confidence = 0.7
include_safety_levels = ["safe", "requires_review"]
max_per_finding = 3
generate_refactor_calls = true
```

## Checklists

### Tool Documentation Updates
- [ ] Update `docs/tools/analysis.md` with suggestions
- [ ] Update `docs/tools/refactoring.md` with workflow examples
- [ ] Add suggestion output to all tool examples
- [ ] Document suggestion structure (tool, arguments, confidence, reason, safety)
- [ ] Add "See Also" links between analysis and refactoring docs

### New Documentation Pages
- [ ] Create `docs/features/actionable_suggestions.md`
- [ ] Create `docs/guides/analysis_workflow.md`
- [ ] Create `docs/guides/ci_integration.md`
- [ ] Create `docs/configuration/analysis.md`
- [ ] Add all new pages to navigation

### Quick Start Updates
- [ ] Add suggestions to quick start guide
- [ ] Add screenshot/example of suggestion output
- [ ] Add workflow: analyze → review → execute → verify
- [ ] Link to detailed guides

### API Reference Updates
- [ ] Update `analyze.quality` response schema
- [ ] Update `analyze.dead_code` response schema
- [ ] Update all analysis tools with suggestion output
- [ ] Add `ActionableSuggestion` type documentation
- [ ] Document confidence and safety level meanings

### Configuration Documentation
- [ ] Document `.typemill/analysis.toml` format
- [ ] Document all suggestion configuration options
- [ ] Document preset configurations
- [ ] Add examples for common scenarios
- [ ] Document environment variable overrides

### Example Updates
- [ ] Update all code examples to show suggestions
- [ ] Add real-world workflow examples
- [ ] Add CI/CD integration examples
- [ ] Add before/after examples
- [ ] Add troubleshooting examples

### Website Updates
- [ ] Update website home page features
- [ ] Update website tools page with suggestions
- [ ] Update website docs page navigation
- [ ] Add suggestions to feature highlights
- [ ] Update screenshots/demos

## Success Criteria

1. **Completeness**: All analysis tools documented with suggestions
2. **Examples**: Every tool doc has working example with suggestion output
3. **Workflows**: At least 3 workflow guides (interactive, CI, refactoring)
4. **Discoverability**: New features visible in quick start and home page
5. **Configuration**: All config options documented with examples
6. **Navigation**: New docs linked in navigation and "See Also" sections

## Benefits

1. **User Awareness**: Users discover and use actionable suggestions
2. **Reduced Support**: Self-service documentation answers common questions
3. **Faster Onboarding**: New users understand suggestion workflow immediately
4. **Better Adoption**: Clear examples drive feature usage
5. **Consistency**: Unified documentation across all tools
6. **Maintainability**: Documentation matches implementation
