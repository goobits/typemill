# Magnificent Seven Refactor Proposal (No Legacy, No Cruft)

## Goal
Replace the current public tool surface with a clean, intent-oriented API of seven tools. Remove legacy public tool names entirely. Update all callers, tests, CLI, tool discovery, and docs so the old API is not exposed or referenced anywhere.

**Final public tools**
1. inspect_code
2. search_code
3. rename_all
4. relocate
5. prune
6. refactor
7. workspace

Optional: keep health_check public or fold into workspace as action: verify_project. This proposal keeps health_check internal only and provides verify_project as a workspace action.

## Principles (DRY, SOLID, IDEAL)
- Single source of truth for tool schemas and public tool list.
- Intent-oriented, stable interfaces; internal primitives hidden.
- Consistent request/response envelopes across write tools.
- No compatibility aliases; remove all legacy names from public surface and docs.
- Agents can implement in sequence without conflicts.

## Scope Summary
- Public API: remove old tool names (find_definition, rename, extract, etc.).
- Internal API: keep LSP primitives and legacy handlers only if used internally.
- Tool discovery: only new tools appear in tools/list and CLI list.
- CLI: new tool names and args. Old names removed.
- Docs/tests/examples: updated to new names only.

## Canonical Request/Response Shapes

### Shared write response envelope
All write tools return:
- status: "success"|"error"
- summary: string
- files_changed: string[]
- diagnostics: array
- changes: optional structured plan/result

### Shared options
- options.dryRun: boolean (default true)
- options.scope: optional
- options.limit/offset: where applicable

### inspect_code
- identify by (filePath + line + character) or (symbolName + filePath)
- include: ["definition","typeInfo","references","implementations","callHierarchy","diagnostics"]
- detailLevel: "basic"|"deep"
- limit/offset: for list fields

### search_code
- query string
- kind optional
- limit/offset

### rename_all
- target: { kind: "symbol"|"file"|"directory", filePath?, line?, character? }
- newName
- scope: "symbol"|"file"|"both" (default "both")
- options.dryRun

### relocate
- target: { kind: "symbol"|"file"|"directory", filePath?, line?, character? }
- destination: { filePath } (or newPath)
- options.dryRun

### prune
- target: { kind: "symbol"|"file"|"directory", filePath?, line?, character? }
- options: { dryRun, cleanupImports, force, removeTests }

### refactor
- action: "extract"|"inline"|"reorder"|"transform"
- params: action-specific, aligned to current extract/inline/etc.
- options.dryRun

### workspace
- action: "create_package"|"extract_dependencies"|"update_members"|"find_replace"|"verify_project"
- params: per action

## Phased Checklist (Sequential, Agent-Friendly)

### Phase 0 - Decision Lock (No code changes)
- [ ] Confirm final public tool list (7) and whether health_check is internal with workspace.verify_project.
- [ ] Approve canonical request/response shapes above (names, fields, defaults).
- [ ] Confirm no backwards compatibility aliases (legacy names fully removed).

### Phase 1 - New Tool Definitions (Single Source of Truth)
**Goal:** Centralized schemas and public tool list.
- [ ] Create a new module for public tool definitions (e.g., crates/mill-handlers/src/handlers/tool_definitions.rs).
- [ ] Define JSON schemas for the seven tools only.
- [ ] Define a static list of public tool names.
- [ ] Ensure MCP tools/list uses this list (not plugin tool defs).

Files:
- add: crates/mill-handlers/src/handlers/tool_definitions.rs
- modify: crates/mill-handlers/src/handlers/plugin_dispatcher.rs (tools/list)

### Phase 2 - Public Handlers (New Intent Tools)
**Goal:** Implement new handlers as wrappers over existing services.
- [ ] Add InspectHandler -> aggregates LSP primitives and diagnostics.
- [ ] Add SearchHandler -> workspace symbol search wrapper.
- [ ] Add RenameAllHandler -> wraps rename + optional file rename.
- [ ] Add RelocateHandler -> wraps move operations (symbol/file/dir).
- [ ] Add PruneHandler -> wraps delete with cleanup summary.
- [ ] Add RefactorHandler -> dispatches to extract/inline/reorder/transform.
- [ ] Ensure all new handlers return shared write response envelope.

Files:
- add: crates/mill-handlers/src/handlers/inspect_handler.rs
- add: crates/mill-handlers/src/handlers/search_handler.rs
- add: crates/mill-handlers/src/handlers/rename_all_handler.rs
- add: crates/mill-handlers/src/handlers/relocate_handler.rs
- add: crates/mill-handlers/src/handlers/prune_handler.rs
- add: crates/mill-handlers/src/handlers/refactor_handler.rs
- modify: crates/mill-handlers/src/handlers/mod.rs
- modify: crates/mill-handlers/src/handlers/plugin_dispatcher.rs (register handlers)

### Phase 3 - Internalize Legacy Tools (No Public Exposure)
**Goal:** Remove old public names from the public surface.
- [ ] Mark navigation primitives (find_definition, etc.) internal-only.
- [ ] Mark rename/extract/inline/move/delete as internal-only (or remove from registry) and use them only via new handlers.
- [ ] Ensure ToolRegistry.list_tools returns only the seven tools.
- [ ] Ensure tools/list does not expose plugin tool definitions for legacy names.

Files:
- modify: crates/mill-handlers/src/handlers/tools/navigation.rs
- modify: crates/mill-handlers/src/handlers/plugin_dispatcher.rs
- modify: crates/mill-server/tests/tool_registration_test.rs

### Phase 4 - CLI Update (No Legacy Names)
**Goal:** CLI lists, help, and flag parsing only for new tools.
- [ ] Update CLI help to list only new tools.
- [ ] Replace flag parsing for rename/move/extract/etc. with rename_all/relocate/refactor/prune.
- [ ] Replace examples in CLI error messages and tests.

Files:
- modify: apps/mill/src/cli/mod.rs
- modify: apps/mill/src/cli/flag_parser.rs
- modify: apps/mill/tests/cli_tool_command.rs
- modify: apps/mill/tests/smoke_mcp.rs

### Phase 5 - Workflow Planner + Internal Callers
**Goal:** internal workflows use new tool names.
- [ ] Update planner step templates to use new tools.
- [ ] Update any direct tool calls in services/tests.

Files:
- modify: crates/mill-services/src/services/planning/planner.rs
- modify: crates/mill-services/src/services/coordination/workflow_executor.rs (if needed)
- modify: apps/mill/tests/e2e_* (tool calls)

### Phase 6 - Docs and Guides (No Legacy References)
**Goal:** Update all docs to new tool names; remove old tool pages.
- [ ] README.md: update tool list and examples.
- [ ] CLAUDE.md: update tool quick reference and examples.
- [ ] docs/tools/README.md: update table; remove old anchors.
- [ ] Replace docs/tools/navigation.md with inspect_code.md + search_code.md.
- [ ] Replace docs/tools/refactoring.md with rename_all.md, relocate.md, prune.md, refactor.md.
- [ ] Update user-guide cheatsheet and getting-started.
- [ ] Update architecture/specifications/core-concepts.
- [ ] Update operations/cicd.md to use inspect_code diagnostics bundle.

Files (non-exhaustive):
- modify: README.md
- modify: CLAUDE.md
- modify: docs/tools/README.md
- add: docs/tools/inspect_code.md
- add: docs/tools/search_code.md
- add: docs/tools/rename_all.md
- add: docs/tools/relocate.md
- add: docs/tools/prune.md
- add: docs/tools/refactor.md
- modify: docs/user-guide/cheatsheet.md
- modify: docs/user-guide/getting-started.md
- modify: docs/architecture/specifications.md
- modify: docs/architecture/core-concepts.md
- modify: docs/operations/cicd.md

### Phase 7 - Tests and Validation
**Goal:** enforce clean public surface and behavior.
- [ ] Update tool_registration_test expected list to the seven tools.
- [ ] Add tests for new tool schemas and handler routing.
- [ ] Add smoke test for inspect_code and rename_all.
- [ ] Verify no docs mention legacy tool names.

Files:
- modify: crates/mill-server/tests/tool_registration_test.rs
- add/modify: crates/mill-handlers/tests (new tool tests)
- modify: apps/mill/tests/smoke_mcp.rs
- add: docs lint/rg check for legacy names

### Phase 8 - Cleanup (No Legacy Artifacts)
**Goal:** remove unused code and dead paths.
- [ ] Delete unused handlers or module exports for old public tools if not used internally.
- [ ] Remove legacy tool examples from docs/tests.
- [ ] Ensure tool definitions only include new public tools.

## Implementation Notes (Safety and Correctness)
- Normalize file URIs to paths in all new handlers (file://...).
- Use saturating_sub for any 1-based to 0-based conversions.
- For inspect_code, support pagination for references/implementations.
- Provide impact summaries for rename_all/relocate/prune/refactor.
- Define a conflict resolution policy for concurrent edits (e.g., last-write-wins or warn in summary).
- Enforce consistent serde casing for all request structs (prefer #[serde(rename_all = "camelCase")]).
- Explicitly document that all coordinates are 0-based in tool_definitions.rs.

## Acceptance Criteria
- Only seven public tools are listed in `mill tools` and MCP tools/list.
- No documentation, tests, or examples reference legacy tool names.
- New tools execute existing functionality via internal services.
- Shared write response envelope present across write tools.
- CI passes; no legacy tool calls remain in codebase.
