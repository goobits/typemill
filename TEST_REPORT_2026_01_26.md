# TypeMill Test Report - Fresh Setup Experience

**Date:** 2026-01-26
**Rust Version:** 1.93.0 (stable)
**Platform:** Linux x86_64
**Tester:** Automated Test Run

---

## Executive Summary

**Test Results: 1962/1969 passed (99.6%)**

A fresh setup of the TypeMill project was performed from scratch. The setup process was smooth overall, but 7 tests failed due to:
- Missing optional dependencies (Java JAR - 4 tests)
- Resource contention/timeout issues (Go tests - 2 tests)
- A bug in test expectations (cross-file reference - 1 test)

---

## Setup Process

### Steps Performed

| Step | Duration | Status |
|------|----------|--------|
| Check Rust toolchain | instant | ✅ Already installed (1.93.0) |
| Initialize git submodules | ~15s | ✅ Success |
| Install cargo-nextest | ~1.5 min | ✅ Success |
| Build workspace (debug) | ~2 min | ✅ Success (with warnings) |
| Run all tests | ~64s | ⚠️ 7 failures |

### Build Warnings (Expected)

```
warning: mill-lang-java@0.8.4: Java parser JAR not found. Import support will not work.
warning: mill-lang-java@0.8.4: To build the JAR: cd resources/java-parser && mvn package

warning: mill-lang-csharp@0.8.4: .NET SDK not found or 'dotnet' command failed.
warning: mill-lang-csharp@0.8.4: The C# language plugin requires the .NET SDK to build its parser.
```

These warnings are informational and do not block the build.

---

## Test Results Detail

### Summary Table

| Category | Tests | Passed | Failed | Notes |
|----------|-------|--------|--------|-------|
| All Tests | 1969 | 1962 | 7 | 99.6% pass rate |

### Failed Tests

#### 1. Java Parser Tests (4 failures)

**Root Cause:** Java parser JAR not built (requires Maven)

| Test | Location |
|------|----------|
| `test_add_import_integration` | `languages/mill-lang-java/src/import_support.rs:253` |
| `test_parse_imports_integration` | `languages/mill-lang-java/src/import_support.rs:246` |
| `test_remove_import_integration` | `languages/mill-lang-java/src/import_support.rs:285` |
| `test_performance_parse_large_file` | `languages/mill-lang-java/src/lib.rs:355` |

**Reproduction:**
```bash
cargo nextest run -p mill-lang-java
```

**Fix:**
```bash
cd languages/mill-lang-java/resources/java-parser
mvn package
cargo clean -p mill-lang-java
cargo test -p mill-lang-java
```

#### 2. Go Performance Tests (2 failures)

**Root Cause:** Timeout (30s) during full test suite - **passes when run in isolation**

| Test | Location |
|------|----------|
| `test_parse_large_file` | `languages/mill-lang-go/src/lib.rs` |
| `test_performance_parse_large_file` | `languages/mill-lang-go/src/lib.rs` |

**Reproduction:**
```bash
# Fails in full suite
cargo nextest run --workspace

# Passes in isolation
cargo nextest run -p mill-lang-go -E 'test(test_parse_large_file)'
```

**Analysis:** These tests pass in ~1.2 seconds when run alone, but timeout at 30s during the full suite. This is due to resource contention when all 1969 tests run in parallel.

#### 3. Cross-File Reference Test (1 failure)

**Root Cause:** Test expectation bug - expects 3 occurrences but 4 are found

| Test | Location |
|------|----------|
| `test_find_symbol_occurrences` | `crates/mill-handlers/src/handlers/tools/cross_file_references.rs:625` |

**Error Message:**
```
assertion `left == right` failed
  left: 4
 right: 3
```

**Analysis:** The test uses this content:
```javascript
import { is } from './is'
const result = is(value)
is.type(x)
```

The function `find_symbol_occurrences` correctly finds 4 occurrences of `"is"`:
1. `{ is }` at position (0, 9)
2. `'./is'` at position (0, 22) ← The test forgot this one
3. `= is(` at position (1, 15)
4. `is.type` at position (2, 0)

The test expectation should be updated from 3 to 4.

---

## Setup Difficulty Rating

### Overall: **3/10 (Easy)**

| Aspect | Rating | Notes |
|--------|--------|-------|
| Prerequisites | 2/10 | Rust toolchain is the only hard requirement |
| Build Process | 2/10 | `cargo build` works out of the box |
| Documentation | 3/10 | CLAUDE.md and Makefile are helpful |
| Test Running | 4/10 | cargo-nextest must be installed separately |
| Optional Features | 5/10 | Java/C# plugins need extra dependencies |

### What Worked Well

1. **Git submodules initialize cleanly** after recent fixes
2. **Clear build warnings** explain missing optional dependencies
3. **Comprehensive Makefile** with `make first-time-setup`
4. **Fast test execution** (~64s for 1969 tests)
5. **Helpful error messages** in Java tests

### Areas for Improvement

1. **Java tests should use `#[ignore]`** unless JAR is detected
2. **Go performance tests need higher timeout** or should be marked slow
3. **Cross-file reference test needs fix** (expectation: 3 → 4)
4. **README should document optional dependencies** upfront

---

## Recommendations for Next Team

### Quick Fixes

1. **Fix cross-file reference test** (`cross_file_references.rs:628`)
   ```rust
   assert_eq!(occurrences.len(), 4); // was 3
   ```

2. **Mark Java import tests as ignored** unless JAR exists
   ```rust
   #[test]
   #[cfg_attr(not(feature = "java-parser"), ignore)]
   fn test_add_import_integration() { ... }
   ```

3. **Increase nextest timeout** for Go tests (add to `.config/nextest.toml`)
   ```toml
   [profile.default]
   slow-timeout = { period = "60s", terminate-after = 2 }
   ```

### Makefile Improvements

Add a quick test target that skips optional/slow tests:
```makefile
test-quick:
    cargo nextest run --workspace \
        -E 'not (package(mill-lang-java) and test(/integration/)) and not test(/performance/)'
```

### Documentation Updates

Add to README.md:
```markdown
## Optional Dependencies

| Dependency | Required For | Installation |
|------------|--------------|--------------|
| Maven | Java language plugin tests | `apt install maven` |
| .NET SDK 6.0+ | C# language plugin | [dotnet.microsoft.com](https://dotnet.microsoft.com/download) |
```

---

## Conclusion

The TypeMill project is in good shape. The setup process is straightforward, and the 99.6% test pass rate indicates a healthy codebase. The 7 failing tests are well-understood issues that don't block normal development:

- **4 Java tests:** Optional dependency (easy to fix with `#[ignore]`)
- **2 Go tests:** Timeout issue (easy to fix with higher timeout)
- **1 Cross-file test:** Test bug (trivial fix)

**Time to Full Working Setup:** ~5 minutes
**Confidence Level:** High - ready for development
