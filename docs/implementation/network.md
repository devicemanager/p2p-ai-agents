# Network Protocol Implementation

This document details the implementation of the P2P networking system in Rust, including protocol handling, peer discovery, and secure communication.

## Network Types and Definitions

```rust
use libp2p::{
    core::{multiaddr::MultiAddr, PeerId},
    identity::{Keypair, PublicKey},
    swarm::{NetworkBehaviour, SwarmEvent},
    Transport,
};
use tokio::sync::{mpsc, RwLock};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: MultiAddr,
    pub bootstrap_peers: Vec<MultiAddr>,
    pub max_peers: usize,
    pub connection_timeout: Duration,
    pub ping_interval: Duration,
    pub protocol_timeout: Duration,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<MultiAddr>,
    pub protocols: HashSet<String>,
    pub last_seen: DateTime<Utc>,
    pub latency: Option<Duration>,
    pub reputation: i32,
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Task distribution message
    TaskDistribution {
        task_id: TaskId,
        task: Task,
        target_peers: Vec<PeerId>,
    },
    
    /// Task result message
    TaskResult {
        task_id: TaskId,
        result: TaskResult,
        source_peer: PeerId,
    },
    
    /// Peer discovery message
    PeerDiscovery {
        known_peers: Vec<PeerInfo>,
        query: Option<PeerQuery>,
    },
    
    /// Resource update message
    ResourceUpdate {
        peer_id: PeerId,
        resources: ResourceUsage,
    },
    
    /// Health check message
    HealthCheck {
        peer_id: PeerId,
        status: HealthStatus,
    },
}

/// Network protocol types
#[derive(Debug, Clone, NetworkBehaviour)]
pub enum NetworkProtocol {
    /// Task distribution protocol
    TaskProtocol {
        #[behaviour(ignore)]
        events: mpsc::Sender<TaskProtocolEvent>,
    },
    
    /// Peer discovery protocol
    DiscoveryProtocol {
        #[behaviour(ignore)]
        events: mpsc::Sender<DiscoveryEvent>,
    },
    
    /// Resource management protocol
    ResourceProtocol {
        #[behaviour(ignore)]
        events: mpsc::Sender<ResourceEvent>,
    },
    
    /// Health check protocol
    HealthProtocol {
        #[behaviour(ignore)]
        events: mpsc::Sender<HealthEvent>,
    },
}
```

## Network Manager Implementation

```rust
pub struct NetworkManager {
    swarm: Swarm<NetworkProtocol>,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    config: NetworkConfig,
    event_sender: mpsc::Sender<NetworkEvent>,
    event_receiver: mpsc::Receiver<NetworkEvent>,
    metrics: Arc<NetworkMetricsCollector>,
}

impl NetworkManager {
    pub async fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        // Generate or load identity
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        // Create transport
        let transport = libp2p::tokio::Transport::new(
            libp2p::tls::Config::new(&local_key),
            libp2p::noise::Config::new(&local_key),
        )
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(libp2p::noise::Config::new(&local_key))
        .multiplex(libp2p::yamux::Config::default())
        .boxed();
        
        // Create protocols
        let (task_sender, task_receiver) = mpsc::channel(32);
        let (discovery_sender, discovery_receiver) = mpsc::channel(32);
        let (resource_sender, resource_receiver) = mpsc::channel(32);
        let (health_sender, health_receiver) = mpsc::channel(32);
        
        let protocols = NetworkProtocol::new(
            task_sender,
            discovery_sender,
            resource_sender,
            health_sender,
        );
        
        // Create swarm
        let mut swarm = Swarm::new(transport, protocols, local_peer_id);
        
        // Listen on configured address
        swarm.listen_on(config.listen_addr.clone())?;
        
        // Bootstrap with known peers
        for addr in &config.bootstrap_peers {
            swarm.dial(addr.clone())?;
        }
        
        // Create event channels
        let (event_sender, event_receiver) = mpsc::channel(64);
        
        Ok(Self {
            swarm,
            peers: Arc::new(RwLock::new(HashMap::new())),
            config,
            event_sender,
            event_receiver,
            metrics: Arc::new(NetworkMetricsCollector::new()),
        })
    }
    
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        // Start network event loop
        let mut swarm = self.swarm.clone();
        let peers = self.peers.clone();
        let event_sender = self.event_sender.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            Self::event_loop(swarm, peers, event_sender, metrics).await
        });
        
        Ok(())
    }
    
    async fn event_loop(
        mut swarm: Swarm<NetworkProtocol>,
        peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        event_sender: mpsc::Sender<NetworkEvent>,
        metrics: Arc<NetworkMetricsCollector>,
    ) {
        loop {
            match swarm.next().await {
                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                    info!("Listening on {}", address);
                    metrics.record_listen_addr(&address);
                }
                
                Some(SwarmEvent::ConnectionEstablished { peer_id, .. }) => {
                    info!("Connected to peer {}", peer_id);
                    metrics.record_connection(&peer_id);
                    
                    // Update peer info
                    let mut peers = peers.write().await;
                    peers.entry(peer_id).or_insert_with(|| PeerInfo {
                        peer_id,
                        addresses: Vec::new(),
                        protocols: HashSet::new(),
                        last_seen: Utc::now(),
                        latency: None,
                        reputation: 0,
                    });
                }
                
                Some(SwarmEvent::ConnectionClosed { peer_id, .. }) => {
                    info!("Disconnected from peer {}", peer_id);
                    metrics.record_disconnection(&peer_id);
                    
                    // Update peer info
                    let mut peers = peers.write().await;
                    if let Some(peer) = peers.get_mut(&peer_id) {
                        peer.last_seen = Utc::now();
                    }
                }
                
                Some(SwarmEvent::Behaviour(protocol_event)) => {
                    match protocol_event {
                        NetworkProtocol::TaskProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_task_event(event, &event_sender).await;
                            }
                        }
                        
                        NetworkProtocol::DiscoveryProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_discovery_event(event, &event_sender).await;
                            }
                        }
                        
                        NetworkProtocol::ResourceProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_resource_event(event, &event_sender).await;
                            }
                        }
                        
                        NetworkProtocol::HealthProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_health_event(event, &event_sender).await;
                            }
                        }
                    }
                }
                
                _ => {}
            }
        }
    }
}
```

## Protocol Implementations

### Task Protocol

```rust
pub struct TaskProtocol {
    events: mpsc::Sender<TaskProtocolEvent>,
}

impl TaskProtocol {
    pub async fn send_task(
        &self,
        task: Task,
        target_peers: Vec<PeerId>,
    ) -> Result<(), NetworkError> {
        let message = NetworkMessage::TaskDistribution {
            task_id: task.id,
            task,
            target_peers,
        };
        
        self.send_message(message).await
    }
    
    pub async fn send_result(
        &self,
        task_id: TaskId,
        result: TaskResult,
        source_peer: PeerId,
    ) -> Result<(), NetworkError> {
        let message = NetworkMessage::TaskResult {
            task_id,
            result,
            source_peer,
        };
        
        self.send_message(message).await
    }
    
    async fn send_message(&self, message: NetworkMessage) -> Result<(), NetworkError> {
        let event = TaskProtocolEvent::SendMessage(message);
        self.events.send(event).await?;
        Ok(())
    }
}
```

### Discovery Protocol

```rust
pub struct DiscoveryProtocol {
    events: mpsc::Sender<DiscoveryEvent>,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
}

impl DiscoveryProtocol {
    pub async fn start_discovery(&self) -> Result<(), NetworkError> {
        // Start DHT
        let event = DiscoveryEvent::StartDHT;
        self.events.send(event).await?;
        
        // Bootstrap with known peers
        let event = DiscoveryEvent::Bootstrap;
        self.events.send(event).await?;
        
        Ok(())
    }
    
    pub async fn find_peers(&self, query: PeerQuery) -> Result<Vec<PeerInfo>, NetworkError> {
        let event = DiscoveryEvent::FindPeers(query);
        self.events.send(event).await?;
        
        // Wait for response
        let peers = self.peers.read().await;
        Ok(peers.values().cloned().collect())
    }
    
    pub async fn announce_peer(&self, peer_info: PeerInfo) -> Result<(), NetworkError> {
        let event = DiscoveryEvent::AnnouncePeer(peer_info);
        self.events.send(event).await?;
        Ok(())
    }
}
```

### Resource Protocol

```rust
pub struct ResourceProtocol {
    events: mpsc::Sender<ResourceEvent>,
    resource_monitor: Arc<ResourceMonitor>,
}

impl ResourceProtocol {
    pub async fn start_monitoring(&self) -> Result<(), NetworkError> {
        let monitor = self.resource_monitor.clone();
        let events = self.events.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Ok(usage) = monitor.current_usage().await {
                    let event = ResourceEvent::UpdateResources(usage);
                    if let Err(e) = events.send(event).await {
                        error!("Failed to send resource update: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn get_peer_resources(&self, peer_id: PeerId) -> Result<ResourceUsage, NetworkError> {
        let event = ResourceEvent::GetPeerResources(peer_id);
        self.events.send(event).await?;
        
        // Wait for response
        // Implementation depends on your event handling system
        unimplemented!()
    }
}
```

### Health Protocol

```rust
pub struct HealthProtocol {
    events: mpsc::Sender<HealthEvent>,
}

impl HealthProtocol {
    pub async fn start_health_checks(&self) -> Result<(), NetworkError> {
        let events = self.events.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let event = HealthEvent::CheckHealth;
                if let Err(e) = events.send(event).await {
                    error!("Failed to send health check: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn report_health(&self, status: HealthStatus) -> Result<(), NetworkError> {
        let event = HealthEvent::ReportHealth(status);
        self.events.send(event).await?;
        Ok(())
    }
}
```

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Transport error: {0}")]
    TransportError(#[from] libp2p::TransportError),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    
    #[error("Message error: {0}")]
    MessageError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl NetworkError {
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            NetworkError::TransportError(_) |
            NetworkError::ConnectionError(_) |
            NetworkError::TimeoutError(_)
        )
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_peer_discovery() {
        let config = NetworkConfig::default();
        let mut manager = NetworkManager::new(config).await.unwrap();
        manager.start().await.unwrap();
        
        // Wait for bootstrap
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Check connected peers
        let peers = manager.get_peers().await.unwrap();
        assert!(!peers.is_empty());
    }
    
    #[test]
    async fn test_task_distribution() {
        let config = NetworkConfig::default();
        let mut manager = NetworkManager::new(config).await.unwrap();
        manager.start().await.unwrap();
        
        let task = Task {
            id: TaskId::new(),
            task_type: TaskType::TextProcessing,
            input: TaskInput::Text("Test".to_string()),
            ..Default::default()
        };
        
        let result = manager.distribute_task(task, vec![]).await;
        assert!(result.is_ok());
    }
    
    #[test]
    async fn test_health_check() {
        let config = NetworkConfig::default();
        let mut manager = NetworkManager::new(config).await.unwrap();
        manager.start().await.unwrap();
        
        let status = manager.check_health().await.unwrap();
        assert_eq!(status, HealthStatus::Healthy);
    }
}
```

## Performance Optimization

1. **Connection Management**
   - Implement connection pooling
   - Handle connection limits
   - Optimize connection reuse
   - Manage connection lifecycle

2. **Message Handling**
   - Implement message batching
   - Use efficient serialization
   - Handle backpressure
   - Optimize message sizes

3. **Peer Discovery**
   - Use efficient DHT implementation
   - Implement caching
   - Optimize peer selection
   - Handle peer churn

4. **Resource Usage**
   - Monitor bandwidth usage
   - Implement rate limiting
   - Handle resource exhaustion
   - Optimize protocol overhead

## Metrics and Monitoring

```rust
use metrics::{counter, gauge, histogram};

impl NetworkManager {
    async fn record_metrics(&self) {
        // Connection metrics
        let peers = self.peers.read().await;
        gauge!("network.peers.total", peers.len() as f64);
        gauge!("network.peers.active", peers.values()
            .filter(|p| p.last_seen > Utc::now() - Duration::minutes(5))
            .count() as f64);
        
        // Protocol metrics
        counter!("network.messages.sent", 1);
        counter!("network.messages.received", 1);
        histogram!("network.message.size", message_size as f64);
        
        // Performance metrics
        histogram!("network.latency", latency.as_secs_f64());
        histogram!("network.bandwidth", bandwidth as f64);
        
        // Error metrics
        counter!("network.errors.total", 1);
        counter!("network.errors.by_type", 1, "type" => error_type);
    }
}
``` 