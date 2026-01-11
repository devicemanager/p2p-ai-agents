# Story 3.6: Integration Tests & Coverage

**Story ID**: 3.6  
**Story Key**: 3-6-integration-tests-coverage  
**Epic**: Week 3 - MVP Implementation  
**Owner**: Amelia (Dev)  
**Status**: complete  
**Priority**: High  
**Effort**: 0.5 days  
**Created**: 2026-01-10  
**Completed**: 2026-01-11  
**Dependencies**: Stories 3.2-3.5 (All components + demo)

---

## Story

**As a** developer  
**I want** comprehensive integration tests and high code coverage  
**So that** I can confidently claim the MVP is production-ready and launch-worthy

**Context**: Create integration tests that validate full system behavior and measure code coverage per project standards (90%+ overall).

**Blocks**: Story 4.1 (Launch readiness requires test validation)

---

## Acceptance Criteria

### AC1: Integration Test Suite
- [ ] Test file: `tests/integration_test.rs` created
- [ ] Test: 2-agent discovery and connection
- [ ] Test: Task exchange end-to-end
- [ ] Test: Error handling (timeout, connection failure)
- [ ] All tests pass with `cargo test --test integration_test`

### AC2: Code Coverage
- [ ] Overall coverage: ≥90% (project requirement)
- [ ] Critical paths coverage: ≥95%
- [ ] Identity module: ≥95%
- [ ] Network module: ≥90%
- [ ] Task module: ≥95%
- [ ] Coverage report generated: `lcov.info`

### AC3: Test Quality
- [ ] Tests are deterministic (no flakiness)
- [ ] Tests run in <60 seconds total
- [ ] Tests isolated (no shared state)
- [ ] Clear test names describe what's tested

### AC4: CI Integration
- [ ] Tests run automatically via `make test`
- [ ] Coverage report generated via `make coverage`
- [ ] CI passing badge ready for README

---

## Tasks/Subtasks

### Task 1: Set Up Integration Test Infrastructure
- [ ] Create `tests/integration_test.rs`
- [ ] Add test helpers (create_test_agent, wait_for_discovery)
- [ ] Configure test logging (tracing-subscriber)
- [ ] Test: Verify test infrastructure works

### Task 2: Write Discovery Integration Test
- [ ] Test: `test_two_agents_discover_each_other`
- [ ] Start 2 agents in test
- [ ] Wait for discovery (timeout: 10s)
- [ ] Assert both agents see each other
- [ ] Verify test passes

### Task 3: Write Task Exchange Integration Test
- [ ] Test: `test_end_to_end_task_exchange`
- [ ] Start 2 agents
- [ ] Wait for discovery
- [ ] Agent A sends task to Agent B
- [ ] Agent B executes and responds
- [ ] Assert result received and correct
- [ ] Verify test passes

### Task 4: Write Error Handling Tests
- [ ] Test: `test_discovery_timeout`
  - [ ] Start 1 agent (no peers to discover)
  - [ ] Wait for timeout
  - [ ] Assert timeout error returned
- [ ] Test: `test_task_send_to_unknown_peer`
  - [ ] Try to send task to non-existent peer
  - [ ] Assert error returned
- [ ] Verify all error tests pass

### Task 5: Measure Code Coverage
- [ ] Install cargo-llvm-cov: `cargo install cargo-llvm-cov`
- [ ] Run: `cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info`
- [ ] Review coverage report
- [ ] Identify uncovered lines
- [ ] Add tests to reach 90% threshold
- [ ] Verify coverage ≥90%

### Task 6: Coverage Gap Analysis
- [ ] Check module-by-module coverage
- [ ] Prioritize critical paths (identity, network, task)
- [ ] Add unit tests for uncovered branches
- [ ] Re-run coverage measurement
- [ ] Verify all modules meet targets

### Task 7: CI Configuration
- [ ] Verify `make test` runs all tests
- [ ] Verify `make coverage` generates lcov.info
- [ ] Update codecov.yml configuration (if needed)
- [ ] Add coverage badge to README.md
- [ ] Document coverage requirements in CONTRIBUTING.md

### Task 8: Documentation
- [ ] Update docs/development/testing-guide.md
- [ ] Document how to run integration tests
- [ ] Document how to generate coverage reports
- [ ] Add examples of writing new tests

---

## Dev Notes

### Technical Context

**Project Coverage Requirements** (from docs/development/testing-guide.md):
- Overall: 90%+
- Critical paths: 95%+
- Security-critical code: 100%

**From Week 3 Implementation Summary**:
```
Tests Created: 7 integration tests
Coverage: 91% (231 tests passing)
```

**Test Structure**:
```
tests/
└── integration_test.rs  # ~300 LOC
```

### Implementation Guidance

**Integration Test Template**:
```rust
#[tokio::test]
async fn test_two_agents_discover_each_other() {
    // Arrange
    let identity_a = AgentIdentity::generate();
    let identity_b = AgentIdentity::generate();
    let mut agent_a = P2PAgent::new(identity_a).await.unwrap();
    let mut agent_b = P2PAgent::new(identity_b).await.unwrap();
    
    // Act
    tokio::spawn(async move { agent_a.start().await });
    tokio::spawn(async move { agent_b.start().await });
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Assert
    assert_eq!(agent_a.list_peers().len(), 1);
    assert_eq!(agent_b.list_peers().len(), 1);
}
```

### Coverage Measurement Commands

```bash
# Install tool (once)
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# View HTML report
cargo llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html
```

### Coverage Targets by Module

| Module | Target | Rationale |
|--------|--------|-----------|
| `src/identity.rs` | 95% | Security-critical (crypto) |
| `src/network/` | 90% | Core functionality |
| `src/task/` | 95% | Business logic |
| `examples/mvp_demo.rs` | N/A | Demo code, not library |

### Test Quality Checklist

- [ ] Tests are fast (<1s per test)
- [ ] Tests are isolated (no shared state)
- [ ] Tests are deterministic (no random failures)
- [ ] Test names describe behavior (not implementation)
- [ ] Arrange-Act-Assert pattern used
- [ ] No `unwrap()` in test code (use `?` or `expect()`)

### CI Pipeline Validation

**Commands that must pass**:
```bash
cargo test --all-features --workspace  # All tests
cargo clippy --all-targets --all-features  # Linting
cargo fmt --check  # Formatting
make coverage  # Coverage report
```

---

## Dev Agent Record

### Debug Log
*To be populated during implementation*

### Implementation Plan
*To be documented as tasks are executed*

### Completion Notes
*To be filled when story is complete*

---

## File List

### Files Created
*To be updated as implementation proceeds*

### Files Modified
*To be updated as implementation proceeds*

---

## Change Log

- **2026-01-10**: Story created (Amelia) - Final validation for launch readiness

---

## Status

**Current**: not-started  
**Next**: ready-for-dev (after Stories 3.2-3.5 complete)  
**Goal**: review (tests pass, coverage ≥90%, CI green)
