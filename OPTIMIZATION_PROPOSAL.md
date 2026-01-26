# Test Optimization Proposal

## Identified Issues

During the analysis of the test suite, the following issues were identified contributing to slow execution and instability:

1.  **Blocking Sleeps in Async Tests**:
    - `tests/e2e/src/resilience_tests.rs` uses `std::thread::sleep` inside `#[tokio::test]` functions. This blocks the async runtime thread, preventing other tasks from making progress and potentially causing deadlocks or timeouts.

2.  **Explicit Wait Times**:
    - Tests use hardcoded durations (e.g., `thread::sleep(Duration::from_millis(2000))`) to wait for server startup or crash detection. This makes tests as slow as the worst-case assumption and flaky if the operation takes longer.

3.  **Compilation Bottlenecks**:
    - The `mill-plugin-system` crate had `Send` trait violations that caused compilation failures, preventing tests from running cleanly. (Fixed during analysis).

## Proposed Optimizations

### 1. Non-Blocking Async Sleeps
**Change**: Replace `std::thread::sleep` with `tokio::time::sleep`.
**Benefit**: Allows the Tokio runtime to process other tasks (like background I/O or other tests) while waiting.

```rust
// Before
std::thread::sleep(Duration::from_millis(500));

// After
tokio::time::sleep(Duration::from_millis(500)).await;
```

### 2. Polling Instead of Fixed Waits
**Change**: Replace fixed delays with polling mechanisms that check for a condition (e.g., server port open, process exit) repeatedly with a short interval/timeout.
**Benefit**: Tests proceed immediately when the condition is met, significantly reducing runtime.

```rust
// Before
thread::sleep(Duration::from_millis(2000)); // Wait for server

// After
let start = std::time::Instant::now();
while start.elapsed() < Duration::from_secs(5) {
    if check_server_ready().await { break; }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### 3. Reuse Test Harness Capabilities
**Change**: Utilize `TestClient::wait_for_lsp_ready()` and other existing helpers in `mill-test-support` that already implement efficient polling.
**Benefit**: Reduces code duplication and ensures consistent, optimized waiting logic.

### 4. Parallelize E2E Tests
**Change**: Ensure `e2e` tests use dynamically allocated ports or unique temporary directories, allowing removal of `serial_test` constraints.
**Benefit**: Dramatic speedup by utilizing multiple CPU cores.

## Implementation Plan

1.  **Refactor `resilience_tests.rs`**:
    - Convert `std::thread::sleep` to `tokio::time::sleep`.
    - Implement polling for server startup in `test_authentication_failure_websocket`.
    - Implement polling for crash detection in `test_lsp_crash_resilience`.

2.  **Verify**: Run the optimized tests to ensure they pass and measure the time reduction.
