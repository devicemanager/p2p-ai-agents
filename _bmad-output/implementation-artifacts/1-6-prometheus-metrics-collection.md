# Story 1.6: Prometheus Metrics Collection

Status: ready-for-dev

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

## Notes

- Use lazy_static for global metrics registry
- Metrics should have minimal performance impact (<1% overhead)
- Follow Prometheus naming conventions
- Document metrics in docs/metrics.md
