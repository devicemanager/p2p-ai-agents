# Testing & Operations

*Part of Network Implementation Guide*

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
