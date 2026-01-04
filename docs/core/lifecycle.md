# Agent Lifecycle Management

## Overview

The Agent Lifecycle Management system provides robust startup, shutdown, and crash recovery capabilities for P2P AI agents. It ensures data integrity, graceful handling of signals, and proper state persistence across restarts.

## Features

### 1. Graceful Startup
- Configuration loading and validation
- Storage backend initialization
- Identity loading or generation
- Service initialization
- State recovery from previous runs
- Comprehensive startup logging

### 2. Signal Handling
- SIGTERM handling (Unix/Linux)
- SIGINT handling (Ctrl+C)
- Graceful shutdown initiation
- Automatic signal registration

### 3. Graceful Shutdown
- Stops accepting new connections/tasks
- Completes in-flight operations (with timeout)
- Persists current state to storage
- Stops all services gracefully
- Comprehensive shutdown logging
- Exit code 0 on successful shutdown

### 4. State Persistence
- Automatic state persistence during operation
- State includes:
  - Peer ID
  - Last started timestamp
  - Last stopped timestamp (if applicable)
  - Task processing statistics
  - Shutdown history (clean vs unclean)

### 5. Crash Recovery
- Detects unclean shutdowns on restart
- Logs recovery information
- Tracks crash statistics
- Recovers operational state

## Architecture

```
┌─────────────────────────────────────────┐
│        LifecycleManager                  │
├─────────────────────────────────────────┤
│                                          │
│  ┌──────────┐  ┌──────────┐  ┌────────┐│
│  │ Startup  │→ │ Running  │→ │Shutdown││
│  └──────────┘  └──────────┘  └────────┘│
│        ↓             ↓            ↓     │
│  ┌──────────────────────────────────┐  │
│  │     State Persistence            │  │
│  └──────────────────────────────────┘  │
│                                          │
│  ┌──────────────────────────────────┐  │
│  │     Signal Handling              │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## Usage

### Basic Usage

```rust
use p2p_ai_agents::prelude::*;
use p2p_ai_agents::application::lifecycle::LifecycleManager;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_default_logging()?;

    // Create application
    let app = Application::new();
    
    // Create lifecycle manager
    let lifecycle = Arc::new(
        LifecycleManager::new(app)
            .with_shutdown_timeout(Duration::from_secs(30))
    );

    // Startup
    lifecycle.startup().await?;

    // Register signal handlers
    let lifecycle_signals = lifecycle.clone();
    tokio::spawn(async move {
        let _ = lifecycle_signals.handle_signals().await;
    });

    // Application logic here...

    // Graceful shutdown
    lifecycle.shutdown().await?;

    Ok(())
}
```

### Custom Shutdown Timeout

```rust
let lifecycle = LifecycleManager::new(app)
    .with_shutdown_timeout(Duration::from_secs(60));
```

### Checking Lifecycle State

```rust
if let Some(state) = lifecycle.state().await {
    println!("Peer ID: {}", state.peer_id);
    println!("Started: {}", state.last_started);
    println!("Tasks Processed: {}", state.tasks_processed);
    println!("Successful Shutdowns: {}", state.successful_shutdowns);
    println!("Unclean Shutdowns: {}", state.unclean_shutdowns);
}
```

## State File Format

The lifecycle state is persisted as JSON in `data/lifecycle_state.json`:

```json
{
  "last_started": "2026-01-04T12:00:00Z",
  "last_stopped": "2026-01-04T13:00:00Z",
  "peer_id": "12D3KooWExample...",
  "tasks_processed": 1337,
  "successful_shutdowns": 42,
  "unclean_shutdowns": 3
}
```

## Signal Handling

### Unix/Linux
- **SIGTERM**: Graceful shutdown (common in containers/systemd)
- **SIGINT**: Graceful shutdown (Ctrl+C in terminal)

### Windows
- **Ctrl+C**: Graceful shutdown

## Shutdown Process

1. **Stop accepting new work**
   - Mark application as stopping
   - Reject new connections/tasks

2. **Complete in-flight operations** (with timeout)
   - Wait for current tasks to finish
   - Flush pending messages
   - Close connections gracefully
   - Force stop after timeout

3. **Persist state**
   - Save current operational state
   - Record shutdown timestamp
   - Update shutdown statistics

4. **Stop services**
   - Stop network services
   - Stop storage services
   - Stop monitoring services

5. **Exit cleanly**
   - Log completion
   - Exit with code 0

## Crash Recovery

On startup, the lifecycle manager:

1. Checks for existing state file
2. If `last_stopped` is `null`:
   - Logs "Detected unclean shutdown"
   - Increments `unclean_shutdowns` counter
   - Logs recovery information
3. Proceeds with normal startup

## Configuration

### Environment Variables

```bash
# Shutdown timeout in seconds
SHUTDOWN_TIMEOUT=30

# Data directory for state persistence
DATA_DIR=./data
```

### Config File (config/agent.yaml)

```yaml
lifecycle:
  shutdown_timeout_secs: 30
  state_persistence:
    enabled: true
    path: "data/lifecycle_state.json"
  crash_recovery:
    enabled: true
    log_recovery: true
```

## Testing

### Running Tests

```bash
# Run lifecycle integration tests
cargo test --test lifecycle_integration

# Run with output
cargo test --test lifecycle_integration -- --nocapture

# Run specific test
cargo test test_agent_graceful_shutdown
```

### Test Coverage

The lifecycle management system has comprehensive test coverage:

- ✅ Graceful startup
- ✅ Graceful shutdown
- ✅ In-flight operations timeout
- ✅ State persistence and recovery
- ✅ Crash detection
- ✅ Double shutdown safety
- ✅ Application state transitions
- ✅ Custom shutdown timeout
- ✅ Multiple startup/shutdown cycles

## Best Practices

### 1. Always Use Lifecycle Manager

```rust
// ❌ Don't do this
let app = Application::new();
app.initialize().await?;
app.start().await?;

// ✅ Do this instead
let lifecycle = LifecycleManager::new(app);
lifecycle.startup().await?;
```

### 2. Set Appropriate Timeouts

```rust
// For quick operations
let lifecycle = LifecycleManager::new(app)
    .with_shutdown_timeout(Duration::from_secs(10));

// For long-running tasks
let lifecycle = LifecycleManager::new(app)
    .with_shutdown_timeout(Duration::from_secs(300));
```

### 3. Always Register Signal Handlers

```rust
let lifecycle = Arc::new(LifecycleManager::new(app));
let lifecycle_signals = lifecycle.clone();

tokio::spawn(async move {
    if let Err(e) = lifecycle_signals.handle_signals().await {
        eprintln!("Signal handler error: {}", e);
    }
});
```

### 4. Check State After Recovery

```rust
lifecycle.startup().await?;

if let Some(state) = lifecycle.state().await {
    if state.unclean_shutdowns > 0 {
        warn!("Agent has {} unclean shutdowns", state.unclean_shutdowns);
        // Consider additional recovery actions
    }
}
```

## Troubleshooting

### State File Not Found
- **Cause**: First run or state file deleted
- **Solution**: Normal, state will be created on startup

### Unclean Shutdown Detected
- **Cause**: Previous process crashed or was killed
- **Solution**: Check logs from previous run, fix crash cause

### Shutdown Timeout
- **Cause**: In-flight operations taking too long
- **Solution**: Increase timeout or investigate blocking operations

### Permission Denied on State File
- **Cause**: File system permissions
- **Solution**: Ensure write permissions on `data/` directory

## Metrics

The lifecycle manager exposes the following metrics:

- `agent_startup_duration_seconds` - Time taken to start up
- `agent_shutdown_duration_seconds` - Time taken to shut down
- `agent_successful_shutdowns_total` - Count of clean shutdowns
- `agent_unclean_shutdowns_total` - Count of crashes
- `agent_uptime_seconds` - Current agent uptime

## Future Enhancements

- [ ] Configurable state persistence backend (Redis, Postgres)
- [ ] Health check integration
- [ ] Automatic restart on failure
- [ ] State migration between versions
- [ ] Distributed state coordination
- [ ] Graceful rolling updates

## References

- Story 1.8: Agent Lifecycle Management
- [Application Module](../src/application/mod.rs)
- [Lifecycle Module](../src/application/lifecycle.rs)
- [Integration Tests](../tests/lifecycle_integration.rs)
- [Example](../examples/agent_lifecycle.rs)
