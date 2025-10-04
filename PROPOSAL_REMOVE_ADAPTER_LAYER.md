# Proposal: Remove cb-lang-rust-adapter and Consolidate Language Architecture

**Status:** Draft
**Created:** 2025-10-04
**Recommendation:** ✅ **REMOVE** cb-lang-rust-adapter

---

## Executive Summary

The `cb-lang-rust-adapter` crate appears to be **unused legacy code** from an incomplete architectural migration. It should be removed to eliminate confusion, reduce maintenance burden, and simplify the codebase.

---

## Current Situation

### Two Conflicting Implementations Exist

**1. Active Implementation** (in use):
- Location: `crates/cb-ast/src/language.rs:290`
- Type: Simple zero-sized type `pub struct RustAdapter;`
- Dependencies: None (uses direct `syn` calls)
- Usage: Registered in `FileService` (line 67):
  ```rust
  adapter_registry.register(Arc::new(cb_ast::language::RustAdapter));
  ```

**2. Unused Implementation** (orphaned):
- Location: `crates/languages/cb-lang-rust-adapter/`
- Type: Complex composable adapter
- Dependencies: `cb-lang-rust`, `cb-plugin-api`, `syn`, `quote`, `tokio`
- Usage: **NOWHERE** - grep shows zero actual usage
- Size: 439 lines of code

### Evidence of Non-Usage

```bash
$ grep -r "cb_lang_rust_adapter" crates/
# Only finds:
# - Its own Cargo.toml
# - Comment in adapter_registry.rs (line 17, 61, 122)
# - Proposal document (01_PROPOSAL_LANG_CRATE.md)

$ grep -r "cb-lang-rust-adapter" crates/
# Only finds:
# - Its own Cargo.toml
# - Not in any Cargo.toml dependencies
# - Not imported anywhere
```

The adapter-registry only references it in:
- Documentation comments (examples)
- Feature-gated code that's **not enabled**: `#[cfg(feature = "rust-adapter")]`

---

## Analysis

### Why the Adapter Exists (Historical Context)

Based on `01_PROPOSAL_LANG_CRATE.md`, there was a planned architecture:

```
Intelligence Layer (cb-lang-rust) → Adapter Layer (cb-lang-rust-adapter)
         ↓                                      ↓
    Pure AST parsing                    Refactoring operations
```

**The Problem:** This migration was **never completed**. The code went from:

1. ✅ Created `cb-lang-rust` (intelligence plugin) - **IN USE**
2. ✅ Created `cb-lang-rust-adapter` (composable adapter) - **NOT IN USE**
3. ❌ Never migrated `FileService` to use the new adapter
4. ❌ Never removed the old `RustAdapter` from `language.rs`

### Why the Split is Questionable

**Claimed Benefits:**
- "Clean separation of parsing vs refactoring"
- "Intelligence layer remains pure and reusable"

**Reality Check:**
1. **cb-lang-rust is already impure** - It has both:
   - AST parsing (`parse()`, `list_functions()`)
   - Manifest operations (`analyze_manifest()`)
   - Import parsing (`parse_imports()`)

2. **The adapter duplicates code** - `cb-lang-rust-adapter` has:
   - Its own `syn` parsing (lines 319-330)
   - Its own visitor pattern (lines 334-438)
   - Calls to `cb_lang_rust::parse_imports()` then does its own work

3. **No reuse evidence** - There's no proof the "intelligence plugin" needs to be pure for other tools. This is premature abstraction.

4. **Two crates = more complexity**:
   - 2 separate packages to version/release
   - Circular-ish dependency concerns
   - More cognitive load for contributors

### The Simple Truth

**The adapter layer adds NO value over having one cohesive language plugin crate.**

A single `cb-lang-rust` crate can provide:
- ✅ AST parsing
- ✅ Symbol extraction
- ✅ Import rewriting
- ✅ Manifest handling
- ✅ Module reference finding

This is what **every other language** does (TypeScript, Python, Go, Java) - they're all in `language.rs` as single implementations.

---

## Recommendation

### ✅ REMOVE cb-lang-rust-adapter

**Rationale:**
1. **Not used** - Zero imports, zero references
2. **Incomplete migration** - The old `RustAdapter` is still active
3. **Premature abstraction** - No evidence splitting helps
4. **Consistency** - Other languages don't have adapters
5. **Maintainability** - Fewer crates = less overhead

### Keep cb-lang-rust as-is

`cb-lang-rust` is a good self-contained plugin:
- Implements `LanguageIntelligencePlugin` trait
- Used by plugin manager
- Clean, tested, documented

### Future: One Crate Per Language (No Adapters)

Following the proposal in `01_PROPOSAL_LANG_CRATE.md`, but **simplified**:

```
crates/languages/
├── cb-lang-rust/         # ✅ All Rust functionality (keep)
├── cb-lang-typescript/   # 🆕 All TypeScript functionality
├── cb-lang-python/       # 🆕 All Python functionality
├── cb-lang-go/           # 🆕 All Go functionality
└── cb-lang-java/         # 🆕 All Java functionality
```

**No `-adapter` suffix. One crate = one language.**

Each crate implements:
- `LanguageIntelligencePlugin` (for LSP/plugin system)
- `LanguageAdapter` (for refactoring operations)

This is **simpler** and **eliminates** the artificial intelligence/adapter split.

---

## Migration Path

### Step 1: Verify No Usage (5 min)
```bash
# Confirm cb-lang-rust-adapter is truly unused
grep -r "cb_lang_rust_adapter" crates/ --exclude-dir=cb-lang-rust-adapter
grep -r "cb-lang-rust-adapter" crates/ --exclude-dir=cb-lang-rust-adapter
grep -r "use cb_lang_rust_adapter" crates/

# Check workspace members
grep "cb-lang-rust-adapter" Cargo.toml
```

### Step 2: Remove from Workspace (2 min)
```toml
# Cargo.toml - Remove from members list
[workspace]
members = [
    # ... other crates ...
    # "crates/languages/cb-lang-rust-adapter",  # ← DELETE THIS
]
```

### Step 3: Delete the Crate (1 min)
```bash
git rm -r crates/languages/cb-lang-rust-adapter/
```

### Step 4: Clean Up References (5 min)
```rust
// crates/cb-ast/src/adapter_registry.rs
// Remove or update doc comments mentioning cb-lang-rust-adapter

// Lines 17, 61, 122 - change examples to use cb_ast::language::RustAdapter
```

### Step 5: Update Proposal Document (3 min)
```bash
# Mark 01_PROPOSAL_LANG_CRATE.md as superseded
# Or update to reflect "no adapter layer" decision
```

### Step 6: Test (5 min)
```bash
cargo test --workspace
cargo build --release
```

### Step 7: Commit (2 min)
```bash
git commit -m "refactor: Remove unused cb-lang-rust-adapter crate

The cb-lang-rust-adapter was created as part of an incomplete
architectural migration to split intelligence and adapter layers.

This split added unnecessary complexity:
- Not actually used anywhere in the codebase
- Old RustAdapter in language.rs is still active
- No evidence the split provides value over single-crate approach
- Other languages (TS, Python, Go, Java) don't use adapters

Keeping cb-lang-rust as the single source of truth for Rust
language support. Future language extractions will follow the
simpler one-crate-per-language pattern.

Refs: 01_PROPOSAL_LANG_CRATE.md"
```

**Total Time:** ~25 minutes

---

## Risks

### Low Risk

- ✅ Crate is unused - removal is safe
- ✅ No external dependencies on it
- ✅ Tests don't reference it
- ✅ Easy to revert if needed (git)

### Potential Objection: "But the architecture proposal!"

**Response:** The proposal in `01_PROPOSAL_LANG_CRATE.md` can be **simplified**. The adapter pattern adds complexity without demonstrated value. A single crate per language is cleaner and matches how other languages work.

---

## Alternative: Keep but Actually Use It

**Not Recommended** because:

1. **Requires more work**:
   - Migrate `FileService` to use `cb_lang_rust_adapter::RustAdapter`
   - Remove old `RustAdapter` from `language.rs`
   - Add `cb-lang-rust-adapter` to Cargo dependencies
   - Test the migration

2. **Creates asymmetry**:
   - Rust has 2 crates (intelligence + adapter)
   - Other languages have 1 implementation (in `language.rs`)
   - Now need to decide: migrate them all? Or keep asymmetry?

3. **No clear benefit**:
   - Current `RustAdapter` in `language.rs` works fine
   - `cb-lang-rust` already provides what's needed
   - Additional layer doesn't solve a real problem

---

## Decision Matrix

| Option | Effort | Complexity | Consistency | Maintenance |
|--------|--------|------------|-------------|-------------|
| **Remove adapter** | Low (25 min) | ✅ Reduced | ✅ Unified | ✅ Less |
| Keep but use it | High (2-3 hrs) | ❌ Increased | ⚠️ Asymmetric | ❌ More |
| Keep as-is (unused) | None | ❌ Confusing | ❌ Worst | ❌ Debt |

---

## Conclusion

**Remove `cb-lang-rust-adapter`** to eliminate unused code and architectural confusion.

Going forward, adopt **one crate per language** without adapter layers:
- Simpler mental model
- Less boilerplate
- Easier to contribute
- Matches existing patterns (TypeScript, Python, etc.)

The intelligence/adapter split was well-intentioned but over-engineered. Let's fix it before it spreads to other languages.

---

## Questions?

1. **Q: What if we need the separation later?**
   A: Git history preserves it. We can reintroduce if there's actual evidence of need.

2. **Q: Doesn't this violate separation of concerns?**
   A: A single cohesive language plugin IS a proper separation - it separates Rust support from other languages. The intelligence/adapter split is internal overengineering.

3. **Q: What about the proposal document's vision?**
   A: Update it. The vision should be "one crate per language" not "two crates per language".

---

**Recommendation: ✅ Proceed with removal**
