# Story 1.7: Structured JSON Logging

Status: done

## Story

As a node operator,
I want my agent to emit structured JSON logs with correlation IDs,
So that I can aggregate and search logs effectively.

## Acceptance Criteria

**AC1: JSON Log Format with Required Fields**
```
Given: the agent is configured with JSON logging
When: any log is emitted
Then: the log is formatted as JSON
And: includes fields: timestamp, level, message, target, correlation_id
And: the output is written to stdout
```

**AC2: Correlation ID Generation and Propagation**
```
Given: a new request or operation starts
When: creating a new span or context
Then: a correlation_id (UUID v4) is generated
And: propagated to all logs within that context
```

**AC3: Error Logging with Context**
```
Given: an error occurs during key loading
When: logging the error
Then: the log includes level: "ERROR"
And: includes error_type and error_message fields
And: includes the correlation_id for tracing
```

**AC4: Log Level Filtering**
```
Given: log level is set to INFO
When: a DEBUG log is emitted
Then: the log is not written to output
And: only INFO, WARN, and ERROR logs are emitted
```

## Implementation Plan

### Files to Create/Modify

1. **Modify: `Cargo.toml`**
   - Add tracing-subscriber with json feature
   - Already has tracing (0.1) and tracing-subscriber (0.3) - add json feature

2. **Create: `src/core/logging.rs`**
   - LoggingConfig struct
   - Initialize JSON logging with correlation IDs
   - Configure log level filtering

3. **Modify: `src/core/mod.rs`**
   - Add logging module export

4. **Modify: `src/lib.rs`**
   - Initialize logging system on startup

5. **Create: `src/core/correlation.rs`**
   - CorrelationId type (UUID v4 wrapper)
   - Tracing span integration for propagation

6. **Create: Tests**
   - JSON format validation
   - Correlation ID propagation
   - Log level filtering
   - Error logging with context

### Technical Details

**Dependencies:**
```toml
tracing = "0.1"  # Already present
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }  # Add json feature
uuid = { version = "1.7", features = ["v4", "serde"] }  # Already present
```

**Logging Architecture (from arch-005):**
- **Format**: Structured JSON logs with correlation IDs
- **Transport**: stdout for container/systemd capture
- **Correlation**: OpenTelemetry-compatible trace context propagation
- **Integration**: Ready for distributed tracing (OpenTelemetry Phase 3)

**JSON Log Structure:**
```json
{
  "timestamp": "2026-01-04T12:00:00.123456Z",
  "level": "INFO",
  "message": "Agent started successfully",
  "target": "p2p_ai_agents::agent",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "peer_id": "12D3KooWABC...",
  "span": {
    "name": "agent_startup",
    "operation": "initialize"
  }
}
```

**Error Log Structure:**
```json
{
  "timestamp": "2026-01-04T12:00:01.456789Z",
  "level": "ERROR",
  "message": "Failed to load private key",
  "target": "p2p_ai_agents::agent::identity",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "error_type": "KeyLoadError",
  "error_message": "File not found: /etc/p2p-agents/keys/private.key",
  "error_source": "std::io::Error"
}
```

**Configuration:**
```yaml
logging:
  format: json  # Options: json, pretty, compact
  level: INFO   # Options: TRACE, DEBUG, INFO, WARN, ERROR
  target_filters:
    - "p2p_ai_agents=DEBUG"
    - "libp2p=INFO"
  output: stdout  # stdout or file path
```

**Correlation ID Propagation Pattern:**
```rust
use tracing::{info, error, instrument};
use uuid::Uuid;

#[instrument(fields(correlation_id = %Uuid::new_v4()))]
async fn handle_request() {
    info!("Processing request");
    // All logs in this span inherit correlation_id
    
    match load_data().await {
        Ok(data) => info!("Data loaded successfully"),
        Err(e) => error!(
            error_type = std::any::type_name_of_val(&e),
            error_message = %e,
            "Failed to load data"
        ),
    }
}
```

### Testing Strategy

1. **Unit Tests:**
   - Logging initialization with different configs
   - JSON format validation
   - Log level filtering behavior
   - Correlation ID generation

2. **Integration Tests:**
   - Capture stdout during test operations
   - Parse JSON logs and validate structure
   - Verify correlation ID propagation across function calls
   - Test error logging with structured fields

3. **Test Coverage Target:** 90%

## Dependencies

- Story 1.1: Agent Identity Generation (done) - provides peer_id for logs
- Story 1.6: Prometheus Metrics Collection (done) - parallel observability feature

## Dev Notes

### Existing Logging Context

**Current State:**
- Project already has `tracing` (0.1) and `tracing-subscriber` (0.3) dependencies
- Basic tracing infrastructure likely in place
- Need to add JSON formatting and correlation ID support

**Architecture Alignment (arch-005: Distributed Tracing):**
- **Current (Story 1.7):** Structured JSON logging with correlation IDs
- **Phase 2:** Correlation IDs compatible with OpenTelemetry trace context
- **Phase 3:** Full OpenTelemetry integration with distributed tracing

**Key Design Decisions:**
1. **Use tracing-subscriber's JSON formatter** - Built-in, mature, OpenTelemetry-compatible
2. **Correlation ID as span field** - Natural propagation using tracing spans
3. **ENV_FILTER for dynamic configuration** - Runtime log level adjustment
4. **stdout output** - Container-friendly, systemd journal compatible

**Previous Story Learnings:**
- From 1.6 (Metrics): Feature-gate optional dependencies (metrics-prometheus feature)
- From 1.3 (Storage): Configuration-driven behavior with sensible defaults
- From 1.1 (Identity): Error handling with structured error types (thiserror)

### Project Structure Notes

**Alignment with Unified Structure:**
- Logging is a core infrastructure concern → `src/core/logging.rs`
- Correlation handling is a core utility → `src/core/correlation.rs`
- Follows established pattern of core/ for cross-cutting concerns
- Consistent with arch-005's observability strategy

**Integration Points:**
- `src/lib.rs` - Initialize logging on crate load
- `src/agent/` - Agent operations will use correlation IDs
- `src/network/` - Network operations will propagate correlation
- `src/storage/` - Storage operations will include correlation context

### Testing Standards Summary

**Coverage Requirements:**
- Overall: 90% minimum
- Critical paths: 95% (error logging, correlation propagation)
- Security-critical: 100% (log sanitization if handling sensitive data)

**Test Organization:**
- Unit tests: In `src/core/logging.rs` and `src/core/correlation.rs`
- Integration tests: In `tests/logging_integration.rs`
- Capture and parse JSON output for validation

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#arch-005-monitoring-observability]
- [Source: _bmad-output/planning-artifacts/epics.md#story-1-7-structured-json-logging]
- [Source: Cargo.toml - tracing dependencies already present]
- [Source: Story 1.6 - Metrics integration patterns for observability]

## Tasks

### Task 1: Configure JSON Logging Dependencies [X]
- [X] Subtask 1.1: Add "json" feature to tracing-subscriber in Cargo.toml
- [X] Subtask 1.2: Verify uuid crate already has v4 and serde features
- [X] Subtask 1.3: Run `cargo check` to validate dependencies

### Task 2: Create Correlation ID Module [X]
- [X] Subtask 2.1: Create src/core/correlation.rs
- [X] Subtask 2.2: Define CorrelationId newtype wrapper around Uuid
- [X] Subtask 2.3: Implement Display, Debug, Serialize traits
- [X] Subtask 2.4: Add helper function to generate new correlation IDs
- [X] Subtask 2.5: Write unit tests for CorrelationId type

### Task 3: Create Logging Configuration Module [X]
- [X] Subtask 3.1: Create src/core/logging.rs
- [X] Subtask 3.2: Define LoggingConfig struct (format, level, target_filters)
- [X] Subtask 3.3: Implement logging initialization with JSON formatter
- [X] Subtask 3.4: Configure EnvFilter for log level filtering
- [X] Subtask 3.5: Add error handling for initialization failures
- [X] Subtask 3.6: Write unit tests for configuration parsing

### Task 4: Integrate Logging into Core Module [X]
- [X] Subtask 4.1: Add `pub mod logging;` to src/core/mod.rs
- [X] Subtask 4.2: Add `pub mod correlation;` to src/core/mod.rs
- [X] Subtask 4.3: Re-export key types (LoggingConfig, CorrelationId)
- [X] Subtask 4.4: Update core module documentation

### Task 5: Initialize Logging on Startup [X]
- [X] Subtask 5.1: Modify src/lib.rs to call logging initialization (added to prelude)
- [X] Subtask 5.2: Add example configuration in config/agent.example.yaml
- [X] Subtask 5.3: Test logging initialization with different log levels
- [X] Subtask 5.4: Verify logs output to stdout in JSON format

### Task 6: Create Integration Tests [X]
- [X] Subtask 6.1: Create tests/logging_integration.rs
- [X] Subtask 6.2: Test AC1: Capture and parse JSON log output
- [X] Subtask 6.3: Test AC2: Verify correlation ID propagation in nested spans
- [X] Subtask 6.4: Test AC3: Validate error logging with structured fields
- [X] Subtask 6.5: Test AC4: Verify log level filtering (DEBUG filtered when level=INFO)
- [X] Subtask 6.6: Test multi-threaded correlation ID isolation

### Task 7: Update Example Spans in Existing Code [X]
- [X] Subtask 7.1: Add example instrumented function in agent/identity.rs
- [X] Subtask 7.2: Add correlation_id field to key error logging
- [X] Subtask 7.3: Test error logging in agent identity generation flow
- [X] Subtask 7.4: Verify JSON logs include all required fields

### Task 8: Documentation [X]
- [X] Subtask 8.1: Document logging configuration in docs/core/logging.md
- [X] Subtask 8.2: Add logging examples to example/structured_logging.rs
- [X] Subtask 8.3: Document correlation ID usage patterns
- [X] Subtask 8.4: Update README.md with logging setup instructions

## Notes

- Use `tracing::instrument` macro for automatic correlation ID propagation
- JSON formatter automatically includes timestamp, level, target
- Correlation IDs should be generated once per top-level operation
- Nested spans inherit correlation_id from parent span
- Consider adding `RUST_LOG` environment variable support for runtime configuration
- Ensure no sensitive data (keys, passwords) appears in logs

## Dev Agent Record

### Agent Model Used

Claude 3.5 Sonnet (via GitHub Copilot CLI)

### Debug Log References

N/A - Implementation was straightforward with no major debugging required.

### Completion Notes List

1. **Structured JSON Logging Implemented**: Complete implementation of JSON-formatted logging with correlation IDs
2. **Correlation ID Module**: Created `src/core/correlation.rs` with UUID v4-based correlation ID type
3. **Logging Configuration Module**: Created `src/core/logging.rs` with flexible configuration and initialization
4. **Test Coverage**: Comprehensive integration tests in `tests/logging_integration.rs` covering all acceptance criteria
5. **Documentation**: Created detailed guide at `docs/core/logging.md` with examples and best practices
6. **Example Application**: Created `examples/structured_logging.rs` demonstrating real-world usage
7. **Enhanced Agent Identity**: Added instrumentation and structured error logging to `src/agent/identity.rs`
8. **Configuration Update**: Updated `config/agent.example.yaml` with logging configuration
9. **All Acceptance Criteria Met**:
   - AC1: JSON format with timestamp, level, message, target, correlation_id ✅
   - AC2: UUID v4 correlation IDs generated and propagated via spans ✅
   - AC3: Structured error logging with error_type and error_message ✅
   - AC4: Log level filtering working correctly ✅

### File List

**Created Files:**
- `src/core/correlation.rs` - Correlation ID type and utilities
- `src/core/logging.rs` - Logging configuration and initialization
- `tests/logging_integration.rs` - Comprehensive integration tests
- `examples/structured_logging.rs` - Example demonstrating logging usage
- `docs/core/logging.md` - Complete logging documentation

**Modified Files:**
- `Cargo.toml` - Added "json" feature to tracing-subscriber, added structured_logging example
- `src/core/mod.rs` - Added correlation and logging modules, exported types
- `src/lib.rs` - Added logging types to prelude
- `src/agent/identity.rs` - Added instrumentation and structured error logging
- `config/agent.example.yaml` - Updated logging configuration section
- `README.md` - Added logging documentation link and feature mention
