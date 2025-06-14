# Basic Usage Examples

This guide demonstrates basic usage of the P2P network module.

## Network Initialization
```rust
use p2p_ai_agents::network::{NetworkConfig, ProtocolConfig, ResourceLimits, SecurityConfig, NetworkManager};

#[tokio::main]
async fn main() {
    let config = NetworkConfig {
        listen_addr: "127.0.0.1:8080".parse().unwrap(),
        bootstrap_peers: vec![],
        max_peers: 50,
        protocol_config: ProtocolConfig {},
        resource_limits: ResourceLimits {
            max_bandwidth: 1024 * 1024,
            max_memory: 512 * 1024 * 1024,
            max_connections: 100,
        },
        security_config: SecurityConfig {},
    };
    let mut manager = NetworkManager::new(config);
    // manager.initialize().await?; // Uncomment if implemented
    // manager.start().await?;
}
```

## Peer Connection
```rust
use p2p_ai_agents::network::{PeerInfo, PeerId, Multiaddr, PeerCapabilities, ConnectionStatus};
use chrono::Utc;

let peer = PeerInfo {
    peer_id: PeerId("QmPeer...".to_string()),
    addresses: vec![Multiaddr("/ip4/127.0.0.1/tcp/8080".to_string())],
    last_seen: Utc::now(),
    reputation: 100,
    capabilities: PeerCapabilities,
    status: ConnectionStatus::Connected,
};
// manager.connected_peers.lock().await.push(peer.addresses[0].clone().into()); // Example stub
```

## Message Sending and Receiving
```rust
use p2p_ai_agents::network::{NetworkMessage, NetworkManager};
use tokio::sync::Mutex;
use std::sync::Arc;

let msg = NetworkMessage {
    from: "peer1".to_string(),
    to: "peer2".to_string(),
    content: b"hello".to_vec(),
};
// Simulate sending a message by pushing to the manager's queue
manager.messages.lock().await.push(msg.clone());
// Simulate receiving a message
let received = manager.messages.lock().await.pop();
assert_eq!(received.unwrap().content, b"hello".to_vec());
```

## Error Handling
```rust
use p2p_ai_agents::network::NetworkError;

fn handle_error(err: NetworkError) {
    match err {
        NetworkError::NotInitialized => println!("Network not initialized"),
        NetworkError::Transport(e) => println!("Transport error: {}", e),
        _ => println!("Other error: {:?}", err),
    }
}

// Example: Handling a failing operation
let result: Result<(), NetworkError> = Err(NetworkError::NotInitialized);
if let Err(e) = result {
    handle_error(e);
}
```

## Protocol Usage (Advanced)
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum ProtocolState {
    Init,
    Active,
    Closed,
}

let mut state = ProtocolState::Init;
// Transition to Active
state = ProtocolState::Active;
assert_eq!(state, ProtocolState::Active);
// Transition to Closed
state = ProtocolState::Closed;
assert_eq!(state, ProtocolState::Closed);
```

## Metrics Usage (Advanced)
```rust
// Assume MetricsCollector is implemented in the network module
use p2p_ai_agents::network::MetricsCollector;

let mut metrics = MetricsCollector::new();
metrics.increment("messages_sent");
metrics.increment("messages_sent");
let count = metrics.get("messages_sent");
assert_eq!(count, 2);
```
