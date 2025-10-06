# Java Import/Workspace Support - AST-Based Implementation Plan

## Architecture: Extend Existing JavaParser Tool

### Current Architecture
```
┌─────────────────┐         ┌──────────────────────┐
│ Rust Plugin     │  stdin  │ Java Parser JAR      │
│ (import_support)│ ──────> │ (javaparser-core)    │
│                 │ <────── │                      │
└─────────────────┘  stdout └──────────────────────┘
                      JSON
```

**Current Commands:**
- `extract-symbols` - Returns classes and methods as JSON

**New Commands Needed:**
- `parse-imports` - Extract import statements
- `add-import` - Add import to source
- `remove-import` - Remove import from source
- `rewrite-imports` - Rename/move imports
- `parse-package` - Get package declaration

---

## Phase 1: Extend Java Parser JAR (Import Support)

### 1.1 Add Import Parsing Command

**File**: `Parser.java` (extend existing)

```java
// Add to switch statement in main()
case "parse-imports":
    parseImports(source);
    break;
```

**New Method**:
```java
private static void parseImports(String source) {
    JavaParser javaParser = new JavaParser();
    ParseResult<CompilationUnit> result = javaParser.parse(source);

    if (!result.isSuccessful() || !result.getResult().isPresent()) {
        System.out.println("[]");
        return;
    }

    CompilationUnit cu = result.getResult().get();
    List<ImportInfo> imports = new ArrayList<>();

    // Extract all import declarations
    cu.getImports().forEach(importDecl -> {
        imports.add(new ImportInfo(
            importDecl.getNameAsString(),
            importDecl.isStatic(),
            importDecl.isAsterisk()
        ));
    });

    Gson gson = new Gson();
    System.out.println(gson.toJson(imports));
}

private static class ImportInfo {
    String path;      // "java.util.List"
    boolean isStatic; // import static X
    boolean isWildcard; // import X.*

    ImportInfo(String path, boolean isStatic, boolean isWildcard) {
        this.path = path;
        this.isStatic = isStatic;
        this.isWildcard = isWildcard;
    }
}
```

**Output Example**:
```json
[
  {"path": "java.util.List", "isStatic": false, "isWildcard": false},
  {"path": "java.util.ArrayList", "isStatic": false, "isWildcard": false},
  {"path": "org.junit.Assert", "isStatic": true, "isWildcard": true}
]
```

### 1.2 Add Import Manipulation Commands

**Add Import**:
```java
case "add-import":
    String importToAdd = args[1]; // e.g., "java.util.HashMap"
    addImport(source, importToAdd);
    break;

private static void addImport(String source, String importPath) {
    JavaParser javaParser = new JavaParser();
    ParseResult<CompilationUnit> result = javaParser.parse(source);

    if (!result.isSuccessful() || !result.getResult().isPresent()) {
        System.out.println(source); // Return unchanged
        return;
    }

    CompilationUnit cu = result.getResult().get();

    // Check if import already exists
    boolean exists = cu.getImports().stream()
        .anyMatch(i -> i.getNameAsString().equals(importPath));

    if (!exists) {
        cu.addImport(importPath);
    }

    System.out.println(cu.toString());
}
```

**Remove Import**:
```java
case "remove-import":
    String importToRemove = args[1];
    removeImport(source, importToRemove);
    break;

private static void removeImport(String source, String importPath) {
    JavaParser javaParser = new JavaParser();
    ParseResult<CompilationUnit> result = javaParser.parse(source);

    if (!result.isSuccessful() || !result.getResult().isPresent()) {
        System.out.println(source);
        return;
    }

    CompilationUnit cu = result.getResult().get();

    // Remove matching import
    cu.getImports().removeIf(i -> i.getNameAsString().equals(importPath));

    System.out.println(cu.toString());
}
```

**Rewrite Imports**:
```java
case "rewrite-imports":
    String oldPath = args[1];
    String newPath = args[2];
    rewriteImports(source, oldPath, newPath);
    break;

private static void rewriteImports(String source, String oldPath, String newPath) {
    JavaParser javaParser = new JavaParser();
    ParseResult<CompilationUnit> result = javaParser.parse(source);

    if (!result.isSuccessful() || !result.getResult().isPresent()) {
        System.out.println(source);
        return;
    }

    CompilationUnit cu = result.getResult().get();

    // Find and replace import declarations
    cu.getImports().forEach(importDecl -> {
        String currentPath = importDecl.getNameAsString();
        if (currentPath.equals(oldPath) || currentPath.startsWith(oldPath + ".")) {
            String updatedPath = currentPath.replace(oldPath, newPath);
            importDecl.setName(updatedPath);
        }
    });

    System.out.println(cu.toString());
}
```

**Parse Package**:
```java
case "parse-package":
    parsePackage(source);
    break;

private static void parsePackage(String source) {
    JavaParser javaParser = new JavaParser();
    ParseResult<CompilationUnit> result = javaParser.parse(source);

    if (!result.isSuccessful() || !result.getResult().isPresent()) {
        System.out.println("null");
        return;
    }

    CompilationUnit cu = result.getResult().get();
    String packageName = cu.getPackageDeclaration()
        .map(pd -> pd.getNameAsString())
        .orElse(null);

    System.out.println(packageName != null ? packageName : "null");
}
```

### 1.3 Build Updated JAR

```bash
cd crates/languages/cb-lang-java/resources/java-parser
mvn clean package
# Creates: target/java-parser-1.0.0.jar
```

---

## Phase 2: Rust Import Support Implementation

### 2.1 Create `import_support.rs`

**File**: `crates/languages/cb-lang-java/src/import_support.rs`

```rust
//! Java import support using AST-based JavaParser tool

use cb_plugin_api::import_support::ImportSupport;
use serde::Deserialize;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;
use tempfile::Builder;
use tracing::{debug, warn};

/// Embedded JavaParser JAR
const JAVA_PARSER_JAR: &[u8] =
    include_bytes!("../resources/java-parser/target/java-parser-1.0.0.jar");

/// Java import support implementation using AST parsing
pub struct JavaImportSupport;

impl JavaImportSupport {
    pub fn new() -> Self {
        Self
    }

    /// Run JavaParser command and return output
    fn run_parser_command(&self, command: &str, source: &str, args: &[&str]) -> Result<String, String> {
        // Write JAR to temp file
        let tmp_dir = Builder::new()
            .prefix("codebuddy-java-parser")
            .tempdir()
            .map_err(|e| format!("Failed to create temp dir: {}", e))?;

        let jar_path = tmp_dir.path().join("java-parser.jar");
        std::fs::write(&jar_path, JAVA_PARSER_JAR)
            .map_err(|e| format!("Failed to write JAR: {}", e))?;

        // Build command args
        let mut cmd_args = vec!["-jar", jar_path.to_str().unwrap(), command];
        cmd_args.extend_from_slice(args);

        // Spawn Java process
        let mut child = Command::new("java")
            .args(&cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn Java: {}", e))?;

        // Write source to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes())
                .map_err(|e| format!("Failed to write stdin: {}", e))?;
        }

        // Get output
        let output = child.wait_with_output()
            .map_err(|e| format!("Failed to wait for process: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("JavaParser failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[derive(Debug, Deserialize)]
struct ImportInfo {
    path: String,
    #[serde(rename = "isStatic")]
    is_static: bool,
    #[serde(rename = "isWildcard")]
    is_wildcard: bool,
}

impl ImportSupport for JavaImportSupport {
    fn parse_imports(&self, content: &str) -> Vec<String> {
        match self.run_parser_command("parse-imports", content, &[]) {
            Ok(json_output) => {
                match serde_json::from_str::<Vec<ImportInfo>>(&json_output) {
                    Ok(imports) => imports.into_iter().map(|i| {
                        if i.is_static {
                            format!("static {}", i.path)
                        } else {
                            i.path
                        }
                    }).collect(),
                    Err(e) => {
                        warn!(error = %e, "Failed to parse imports JSON");
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to parse imports");
                Vec::new()
            }
        }
    }

    fn rewrite_imports_for_rename(
        &self,
        content: &str,
        old_name: &str,
        new_name: &str,
    ) -> (String, usize) {
        match self.run_parser_command("rewrite-imports", content, &[old_name, new_name]) {
            Ok(new_content) => {
                let changes = if new_content != content { 1 } else { 0 };
                (new_content, changes)
            }
            Err(e) => {
                warn!(error = %e, old_name = %old_name, new_name = %new_name,
                      "Failed to rewrite imports");
                (content.to_string(), 0)
            }
        }
    }

    fn rewrite_imports_for_move(
        &self,
        content: &str,
        old_path: &Path,
        new_path: &Path,
    ) -> (String, usize) {
        // Convert file paths to Java package paths
        let old_package = file_path_to_package(old_path).unwrap_or_default();
        let new_package = file_path_to_package(new_path).unwrap_or_default();

        if old_package.is_empty() || new_package.is_empty() {
            return (content.to_string(), 0);
        }

        self.rewrite_imports_for_rename(content, &old_package, &new_package)
    }

    fn contains_import(&self, content: &str, module: &str) -> bool {
        let imports = self.parse_imports(content);
        imports.iter().any(|imp| {
            imp == module ||
            imp.ends_with(&format!(".{}", module)) ||
            (imp.ends_with(".*") && module.starts_with(&imp[..imp.len()-2]))
        })
    }

    fn add_import(&self, content: &str, module: &str) -> String {
        if self.contains_import(content, module) {
            return content.to_string();
        }

        match self.run_parser_command("add-import", content, &[module]) {
            Ok(new_content) => new_content,
            Err(e) => {
                warn!(error = %e, module = %module, "Failed to add import");
                content.to_string()
            }
        }
    }

    fn remove_import(&self, content: &str, module: &str) -> String {
        match self.run_parser_command("remove-import", content, &[module]) {
            Ok(new_content) => new_content,
            Err(e) => {
                warn!(error = %e, module = %module, "Failed to remove import");
                content.to_string()
            }
        }
    }
}

/// Convert file path to Java package path
/// Example: src/main/java/com/example/Foo.java -> com.example.Foo
fn file_path_to_package(path: &Path) -> Option<String> {
    let path_str = path.to_str()?;

    // Find source root markers
    let markers = ["src/main/java/", "src/test/java/", "src/"];

    for marker in &markers {
        if let Some(idx) = path_str.find(marker) {
            let package_part = &path_str[idx + marker.len()..];
            let package_path = package_part
                .trim_end_matches(".java")
                .replace('/', ".");
            return Some(package_path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_to_package() {
        let path = Path::new("src/main/java/com/example/UserService.java");
        let package = file_path_to_package(path);
        assert_eq!(package, Some("com.example.UserService".to_string()));
    }

    // Note: Integration tests require Java runtime and built JAR
    // Skip for now, will add when JAR is built
}
```

### 2.2 Wire to Plugin

**File**: `crates/languages/cb-lang-java/src/lib.rs`

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
            import_support: import_support::JavaImportSupport::new(),
        }
    }
}

impl LanguagePlugin for JavaPlugin {
    fn capabilities(&self) -> LanguageCapabilities {
        LanguageCapabilities {
            imports: true,  // ✅ Enabled!
            workspace: false,
        }
    }

    fn import_support(&self) -> Option<&dyn ImportSupport> {
        Some(&self.import_support)
    }
}
```

---

## Phase 3: Workspace Support (Maven Multi-Module)

### 3.1 Maven Workspace Detection (XML-Based)

**File**: `crates/languages/cb-lang-java/src/workspace_support.rs`

```rust
//! Java workspace support for Maven multi-module projects

use cb_plugin_api::workspace_support::WorkspaceSupport;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

pub struct JavaWorkspaceSupport;

impl JavaWorkspaceSupport {
    pub fn new() -> Self {
        Self
    }
}

impl WorkspaceSupport for JavaWorkspaceSupport {
    fn add_workspace_member(&self, content: &str, member: &str) -> String {
        // Parse XML, find <modules>, add <module>, serialize
        add_module_to_pom(content, member).unwrap_or_else(|_| content.to_string())
    }

    fn remove_workspace_member(&self, content: &str, member: &str) -> String {
        remove_module_from_pom(content, member).unwrap_or_else(|_| content.to_string())
    }

    fn is_workspace_manifest(&self, content: &str) -> bool {
        content.contains("<modules>") || content.contains("<modules/>")
    }

    fn list_workspace_members(&self, content: &str) -> Vec<String> {
        extract_modules(content).unwrap_or_default()
    }

    fn update_package_name(&self, content: &str, new_name: &str) -> String {
        update_artifact_id(content, new_name).unwrap_or_else(|_| content.to_string())
    }
}

// XML manipulation helpers (using quick-xml)
fn extract_modules(pom: &str) -> Result<Vec<String>, String> {
    // ... implementation
}

fn add_module_to_pom(pom: &str, module: &str) -> Result<String, String> {
    // ... implementation
}

fn remove_module_from_pom(pom: &str, module: &str) -> Result<String, String> {
    // ... implementation
}

fn update_artifact_id(pom: &str, new_name: &str) -> Result<String, String> {
    // ... implementation
}
```

**Note**: Workspace support uses XML manipulation (no AST needed for pom.xml).

---

## Implementation Timeline

### Week 1: Java Parser Extension + Import Support

| Day | Task | Deliverable |
|-----|------|-------------|
| 1 | Extend Parser.java with parse-imports command | JAR supports `parse-imports` |
| 2 | Add add-import, remove-import commands | JAR supports manipulation |
| 3 | Add rewrite-imports, parse-package commands | JAR feature-complete |
| 4 | Build JAR, create import_support.rs | Rust wrapper working |
| 5 | Wire to plugin, add tests | imports: true ✅ |

### Week 2: Workspace Support (Optional)

| Day | Task | Deliverable |
|-----|------|-------------|
| 6 | Create workspace_support.rs with XML parsing | Basic structure |
| 7 | Implement add/remove module methods | Module manipulation works |
| 8 | Implement is_workspace, list_members | Query methods work |
| 9 | Implement update_package_name | Full feature set |
| 10 | Tests, wire to plugin | workspace: true ✅ |

---

## Key Advantages of AST Approach

### ✅ Pros
1. **100% Accurate** - No regex edge cases
2. **Handles Complex Code** - Comments, strings, multi-line imports
3. **Preserves Formatting** - JavaParser maintains style
4. **Type-Safe** - Compiler-checked transformations
5. **Extensible** - Easy to add more commands

### ⚠️ Cons
1. **Requires Java Runtime** - Must have `java` in PATH
2. **Subprocess Overhead** - ~50-100ms per operation
3. **JAR Build Step** - Requires Maven to build initially
4. **Error Recovery** - Subprocess failures need graceful handling

---

## Effort Estimate (AST-Based)

| Component | LOC | Days |
|-----------|-----|------|
| **Java Parser Extensions** | | |
| - parse-imports command | 30 | 0.5 |
| - add/remove import commands | 60 | 1.0 |
| - rewrite-imports command | 40 | 0.5 |
| - parse-package command | 20 | 0.25 |
| **Rust Import Support** | | |
| - import_support.rs structure | 50 | 0.5 |
| - run_parser_command helper | 80 | 1.0 |
| - ImportSupport trait impl | 120 | 1.5 |
| - File path helpers | 40 | 0.5 |
| - Tests | 70 | 0.75 |
| **Subtotal Import Support** | ~510 | **6 days** |
| | | |
| **Workspace Support** | | |
| - workspace_support.rs | 200 | 2.5 |
| - XML manipulation helpers | 150 | 2.0 |
| - Tests | 50 | 0.5 |
| **Subtotal Workspace** | ~400 | **5 days** |
| | | |
| **Grand Total** | ~910 | **11 days** |

**Comparison to Regex Approach:**
- Regex: 3 days (imports only)
- AST: 6 days (imports only)
- **Trade-off**: 2x time for 100% accuracy

---

## Success Criteria

### Phase 1: Import Support
- ✅ JavaParser JAR extended with 5 new commands
- ✅ JAR built and embedded in Rust binary
- ✅ import_support.rs implements all 6 ImportSupport methods
- ✅ Handles static imports, wildcards, package declarations
- ✅ Graceful fallback when Java unavailable (return empty/unchanged)
- ✅ All integration tests passing
- ✅ `capabilities().imports == true`

### Phase 2: Workspace Support
- ✅ workspace_support.rs implements all 5 WorkspaceSupport methods
- ✅ Maven multi-module projects supported
- ✅ Can add/remove/list modules
- ✅ Can update package names
- ✅ All tests passing
- ✅ `capabilities().workspace == true`

---

## Risk Mitigation

### Risk: Java Runtime Not Available
**Mitigation**:
- Detect Java availability at runtime
- Return empty results gracefully (don't crash)
- Document requirement in README
- Consider adding "java_available" to capabilities in future

### Risk: JAR Build Complexity
**Mitigation**:
- Add build instructions to README
- Consider CI/CD to pre-build JAR
- Could embed pre-built JAR in repo (but increases size)

### Risk: Performance (Subprocess Overhead)
**Mitigation**:
- Cache JAR extraction (write once per session)
- Consider long-lived Java process (future optimization)
- For batch operations, combine into single call

### Risk: Cross-Platform Issues
**Mitigation**:
- Java is cross-platform by design
- Test on Windows/Linux/macOS
- Use platform-agnostic temp directories

---

## Next Steps

**Option A**: Implement AST-based import support (6 days)
**Option B**: Start with regex, migrate to AST later (3 days MVP + 3 days migration)
**Option C**: Full implementation: Imports (AST) + Workspace (XML) (11 days)

**Recommendation**: **Option A** - Go straight to AST for imports
- Maintains architectural consistency (Rust uses syn, TypeScript uses swc_ecma_parser)
- No technical debt from regex migration
- Production-quality from day 1
- Only 3 extra days vs regex

**Your call!**
