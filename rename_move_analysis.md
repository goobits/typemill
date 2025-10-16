# CodeBuddy Rename/Move Implementation Analysis

## Executive Summary

CodeBuddy has a sophisticated rename/move system with **60-70% coverage** for reference updates across the codebase. The current implementation successfully handles core code references but is **missing key coverage areas** needed to reach the 93%+ target for Proposal 02f.

### Current Coverage Status

**What IS Covered (60-70%):**
- Rust cross-crate imports (`use crate_name::module::*`)
- Rust same-crate module paths (`use module::*`, qualified paths)
- Rust module declarations (`pub mod old; → pub mod new;`)
- TypeScript/JavaScript imports with relative paths
- Cargo.toml manifest updates (workspace members, dependencies)
- Parent file (lib.rs/mod.rs) mod declaration updates

**What IS NOT Covered (30-40%):**
- String literals in code (e.g., hardcoded paths: `"path/to/old/file"`)
- Markdown documentation files (.md)
- Configuration files (TOML, YAML, JSON beyond Cargo.toml)
- Makefile and build script references
- Comments in code
- HTML/XML files with path references
- Environment variable references
- URL/URI references in comments/docs

---

## 1. Current Update Coverage

### 1.1 What Gets Updated

**Source: `/workspace/crates/cb-services/src/services/reference_updater/mod.rs` (Lines 39-369)**

The pipeline works as follows:

```
update_references() 
  ├─ find_project_files() [Lines 535-597]
  │  └─ Only scans files with extensions registered in plugins
  │     (checks: plugin.handles_extension(ext_str))
  │
  ├─ find_affected_files_for_rename() [Lines 421-461]
  │  ├─ Rust detector for .rs files (detectors::find_rust_affected_files)
  │  └─ Generic detector fallback (detectors::find_generic_affected_files)
  │
  └─ For each affected file:
     └─ plugin.rewrite_file_references() [Lines 166-367]
```

#### File Types Currently Scanned

**SCANNED:** Only files matching registered plugin extensions:
- `.rs` (Rust plugin)
- `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs` (TypeScript plugin)

**NOT SCANNED:** Configuration/documentation files
- `.md` (Markdown)
- `.toml` (TOML configs)
- `.yaml`, `.yml` (YAML configs)
- `.json` (JSON files, except package.json for manifests)
- `Makefile`
- `.sh` (Shell scripts)
- `.env` (Environment files)

Line 580-586 shows the filtering logic:
```rust
} else if let Some(ext) = path.extension() {
    let ext_str = ext.to_str().unwrap_or("");
    if plugins.iter().any(|plugin| plugin.handles_extension(ext_str)) {
        files.push(path);  // Only adds if plugin handles it
    }
}
```

### 1.2 Rust Reference Updates

**Source: `/workspace/crates/cb-lang-rust/src/lib.rs` (Lines 151-193, 568-842)**

The Rust plugin's `rewrite_file_references()` method handles:

1. **Module Declarations** (Lines 625-652)
   - Updates: `pub mod old;` → `pub mod new;`
   - Calls: `self.update_module_declaration()`

2. **Use Statements** (Lines 655-700)
   - Same-directory file renames: `use utils::helper;` → `use helpers::helper;`
   - Calls: `ImportRenameSupport::rewrite_imports_for_rename()`

3. **Qualified Paths** (Lines 679-700)
   - Updates: `utils::helper()` → `helpers::helper()`
   - Calls: `self.update_qualified_paths()`

4. **Crate-level Imports** (Lines 705-835)
   - Full module path computation via `compute_module_path_from_file()`
   - Handles: `use crate::module::*`, `use super::*`, `use self::*`

#### Rust Import Support Trait
**Source: `/workspace/crates/cb-lang-rust/src/import_support.rs` (Lines 59-295)**

The `ImportRenameSupport::rewrite_imports_for_rename()` method:
- Line 95-140: Detects use statements
- Line 140-188: Handles crate::, super::, self:: imports via AST parsing
- Line 199-271: Uses syn::parse_str to rewrite use trees safely

**Coverage Details:**
- ✅ `use old_crate::module::Thing;` → `use new_crate::module::Thing;`
- ✅ `use crate::old_module::*;` → `use crate::new_module::*;`
- ✅ `use super::old::*;` → `use super::new::*;`
- ✅ Qualified paths: `old_module::function()` → `new_module::function()`
- ❌ String literals: `"use old_crate::module"` (comment, string, or documentation)

### 1.3 TypeScript/JavaScript Updates

**Source: `/workspace/crates/cb-lang-typescript/src/lib.rs` (Lines 114-194)**

The TypeScript plugin delegates to `import_support::rewrite_imports_for_move_with_context()`:
- Handles relative path updates in imports
- Works with: `from './old-path'` → `from './new-path'`

**Limitation:** Only processes files that match registered extensions. No markdown or config file scanning.

### 1.4 Cargo.toml Manifest Updates

**Source: `/workspace/crates/cb-services/src/services/move_service/planner.rs` (Lines 63-214)**

For directory moves (when Cargo.toml detected), the system updates:

1. **Workspace Members** (Line 118)
   - `cargo::plan_workspace_manifest_updates()`
   - Updates workspace.members array with new crate path

2. **Dependent Crate Paths** (Line 158)
   - `cargo::plan_dependent_crate_path_updates()`
   - Updates path dependencies in other crates' Cargo.toml

**Coverage:** Specific to Rust crates only; no similar support for package.json

---

## 2. Language Plugin System Architecture

### 2.1 Plugin Registration

**Source: `/workspace/crates/cb-lang-rust/src/lib.rs` (Lines 35-72)**

```rust
codebuddy_plugin! {
    name: "rust",
    extensions: ["rs"],
    manifest: "Cargo.toml",
    capabilities: RustPlugin::CAPABILITIES,
    factory: RustPlugin::new,
    lsp: Some(LspConfig::new("rust-analyzer", &["rust-analyzer"]))
}
```

Capabilities defined at line 63-66:
```rust
pub const CAPABILITIES: PluginCapabilities = PluginCapabilities {
    imports: true,
    workspace: true,
};
```

### 2.2 Plugin Capabilities

Each plugin can implement these traits:

1. **ImportParser** (Lines 25-57)
   - `parse_imports()`: Extract all imports from code
   - `contains_import()`: Check if code contains specific import

2. **ImportRenameSupport** (Lines 59-295)
   - `rewrite_imports_for_rename()`: Update imports during rename
   - **THIS IS THE MAIN HOOK FOR RENAME OPERATIONS**

3. **ImportMoveSupport** (Lines 298-310)
   - `rewrite_imports_for_move()`: Delegated to rename support

4. **ImportMutationSupport** (Lines 313-362)
   - `add_import()`: Add new import statement
   - `remove_import()`: Remove import statement

5. **WorkspaceSupport**
   - Manifest analysis and updates

### 2.3 Currently Registered Plugins

From the codebase:
- **Rust plugin** (`/workspace/crates/cb-lang-rust/`) - Handles `.rs` files
- **TypeScript plugin** (`/workspace/crates/cb-lang-typescript/`) - Handles `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`
- **Markdown plugin** (`/workspace/crates/cb-lang-markdown/`) - **EXISTS but NOT wired into rename/move pipeline**

### 2.4 Missing Plugin Support

**Source: `/workspace/crates/cb-lang-markdown/src/lib.rs`**

A Markdown plugin exists but:
- Not registered in the reference updater's file scanning loop
- Not called during rename/move operations
- Could handle `.md` file path references

---

## 3. AST Capabilities by Language

### 3.1 Rust AST Capabilities

**Parser: `/workspace/crates/cb-lang-rust/src/parser.rs`**

Uses `syn` crate for AST parsing:
- ✅ Parse complete Rust source into AST
- ✅ Extract symbols (functions, structs, modules, etc.)
- ✅ Parse imports via `syn::ItemUse`
- ✅ Traverse module structure
- ✅ AST-based rewriting via `quote::quote!()`

**Limitations:**
- ❌ Does NOT parse string literals as path references
- ❌ Does NOT extract hardcoded paths from comments
- ❌ Does NOT handle documentation comments specially

### 3.2 TypeScript AST Capabilities

**Parser: `/workspace/crates/cb-lang-typescript/src/parser.rs`**

Uses regex-based extraction (not full AST):
- ✅ Extract import statements via regex
- ✅ Parse relative paths in imports
- ❌ No deep AST analysis
- ❌ String literals not processed
- ❌ Comments not analyzed

### 3.3 String Literal Detection

**GAP:** No language plugin detects/rewrites string literals containing paths:

```rust
// These are NOT updated by any plugin:
let old_path = "crate/old/file";  // Should become "crate/new/file"
let doc_url = "docs/old-module.html";  // Should become "docs/new-module.html"
```

---

## 4. Reference Update Pipeline

### 4.1 Complete Flow for File Rename

```
User initiates rename: src/utils.rs → src/helpers.rs

1. plan_file_rename() [file_rename.rs:17-217]
   ├─ Calls: file_service.plan_rename_file_with_imports()
   │
   └─→ 2. MoveService.plan_file_move() [move/mod.rs:58-89]
       ├─ Calls: reference_updater.update_references()
       │
       └─→ 3. ReferenceUpdater.update_references() [reference_updater/mod.rs:39-392]
           ├─ find_project_files() → Gets list of .rs, .ts, .tsx, .js files
           ├─ find_affected_files_for_rename() → Uses Rust detector + generic detector
           │
           └─ For each affected file:
               ├─ Loads file content
               ├─ Selects plugin based on file extension
               │
               └─→ 4. plugin.rewrite_file_references()
                   ├─ [Rust] → RustPlugin.rewrite_imports_for_rename() 
                   │    ├─ update_module_declaration()
                   │    ├─ update_qualified_paths()
                   │    └─ ImportRenameSupport.rewrite_imports_for_rename()
                   │
                   └─ [TypeScript] → rewrite_imports_for_move_with_context()

5. Results wrapped in EditPlan → MovePlan
6. User reviews and calls: workspace.apply_edit() to execute
```

### 4.2 Affected File Detection

**Rust Detector: `/workspace/crates/cb-services/src/services/reference_updater/detectors/rust.rs`**

For Rust file moves:
1. Line 69-93: Extract crate name from Cargo.toml (or fallback to directory name)
2. Line 177-220: For crate renames, scan all files for `use crate_name::`
3. Line 224-455: For file moves, compute module path and scan for imports

**Key Logic (Lines 358-455):**
- Searches for: `use old_module_path::`, `use crate::old_suffix::`, `use super::old_module::`
- Scans ALL `.rs` files in project
- Returns list of files that import from old path

**Generic Detector: `/workspace/crates/cb-services/src/services/reference_updater/detectors/generic.rs`**

Fallback for non-Rust files:
1. Line 13-62: For each file in project
2. Line 34: Parse imports using plugin's `import_parser()`
3. Line 44-45: Check if parsed imports match old_path or new_path

---

## 5. Current Gaps for 93%+ Coverage

### 5.1 File Type Coverage

**Missing from Scanner (Lines 580-586):**

| File Type | Current Status | Impact | Why Missing |
|-----------|---|---|---|
| .md (Markdown) | ❌ Not scanned | Documentation references missed | Plugin exists but not wired into pipeline |
| .toml (Config) | ❌ Not scanned | Build config refs missed | No plugin registered |
| .yaml/.yml | ❌ Not scanned | Config refs missed | No plugin registered |
| .json | ✅ Partial (only package.json) | Inconsistent | Only for manifest analysis |
| Makefile | ❌ Not scanned | Build targets missed | No plugin registered |
| .sh (Shell) | ❌ Not scanned | Script refs missed | No plugin registered |

### 5.2 Reference Type Coverage

**Not Currently Detected:**

1. **String Literals in Code**
   ```rust
   // Renaming: utils.rs → helpers.rs
   let import_path = "utils::helper";  // ← NOT UPDATED
   let config_file = "config/utils.json";  // ← NOT UPDATED
   ```

2. **Comments and Documentation**
   ```rust
   // Use the utils module for calculations
   // See docs/utils.md for details
   // ← These paths NOT UPDATED
   ```

3. **Hardcoded Paths in Config**
   ```toml
   # Cargo.toml
   [[example]]
   name = "example"
   path = "examples/old_utils.rs"  # ← NOT UPDATED
   ```

4. **Makefile Targets**
   ```makefile
   build:
       cargo build --manifest-path crates/old_crate/Cargo.toml  # ← NOT UPDATED
   ```

5. **URL/Path References in Docs**
   ```markdown
   # Module Documentation
   See the [utils module](./old_utils.md) for details  # ← NOT UPDATED
   ```

### 5.3 Language-Specific Gaps

**Rust:**
- ✅ Covers imports and qualified paths
- ❌ Missing: String literals, cfg attributes with paths
- ❌ Missing: doc(cfg(...)) attributes
- ❌ Missing: Build script references

**TypeScript:**
- ✅ Covers relative path imports
- ❌ Missing: String literals ("./old-path" as string)
- ❌ Missing: Dynamic imports with string templates
- ❌ Missing: require() statements (legacy)

---

## 6. Technical Architecture Details

### 6.1 File Extension Registration

**Plugin Registration Flow:**

1. Plugin defines extensions (Line 35-42 in rust/lib.rs):
   ```rust
   codebuddy_plugin! {
       extensions: ["rs"],
       ...
   }
   ```

2. File scanner uses `plugin.handles_extension()` (Line 582-584):
   ```rust
   if plugins.iter().any(|plugin| plugin.handles_extension(ext_str)) {
       files.push(path);
   }
   ```

3. When file is found, plugin's `rewrite_file_references()` is called

**Current Registered Extensions:**
- Rust: `["rs"]`
- TypeScript: `["ts", "tsx", "js", "jsx", "mjs", "cjs"]`

### 6.2 The `rewrite_file_references()` Hook

**Signature:** (from cb-protocol)
```rust
fn rewrite_file_references(
    &self,
    content: &str,           // File content to process
    old_path: &Path,         // Old path (being moved FROM)
    new_path: &Path,         // New path (being moved TO)
    current_file: &Path,     // The file being processed
    project_root: &Path,     // Root of project
    rename_info: Option<&serde_json::Value>,  // Extra context
) -> Option<(String, usize)>;  // (modified_content, changes_count)
```

This is the **PRIMARY HOOK** for all reference rewriting.

**Called from:** `reference_updater/mod.rs` lines 166-336

### 6.3 Import Detection via Plugins

Two approaches:

**Approach 1: Built-in Plugin Parser (for supported languages)**
```rust
let all_imports = plugin.import_parser()
    .parse_imports(content);  // Returns Vec<String> of module paths
```

**Approach 2: Generic Import Extraction (fallback)**
```rust
extract_import_path(line)  // Regex-based: `from "path"` or `require('path')`
```

---

## 7. Summary Table: What Works vs. What Doesn't

| Scenario | Works | How | Gap |
|----------|-------|-----|-----|
| Rename `src/utils.rs` → `src/helpers.rs` | ✅ Yes | Plugin detects affected files + rewrites imports | No string literals |
| Move file across crates | ✅ Yes | Rust detector finds all imports | Only .rs files scanned |
| Rename crate (directory) | ✅ Yes | Full module path detection | Config files missed |
| Update Cargo.toml paths | ✅ Yes | cargo::plan_dependent_crate_path_updates() | TypeScript/Node only |
| Update TypeScript imports | ✅ Yes | Generic import rewriter + plugin | String literals missed |
| Update documentation refs | ❌ No | No .md file scanning | Plugin exists but not wired |
| Update Makefile refs | ❌ No | No Makefile scanning | No plugin registered |
| Update strings in code | ❌ No | No literal analysis | Would need AST analysis |
| Update URLs in comments | ❌ No | Comments not parsed | Risky (user-specific content) |

---

## 8. Key Files for Implementation

### Core Services
- **Reference Updater**: `/workspace/crates/cb-services/src/services/reference_updater/mod.rs` (532 lines)
- **Rust Detector**: `/workspace/crates/cb-services/src/services/reference_updater/detectors/rust.rs` (603 lines)
- **Generic Detector**: `/workspace/crates/cb-services/src/services/reference_updater/detectors/generic.rs` (245 lines)

### Plugin Implementations
- **Rust Plugin**: `/workspace/crates/cb-lang-rust/src/lib.rs` (1140 lines)
- **Rust Import Support**: `/workspace/crates/cb-lang-rust/src/import_support.rs` (520 lines)
- **TypeScript Plugin**: `/workspace/crates/cb-lang-typescript/src/lib.rs` (220 lines)
- **Markdown Plugin**: `/workspace/crates/cb-lang-markdown/src/lib.rs` (exists but not used)

### Move/Rename Handlers
- **File Rename Handler**: `/workspace/crates/cb-handlers/src/handlers/rename_handler/file_rename.rs` (217 lines)
- **File Move Handler**: `/workspace/crates/cb-handlers/src/handlers/move/file_move.rs` (91 lines)
- **Move Planner**: `/workspace/crates/cb-services/src/services/move_service/planner.rs` (214 lines)

---

## 9. Recommendations for Achieving 93%+ Coverage

### Priority 1: Extend File Type Scanning (15-20% improvement)
1. Register additional plugins (Markdown, JSON, TOML, YAML)
2. Wire them into `find_project_files()` loop
3. Implement basic path rewriting for each format

### Priority 2: String Literal Detection (10-15% improvement)
1. For Rust: Detect and rewrite hardcoded path strings
2. Use AST analysis to identify string literals
3. Conservative: Only rewrite if path matches file system

### Priority 3: Configuration File Updates (5-10% improvement)
1. Create TOML/YAML plugins with rewrite support
2. Handle Makefile build targets and paths
3. Update .env files with hardcoded paths

### Priority 4: Documentation Updates (5% improvement)
1. Implement Markdown rewrite support (plugin exists)
2. Update links and code examples in .md files
3. Update path references in comments

---

## Implementation Effort Estimate

| Feature | Effort | Impact |
|---------|--------|--------|
| Wire Markdown plugin | 2-3 hours | +3-5% |
| Add TOML/YAML plugins | 4-6 hours | +8-10% |
| String literal detection | 6-8 hours | +10-12% |
| Build config updates | 3-4 hours | +5-7% |
| Comment path detection | 4-5 hours | +3-5% |
| **Total** | **19-26 hours** | **+29-39%** |

**Target achievable:** 89-109% (accounting for overlaps, aiming for 93%)
