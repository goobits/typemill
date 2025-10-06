# Java Import & Workspace Support Implementation Plan

## Overview
Add ImportSupport and WorkspaceSupport capabilities to the Java language plugin following the Phase 2 architecture pattern.

## Phase 1: Import Support (Priority: HIGH, Effort: 2-3 days)

### Files to Create
- `crates/languages/cb-lang-java/src/import_support.rs` (~300 LOC)

### Implementation Details

#### 1.1 Regex Patterns
```rust
// Regular import: import java.util.List;
const IMPORT_PATTERN: &str = r"^\s*import\s+(?P<path>[a-zA-Z0-9._*]+)\s*;";

// Static import: import static org.junit.Assert.*;
const STATIC_IMPORT_PATTERN: &str = r"^\s*import\s+static\s+(?P<path>[a-zA-Z0-9._*]+)\s*;";

// Package declaration: package com.example.myapp;
const PACKAGE_PATTERN: &str = r"^\s*package\s+(?P<path>[a-zA-Z0-9._]+)\s*;";
```

#### 1.2 Core Methods

**parse_imports(&self, content: &str) -> Vec<String>**
- Scan for `import` statements using regex
- Extract package paths
- Include both static and regular imports
- Return list like: `["java.util.List", "java.util.ArrayList", "static org.junit.Assert.*"]`

**rewrite_imports_for_rename(&self, content: &str, old: &str, new: &str) -> (String, usize)**
- Find imports matching old package/class name
- Replace with new name
- Example: `import com.old.MyClass` → `import com.new.MyClass`
- Return (updated content, number of changes)

**rewrite_imports_for_move(&self, content: &str, old_path: &Path, new_path: &Path) -> (String, usize)**
- Convert file paths to package paths:
  - `src/main/java/com/example/Foo.java` → `com.example.Foo`
- Update imports referencing the moved class
- Handle source root detection (src/main/java, src/, etc.)

**contains_import(&self, content: &str, module: &str) -> bool**
- Check if specific import exists
- Handle both exact match and wildcard:
  - `module = "java.util.List"` matches `import java.util.List;`
  - `module = "java.util.List"` also matches `import java.util.*;`

**add_import(&self, content: &str, module: &str) -> String**
- Insert point: After package declaration, before first class
- Maintain alphabetical ordering (optional nicety)
- Avoid duplicates
- Format: `import {module};`

**remove_import(&self, content: &str, module: &str) -> String**
- Remove lines matching the import
- Clean up extra blank lines

#### 1.3 Helper Functions

```rust
/// Convert file path to Java package path
/// Example: src/main/java/com/example/Foo.java -> com.example.Foo
fn file_path_to_package(path: &Path) -> Option<String> {
    // Find source root (src/main/java, src/, etc.)
    // Extract package path from directory structure
}

/// Extract package declaration from source
/// Example: "package com.example.myapp;" -> Some("com.example.myapp")
fn parse_package_declaration(content: &str) -> Option<String> {
    // Use PACKAGE_PATTERN regex
}

/// Find insertion point for new import
/// Returns line index after package declaration and existing imports
fn find_import_insertion_point(content: &str) -> usize {
    // Scan for last import statement or package declaration
}
```

#### 1.4 Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_imports() {
        let source = r#"
            package com.example;

            import java.util.List;
            import java.util.ArrayList;
            import static org.junit.Assert.*;

            public class MyClass { }
        "#;

        let support = JavaImportSupport;
        let imports = support.parse_imports(source);
        assert_eq!(imports.len(), 3);
        assert!(imports.contains(&"java.util.List".to_string()));
    }

    #[test]
    fn test_rewrite_imports_for_rename() { /* ... */ }

    #[test]
    fn test_add_import() { /* ... */ }

    // ... 5-6 more tests
}
```

### 1.5 Wire to Plugin

Update `crates/languages/cb-lang-java/src/lib.rs`:

```rust
pub mod import_support;

pub struct JavaPlugin {
    metadata: LanguageMetadata,
    import_support: import_support::JavaImportSupport,
}

impl JavaPlugin {
    pub fn new() -> Self {
        Self {
            metadata: LanguageMetadata::JAVA,
            import_support: import_support::JavaImportSupport,
        }
    }
}

impl LanguagePlugin for JavaPlugin {
    fn capabilities(&self) -> LanguageCapabilities {
        LanguageCapabilities {
            imports: true,   // ✅ Now enabled!
            workspace: false,
        }
    }

    fn import_support(&self) -> Option<&dyn ImportSupport> {
        Some(&self.import_support)
    }
}
```

### 1.6 Effort Estimate
- **Regex patterns**: 0.5 day
- **Core methods**: 1.5 days
- **Helper functions**: 0.5 day
- **Tests**: 0.5 day
- **Total**: 3 days

---

## Phase 2: Maven Workspace Support (Priority: MEDIUM, Effort: 3-4 days)

### Files to Create
- `crates/languages/cb-lang-java/src/workspace_support.rs` (~350 LOC)

### Implementation Details

#### 2.1 XML Manipulation Strategy

Use `quick-xml` crate (already a dependency):
- `quick_xml::de::from_str()` - Parse XML to struct
- `quick_xml::se::to_string()` - Serialize struct back to XML
- OR use `quick_xml::Reader/Writer` for direct manipulation

**Approach**: Similar to Rust's `toml_edit` pattern:
```rust
use quick_xml::events::{Event, BytesStart, BytesText};
use quick_xml::Reader;
```

#### 2.2 Core Methods

**add_workspace_member(&self, content: &str, member: &str) -> String**
```rust
// Parse pom.xml
// Find <modules> section (or create it)
// Add <module>{member}</module>
// Serialize back to string
```

**remove_workspace_member(&self, content: &str, member: &str) -> String**
```rust
// Parse pom.xml
// Find <modules> section
// Remove matching <module> element
// Serialize back
```

**is_workspace_manifest(&self, content: &str) -> bool**
```rust
// Check if <modules> element exists
content.contains("<modules>") || content.contains("<modules/>")
```

**list_workspace_members(&self, content: &str) -> Vec<String>**
```rust
// Parse <modules> section
// Extract all <module> text content
// Return as Vec<String>
```

**update_package_name(&self, content: &str, new_name: &str) -> String**
```rust
// Parse pom.xml
// Update <artifactId> in <project> root
// Optionally update <groupId>
// Serialize back
```

#### 2.3 XML Helper Functions

```rust
/// Parse pom.xml to structured data
fn parse_pom(content: &str) -> Result<PomProject, String> {
    quick_xml::de::from_str(content)
        .map_err(|e| format!("Failed to parse pom.xml: {}", e))
}

/// Serialize pom back to XML string
fn serialize_pom(pom: &PomProject) -> String {
    quick_xml::se::to_string(pom)
        .unwrap_or_else(|_| String::new())
}

/// Extract <modules> section from pom.xml
fn extract_modules(content: &str) -> Vec<String> {
    // Use quick_xml::Reader to scan for <module> elements
}
```

#### 2.4 Data Structures

```rust
#[derive(Debug, Serialize, Deserialize)]
struct PomProject {
    #[serde(rename = "groupId")]
    group_id: Option<String>,

    #[serde(rename = "artifactId")]
    artifact_id: String,

    version: Option<String>,

    modules: Option<PomModules>,

    dependencies: Option<PomDependencies>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PomModules {
    #[serde(rename = "module")]
    module: Vec<String>,
}
```

#### 2.5 Tests
```rust
#[test]
fn test_add_workspace_member() {
    let pom = r#"
        <project>
            <artifactId>parent</artifactId>
            <modules>
                <module>module-a</module>
            </modules>
        </project>
    "#;

    let support = JavaWorkspaceSupport;
    let result = support.add_workspace_member(pom, "module-b");
    assert!(result.contains("<module>module-b</module>"));
}

#[test]
fn test_is_workspace_manifest() { /* ... */ }

// ... 5-6 more tests
```

### 2.6 Wire to Plugin

Update `lib.rs`:
```rust
pub mod workspace_support;

pub struct JavaPlugin {
    metadata: LanguageMetadata,
    import_support: import_support::JavaImportSupport,
    workspace_support: workspace_support::JavaWorkspaceSupport,
}

impl LanguagePlugin for JavaPlugin {
    fn capabilities(&self) -> LanguageCapabilities {
        LanguageCapabilities {
            imports: true,
            workspace: true,  // ✅ Now enabled!
        }
    }

    fn workspace_support(&self) -> Option<&dyn WorkspaceSupport> {
        Some(&self.workspace_support)
    }
}
```

### 2.7 Effort Estimate
- **XML parsing setup**: 0.5 day
- **Core methods**: 2 days
- **Helper functions**: 0.5 day
- **Tests**: 0.5 day
- **Debugging XML quirks**: 0.5 day
- **Total**: 4 days

---

## Phase 3: Gradle Support (Priority: LOW, Effort: 5-7 days)

### Status: DEFERRED

**Reasons to defer:**
1. Gradle files are code, not data (very hard to parse/manipulate)
2. Maven is more common in enterprise Java
3. Much higher complexity for marginal benefit
4. No good Gradle parser in Rust ecosystem

**If needed later:**
- Use simple regex for basic `include` statements
- Or shell out to `gradle` CLI for queries
- Or wait for tree-sitter-groovy/kotlin to mature

---

## Implementation Order

### Week 1: Import Support
1. Day 1: Setup + regex patterns + parse_imports
2. Day 2: Rewrite methods + path conversion helpers
3. Day 3: Add/remove imports + tests + wire to plugin

### Week 2: Maven Workspace (Optional)
4. Day 4: XML parsing + is_workspace + list_members
5. Day 5: Add/remove members
6. Day 6: Update package name + tests
7. Day 7: Integration testing + wire to plugin

## Success Criteria

### Phase 1 (Import Support)
- ✅ Can parse imports from Java source
- ✅ Can add/remove imports correctly
- ✅ Can rewrite imports when classes move
- ✅ All 6 ImportSupport methods implemented
- ✅ 10+ tests passing
- ✅ Capabilities flag updated: `imports: true`

### Phase 2 (Maven Workspace)
- ✅ Can detect Maven multi-module projects
- ✅ Can list workspace members
- ✅ Can add/remove modules from parent pom
- ✅ All 5 WorkspaceSupport methods implemented
- ✅ 10+ tests passing
- ✅ Capabilities flag updated: `workspace: true`

## Alternatives Considered

### Alternative 1: Use JavaParser AST for imports
**Pros**: 100% accurate, handles edge cases
**Cons**: Requires Java runtime, slower, JAR dependency
**Decision**: Regex is good enough for MVP

### Alternative 2: Shell out to Maven/Gradle CLI
**Pros**: Delegates to official tools
**Cons**: Slow, requires tools installed, hard to test
**Decision**: Direct XML manipulation is better

### Alternative 3: Skip workspace support entirely
**Pros**: Less work
**Cons**: Incomplete feature parity with Rust
**Decision**: Implement Maven support, skip Gradle

## Risk Assessment

### Low Risk
- Import parsing with regex (proven pattern, TypeScript/Go use it)
- Maven XML manipulation (quick-xml is mature)

### Medium Risk
- File path → package path conversion (edge cases with non-standard structures)
- Maven module relative paths (need to handle ../other-module cases)

### High Risk (Mitigated by deferring)
- Gradle support (complexity too high, deferred)

## Dependencies Required

### Already Available
- ✅ `regex` crate (workspace dependency)
- ✅ `quick-xml` crate (already in Java plugin)
- ✅ `serde` for XML deserialization

### New Dependencies Needed
- None! Everything we need is already available.

## Testing Strategy

### Unit Tests
- Each method in ImportSupport (6 methods × 2-3 tests = 12-18 tests)
- Each method in WorkspaceSupport (5 methods × 2-3 tests = 10-15 tests)
- Helper functions (5-10 tests)
- **Total**: ~30-40 unit tests

### Integration Tests
- Parse real-world pom.xml files
- Handle malformed XML gracefully
- Test with actual Java source files

### Edge Cases to Test
- Empty import blocks
- Imports in comments
- Wildcard imports
- Static imports
- Package-less files (default package)
- Multi-module Maven projects with relative paths
- Missing <modules> section (create it)

## Documentation Updates

After implementation, update:
1. `crates/languages/cb-lang-java/README.md` - Document capabilities
2. `API.md` - Update Java plugin capabilities table
3. `crates/languages/README.md` - Reference Java as example
4. `00_PROPOSAL_LANGUAGE_PLUGIN_REFACTOR.md` - Mark Java as complete

## Future Enhancements (Post-MVP)

1. **AST-based Import Parsing** (when JAR is built)
   - Use JavaParser for 100% accuracy
   - Fallback to regex when Java unavailable

2. **Smart Import Ordering**
   - Group imports by package
   - Separate static imports
   - Follow Google/Oracle Java style guide

3. **Gradle Support** (if demanded)
   - Basic regex-based include manipulation
   - Or wait for better Groovy parser

4. **Import Optimization**
   - Remove unused imports
   - Convert wildcard imports to explicit
   - Suggest missing imports

## Timeline Summary

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Import Support | 3 days | Ready to start |
| Phase 2: Maven Workspace | 4 days | Optional |
| Phase 3: Gradle Support | 7 days | Deferred |

**Minimum Viable**: Phase 1 only (3 days)
**Full Maven Support**: Phase 1 + 2 (7 days)
**Complete**: All phases (14 days)

**Recommendation**: Start with Phase 1, validate with users, then decide on Phase 2.
