# Error Types

This document describes the error types used in the P2P network implementation.

## Error Definitions
- `NetworkError`: General network errors (already implemented in Rust).
- `DiscoveryError`: Errors related to peer discovery (already implemented).
- `TransportError`: Errors related to network transport (already implemented).
- `TaskError`: Task protocol-specific errors (see Protocol-Specific Error Types below).
- `ResourceError`: Resource protocol-specific errors (see Protocol-Specific Error Types below).
- `HealthError`: Health monitoring protocol errors (see Protocol-Specific Error Types below).

## Error Handling
- All error types implement `std::error::Error` via `thiserror`.
- Use `Result<T, E>` for all fallible operations.
- Use error enums for domain-specific errors.

## Recovery Strategies
- Log errors and attempt retries where appropriate.
- For fatal errors, shut down the affected subsystem cleanly.
- For recoverable errors, provide feedback to the user or system.

## Examples
```rust
use p2p_ai_agents::network::{NetworkError, DiscoveryError, TransportError};

fn handle_error(err: NetworkError) {
    match err {
        NetworkError::NotInitialized => println!("Network not initialized"),
        NetworkError::Transport(e) => println!("Transport error: {}", e),
        _ => println!("Other error: {:?}", err),
    }
}
```

## Protocol-Specific Error Types

### Task Protocol Errors
```rust
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },
    #[error("Task execution failed: {reason}")]
    ExecutionFailed { reason: String },
    #[error("Task timeout after {duration}ms")]
    Timeout { duration: u64 },
    #[error("Invalid task payload: {details}")]
    InvalidPayload { details: String },
}
```

### Discovery Protocol Errors
```rust
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Peer discovery timeout")]
    DiscoveryTimeout,
    #[error("Invalid peer response: {reason}")]
    InvalidResponse { reason: String },
    #[error("Bootstrap node unreachable: {address}")]
    BootstrapUnreachable { address: String },
}
```

### Resource Protocol Errors
```rust
#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Insufficient resources: requested {requested}, available {available}")]
    InsufficientResources { requested: u64, available: u64 },
    #[error("Resource type not supported: {resource_type}")]
    UnsupportedType { resource_type: String },
    #[error("Resource allocation failed: {reason}")]
    AllocationFailed { reason: String },
}
```

### Error Recovery Strategies
```rust
impl NetworkError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            NetworkError::ConnectionFailed { .. } => true,
            NetworkError::Timeout { .. } => true,
            NetworkError::InvalidMessage { .. } => false,
            NetworkError::AuthenticationFailed { .. } => false,
        }
    }
    
    pub fn retry_delay(&self) -> Duration {
        match self {
            NetworkError::ConnectionFailed { .. } => Duration::from_secs(5),
            NetworkError::Timeout { .. } => Duration::from_secs(2),
            _ => Duration::from_secs(1),
        }
    }
}
```

### Related Documentation
- [Network Manager](../network-manager.md) - Error handling in network operations
- [Protocol Types](protocol-types.md) - Protocol definitions
- [Message Types](message-types.md) - Message processing errors
