# Network API

*Part of P2P AI Agents API Reference*

---

## Network API

### NetworkManager

Core network management for P2P communication.

```rust
pub struct NetworkManager {
    // Internal fields...
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(config: NetworkConfig) -> Result<Self, NetworkError>
    
    /// Start the network manager
    pub async fn start(&self) -> Result<(), NetworkError>
    
    /// Stop the network manager
    pub async fn stop(&self) -> Result<(), NetworkError>
    
    /// Send a message to a peer
    pub async fn send_message(&self, peer_id: &str, message: NetworkMessage) -> Result<(), NetworkError>
    
    /// Broadcast a message to all peers
    pub async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), NetworkError>
    
    /// Get list of connected peers
    pub fn get_peers(&self) -> Vec<PeerInfo>
    
    /// Subscribe to network events
    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<NetworkEvent>
}
```

### NetworkConfig

Configuration for network behavior and protocols.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Local listening address
    pub listen_address: String,
    
    /// Bootstrap peers for initial connections
    pub bootstrap_peers: Vec<String>,
    
    /// Maximum number of peer connections
    pub max_peers: usize,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Protocol configurations
    pub protocols: ProtocolConfig,
}
```

### NetworkMessage

Message format for peer-to-peer communication.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Message ID for tracking
    pub id: String,
    
    /// Source agent ID
    pub source: AgentId,
    
    /// Target agent ID (None for broadcast)
    pub target: Option<AgentId>,
    
    /// Message payload
    pub payload: Vec<u8>,
    
    /// Message timestamp
    pub timestamp: SystemTime,
    
    /// Message type
    pub message_type: MessageType,
}

impl NetworkMessage {
    /// Create a new message
    pub fn new(source: AgentId, target: Option<AgentId>, payload: Vec<u8>) -> Self
    
    /// Create a broadcast message
    pub fn broadcast(source: AgentId, payload: Vec<u8>) -> Self
}
```

**Example:**

```rust
use p2p_ai_agents::prelude::*;

#[tokio::main]
async fn main() -> Result<(), NetworkError> {
    let config = NetworkConfig {
        listen_address: "127.0.0.1:0".to_string(),
        bootstrap_peers: vec!["127.0.0.1:8000".to_string()],
        max_peers: 50,
        connection_timeout: 30,
        protocols: ProtocolConfig::default(),
    };
    
    let network = NetworkManager::new(config).await?;
    network.start().await?;
    
    // Send a message
    let agent_id = AgentId::new();
    let message = NetworkMessage::broadcast(agent_id, b"Hello network!".to_vec());
    network.broadcast_message(message).await?;
    
    // Get connected peers
    let peers = network.get_peers();
    println!("Connected to {} peers", peers.len());
    
    Ok(())
}
```

### Peer Discovery

Automatic peer discovery and connection management.

```rust
pub struct DiscoveryManager {
    // Internal fields...
}

impl DiscoveryManager {
    /// Create a new discovery manager
    pub fn new(config: DiscoveryConfig) -> Self
    
    /// Start peer discovery
    pub async fn start(&self) -> Result<(), NetworkError>
    
    /// Stop peer discovery
    pub async fn stop(&self) -> Result<(), NetworkError>
    
    /// Manually add a peer
    pub async fn add_peer(&self, peer_info: PeerInfo) -> Result<(), NetworkError>
    
    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &str) -> Result<(), NetworkError>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer identifier
    pub id: String,
    
    /// Peer network address
    pub address: String,
    
    /// Peer capabilities
    pub capabilities: Vec<String>,
    
    /// Last seen timestamp
    pub last_seen: SystemTime,
}
```
