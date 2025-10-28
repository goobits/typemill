# Proposal 14: Markdown Analysis Extensions

## Problem

Markdown plugin supports link tracking for renames but lacks quality analysis:
- No broken link detection (404s, invalid anchors)
- No structure validation (heading hierarchy, duplicates)
- No formatting checks (missing alt text, code language tags)
- No documentation completeness analysis (README sections)
- Analysis handlers don't leverage markdown plugin's link parsing capabilities

Current `analyze.quality` and `analyze.documentation` don't have markdown-specific detection functions.

## Solution

Extend analysis handlers with markdown-specific detection functions that use the markdown plugin's existing capabilities (symbol parsing for headers, import parsing for links).

### Architecture
- Analysis handlers call `language_plugins.get_plugin_for_file()` to get markdown plugin
- Use `plugin.parse()` to get headers as symbols
- Use `plugin.import_parser()` to get links
- Add new `kind` values to `analyze.quality` and `analyze.documentation`

### New Analysis Kinds

**`analyze.quality`:**
- `kind: "link_validity"` - Broken links, invalid anchors, circular dependencies
- `kind: "markdown_structure"` - Heading hierarchy, duplicate headings, empty sections
- `kind: "markdown_formatting"` - Code block language tags, image alt text, table consistency

**`analyze.documentation`:**
- `kind: "completeness"` (enhanced) - README required sections, cross-references

## Checklists

### Phase 1: Link Validity (Priority 1)
- [ ] Add `detect_broken_links()` function in `crates/mill-handlers/src/handlers/tools/analysis/quality.rs`
- [ ] Implement file existence checking for markdown links
- [ ] Implement anchor validation (check `#heading` exists in target file)
- [ ] Add `link_validity` kind to quality handler dispatcher
- [ ] Test with broken links, invalid anchors, relative paths
- [ ] Document `link_validity` kind in `docs/tools/analysis.md`

### Phase 2: Structure Validation (Priority 2)
- [ ] Add `detect_structure_issues()` function in quality.rs
- [ ] Implement heading hierarchy validation (no skipped levels)
- [ ] Implement duplicate heading detection
- [ ] Implement empty section detection
- [ ] Add `markdown_structure` kind to quality handler dispatcher
- [ ] Test with various heading structures
- [ ] Document `markdown_structure` kind in docs

### Phase 3: Formatting Checks (Priority 3)
- [ ] Add `detect_formatting_issues()` function in quality.rs
- [ ] Implement code block language tag checking
- [ ] Implement image alt text checking
- [ ] Implement table column consistency checking
- [ ] Implement list bullet style consistency
- [ ] Add `markdown_formatting` kind to quality handler dispatcher
- [ ] Test with various markdown formatting patterns
- [ ] Document `markdown_formatting` kind in docs

### Phase 4: Documentation Completeness (Priority 4)
- [ ] Add `detect_documentation_completeness()` in `documentation.rs`
- [ ] Implement README section detection (Installation, Usage, Examples, Contributing, License)
- [ ] Enhance existing `completeness` kind with markdown-specific checks
- [ ] Test with READMEs missing sections
- [ ] Update docs with markdown completeness criteria

### Integration
- [ ] Update `docs/tools/analysis.md` with all new markdown analysis kinds
- [ ] Add markdown analysis examples to cheatsheet
- [ ] Add integration tests for markdown analysis in `tests/e2e/`
- [ ] Update CLAUDE.md with markdown analysis capabilities

## Success Criteria

- `analyze.quality` supports `link_validity`, `markdown_structure`, and `markdown_formatting` kinds for markdown files
- Link validation detects broken file links and invalid heading anchors
- Structure validation catches heading hierarchy issues and duplicates
- Formatting validation identifies missing alt text and code language tags
- `analyze.documentation` completeness check identifies missing README sections
- All analysis functions use language plugin architecture (no hardcoded markdown assumptions)
- Documentation updated with markdown analysis examples
- Integration tests pass for all new analysis kinds

## Benefits

- **Documentation Quality**: Catch broken links before users encounter them
- **Consistency**: Enforce markdown formatting standards across projects
- **Accessibility**: Detect missing alt text for images
- **Structure**: Maintain proper document hierarchy
- **Maintenance**: Identify documentation gaps in READMEs
- **Architecture**: Demonstrates proper plugin integration for language-specific analysis
