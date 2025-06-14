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

## Message Types Definition

### Core Message Types
```rust
// Task-related messages
pub enum TaskMessage {
    Request { task_id: String, payload: Vec<u8> },
    Response { task_id: String, result: TaskResult },
    Status { task_id: String, status: TaskStatus },
}

// Resource-related messages  
pub enum ResourceMessage {
    Query { resource_type: String },
    Update { resources: Vec<ResourceInfo> },
    Request { resource_id: String, amount: u64 },
}

// Health monitoring messages
pub enum HealthMessage {
    Ping { timestamp: u64 },
    Pong { timestamp: u64, latency: u64 },
    Status { metrics: HealthMetrics },
}
```

### Message Serialization
All messages use efficient binary serialization via `serde` and `bincode`:

```rust
use serde::{Serialize, Deserialize};

impl NetworkMessage {
    pub fn serialize(&self) -> Result<Vec<u8>, SerializationError> {
        bincode::serialize(self).map_err(SerializationError::from)
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, SerializationError> {
        bincode::deserialize(data).map_err(SerializationError::from)
    }
}
```

### Related Documentation
- [Protocol Types](protocol-types.md) - Protocol definitions that use these messages
- [Error Types](error-types.md) - Message error handling
- [Network Manager](../network-manager.md) - Message routing and processing
