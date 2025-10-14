# Markdown Link Detection and Updates

## Problem

Moving or renaming markdown files breaks cross-references across documentation, creating dead links that aren't detected or updated by CodeBuddy's rename operations.

**Discovered during dogfooding:** When attempting to move `docs/development/logging_guidelines.md` → `docs/logging_guidelines.md`:

```json
// rename.plan output
{
  "summary": {
    "affected_files": 1,  // ❌ Wrong - should be 6
    "text_edits": 0       // ❌ Wrong - should be 7
  }
}
```

**Reality:** 5 files contain 7 markdown link references that would break:
- `CLAUDE.md` (2 references)
- `AGENTS.md` (2 references)
- `CONTRIBUTING.md` (1 reference)
- `docs/development/languages/plugin_development_guide.md` (2 references)

**Impact:** After applying the rename plan, all 7 links become 404s.

## Root Cause

The import path extractor (`crates/cb-ast/src/import_updater/file_scanner.rs:extract_import_path`) only detects:
- ES6 imports: `from './path'`
- CommonJS: `require('./path')`

**Missing:**
- Markdown links: `[text](path/to/file.md)`
- Markdown reference links: `[text][ref]` + `[ref]: path`
- HTML anchor links: `<a href="path">text</a>`

```rust
// Current implementation
pub fn extract_import_path(line: &str) -> Option<String> {
    if line.contains("from") { /* ES6 */ }
    if line.contains("require") { /* CommonJS */ }

    // ❌ NO MARKDOWN SUPPORT
    None
}
```

## Solution

Add markdown link detection to the import path extraction and reference tracking system.

### Implementation Approach

**Phase 1:** Extend `extract_import_path` to detect markdown syntax
**Phase 2:** Add markdown file type handling to import updater
**Phase 3:** Create test fixtures and integration tests
**Phase 4:** Validate with real documentation moves

## Checklists

### Phase 1: Link Pattern Detection

- [ ] Add markdown inline link regex: `\[([^\]]+)\]\(([^)]+)\)`
- [ ] Extract path from capture group 2
- [ ] Support relative paths: `./file.md`, `../dir/file.md`, `docs/file.md`
- [ ] Support absolute paths from project root: `/docs/file.md`
- [ ] Support anchors: `path/file.md#section` (preserve anchor)
- [ ] Handle URL-encoded paths: `docs/my%20file.md`

**Implementation location:** `crates/cb-ast/src/import_updater/file_scanner.rs`

```rust
pub fn extract_import_path(line: &str) -> Option<String> {
    // Existing: ES6 and CommonJS
    // ...

    // NEW: Markdown inline links [text](path)
    if let Some(caps) = MARKDOWN_LINK_REGEX.captures(line) {
        if let Some(path_match) = caps.get(2) {
            let path = path_match.as_str();
            // Skip external URLs
            if !path.starts_with("http://") && !path.starts_with("https://") {
                // Strip anchor if present
                let path_without_anchor = path.split('#').next().unwrap_or(path);
                return Some(path_without_anchor.to_string());
            }
        }
    }

    None
}
```

### Phase 2: Markdown Reference Link Support

- [ ] Detect reference-style links: `[text][ref]`
- [ ] Detect reference definitions: `[ref]: path/to/file.md`
- [ ] Track references across the same file
- [ ] Update both reference usage and definition on file move

**Pattern examples:**
```markdown
See [the guide][guide] for details.

[guide]: docs/development/guide.md
```

When `docs/development/guide.md` moves, update the definition line.

### Phase 3: File Type Handling

- [ ] Add `.md` extension to supported file types in import scanner
- [ ] Ensure markdown files are included in `find_project_files`
- [ ] Don't skip `.md` files in rename operations
- [ ] Add language detection for markdown (file extension `.md`, `.markdown`)

**Verify:** `find_project_files` should return markdown files alongside source code files.

### Phase 4: Path Resolution

- [ ] Handle different path styles in markdown:
  - Relative to current file: `./file.md`, `../dir/file.md`
  - Relative to project root: `docs/file.md` (no leading `.` or `/`)
  - Absolute from root: `/docs/file.md`
- [ ] Resolve ambiguous bare filenames: `CONTRIBUTING.md` could be `./CONTRIBUTING.md` or `docs/CONTRIBUTING.md`
- [ ] Prioritize exact path matches over basename matches
- [ ] Support case-insensitive filesystems (macOS/Windows)

### Phase 5: Test Fixtures

- [ ] Create `MARKDOWN_RENAME_FILE_TESTS` in `crates/cb-test-support/src/harness/mcp_fixtures.rs`
- [ ] Test case: Single file with relative link
- [ ] Test case: Multiple files with links to moved file
- [ ] Test case: Nested directory paths (`../../docs/file.md`)
- [ ] Test case: Reference-style links
- [ ] Test case: Mixed content (markdown + code blocks with imports)
- [ ] Test case: Links with anchors (`file.md#section`)
- [ ] Test case: External URLs (should NOT be updated)

**Example fixture:**
```rust
pub const MARKDOWN_RENAME_FILE_TESTS: &[RenameFileTestCase] = &[
    RenameFileTestCase {
        test_name: "markdown_relative_link_update",
        initial_files: &[
            ("docs/guide.md", "# Guide\nContent here."),
            ("README.md", "See [guide](docs/guide.md) for details."),
        ],
        old_file_path: "docs/guide.md",
        new_file_path: "docs/user-guide.md",
        expect_success: true,
        expected_import_updates: &[
            ("README.md", "See [guide](docs/user-guide.md)"),
        ],
    },
    // More cases...
];
```

### Phase 6: Integration Tests

- [ ] Add `test_markdown_file_rename_updates_links` to `integration-tests/src/test_rename_with_imports.rs`
- [ ] Test moving file between directories
- [ ] Test renaming file in same directory
- [ ] Test multiple markdown files referencing the same file
- [ ] Test markdown file in subdirectory referencing parent

### Phase 7: Dry-Run Preview Validation

- [ ] Verify dry-run shows correct `affected_files` count for markdown renames
- [ ] Verify `import_updates.files_to_modify` lists all markdown files with references
- [ ] Verify preview shows old/new link text in edit plan

**Test with real example:**
```bash
codebuddy tool rename.plan '{
  "target": {"kind": "file", "path": "docs/development/logging_guidelines.md"},
  "new_name": "docs/logging_guidelines.md"
}'

# Expected output:
# "affected_files": 6  (1 renamed + 5 with links)
# "documentChanges": [...] (should include text edits for CLAUDE.md, etc.)
```

### Phase 8: Edge Cases

- [ ] Handle image links: `![alt](images/diagram.png)`
- [ ] Handle autolinks: `<http://example.com>` (skip external)
- [ ] Handle inline code: `` `[not a link](path)` `` (skip)
- [ ] Handle code blocks: Don't parse markdown syntax inside fenced code blocks
- [ ] Handle escaped brackets: `\[not a link\](path)` (skip)
- [ ] Handle malformed links: `[unclosed](path` (graceful failure)

## Success Criteria

- [ ] Moving `docs/development/logging_guidelines.md` → `docs/logging_guidelines.md` shows 6 affected files
- [ ] Dry-run preview includes 7 text edits across 5 markdown files
- [ ] Applying the edit updates all markdown links correctly
- [ ] Test suite includes 8+ markdown rename test cases
- [ ] External URLs (https://) are never modified
- [ ] Anchors in links are preserved: `file.md#section` → `newfile.md#section`
- [ ] Zero false positives (markdown in code blocks ignored)

## Benefits

- **Prevents broken documentation** - Dead links caught before commit
- **Enables confident refactoring** - Move docs without manual link updates
- **Improves dogfooding** - CodeBuddy can manage its own documentation
- **Better dry-run accuracy** - Preview shows all affected documentation
- **Reduces manual work** - No grep/sed scripts needed for doc reorganization

## Technical Notes

### Regex Pattern for Markdown Links

```rust
// Inline links: [text](path)
static MARKDOWN_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap()
});

// Reference definitions: [ref]: path
static MARKDOWN_REF_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\[([^\]]+)\]:\s+(.+)$").unwrap()
});
```

### Excluding Code Blocks

When scanning markdown files, skip content inside fenced code blocks:

```rust
fn is_inside_code_block(lines: &[&str], current_index: usize) -> bool {
    let mut in_block = false;
    for (i, line) in lines.iter().enumerate() {
        if i >= current_index {
            break;
        }
        if line.starts_with("```") || line.starts_with("~~~") {
            in_block = !in_block;
        }
    }
    in_block
}
```

### Path Resolution Priority

When resolving `CONTRIBUTING.md`:
1. Try exact match: `./CONTRIBUTING.md` (same directory)
2. Try project root: `<project_root>/CONTRIBUTING.md`
3. Try basename search: Find any file named `CONTRIBUTING.md` in project
4. If multiple matches, choose closest (fewest directory hops)

### Testing Strategy

**Unit tests:** Path extraction and pattern matching
**Integration tests:** End-to-end rename with link updates
**Fixtures:** Comprehensive markdown scenarios
**Manual validation:** Real documentation moves in this repository

## References

- **Markdown spec:** CommonMark links: https://spec.commonmark.org/0.30/#links
- **Existing tests:** `integration-tests/src/test_rename_with_imports.rs`
- **Import updater:** `crates/cb-ast/src/import_updater/`
- **Discovery issue:** Proposal 04 (TypeMill rename dogfooding)

## Related Work

- **Proposal 00:** Rust move test coverage (established test pattern)
- **Proposal 04:** TypeMill rename (dogfooding effort that discovered this gap)
- **Proposal 05:** Fix search_symbols and workspace analysis (parallel dogfooding fixes)
