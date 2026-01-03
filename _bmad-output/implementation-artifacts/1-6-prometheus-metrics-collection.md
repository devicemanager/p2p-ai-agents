# Story 1.6: Prometheus Metrics Collection

Status: needs-rework

## Story

As a node operator,
I want my agent to expose performance and resource metrics via Prometheus endpoint,
So that I can monitor agent health and performance.

## Acceptance Criteria

**AC1: Prometheus Endpoint with Standard Metrics**
```
Given: the agent is running with metrics enabled
When: accessing http://localhost:8080/metrics
Then: the response contains Prometheus-formatted metrics
And: includes process_cpu_usage gauge
And: includes process_memory_bytes gauge
And: includes agent_peers_connected gauge
And: the response has Content-Type: text/plain
```

**AC2: Message Processing Metrics**
```
Given: the agent processes a message
When: the message is successfully handled
Then: the messages_received_total counter is incremented
And: the message_processing_duration_seconds histogram records the duration
```

**AC3: Storage Operation Metrics**
```
Given: a storage operation completes
When: recording the operation metrics
Then: storage_operations_total counter is incremented
And: labeled with operation type (get/put/delete) and backend (local/redis)
And: storage_operation_duration_seconds histogram records the duration
```

**AC4: Metrics Endpoint Disable Option**
```
Given: metrics endpoint is disabled in configuration
When: the agent starts
Then: no HTTP server is started on port 8080
And: the agent logs "Metrics endpoint disabled"
```

## Implementation Plan

### Files to Create/Modify

1. **Create: `src/metrics/mod.rs`**
   - MetricsCollector struct
   - Prometheus exporter setup
   - Standard metric definitions

2. **Create: `src/metrics/server.rs`**
   - HTTP server for /metrics endpoint
   - Prometheus text format encoder

3. **Modify: `src/storage/local.rs`**
   - Add metrics recording to Storage operations
   - Instrument get/put/delete with timing

4. **Modify: `src/storage/redis.rs`**
   - Add metrics recording to Redis operations

5. **Modify: `Cargo.toml`**
   - Add metrics dependencies (prometheus, hyper)

6. **Create: Tests**
   - Metrics collection tests
   - HTTP endpoint tests

### Technical Details

**Dependencies:**
```toml
prometheus = "0.13"
hyper = { version = "0.14", features = ["server", "http1"] }
tokio = { version = "1.36", features = ["rt-multi-thread", "net"] }
```

**Key Metrics:**
- **Gauges:**
  - `process_cpu_usage` - CPU usage percentage
  - `process_memory_bytes` - Memory usage in bytes
  - `agent_peers_connected` - Number of connected peers

- **Counters:**
  - `messages_received_total` - Total messages received
  - `storage_operations_total{operation, backend}` - Storage ops by type

- **Histograms:**
  - `message_processing_duration_seconds` - Message processing time
  - `storage_operation_duration_seconds{operation, backend}` - Storage op time

**Configuration:**
```yaml
metrics:
  enabled: true
  port: 8080
  path: /metrics
```

### Testing Strategy

1. **Unit Tests:**
   - Metric registration and recording
   - Counter increments
   - Gauge updates
   - Histogram observations

2. **Integration Tests:**
   - HTTP endpoint responds correctly
   - Prometheus format validation
   - Metrics updated during operations

3. **Test Coverage Target:** 90%

## Dependencies

- Story 1.3: Basic Local Storage Backend (done)
- Story 1.5: Redis Storage Backend (done)

## Tasks

### Task 1: Create Metrics Module [x]
- [x] Subtask 1.1: Define MetricsCollector struct
- [x] Subtask 1.2: Define MetricsConfig struct
- [x] Subtask 1.3: Register Prometheus metrics (gauges, counters, histograms)
- [x] Subtask 1.4: Implement metric recording methods

### Task 2: Create HTTP Endpoint [x]
- [x] Subtask 2.1: Create MetricsServer struct
- [x] Subtask 2.2: Implement HTTP handler for /metrics endpoint
- [x] Subtask 2.3: Implement Prometheus text format encoder
- [x] Subtask 2.4: Add configuration option to enable/disable endpoint

### Task 3: Write Unit Tests [x]
- [x] Subtask 3.1: Test metric registration
- [x] Subtask 3.2: Test counter increments with assertions
- [x] Subtask 3.3: Test gauge updates with assertions
- [x] Subtask 3.4: Test histogram observations with assertions
- [x] Subtask 3.5: Test endpoint disable option

### Task 4: Integration with Storage Layer [~]
- [ ] Subtask 4.1: Add MetricsCollector to LocalStorageBackend
- [ ] Subtask 4.2: Instrument local storage operations with timing
- [ ] Subtask 4.3: Add MetricsCollector to RedisStorageBackend
- [ ] Subtask 4.4: Instrument Redis operations with timing
- [ ] Subtask 4.5: Write integration tests for storage metrics

### Task 5: Integration with Message Processing [~]
- [ ] Subtask 5.1: Identify message processing entry point
- [ ] Subtask 5.2: Add MetricsCollector to message handler
- [ ] Subtask 5.3: Record message receipt and processing duration
- [ ] Subtask 5.4: Write integration tests for message metrics

### Task 6: Documentation [ ]
- [ ] Subtask 6.1: Document metrics in docs/metrics.md
- [ ] Subtask 6.2: Add configuration examples
- [ ] Subtask 6.3: Document integration points

## Notes

- Use lazy_static for global metrics registry
- Metrics should have minimal performance impact (<1% overhead)
- Follow Prometheus naming conventions
- Document metrics in docs/metrics.md

## Dev Agent Record

### File List
- CREATED: src/metrics/mod.rs
- CREATED: src/metrics/prometheus_exporter.rs
- MODIFIED: src/lib.rs (added metrics module)
- MODIFIED: Cargo.toml (added prometheus, hyper, lazy_static dependencies)

### Change Log
- [2026-01-03 Initial] Created metrics module with Prometheus exporter
- [2026-01-03 Initial] Defined standard metrics (CPU, memory, peers, messages, storage)
- [2026-01-03 Initial] Implemented HTTP server for /metrics endpoint
- [2026-01-03 Initial] Added unit tests for metric recording
- [2026-01-03 Rework] Added proper test assertions to validate metric increments
- [2026-01-03 Rework] Fixed unused field warnings (MetricsServer, MetricsCollector)
- [2026-01-03 Rework] Added task list structure for tracking implementation progress
- [2026-01-03 Rework] Increased test coverage with comprehensive assertions

### Implementation Decisions
- Used lazy_static for global metrics registry (per Notes)
- Feature-gated prometheus dependencies (metrics-prometheus feature)
- Provided no-op implementations when feature disabled
- Added assertions to tests to validate actual metric values using prometheus::gather()
- Removed unused config field from MetricsServer (not needed for static implementation)
- Kept config field in MetricsCollector with #[allow(dead_code)] for future extensibility

### Test Coverage Improvements
- test_record_storage_operation: Now validates counter increments by 2 and histogram existence
- test_record_message_metrics: Now validates counter increments and histogram population
- test_update_gauges: Now validates exact gauge values (peers=5, cpu=25.5, memory=104857600)
- All metrics tests now use prometheus::gather() to verify actual metric values
- Test suite: 146 unit tests + 28 integration tests = 174 total tests passing

---

## 🔥 Code Review Issues - Remediation Required

**Review Date:** 2026-01-03  
**Reviewer:** Amelia (Adversarial Code Review Agent)  
**Status:** NEEDS REWORK - 2 Critical, 5 Medium, 1 Low issues found

### 🔴 Critical Issues

**ISSUE-CR-1.6-001: AC3 Storage Metrics Integration Missing** ⚠️ CRITICAL
```yaml
Severity: HIGH
Acceptance Criteria: AC3 - Storage Operation Metrics
Problem: |
  - Metrics functions exist in prometheus_exporter.rs
  - record_storage_operation() never called by storage layer
  - src/storage/local.rs NOT modified (per git diff)
  - src/storage/redis.rs NOT modified (per git diff)
  - AC3 marked complete but functionally NOT implemented
Required Action: |
  1. Add MetricsCollector field to LocalStorageBackend struct
  2. Add MetricsCollector field to RedisStorageBackend struct  
  3. Instrument get/put/delete operations with timing:
     - Start timer before operation
     - Record duration after operation completes
     - Call collector.record_storage_operation(op, backend, duration_ms)
  4. Write integration tests verifying metrics incremented during storage ops
Test Validation:
  - Create storage backend → perform get/put/delete → verify metrics counters increased
  - Check histogram buckets contain duration observations
Blocking: YES - AC3 cannot be considered complete
```

**ISSUE-CR-1.6-002: AC2 Message Metrics Integration Missing** ⚠️ CRITICAL
```yaml
Severity: HIGH
Acceptance Criteria: AC2 - Message Processing Metrics
Problem: |
  - record_message_received() function exists but never called
  - record_message_duration() function exists but never called
  - No integration with message processing layer
  - AC2 marked complete but functionally NOT implemented
Required Action: |
  1. Identify message processing entry point (likely in network/ or application/)
  2. Add MetricsCollector to message handler
  3. Call record_message_received() when message arrives
  4. Wrap message processing with timing, call record_message_duration()
  5. Write integration tests verifying metrics during message processing
Test Validation:
  - Process test message → verify messages_received_total incremented
  - Verify message_processing_duration_seconds histogram updated
Blocking: YES - AC2 cannot be considered complete
```

### 🟡 Medium Issues

**ISSUE-CR-1.6-003: Test Assertions Missing** ⚠️ MEDIUM
```yaml
Severity: MEDIUM
Problem: |
  Test functions exist but have no assertions:
  - test_record_storage_operation: comment says "Just verify it doesn't panic"
  - test_record_message_metrics: no assertions
  - test_update_gauges: no assertions
  Tests call functions but don't validate behavior
Required Action: |
  1. Add assertions to verify counter increments:
     before = get_counter_value()
     call_function()
     after = get_counter_value()
     assert!(after > before)
  2. Use prometheus::gather() to read actual metric values
  3. Verify histogram buckets populated
  4. Check gauge values actually set correctly
File: src/metrics/prometheus_exporter.rs lines 234-261
```

**ISSUE-CR-1.6-004: Story Structure Missing Task List** ⚠️ MEDIUM
```yaml
Severity: MEDIUM
Problem: |
  Story has "Implementation Plan" but no executable Tasks/Subtasks checklist
  Cannot track what was done vs planned
  No [ ] / [x] checkbox tracking
Required Action: |
  Add proper task structure after Implementation Plan:
  
  ## Tasks
  
  ### Task 1: Create Metrics Module [x]
  - [x] Subtask 1.1: Define MetricsCollector struct
  - [x] Subtask 1.2: Define MetricsConfig struct
  - [x] Subtask 1.3: Register Prometheus metrics (gauges, counters, histograms)
  
  ### Task 2: Integrate Storage Metrics [ ]
  - [ ] Subtask 2.1: Add MetricsCollector to LocalStorageBackend
  - [ ] Subtask 2.2: Instrument local storage operations
  - [ ] Subtask 2.3: Add MetricsCollector to RedisStorageBackend
  - [ ] Subtask 2.4: Instrument Redis operations
  
  ### Task 3: Integrate Message Metrics [ ]
  - [ ] Subtask 3.1: Add MetricsCollector to message handler
  - [ ] Subtask 3.2: Record message receipt and processing duration
  
  ### Task 4: HTTP Endpoint [ ]
  - [ ] Subtask 4.1: Create MetricsServer with HTTP handler
  - [ ] Subtask 4.2: Test endpoint responds with Prometheus format
  
  ### Task 5: Tests [ ]
  - [ ] Subtask 5.1: Write unit tests with assertions
  - [ ] Subtask 5.2: Write integration tests for storage metrics
  - [ ] Subtask 5.3: Write integration tests for message metrics
```

**ISSUE-CR-1.6-005: Unused Field Warning** ⚠️ MEDIUM
```yaml
Severity: MEDIUM
Problem: |
  Compiler warning in production code:
  warning: field `config` is never read
    --> src/metrics/prometheus_exporter.rs:142:5
  MetricsServer.config field declared but never used
Required Action: |
  Option A: Use config field (e.g., log config on server start)
  Option B: Remove field if truly not needed
  Option C: Add #[allow(dead_code)] with comment explaining future use
File: src/metrics/prometheus_exporter.rs line 142
```

**ISSUE-CR-1.6-006: Story Claims Files Modified That Weren't** ⚠️ MEDIUM
```yaml
Severity: MEDIUM
Problem: |
  Implementation Plan says:
  - "Modify: src/storage/local.rs - Add metrics recording"
  - "Modify: src/storage/redis.rs - Add metrics recording"
  Git commit 6f92a60 does NOT include these files
Required Action: |
  Either:
  A. Actually modify these files (ties to ISSUE-CR-1.6-001)
  B. Update Implementation Plan to reflect what was actually done
```

**ISSUE-CR-1.6-007: Integration Tests Missing** ⚠️ MEDIUM
```yaml
Severity: MEDIUM
Problem: |
  Testing Strategy says "Integration Tests" but none exist
  - No HTTP endpoint integration test
  - No end-to-end metrics collection test
  - No validation that metrics endpoint serves correct format
Required Action: |
  Create tests/integration_metrics.rs:
  1. Start MetricsServer
  2. Perform operations that should increment metrics
  3. GET http://localhost:8080/metrics
  4. Parse response, verify counters increased
  5. Verify Content-Type: text/plain; version=0.0.4
```

### 🟢 Low Issues

**ISSUE-CR-1.6-008: Missing Dev Agent Record Section** ⚠️ LOW
```yaml
Severity: LOW
Problem: |
  Story missing "## Dev Agent Record" section
  Cannot track implementation decisions, changes made, or files modified
Required Action: |
  Add section after Notes:
  
  ## Dev Agent Record
  
  ### File List
  - CREATED: src/metrics/mod.rs
  - CREATED: src/metrics/prometheus_exporter.rs
  - MODIFIED: src/lib.rs (added metrics module)
  - MODIFIED: Cargo.toml (added prometheus, hyper, lazy_static dependencies)
  
  ### Change Log
  - [timestamp] Created metrics module with Prometheus exporter
  - [timestamp] Defined standard metrics (CPU, memory, peers, messages, storage)
  - [timestamp] Implemented HTTP server for /metrics endpoint
  - [timestamp] Added unit tests for metric recording
  
  ### Implementation Decisions
  - Used lazy_static for global metrics registry (per Notes)
  - Feature-gated prometheus dependencies (metrics-prometheus feature)
  - Provided no-op implementations when feature disabled
```

---

## Remediation Plan

**Priority 1 (Blocking):**
1. Integrate storage metrics (ISSUE-CR-1.6-001) - AC3 incomplete
2. Integrate message metrics (ISSUE-CR-1.6-002) - AC2 incomplete
3. Add test assertions (ISSUE-CR-1.6-003) - Tests not validating

**Priority 2 (Should Fix):**
4. Add task list structure (ISSUE-CR-1.6-004)
5. Fix unused field warning (ISSUE-CR-1.6-005)
6. Update story accuracy (ISSUE-CR-1.6-006)
7. Add integration tests (ISSUE-CR-1.6-007)

**Priority 3 (Nice to Have):**
8. Add Dev Agent Record (ISSUE-CR-1.6-008)

**Recommendation:** Create follow-up story "1.6.1: Complete Metrics Integration" to address Critical issues, or reopen this story as "needs-rework".

---

## ✅ Minor Issues Remediation - Completed

**Remediation Date:** 2026-01-03  
**Developer:** Amelia (Dev Agent)  
**Status:** PARTIAL FIX - Minor Issues 1-3 Resolved

### Issues Addressed

**✅ ISSUE-CR-1.6-003: Test Assertions Missing** - FIXED
- Added comprehensive assertions to test_record_storage_operation
  - Validates counter increments by exactly 2
  - Verifies histogram metric exists
- Added assertions to test_record_message_metrics
  - Validates counter increments
  - Verifies histogram population
- Added assertions to test_update_gauges
  - Validates exact gauge values for peers, CPU, and memory
- All tests now use prometheus::gather() to verify actual metric values

**✅ ISSUE-CR-1.6-004: Story Structure Missing Task List** - FIXED
- Added complete Tasks section with 6 main tasks
- Added 23 subtasks with proper checkbox tracking
- Marked completed tasks [x] and incomplete tasks [ ]
- Tasks 1-3 complete (module creation, HTTP endpoint, unit tests)
- Tasks 4-5 incomplete (storage/message integration) - aligns with critical issues
- Task 6 incomplete (documentation)

**✅ ISSUE-CR-1.6-005: Unused Field Warning** - FIXED
- Removed unused `config` field from MetricsServer struct
- Added #[allow(dead_code)] with comment to MetricsCollector.config field
- Documented reason: "Config may be used for future runtime configuration"
- All compiler warnings resolved

**✅ ISSUE-CR-1.6-008: Missing Dev Agent Record Section** - FIXED
- Added comprehensive Dev Agent Record section
- Documented all created and modified files
- Added detailed change log with timestamps
- Documented implementation decisions
- Documented test coverage improvements

### Test Results
```
Unit Tests: 146 passed (0 failed, 2 ignored)
Integration Tests: 28 passed
Total: 174 tests passing
Coverage: Report generated successfully (lcov.info)
Build: Clean with no warnings
```

### Files Modified
- `src/metrics/prometheus_exporter.rs` - Added test assertions, fixed unused fields
- `_bmad-output/implementation-artifacts/1-6-prometheus-metrics-collection.md` - Added tasks, Dev Agent Record

### Remaining Work
**Critical Issues Still Outstanding:**
- ISSUE-CR-1.6-001: Storage metrics integration (AC3 incomplete)
- ISSUE-CR-1.6-002: Message metrics integration (AC2 incomplete)

**Medium Issues Still Outstanding:**
- ISSUE-CR-1.6-006: Update story accuracy for unmodified files
- ISSUE-CR-1.6-007: Add integration tests for metrics collection

**Next Step:** Address critical issues 1-2 in separate remediation or create Story 1.6.1
