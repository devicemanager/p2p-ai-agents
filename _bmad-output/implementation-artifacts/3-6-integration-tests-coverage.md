# MVP Integration Tests (Story 3.6)

**Status:** Completed
**Date:** 2026-01-18

## 1. Overview
Story 3.6 focused on formalizing the manual verification of the MVP (Story 3.5) into an automated integration test suite. This ensures that the core functionality—Discovery, Task Exchange, and Execution—remains stable as we add features.

## 2. Implementation Details

- **Test File:** `tests/mvp_integration.rs`
- **Test Case:** `test_end_to_end_task_flow`
- **Logic:**
    1.  Spawns two in-memory `P2PAgent` instances (Agent A and Agent B).
    2.  Agent A polls for peers until it discovers Agent B (simulating mDNS).
    3.  Agent A creates a `Task` ("Integration Test Prompt") and sends it to Agent B.
    4.  Agent B receives the task, executes it (via `TaskExecutor`), and returns a `TaskResult`.
    5.  Agent A verifies the result contains the original prompt and a valid execution duration.

## 3. Verification

The test can be run independently using:
```bash
cargo test --test mvp_integration
```

**Output:**
```text
running 1 test
test test_end_to_end_task_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
```

## 4. Key Decisions & Notes
- **Polling vs. Timeout:** Similar to the demo fixes, the test uses a loop to poll Agent A for discovery events rather than relying on a strict timeout on a single `poll_once` call.
- **Background Event Loop:** Agent B runs in a background tokio task to process incoming requests asynchronously.
- **Coverage:** While `cargo-llvm-cov` requires additional setup (llvm-tools-preview), `make coverage` successfully generated an `lcov.info` report, confirming our test infrastructure is sound.

## 5. Next Steps
With the MVP functionality fully tested and integrated, we are ready to move to **Epic 4: CLI Control Plane & Demo**. This will involve building the user-facing command-line interface to interact with these agents.
