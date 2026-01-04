# Structured JSON Logging

## Overview

The P2P AI Agents system uses structured JSON logging with correlation IDs for distributed tracing and effective log aggregation. This approach enables tracking operations across the distributed system and facilitates debugging in production environments.

## Features

- **JSON Format**: All logs are emitted as structured JSON for easy parsing
- **Correlation IDs**: UUID v4 correlation IDs track operations across the system
- **Structured Fields**: Logs include timestamp, level, message, target, and correlation_id
- **Error Context**: Error logs include error_type and error_message fields
- **Log Level Filtering**: Dynamic log level configuration per target
- **Container-Friendly**: Outputs to stdout for container and systemd journal capture

## Configuration

### Basic Configuration

Configure logging in your `agent.yaml` file:

```yaml
logging:
  # Log format: json, pretty, or compact
  format: "json"
  
  # Log level: TRACE, DEBUG, INFO, WARN, ERROR
  level: "INFO"
  
  # Target-specific filters (optional)
  target_filters:
    - "p2p_ai_agents=DEBUG"
    - "libp2p=INFO"
  
  # Output destination: stdout or file path
  output: "stdout"
```

### Environment Variable Override

You can override the log level using the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
RUST_LOG=p2p_ai_agents=trace,libp2p=info cargo run
```

### Programmatic Configuration

```rust
use p2p_ai_agents::core::logging::{LoggingConfig, LogFormat, init_logging};

let config = LoggingConfig {
    format: LogFormat::Json,
    level: "DEBUG".to_string(),
    target_filters: vec!["p2p_ai_agents=TRACE".to_string()],
    output: "stdout".to_string(),
};

init_logging(&config)?;
```

## Usage

### Basic Logging

```rust
use tracing::{info, debug, warn, error};

info!("Application started");
debug!(config = ?config, "Configuration loaded");
warn!(count = retries, "Retry attempt");
error!(error = %err, "Operation failed");
```

### Correlation IDs

Correlation IDs enable tracking a request or operation through the entire system:

```rust
use p2p_ai_agents::core::CorrelationId;
use tracing::{info, instrument};

#[instrument(fields(correlation_id = %CorrelationId::new()))]
async fn handle_request(user_id: u32) {
    info!(user_id = user_id, "Processing request");
    // All logs in this span will include the correlation_id
}
```

### Structured Error Logging

```rust
use tracing::error;

match load_file().await {
    Ok(data) => info!("File loaded successfully"),
    Err(e) => error!(
        error_type = std::any::type_name_of_val(&e),
        error_message = %e,
        "Failed to load file"
    ),
}
```

### Nested Spans

Correlation IDs automatically propagate through nested spans:

```rust
#[instrument(fields(correlation_id = %CorrelationId::new()))]
async fn outer_function() {
    info!("Outer function");
    inner_function().await;  // Inherits correlation_id
}

#[instrument]
async fn inner_function() {
    info!("Inner function - correlation_id inherited");
}
```

## Log Format Examples

### JSON Log Entry

```json
{
  "timestamp": "2026-01-04T12:00:00.123456Z",
  "level": "INFO",
  "message": "Agent started successfully",
  "target": "p2p_ai_agents::agent",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "peer_id": "12D3KooWABC...",
  "span": {
    "name": "agent_startup"
  }
}
```

### Error Log Entry

```json
{
  "timestamp": "2026-01-04T12:00:01.456789Z",
  "level": "ERROR",
  "message": "Failed to load private key",
  "target": "p2p_ai_agents::agent::identity",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "error_type": "std::io::Error",
  "error_message": "No such file or directory",
  "path": "/etc/p2p-agents/keys/private.key",
  "span": {
    "name": "load_from_dir"
  }
}
```

## Log Levels

The system supports standard log levels:

- **TRACE**: Very detailed information, typically only enabled during development
- **DEBUG**: Detailed information useful for debugging
- **INFO**: General informational messages about system operation
- **WARN**: Warning messages about potential issues
- **ERROR**: Error messages about failures

## Best Practices

### 1. Use Correlation IDs for Top-Level Operations

Add correlation IDs to the entry point of operations:

```rust
#[instrument(fields(correlation_id = %CorrelationId::new()))]
async fn handle_network_message(msg: Message) {
    // Processing...
}
```

### 2. Add Context to Logs

Include relevant context in log entries:

```rust
info!(
    user_id = user_id,
    operation = "update_profile",
    "Processing user update"
);
```

### 3. Structure Error Logs

Always include error type and message for errors:

```rust
error!(
    error_type = std::any::type_name_of_val(&err),
    error_message = %err,
    "Operation failed"
);
```

### 4. Use Appropriate Log Levels

- Use `DEBUG` for detailed debugging information
- Use `INFO` for normal operational messages
- Use `WARN` for recoverable issues
- Use `ERROR` for failures requiring attention

### 5. Avoid Logging Sensitive Data

Never log:
- Private keys or passwords
- Personal identifiable information (PII)
- Authentication tokens
- Other sensitive credentials

## Integration with Monitoring

### Prometheus Integration

Logs can be ingested by log aggregation systems and correlated with metrics:

```rust
// Metrics are collected separately (Story 1.6)
metrics::counter!("errors_total", 1);

// Logs provide detailed context
error!(
    correlation_id = %correlation_id,
    error_type = "NetworkError",
    "Network connection failed"
);
```

### Log Aggregation

JSON logs can be easily ingested by:
- **ELK Stack** (Elasticsearch, Logstash, Kibana)
- **Grafana Loki**
- **CloudWatch Logs**
- **Datadog**
- **Splunk**

### Correlation with Traces

The correlation ID format is compatible with OpenTelemetry trace IDs, enabling future integration with distributed tracing systems.

## Performance Considerations

- Structured logging has minimal performance overhead
- JSON formatting is done asynchronously
- Log levels are checked before expensive operations
- Correlation IDs are generated once per operation

## Troubleshooting

### Logs Not Appearing

Check the log level configuration:

```bash
RUST_LOG=debug cargo run
```

### JSON Format Not Working

Ensure the `json` feature is enabled for `tracing-subscriber`:

```toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### Correlation IDs Missing

Make sure to use the `#[instrument]` macro with correlation_id field:

```rust
#[instrument(fields(correlation_id = %CorrelationId::new()))]
fn my_function() {
    // ...
}
```

## Examples

See the `examples/structured_logging.rs` file for a complete example demonstrating:
- Correlation ID generation
- Nested spans
- Error logging with context
- Multi-threaded operations

Run it with:
```bash
cargo run --example structured_logging
```

## References

- [tracing documentation](https://docs.rs/tracing/)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/)
- [OpenTelemetry Context Propagation](https://opentelemetry.io/docs/concepts/context-propagation/)
