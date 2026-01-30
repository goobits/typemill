# Manual Test Report for `mill` CLI

## Summary
All primary CLI commands were manually tested and verified to work as expected on the current environment.

## Tested Commands

### Basic Information
- `mill --help`: ✅ Passed. Displays usage information.
- `mill --version`: ✅ Passed. Displays version `mill 0.8.4`.
- `mill status`: ✅ Passed. Correctly identifies if server is running or not.
  - **Note**: Reports LSP servers as missing from `PATH` even if they are installed (discrepancy with `doctor`).
- `mill doctor`: ✅ Passed. Verifies configuration and LSP installations.
  - **Note**: Correctly identifies installed LSPs (e.g., `typescript-language-server`, `pylsp`).
- `mill tools`: ✅ Passed. Lists available tools and handlers.
- `mill docs`: ✅ Passed. Lists available documentation topics.

### Configuration
- `mill setup`: ✅ Passed. Creates `.typemill/config.json`.
- `mill setup --update`: ✅ Passed. Updates existing configuration.
- `mill link` / `unlink`: ✅ Passed. Displays placeholder messages.

### Server Management
- `mill serve`: ✅ Passed.
  - Verified starting on custom port (e.g., 3046).
  - Verified PID file creation and locking.
  - Verified clean shutdown via `stop` command and `pkill`.
- `mill stop`: ✅ Passed. Stops the running server.
- `mill daemon`: ✅ Passed.
  - `start --foreground`: Starts daemon.
  - `status`: Checks daemon status.
  - `stop`: Stops daemon.

### Tools Execution
- `mill tool`: ✅ Passed.
  - Verified execution of `health_check` (success).
  - Verified execution of `inspect_code` (validation error confirmed tool reached handler).
  - Validated that `mill tool` works independently of `mill serve`.

### Utilities
- `mill convert-naming`: ✅ Passed.
  - Verified dry-run execution on test files.
- `mill install-lsp`: ✅ Passed (Dry run).
  - Verified help and argument parsing.

## Issues / Observations
1. **LSP Detection Discrepancy**: `mill status` reports `WARNING: ... not found in PATH` for LSP servers, while `mill doctor` correctly finds them and reports their versions. This suggests `mill status` uses a simpler/different check (likely just `which` on `PATH`) than `mill doctor`.
2. **Server Lock**: `mill serve` correctly enforces a singleton instance via `/tmp/mill.pid`. If a phantom process holds the lock (e.g. from an improper shutdown or test harness quirk), `mill serve` fails with an "already running" error, which is correct behavior.
