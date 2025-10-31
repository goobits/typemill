# Proposal 21: Fix Java Plugin Missing Critical Traits

**Status**: Ready for Implementation
**Scope**: Java language plugin
**Priority**: CRITICAL

## Problem

The Java plugin is missing **4 critical traits** that block production use:
- `ManifestUpdater` - Cannot update pom.xml dependencies
- `ModuleReferenceScanner` - Cannot scan for module references
- `ImportAnalyzer` - Cannot analyze import graphs
- `LspInstaller` - Cannot install `jdtls` LSP server

**Impact**: Java plugin has only 75% feature parity compared to other languages. Users cannot update dependencies, scan references, or analyze imports.

**Evidence**: `languages/mill-lang-java/src/lib.rs` - Only delegates `import_support`, `workspace_support`, and `project_factory`. Missing 4 trait delegations.

## Solution

Implement all 4 missing traits following patterns from Go and C# plugins.

## Checklists

### Implement ManifestUpdater

- [ ] Create `manifest_updater.rs` module in `languages/mill-lang-java/src/`
- [ ] Define `JavaManifestUpdater` struct
- [ ] Implement `ManifestUpdater` trait with methods:
  - [ ] `add_dependency(path, dep)` - Add dependency to pom.xml
  - [ ] `remove_dependency(path, dep)` - Remove dependency from pom.xml
  - [ ] `update_package_name(path, old, new)` - Update artifact ID in pom.xml
  - [ ] `generate_manifest(package_name, deps)` - Generate new pom.xml
- [ ] Parse pom.xml using `quick_xml` (already in dependencies)
- [ ] Handle `<dependencies>` section properly
- [ ] Handle `<groupId>`, `<artifactId>`, `<version>` correctly
- [ ] Add tests for each method (minimum 10 tests):
  - [ ] Test adding dependency to empty pom.xml
  - [ ] Test adding dependency to existing dependencies
  - [ ] Test removing dependency
  - [ ] Test updating artifact ID
  - [ ] Test generating new pom.xml
  - [ ] Test with invalid XML (error handling)
  - [ ] Test with missing dependencies section
  - [ ] Test with complex pom.xml (parent, modules)
  - [ ] Test with properties/variables
  - [ ] Test preserving formatting and comments
- [ ] Add to `impl_capability_delegations!` in `lib.rs`

### Implement ModuleReferenceScanner

- [ ] Create `module_reference_scanner.rs` module
- [ ] Define `JavaModuleReferenceScanner` struct
- [ ] Implement `ModuleReferenceScanner` trait:
  - [ ] `scan_references(content, module_name)` - Find import statements referencing module
- [ ] Detect Java import patterns:
  - [ ] `import com.example.Module;` (single import)
  - [ ] `import com.example.*;` (wildcard import)
  - [ ] `import static com.example.Module.method;` (static import)
- [ ] Return line numbers (1-indexed) for all matches
- [ ] Exclude comments (`//` and `/* */`)
- [ ] Exclude string literals
- [ ] Add tests (minimum 8 tests):
  - [ ] Test finding single import
  - [ ] Test finding wildcard import
  - [ ] Test finding static import
  - [ ] Test excluding comments
  - [ ] Test excluding strings
  - [ ] Test multiple references on different lines
  - [ ] Test no matches (empty result)
  - [ ] Test qualified references in code (e.g., `Module.method()`)
- [ ] Add to `impl_capability_delegations!` in `lib.rs`

### Implement ImportAnalyzer

- [ ] Create `import_analyzer.rs` module
- [ ] Define `JavaImportAnalyzer` struct
- [ ] Implement `ImportAnalyzer` trait:
  - [ ] `analyze_imports(source)` - Parse and categorize imports
- [ ] Categorize imports:
  - [ ] External dependencies (group ID ≠ current package)
  - [ ] Internal modules (same package)
  - [ ] Standard library (java.*, javax.*)
- [ ] Parse import structure:
  - [ ] Extract package path
  - [ ] Extract class name
  - [ ] Detect wildcard imports
  - [ ] Detect static imports
- [ ] Add tests (minimum 10 tests):
  - [ ] Test single class import
  - [ ] Test wildcard import
  - [ ] Test static import
  - [ ] Test standard library imports (java.util.*)
  - [ ] Test external dependency imports
  - [ ] Test internal package imports
  - [ ] Test mixed imports
  - [ ] Test invalid import syntax (error handling)
  - [ ] Test empty source
  - [ ] Test imports in comments (should ignore)
- [ ] Add to `impl_capability_delegations!` in `lib.rs`

### Implement LspInstaller

- [ ] Create `lsp_installer.rs` module
- [ ] Define `JavaLspInstaller` struct
- [ ] Implement `LspInstaller` trait:
  - [ ] `check_lsp_installed()` - Check if `jdtls` is available
  - [ ] `install_lsp()` - Download and install Eclipse JDT Language Server
  - [ ] `get_install_command()` - Return install instructions
- [ ] Detection logic:
  - [ ] Check for `jdtls` in PATH
  - [ ] Check for `/usr/local/bin/jdtls`
  - [ ] Check for `~/.local/share/nvim/mason/bin/jdtls` (common LSP manager)
  - [ ] Check for Eclipse JDT LS JAR in standard locations
- [ ] Installation logic:
  - [ ] Download latest Eclipse JDT LS from GitHub releases
  - [ ] Extract to `~/.mill/lsp/jdtls/`
  - [ ] Create wrapper script for execution
  - [ ] Make executable (`chmod +x`)
- [ ] Add tests (minimum 5 tests):
  - [ ] Test check when installed (mocked)
  - [ ] Test check when not installed
  - [ ] Test get_install_command returns valid command
  - [ ] Test install creates correct directory structure
  - [ ] Test install creates executable wrapper
- [ ] Add to `impl_capability_delegations!` in `lib.rs`

### Integration and Testing

- [ ] Update `lib.rs` capability delegations:
  ```rust
  impl_capability_delegations! {
      manifest_updater => {
          manifest_updater: ManifestUpdater,
      },
      module_reference_scanner => {
          module_reference_scanner: ModuleReferenceScanner,
      },
      import_analyzer => {
          import_analyzer: ImportAnalyzer,
      },
      lsp_installer => {
          lsp_installer: LspInstaller,
      },
  }
  ```
- [ ] Add integration tests for full workflows:
  - [ ] Test adding dependency then scanning for its usage
  - [ ] Test updating package name then analyzing imports
  - [ ] Test LSP installation then using LSP features
- [ ] Run full Java plugin test suite (should be 28 → 61 tests)
- [ ] Verify all tests pass
- [ ] Run clippy with `-D warnings`
- [ ] Test with real Java project (e.g., Spring Boot sample)

### Documentation

- [ ] Document each new trait in rustdoc
- [ ] Add usage examples for each capability
- [ ] Document pom.xml parsing limitations (if any)
- [ ] Update Java plugin README with new capabilities
- [ ] Add troubleshooting section for LSP installation

## Success Criteria

- [ ] All 4 traits implemented and tested
- [ ] Minimum 33 new tests added (10 + 8 + 10 + 5)
- [ ] Total test count: 28 → 61 tests (54% of Rust baseline)
- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] Works with real Java projects
- [ ] Java plugin has 100% feature parity with Go/C# plugins

## Benefits

- **Feature Completeness**: Java plugin reaches 100% parity with other languages
- **Production Ready**: All critical workflows supported
- **Developer Experience**: Can manage dependencies, scan references, analyze imports
- **Consistency**: Follows same patterns as other language plugins
- **LSP Integration**: Automated LSP installation improves setup experience

## Implementation Notes

### ManifestUpdater Example (pom.xml)

**Before**:
```xml
<project>
  <dependencies>
    <dependency>
      <groupId>org.junit</groupId>
      <artifactId>junit-jupiter</artifactId>
      <version>5.9.0</version>
    </dependency>
  </dependencies>
</project>
```

**After adding dependency**:
```xml
<project>
  <dependencies>
    <dependency>
      <groupId>org.junit</groupId>
      <artifactId>junit-jupiter</artifactId>
      <version>5.9.0</version>
    </dependency>
    <dependency>
      <groupId>com.google.guava</groupId>
      <artifactId>guava</artifactId>
      <version>31.1-jre</version>
    </dependency>
  </dependencies>
</project>
```

### ModuleReferenceScanner Example

**Source**:
```java
package com.example;

import com.example.utils.Helper;  // Line 3 - Match!
import com.example.utils.*;       // Line 4 - Match!
// import com.example.utils.Test; // Comment - Ignore
import java.util.List;

public class Main {
    Helper.doSomething();          // Line 9 - Match (qualified reference)
}
```

**scan_references(source, "utils")** returns:
```json
[
  {"line": 3, "column": 24},
  {"line": 4, "column": 24},
  {"line": 9, "column": 5}
]
```

## References

- Go's `manifest.rs` for ManifestUpdater pattern
- C#'s `module_reference_scanner.rs` for scanner pattern
- Go's `import_analyzer.rs` for analyzer pattern
- Swift's `lsp_installer.rs` for installer pattern
- Eclipse JDT LS: https://github.com/eclipse/eclipse.jdt.ls

## Estimated Effort

- ManifestUpdater: 4 hours
- ModuleReferenceScanner: 2 hours
- ImportAnalyzer: 3 hours
- LspInstaller: 3 hours
- Testing: 4 hours
- **Total: 16 hours (~2 days)**
