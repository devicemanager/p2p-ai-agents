# Protocol Implementations

This document details the implementation of the various network protocols used in the P2P system.

## Protocol Overview

The system implements several protocols for different purposes:
- Task Protocol: Handles task distribution and results
- Discovery Protocol: Manages peer discovery and DHT operations
- Resource Protocol: Handles resource usage updates and queries
- Health Protocol: Manages health checks and status updates

## Task Protocol

```rust
use libp2p::{
    core::upgrade,
    request_response::{RequestResponse, RequestResponseConfig, RequestResponseEvent},
    swarm::NetworkBehaviour,
};

/// Task protocol implementation
#[derive(NetworkBehaviour)]
pub struct TaskProtocol {
    /// Request-response protocol for task distribution
    request_response: RequestResponse<TaskCodec>,
    /// Event sender for task events
    #[behaviour(ignore)]
    events: mpsc::Sender<TaskProtocolEvent>,
}

impl TaskProtocol {
    /// Create a new task protocol
    pub fn new(events: mpsc::Sender<TaskProtocolEvent>) -> Self {
        let config = RequestResponseConfig::default()
            .with_request_timeout(Duration::from_secs(30))
            .with_max_concurrent_requests(100);
            
        let codec = TaskCodec::default();
        let request_response = RequestResponse::new(codec, (), config);
        
        Self {
            request_response,
            events,
        }
    }
    
    /// Handle incoming task request
    async fn handle_task_request(
        &mut self,
        request: TaskRequest,
        responder: oneshot::Sender<TaskResponse>,
    ) -> Result<(), NetworkError> {
        // Process task request
        let task = request.task;
        let result = self.process_task(task).await?;
        
        // Send response
        let response = TaskResponse { result };
        responder.send(response).map_err(|_| NetworkError::ProtocolError("Failed to send response".into()))?;
        
        Ok(())
    }
    
    /// Process a task
    async fn process_task(&mut self, task: Task) -> Result<TaskResult, NetworkError> {
        // Notify task event handler
        self.events.send(TaskProtocolEvent::TaskReceived(task.clone())).await?;
        
        // Process task based on type
        let result = match task.task_type {
            TaskType::Processing => self.process_computation_task(task).await?,
            TaskType::Vector => self.process_vector_task(task).await?,
            TaskType::Storage => self.process_storage_task(task).await?,
        };
        
        // Notify task completion
        self.events.send(TaskProtocolEvent::TaskCompleted(task.id, result.clone())).await?;
        
        Ok(result)
    }
}
```

## Discovery Protocol

```rust
use libp2p::{
    kad::{Kademlia, KademliaConfig, KademliaEvent},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
};

/// Discovery protocol implementation
#[derive(NetworkBehaviour)]
pub struct DiscoveryProtocol {
    /// Kademlia DHT for peer discovery
    kademlia: Kademlia,
    /// mDNS for local network discovery
    mdns: Mdns,
    /// Event sender for discovery events
    #[behaviour(ignore)]
    events: mpsc::Sender<DiscoveryEvent>,
}

impl DiscoveryProtocol {
    /// Create a new discovery protocol
    pub fn new(events: mpsc::Sender<DiscoveryEvent>) -> Self {
        // Configure Kademlia
        let config = KademliaConfig::default();
        let kademlia = Kademlia::new(PeerId::random(), config);
        
        // Configure mDNS
        let config = MdnsConfig::default();
        let mdns = Mdns::new(config).expect("Failed to create mDNS");
        
        Self {
            kademlia,
            mdns,
            events,
        }
    }
    
    /// Start peer discovery
    pub async fn start_discovery(&mut self) -> Result<(), NetworkError> {
        // Bootstrap Kademlia
        self.kademlia.bootstrap()?;
        
        // Start mDNS discovery
        self.mdns.start_discovery()?;
        
        // Notify discovery started
        self.events.send(DiscoveryEvent::StartDHT).await?;
        
        Ok(())
    }
    
    /// Find peers matching query
    pub async fn find_peers(&mut self, query: PeerQuery) -> Result<Vec<PeerInfo>, NetworkError> {
        // Search in Kademlia
        let kad_peers = self.kademlia.find_peers(query.clone()).await?;
        
        // Search in mDNS
        let mdns_peers = self.mdns.find_peers(query.clone()).await?;
        
        // Combine and deduplicate results
        let mut peers = HashSet::new();
        peers.extend(kad_peers);
        peers.extend(mdns_peers);
        
        Ok(peers.into_iter().collect())
    }
}
```

## Resource Protocol

```rust
use libp2p::{
    core::upgrade,
    request_response::{RequestResponse, RequestResponseConfig, RequestResponseEvent},
    swarm::NetworkBehaviour,
};

/// Resource protocol implementation
#[derive(NetworkBehaviour)]
pub struct ResourceProtocol {
    /// Request-response protocol for resource updates
    request_response: RequestResponse<ResourceCodec>,
    /// Event sender for resource events
    #[behaviour(ignore)]
    events: mpsc::Sender<ResourceEvent>,
}

impl ResourceProtocol {
    /// Create a new resource protocol
    pub fn new(events: mpsc::Sender<ResourceEvent>) -> Self {
        let config = RequestResponseConfig::default()
            .with_request_timeout(Duration::from_secs(10))
            .with_max_concurrent_requests(50);
            
        let codec = ResourceCodec::default();
        let request_response = RequestResponse::new(codec, (), config);
        
        Self {
            request_response,
            events,
        }
    }
    
    /// Update resource usage
    pub async fn update_resources(&mut self, usage: ResourceUsage) -> Result<(), NetworkError> {
        // Create resource update request
        let request = ResourceRequest { usage };
        
        // Broadcast to all peers
        self.broadcast_resource_update(request).await?;
        
        // Notify resource update
        self.events.send(ResourceEvent::UpdateResources(usage)).await?;
        
        Ok(())
    }
    
    /// Get peer resources
    pub async fn get_peer_resources(&mut self, peer_id: PeerId) -> Result<ResourceUsage, NetworkError> {
        // Create resource query request
        let request = ResourceRequest {
            usage: ResourceUsage::default(),
            query: Some(ResourceQuery { peer_id }),
        };
        
        // Send request to peer
        let response = self.request_response.send_request(&peer_id, request).await?;
        
        Ok(response.usage)
    }
}
```

## Health Protocol

```rust
use libp2p::{
    core::upgrade,
    request_response::{RequestResponse, RequestResponseConfig, RequestResponseEvent},
    swarm::NetworkBehaviour,
};

/// Health protocol implementation
#[derive(NetworkBehaviour)]
pub struct HealthProtocol {
    /// Request-response protocol for health checks
    request_response: RequestResponse<HealthCodec>,
    /// Event sender for health events
    #[behaviour(ignore)]
    events: mpsc::Sender<HealthEvent>,
}

impl HealthProtocol {
    /// Create a new health protocol
    pub fn new(events: mpsc::Sender<HealthEvent>) -> Self {
        let config = RequestResponseConfig::default()
            .with_request_timeout(Duration::from_secs(5))
            .with_max_concurrent_requests(20);
            
        let codec = HealthCodec::default();
        let request_response = RequestResponse::new(codec, (), config);
        
        Self {
            request_response,
            events,
        }
    }
    
    /// Check peer health
    pub async fn check_health(&mut self, peer_id: PeerId) -> Result<HealthStatus, NetworkError> {
        // Create health check request
        let request = HealthRequest;
        
        // Send request to peer
        let response = self.request_response.send_request(&peer_id, request).await?;
        
        // Update peer health status
        self.events.send(HealthEvent::HealthCheckReceived(peer_id, response.status.clone())).await?;
        
        Ok(response.status)
    }
    
    /// Report health status
    pub async fn report_health(&mut self, status: HealthStatus) -> Result<(), NetworkError> {
        // Create health status response
        let response = HealthResponse { status };
        
        // Broadcast to all peers
        self.broadcast_health_status(response).await?;
        
        // Notify health status update
        self.events.send(HealthEvent::ReportHealth(status)).await?;
        
        Ok(())
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_protocol() {
        let (events, _) = mpsc::channel(32);
        let mut protocol = TaskProtocol::new(events);
        
        let task = Task {
            id: TaskId::random(),
            task_type: TaskType::Processing,
            input: TaskInput::Data(vec![1, 2, 3]),
            priority: Priority::Normal,
        };
        
        let result = protocol.process_task(task).await.unwrap();
        assert!(result.is_success());
    }
    
    #[tokio::test]
    async fn test_discovery_protocol() {
        let (events, _) = mpsc::channel(32);
        let mut protocol = DiscoveryProtocol::new(events);
        
        protocol.start_discovery().await.unwrap();
        
        let query = PeerQuery {
            query_type: PeerQueryType::All,
            filter: None,
            limit: Some(10),
        };
        
        let peers = protocol.find_peers(query).await.unwrap();
        assert!(peers.len() <= 10);
    }
}
```

## Notes

- Each protocol is implemented as a separate module
- Protocols use libp2p's request-response pattern for reliable communication
- Discovery protocol combines Kademlia DHT and mDNS for effective peer discovery
- Resource protocol handles resource usage updates and queries
- Health protocol implements periodic health checks and status updates
- All protocols include proper error handling and event notification
- Protocols are tested with unit tests and integration tests
- Performance considerations are implemented for each protocol 