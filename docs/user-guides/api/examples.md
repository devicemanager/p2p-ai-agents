# API Examples

*Part of P2P AI Agents API Reference*

---

## Examples

### Complete Agent Setup

```rust
use p2p_ai_agents::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent configuration
    let agent_config = AgentConfig {
        max_concurrent_tasks: 5,
        resource_limits: ResourceLimits {
            max_memory_mb: 1024,
            max_cpu_percent: 80.0,
            max_disk_mb: 2048,
        },
        ..Default::default()
    };
    
    // Create network configuration
    let network_config = NetworkConfig {
        listen_address: "127.0.0.1:0".to_string(),
        bootstrap_peers: vec![],
        max_peers: 20,
        connection_timeout: 30,
        protocols: ProtocolConfig::default(),
    };
    
    // Create agent
    let agent = Agent::new(agent_config).await?;
    
    // Create network manager
    let network = NetworkManager::new(network_config).await?;
    
    // Start services
    agent.start().await?;
    network.start().await?;
    
    // Submit a task
    let task = Task::new("data_processing".to_string(), b"sample data".to_vec());
    let task_id = agent.submit_task(task).await?;
    
    // Monitor task progress
    loop {
        let status = agent.get_task_status(&task_id).await?;
        match status {
            TaskStatus::Completed { result, .. } => {
                println!("Task completed with result: {:?}", result);
                break;
            }
            TaskStatus::Failed { error, .. } => {
                println!("Task failed: {}", error);
                break;
            }
            TaskStatus::Running { progress, .. } => {
                if let Some(p) = progress {
                    println!("Task progress: {:.1}%", p * 100.0);
                }
            }
            _ => {}
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Cleanup
    agent.stop().await?;
    network.stop().await?;
    
    Ok(())
}
```

### Network Communication

```rust
use p2p_ai_agents::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let network = NetworkManager::new(config).await?;
    
    // Subscribe to network events
    let mut event_receiver = network.subscribe_events();
    
    // Start network
    network.start().await?;
    
    // Handle incoming messages
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                NetworkEvent::MessageReceived { from, message } => {
                    println!("Received message from {}: {:?}", from, message);
                }
                NetworkEvent::PeerConnected { peer_id } => {
                    println!("Peer connected: {}", peer_id);
                }
                NetworkEvent::PeerDisconnected { peer_id } => {
                    println!("Peer disconnected: {}", peer_id);
                }
            }
        }
    });
    
    // Send messages
    let agent_id = AgentId::new();
    let message = NetworkMessage::broadcast(agent_id, b"Hello everyone!".to_vec());
    network.broadcast_message(message).await?;
    
    Ok(())
}
```

### Custom Storage Backend

```rust
use p2p_ai_agents::storage::StorageBackend;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct MemoryStorage {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StorageBackend for MemoryStorage {
    type Error = std::io::Error;
    
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), Self::Error> {
        let mut storage = self.data.write().await;
        storage.insert(key.to_string(), data.to_vec());
        Ok(())
    }
    
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        let storage = self.data.read().await;
        Ok(storage.get(key).cloned())
    }
    
    async fn delete(&self, key: &str) -> Result<bool, Self::Error> {
        let mut storage = self.data.write().await;
        Ok(storage.remove(key).is_some())
    }
    
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, Self::Error> {
        let storage = self.data.read().await;
        let keys: Vec<String> = match prefix {
            Some(p) => storage.keys().filter(|k| k.starts_with(p)).cloned().collect(),
            None => storage.keys().cloned().collect(),
        };
        Ok(keys)
    }
    
    async fn exists(&self, key: &str) -> Result<bool, Self::Error> {
        let storage = self.data.read().await;
        Ok(storage.contains_key(key))
    }
}
```
