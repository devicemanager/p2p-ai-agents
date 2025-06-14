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

## Implementation Notes

### Protocol State Transitions
The protocol states follow this transition model:
```
Initializing → Connecting → Active ↔ Closed
                    ↓         ↑
                 Failed ──────┘
```

### Usage Example
```rust
use crate::network::types::{ProtocolId, ProtocolState, ProtocolConfig};

let protocol = ProtocolConfig {
    id: ProtocolId::Task,
    version: "1.0.0".to_string(),
    state: ProtocolState::Initializing,
};

// Protocol transitions are handled by the NetworkManager
```

### Related Documentation
- [Network Manager](../network-manager.md) - Protocol lifecycle management
- [Message Types](message-types.md) - Protocol message definitions
- [Error Types](error-types.md) - Protocol error handling
