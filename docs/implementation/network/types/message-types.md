# Message Types

This document describes the message types used in the P2P network implementation.

## Message Definitions
- `NetworkMessage`: Represents a message sent between peers in the network.
- `TaskMessage`: Represents a message related to task execution or management.
- `ResourceMessage`: Represents a message related to resource usage or limits.
- `HealthMessage`: Represents a message related to health checks or status.

## Message Formats
All messages are serialized using `serde` (JSON or binary). Each message type includes sender, receiver, and content fields, with additional fields as needed for the message domain.

## Serialization Rules
- All message types derive `Serialize` and `Deserialize`.
- Messages should be versioned if protocol changes are expected.
- Use enums for message variants when possible.

## Examples
```rust
use p2p_ai_agents::network::{NetworkMessage};

let msg = NetworkMessage {
    from: "peer1".to_string(),
    to: "peer2".to_string(),
    content: b"hello".to_vec(),
};
```

## TODO
- Define `TaskMessage`, `ResourceMessage`, and `HealthMessage` types in Rust codebase.
- Add message enums and serialization tests.
