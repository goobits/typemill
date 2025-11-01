# Add pub(crate) Visibility Markers

## Problem

Only 38 instances of `pub(crate)` exist across entire codebase (should be 200+). Many types are marked `pub` when they're only used within their own crate, unnecessarily expanding the public API surface.

Examples:
- `mill-services/src/services/reference_updater/mod.rs:10` - `pub use cache::FileImportInfo;` (only used internally)
- `mill-handlers/src/handlers/common/mod.rs:8` - `pub use checksums::{...}` (internal utilities)

## Solution

Audit all `pub` items across workspace and change to `pub(crate)` where appropriate.

## Checklists

### Audit mill-services
- [ ] Grep for all `pub struct`, `pub enum`, `pub fn` in mill-services
- [ ] Identify which items are only used within mill-services
- [ ] Change internal-only items to `pub(crate)`
- [ ] Verify public API still exported from lib.rs

### Audit mill-handlers
- [ ] Grep for all `pub struct`, `pub enum`, `pub fn` in mill-handlers
- [ ] Identify common/ utilities that should be `pub(crate)`
- [ ] Identify internal tool helpers that should be `pub(crate)`
- [ ] Change internal-only items to `pub(crate)`

### Audit mill-ast
- [ ] Review cache module - mark internal cache types `pub(crate)`
- [ ] Review analyzer module - mark internal analyzer types `pub(crate)`
- [ ] Review transformer module - mark internal helpers `pub(crate)`
- [ ] Keep only public API as `pub`

### Audit mill-foundation
- [ ] Review protocol module internals
- [ ] Review model module internals
- [ ] Review core module internals
- [ ] Mark implementation details as `pub(crate)`
- [ ] Keep stable API types as `pub`

### Audit mill-lsp
- [ ] Review LSP client internals
- [ ] Mark protocol handling details `pub(crate)`
- [ ] Keep public LSP interface as `pub`

### Audit mill-plugin-system
- [ ] Review registry internals
- [ ] Review adapter internals
- [ ] Mark implementation details `pub(crate)`
- [ ] Keep plugin interface as `pub`

### Audit Language Plugins
- [ ] Review rust plugin internals
- [ ] Review typescript plugin internals
- [ ] Review python plugin internals
- [ ] Mark parser internals `pub(crate)`
- [ ] Keep LanguagePlugin implementation as `pub`

### Create Visibility Guidelines
- [ ] Document when to use `pub` vs `pub(crate)`
- [ ] Add guidelines to contributing.md
- [ ] Create checklist for PR reviews

### Verification
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace`
- [ ] Run `cargo nextest run --workspace`
- [ ] Count `pub(crate)` instances (target: 200+)
- [ ] Verify crate public APIs unchanged
- [ ] Check dependent crates still compile

## Success Criteria

- 200+ uses of `pub(crate)` across workspace
- All internal-only types marked `pub(crate)`
- Public API surface reduced by ~70%
- No broken imports in dependent crates
- All tests pass
- Visibility guidelines documented

## Benefits

- Reduced public API surface area
- Clearer distinction between public and internal APIs
- Prevents accidental coupling to internals
- Enables refactoring internals without breaking changes
- Better encapsulation of implementation details
- AI agents see only intended public API
- Prevents future API creep
