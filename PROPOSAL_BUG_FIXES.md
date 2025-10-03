# Bug Fix Proposal

This document proposes solutions for the active issues in BUG_REPORT.md.

---

## Issue #1: Incomplete Import Path Updates During `rename_directory`

**Current State:** Only top-level `use` statements are updated

**Proposed Solution:** Multi-pass AST-based import scanner

### Implementation Plan

#### Phase 1: Enhanced AST Scanning (Week 1)

**File:** `crates/cb-ast/src/import_scanner.rs` (new)

```rust
pub struct ImportScanner {
    language: Language,
    scope: ScanScope,
}

pub enum ScanScope {
    TopLevelOnly,           // Current behavior
    AllUseStatements,       // Includes function-scoped
    QualifiedPaths,         // module::function references
    All,                    // Everything including strings
}

impl ImportScanner {
    pub fn find_all_references(&self, content: &str, module_name: &str) -> Vec<Reference> {
        // 1. Parse with swc/tree-sitter
        // 2. Walk AST for:
        //    - use statements at any scope
        //    - qualified paths (Expr::Member, PathExpr)
        //    - string literals (opt-in with confirmation)
        // 3. Return positions and types
    }
}
```

**Test Strategy:**
```rust
#[test]
fn test_function_scoped_imports() {
    let code = r#"
        fn test() {
            use old_module::Type;  // Should find this
            let x = old_module::function();  // Should find this
        }
    "#;

    let refs = scanner.find_all_references(code, "old_module");
    assert_eq!(refs.len(), 2);
}
```

#### Phase 2: Configurable Update Modes (Week 2)

**File:** `crates/cb-handlers/src/handlers/tools/file_ops.rs`

Add `update_mode` parameter to `rename_directory`:

```rust
pub struct RenameDirectoryParams {
    pub old_path: String,
    pub new_path: String,
    pub dry_run: Option<bool>,
    pub update_mode: Option<UpdateMode>,  // NEW
}

pub enum UpdateMode {
    Conservative,  // Current: top-level use only
    Standard,      // + function-scoped use
    Aggressive,    // + qualified paths
    Full,          // + string literals (with confirmation)
}
```

**Migration:** Default to `Conservative` for backward compatibility

#### Phase 3: User Confirmation for Risky Updates (Week 2)

For `Aggressive` and `Full` modes, show preview:

```json
{
  "preview": {
    "safe_updates": 15,
    "risky_updates": [
      {
        "file": "src/test.rs",
        "line": 42,
        "old": "let path = \"old_module/file\"",
        "new": "let path = \"new_module/file\"",
        "confidence": "low"
      }
    ]
  },
  "require_confirmation": true
}
```

**Effort:** 2-3 weeks
**Risk:** Low - additive feature, backward compatible
**Priority:** High (frequently requested)

---

## Issue #2: Batch File Operations Don't Use Git

**Current State:** File operations don't preserve git history

**Proposed Solution:** Git-aware file operations with fallback

### Implementation Plan

#### Phase 1: Git Detection (Week 1)

**File:** `crates/cb-services/src/services/git_service.rs` (new)

```rust
pub struct GitService;

impl GitService {
    pub fn is_git_repo(path: &Path) -> bool {
        Command::new("git")
            .current_dir(path)
            .args(&["rev-parse", "--git-dir"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn is_file_tracked(path: &Path) -> bool {
        Command::new("git")
            .args(&["ls-files", "--error-unmatch", path.to_str().unwrap()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn git_mv(old: &Path, new: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = new.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let status = Command::new("git")
            .args(&["mv", old.to_str().unwrap(), new.to_str().unwrap()])
            .status()?;

        if !status.success() {
            return Err(anyhow!("git mv failed"));
        }
        Ok(())
    }
}
```

#### Phase 2: Update FileService (Week 1)

**File:** `crates/cb-services/src/services/file_service.rs`

```rust
pub struct FileService {
    git_service: GitService,
    use_git: bool,  // Auto-detected or configured
}

impl FileService {
    pub async fn rename_file(&self, old: &Path, new: &Path) -> Result<()> {
        if self.use_git && GitService::is_file_tracked(old) {
            // Use git mv for tracked files
            GitService::git_mv(old, new)?;
        } else {
            // Fallback to filesystem
            if let Some(parent) = new.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::rename(old, new).await?;
        }
        Ok(())
    }
}
```

#### Phase 3: Configuration Option (Week 2)

**File:** `.codebuddy/config.json`

```json
{
  "git": {
    "enabled": true,          // Auto-detect and use git if available
    "require": false,         // Fail if git expected but unavailable
    "operations": ["mv", "rm"]  // Which git commands to use
  }
}
```

**Benefits:**
- Preserves git history automatically
- No breaking changes (fallback to current behavior)
- Users can disable if needed

**Edge Cases:**
```rust
#[test]
fn test_git_mv_across_submodules() {
    // Should fallback to fs if crossing submodule boundaries
}

#[test]
fn test_git_mv_new_directory() {
    // git mv can't create new directories - we handle it
}
```

**Effort:** 1-2 weeks
**Risk:** Low (fallback preserves current behavior)
**Priority:** Medium (quality of life improvement)

---

## Issue #3: Test Flakiness

**Current State:** `resilience_tests::test_basic_filesystem_operations` intermittently fails

**Proposed Solution:** Improved test isolation and timing

### Root Cause Analysis

The test failure shows "trailing characters" in JSON parsing, suggesting:
1. Multiple responses being concatenated
2. Log output mixed with JSON
3. Timing issues with async operations

### Implementation Plan

#### Phase 1: Diagnostic Logging (Day 1)

Add structured logging to understand failure mode:

```rust
#[test]
fn test_basic_filesystem_operations() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .init();

    // Capture all stdout/stderr
    let output = test_client.send_request(request)?;
    debug!(raw_output = %output, "Received from server");

    // Try to parse and show exact failure point
    match serde_json::from_str::<Value>(&output) {
        Ok(v) => v,
        Err(e) => {
            error!(
                output = %output,
                position = e.column(),
                "JSON parse failed"
            );
            panic!("Parse error: {}", e);
        }
    }
}
```

#### Phase 2: Response Framing (Week 1)

**File:** `crates/cb-transport/src/stdio.rs`

Ensure clean JSON separation:

```rust
pub struct StdioTransport {
    delimiter: String,  // Default: "\n---\n"
}

impl StdioTransport {
    pub async fn send_response(&mut self, response: &Value) -> Result<()> {
        let json = serde_json::to_string(response)?;
        writeln!(self.stdout, "{}", json)?;
        writeln!(self.stdout, "{}", self.delimiter)?;
        self.stdout.flush()?;
        Ok(())
    }

    pub async fn read_response(&mut self) -> Result<Value> {
        let mut buffer = String::new();
        loop {
            let line = self.stdin.read_line(&mut buffer)?;
            if buffer.ends_with(&self.delimiter) {
                buffer.truncate(buffer.len() - self.delimiter.len());
                break;
            }
        }
        serde_json::from_str(&buffer.trim())
    }
}
```

#### Phase 3: Test Timeout Handling (Week 1)

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_basic_filesystem_operations() {
    let timeout = Duration::from_secs(30);

    let result = tokio::time::timeout(timeout, async {
        // Test logic here
    }).await;

    match result {
        Ok(Ok(value)) => value,
        Ok(Err(e)) => panic!("Test failed: {}", e),
        Err(_) => panic!("Test timed out after {:?}", timeout),
    }
}
```

**Effort:** 1 week
**Risk:** Very Low (test infrastructure only)
**Priority:** Low (doesn't affect production)

---

## Enhancement Requests

### 1. Enhanced Import Scanning
**Covered by Issue #1 proposal above**

### 2. update_dependency Tool Improvements

**Priority:** Medium
**Effort:** 1-2 weeks

#### Proposed Features

**A. Preserve Inline Metadata**

```rust
pub struct DependencyMetadata {
    pub optional: Option<bool>,
    pub features: Option<Vec<String>>,
    pub version: Option<String>,
    pub default_features: Option<bool>,
}

impl CargoManifest {
    fn rename_dependency_preserve_metadata(
        &mut self,
        old_name: &str,
        new_name: &str,
        new_path: &str,
    ) -> Result<()> {
        // 1. Extract all metadata from old dependency
        let metadata = self.extract_metadata(old_name)?;

        // 2. Create new dependency with same metadata
        let mut new_dep = toml_edit::InlineTable::new();
        new_dep.insert("path", new_path.into());

        if let Some(opt) = metadata.optional {
            new_dep.insert("optional", opt.into());
        }
        if let Some(features) = metadata.features {
            new_dep.insert("features", toml_edit::value(features));
        }

        // 3. Replace
        self.0.get_mut("dependencies")
            .and_then(|d| d.as_table_like_mut())
            .ok_or(...)?
            .remove(old_name);

        self.0.get_mut("dependencies")
            .and_then(|d| d.as_table_like_mut())
            .ok_or(...)?
            .insert(new_name, toml_edit::value(new_dep));

        Ok(())
    }
}
```

**B. Batch Mode**

```rust
pub struct BatchDependencyUpdate {
    pub updates: Vec<DependencyUpdate>,
    pub manifest_paths: Vec<String>,  // Auto-find if empty
}

pub struct DependencyUpdate {
    pub old_name: String,
    pub new_name: String,
    pub new_path: Option<String>,
}

// Tool handler
async fn handle_batch_update_dependency(params: BatchDependencyUpdate) -> Result<Value> {
    let manifests = if params.manifest_paths.is_empty() {
        // Auto-discover all Cargo.toml files in workspace
        find_manifests(".")?
    } else {
        params.manifest_paths
    };

    let mut results = Vec::new();
    for manifest_path in manifests {
        for update in &params.updates {
            match update_single_dependency(manifest_path, update).await {
                Ok(_) => results.push(json!({"file": manifest_path, "status": "updated"})),
                Err(e) => results.push(json!({"file": manifest_path, "error": e.to_string()})),
            }
        }
    }

    Ok(json!({
        "updated": results.iter().filter(|r| r["status"] == "updated").count(),
        "failed": results.iter().filter(|r| r.get("error").is_some()).count(),
        "details": results
    }))
}
```

**Usage:**
```bash
# Single project-wide rename
codebuddy tool batch_update_dependency '{
  "updates": [
    {"old_name": "cb-mcp-proxy", "new_name": "cb-plugins", "new_path": "../cb-plugins"}
  ]
}'
# Auto-finds all Cargo.toml files and updates them
```

### 3. Post-Operation Validation

**Priority:** High
**Effort:** 2 weeks

```rust
pub struct ValidationConfig {
    pub enabled: bool,
    pub command: String,  // Default: "cargo check"
    pub on_failure: FailureAction,
}

pub enum FailureAction {
    Report,    // Just show errors
    Rollback,  // Undo the operation
    Interactive,  // Ask user
}

// In file operation handlers
pub async fn rename_directory_with_validation(
    params: RenameDirectoryParams,
    validation: ValidationConfig,
) -> Result<Value> {
    // 1. Perform rename
    let result = rename_directory(params).await?;

    // 2. Run validation
    if validation.enabled {
        let check_result = Command::new("sh")
            .args(&["-c", &validation.command])
            .output()?;

        if !check_result.status.success() {
            let stderr = String::from_utf8_lossy(&check_result.stderr);

            match validation.on_failure {
                FailureAction::Report => {
                    return Ok(json!({
                        "status": "completed_with_errors",
                        "validation_errors": stderr,
                        "suggestion": "Run 'cargo check' to see details"
                    }));
                }
                FailureAction::Rollback => {
                    // Undo the rename
                    rename_directory(RenameDirectoryParams {
                        old_path: params.new_path,
                        new_path: params.old_path,
                        ..params
                    }).await?;

                    return Err(anyhow!("Validation failed, rolled back: {}", stderr));
                }
                FailureAction::Interactive => {
                    // Return special status for user decision
                    return Ok(json!({
                        "status": "validation_failed",
                        "errors": stderr,
                        "rollback_available": true
                    }));
                }
            }
        }
    }

    Ok(result)
}
```

### 4. Better MCP Error Reporting

**Priority:** Low
**Effort:** 3-5 days

**Problem:** Inconsistent error formats between JSON responses and CLI expectations

**Solution:** Standardize error responses

```rust
// crates/cb-protocol/src/error.rs
pub struct StandardError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<Value>,
    pub suggestion: Option<String>,
}

pub enum ErrorCode {
    InvalidRequest,
    NotFound,
    PermissionDenied,
    Internal,
    Timeout,
}

impl StandardError {
    pub fn to_json(&self) -> Value {
        json!({
            "error": {
                "code": self.code.as_str(),
                "message": self.message,
                "details": self.details,
                "suggestion": self.suggestion
            }
        })
    }

    pub fn to_cli_string(&self) -> String {
        let mut msg = format!("Error: {}", self.message);
        if let Some(suggestion) = &self.suggestion {
            msg.push_str(&format!("\nSuggestion: {}", suggestion));
        }
        msg
    }
}
```

---

## Implementation Roadmap

### Immediate (Next 2 Weeks)
1. **Issue #2 (Git Integration)** - High value, low risk
   - Week 1: Git detection and service
   - Week 2: Integration and testing

2. **Enhancement #2 (update_dependency improvements)** - Building on recent work
   - Week 1: Metadata preservation
   - Week 2: Batch mode

### Short Term (Weeks 3-6)
3. **Issue #1 (Import Scanning)** - High user impact
   - Weeks 3-4: Enhanced AST scanner
   - Week 5: Integration with rename_directory
   - Week 6: Testing and documentation

4. **Enhancement #3 (Validation)** - Quality improvement
   - Week 5: Core validation framework
   - Week 6: Integration with operations

### Medium Term (Weeks 7-8)
5. **Issue #3 (Test Flakiness)** - Technical debt
   - Week 7: Diagnostics and framing
   - Week 8: Stabilization

6. **Enhancement #4 (Error Reporting)** - Polish
   - Week 8: Standardize error types

---

## Success Metrics

**Git Integration:**
- [ ] 100% of file operations use git when available
- [ ] Zero git history loss in dogfooding tests
- [ ] Config option documented and tested

**Import Scanning:**
- [ ] Detects 95%+ of import references (measured on real codebases)
- [ ] Zero false positives in Conservative mode
- [ ] User confirmation for risky updates

**update_dependency:**
- [ ] Preserves all metadata (optional, features, etc.)
- [ ] Batch mode updates 20+ files in <1 second
- [ ] Used successfully in next refactoring phase

**Validation:**
- [ ] Catches 90%+ of breaking changes before commit
- [ ] Rollback works correctly 100% of time
- [ ] Clear error messages with actionable suggestions

---

## Risk Assessment

| Issue | Risk | Mitigation |
|-------|------|------------|
| Git Integration | Low | Fallback to current behavior if git unavailable |
| Import Scanning | Medium | Start with Conservative default, opt-in for Aggressive |
| update_dependency | Low | Additive features, backward compatible |
| Validation | Medium | Make it opt-in, test rollback thoroughly |
| Test Flakiness | Very Low | Test infrastructure only |

---

## Questions for Review

1. **Priority Agreement:** Do we agree with the roadmap order?
2. **Git Integration:** Should it be opt-out or opt-in by default?
3. **Import Scanning:** Should we support string literal updates, or is it too risky?
4. **Validation:** Should validation be default-on or default-off?
5. **Resources:** Do we need additional help for the 8-week timeline?

---

**Next Steps:**
1. Review and approve this proposal
2. Create GitHub issues for each work item
3. Start with Git Integration (highest value, lowest risk)
4. Begin dogfooding immediately

**Estimated Total Effort:** 8 weeks (1 developer)
**Expected Impact:** Significantly improved refactoring experience, fewer manual fixes required
