# Protocol Types

This document describes the protocol types used in the P2P network implementation.

## Protocol Definitions
- `NetworkProtocol`: Represents a network protocol (stub, to be implemented).
- `TaskProtocol`: Protocol for task-related communication (stub).
- `ResourceProtocol`: Protocol for resource management (stub).
- `HealthProtocol`: Protocol for health/status communication (stub).

## Protocol States
- Each protocol may define its own state machine (e.g., `Init`, `Active`, `Closed`).
- States should be represented as enums in Rust.

## Protocol Transitions
- Protocols should define valid state transitions (e.g., `Init` → `Active` → `Closed`).
- Use Rust enums and methods to enforce transitions.

## Examples
```rust
// Example protocol state enum (to be implemented in Rust)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolState {
    Init,
    Active,
    Closed,
}
```

## TODO
- Implement protocol types and state enums in the Rust codebase.
- Add protocol transition logic and tests.
