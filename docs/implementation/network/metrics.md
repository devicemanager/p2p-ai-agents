# Metrics and Monitoring

This document details the implementation of metrics collection and monitoring in the P2P networking system.

## Metrics Collector

```rust
use metrics::{counter, gauge, histogram};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Metrics collector for network operations
pub struct MetricsCollector {
    /// Network metrics
    network_metrics: Arc<RwLock<NetworkMetrics>>,
    /// Protocol metrics
    protocol_metrics: Arc<RwLock<ProtocolMetrics>>,
    /// Peer metrics
    peer_metrics: Arc<RwLock<PeerMetrics>>,
    /// Resource metrics
    resource_metrics: Arc<RwLock<ResourceMetrics>>,
}

/// Network-level metrics
#[derive(Default)]
struct NetworkMetrics {
    /// Total bytes sent
    bytes_sent: u64,
    /// Total bytes received
    bytes_received: u64,
    /// Number of active connections
    active_connections: usize,
    /// Number of connection attempts
    connection_attempts: u64,
    /// Number of connection failures
    connection_failures: u64,
    /// Average connection latency
    avg_connection_latency: Duration,
}

/// Protocol-level metrics
#[derive(Default)]
struct ProtocolMetrics {
    /// Task protocol metrics
    task_metrics: HashMap<String, u64>,
    /// Discovery protocol metrics
    discovery_metrics: HashMap<String, u64>,
    /// Resource protocol metrics
    resource_metrics: HashMap<String, u64>,
    /// Health protocol metrics
    health_metrics: HashMap<String, u64>,
}

/// Peer-level metrics
#[derive(Default)]
struct PeerMetrics {
    /// Peer-specific metrics
    peer_metrics: HashMap<PeerId, PeerStats>,
    /// Total number of peers
    total_peers: usize,
    /// Number of healthy peers
    healthy_peers: usize,
    /// Number of degraded peers
    degraded_peers: usize,
    /// Number of unhealthy peers
    unhealthy_peers: usize,
}

/// Resource usage metrics
#[derive(Default)]
struct ResourceMetrics {
    /// CPU usage percentage
    cpu_usage: f64,
    /// Memory usage in bytes
    memory_usage: u64,
    /// Network bandwidth usage in bytes per second
    bandwidth_usage: u64,
    /// Storage usage in bytes
    storage_usage: u64,
}
```

## Metrics Collection

```rust
impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            network_metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
            protocol_metrics: Arc::new(RwLock::new(ProtocolMetrics::default())),
            peer_metrics: Arc::new(RwLock::new(PeerMetrics::default())),
            resource_metrics: Arc::new(RwLock::new(ResourceMetrics::default())),
        }
    }
    
    /// Record network metrics
    pub async fn record_network_metrics(&self, metrics: NetworkMetrics) {
        let mut network_metrics = self.network_metrics.write().await;
        network_metrics.bytes_sent += metrics.bytes_sent;
        network_metrics.bytes_received += metrics.bytes_received;
        network_metrics.active_connections = metrics.active_connections;
        network_metrics.connection_attempts += metrics.connection_attempts;
        network_metrics.connection_failures += metrics.connection_failures;
        network_metrics.avg_connection_latency = metrics.avg_connection_latency;
        
        // Update metrics
        counter!("network.bytes_sent", metrics.bytes_sent);
        counter!("network.bytes_received", metrics.bytes_received);
        gauge!("network.active_connections", metrics.active_connections as f64);
        counter!("network.connection_attempts", metrics.connection_attempts);
        counter!("network.connection_failures", metrics.connection_failures);
        histogram!("network.connection_latency", metrics.avg_connection_latency.as_secs_f64());
    }
    
    /// Record protocol metrics
    pub async fn record_protocol_metrics(&self, protocol: &str, metrics: HashMap<String, u64>) {
        let mut protocol_metrics = self.protocol_metrics.write().await;
        match protocol {
            "task" => protocol_metrics.task_metrics.extend(metrics),
            "discovery" => protocol_metrics.discovery_metrics.extend(metrics),
            "resource" => protocol_metrics.resource_metrics.extend(metrics),
            "health" => protocol_metrics.health_metrics.extend(metrics),
            _ => log::warn!("Unknown protocol: {}", protocol),
        }
        
        // Update metrics
        for (name, value) in metrics {
            counter!("protocol.{}.{}", protocol, name, value);
        }
    }
    
    /// Record peer metrics
    pub async fn record_peer_metrics(&self, peer_id: PeerId, stats: PeerStats) {
        let mut peer_metrics = self.peer_metrics.write().await;
        peer_metrics.peer_metrics.insert(peer_id, stats.clone());
        
        // Update peer counts
        peer_metrics.total_peers = peer_metrics.peer_metrics.len();
        peer_metrics.healthy_peers = peer_metrics.peer_metrics.values()
            .filter(|stats| stats.health_status == HealthStatus::Healthy)
            .count();
        peer_metrics.degraded_peers = peer_metrics.peer_metrics.values()
            .filter(|stats| matches!(stats.health_status, HealthStatus::Degraded(_)))
            .count();
        peer_metrics.unhealthy_peers = peer_metrics.peer_metrics.values()
            .filter(|stats| matches!(stats.health_status, HealthStatus::Unhealthy(_)))
            .count();
        
        // Update metrics
        gauge!("peers.total", peer_metrics.total_peers as f64);
        gauge!("peers.healthy", peer_metrics.healthy_peers as f64);
        gauge!("peers.degraded", peer_metrics.degraded_peers as f64);
        gauge!("peers.unhealthy", peer_metrics.unhealthy_peers as f64);
        histogram!("peers.latency", stats.latency.as_secs_f64());
        gauge!("peers.reputation", stats.reputation as f64);
    }
    
    /// Record resource metrics
    pub async fn record_resource_metrics(&self, metrics: ResourceMetrics) {
        let mut resource_metrics = self.resource_metrics.write().await;
        resource_metrics.cpu_usage = metrics.cpu_usage;
        resource_metrics.memory_usage = metrics.memory_usage;
        resource_metrics.bandwidth_usage = metrics.bandwidth_usage;
        resource_metrics.storage_usage = metrics.storage_usage;
        
        // Update metrics
        gauge!("resources.cpu_usage", metrics.cpu_usage);
        gauge!("resources.memory_usage", metrics.memory_usage as f64);
        gauge!("resources.bandwidth_usage", metrics.bandwidth_usage as f64);
        gauge!("resources.storage_usage", metrics.storage_usage as f64);
    }
}
```

## Monitoring System

```rust
/// Monitoring system for network operations
pub struct MonitoringSystem {
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
    /// Monitoring configuration
    config: MonitoringConfig,
    /// Alert manager
    alert_manager: Arc<AlertManager>,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        let metrics = Arc::new(MetricsCollector::new());
        let alert_manager = Arc::new(AlertManager::new(config.alert_config.clone()));
        
        Self {
            metrics,
            config,
            alert_manager,
        }
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<(), NetworkError> {
        // Start metrics collection
        self.start_metrics_collection().await?;
        
        // Start alert monitoring
        self.start_alert_monitoring().await?;
        
        // Start health checks
        self.start_health_checks().await?;
        
        Ok(())
    }
    
    /// Start metrics collection
    async fn start_metrics_collection(&self) -> Result<(), NetworkError> {
        let metrics = self.metrics.clone();
        let interval = self.config.metrics_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                
                // Collect system metrics
                let system_metrics = collect_system_metrics().await;
                metrics.record_resource_metrics(system_metrics).await;
                
                // Collect network metrics
                let network_metrics = collect_network_metrics().await;
                metrics.record_network_metrics(network_metrics).await;
                
                // Collect protocol metrics
                let protocol_metrics = collect_protocol_metrics().await;
                for (protocol, metrics) in protocol_metrics {
                    metrics.record_protocol_metrics(&protocol, metrics).await;
                }
            }
        });
        
        Ok(())
    }
    
    /// Start alert monitoring
    async fn start_alert_monitoring(&self) -> Result<(), NetworkError> {
        let alert_manager = self.alert_manager.clone();
        let metrics = self.metrics.clone();
        let interval = self.config.alert_check_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                
                // Check resource usage
                let resource_metrics = metrics.resource_metrics.read().await;
                if resource_metrics.cpu_usage > 90.0 {
                    alert_manager.trigger_alert(AlertType::HighCpuUsage).await;
                }
                if resource_metrics.memory_usage > 1024 * 1024 * 1024 {
                    alert_manager.trigger_alert(AlertType::HighMemoryUsage).await;
                }
                
                // Check peer health
                let peer_metrics = metrics.peer_metrics.read().await;
                if peer_metrics.unhealthy_peers > 0 {
                    alert_manager.trigger_alert(AlertType::UnhealthyPeers).await;
                }
                
                // Check network status
                let network_metrics = metrics.network_metrics.read().await;
                if network_metrics.connection_failures > 10 {
                    alert_manager.trigger_alert(AlertType::HighConnectionFailures).await;
                }
            }
        });
        
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
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        
        // Record network metrics
        let network_metrics = NetworkMetrics {
            bytes_sent: 1000,
            bytes_received: 2000,
            active_connections: 5,
            connection_attempts: 10,
            connection_failures: 2,
            avg_connection_latency: Duration::from_millis(100),
        };
        collector.record_network_metrics(network_metrics).await;
        
        // Verify metrics
        let metrics = collector.network_metrics.read().await;
        assert_eq!(metrics.bytes_sent, 1000);
        assert_eq!(metrics.bytes_received, 2000);
        assert_eq!(metrics.active_connections, 5);
    }
    
    #[tokio::test]
    async fn test_alert_monitoring() {
        let config = MonitoringConfig {
            metrics_interval: Duration::from_secs(1),
            alert_check_interval: Duration::from_secs(1),
            alert_config: AlertConfig::default(),
        };
        
        let monitoring = MonitoringSystem::new(config);
        monitoring.start().await.unwrap();
        
        // Trigger high CPU usage
        let resource_metrics = ResourceMetrics {
            cpu_usage: 95.0,
            memory_usage: 0,
            bandwidth_usage: 0,
            storage_usage: 0,
        };
        monitoring.metrics.record_resource_metrics(resource_metrics).await;
        
        // Wait for alert check
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Verify alert was triggered
        assert!(monitoring.alert_manager.has_alert(AlertType::HighCpuUsage).await);
    }
}
```

## Notes

- Metrics are collected at multiple levels (network, protocol, peer, resource)
- Metrics are exposed through standard metrics interfaces
- Monitoring system includes alerting capabilities
- Metrics collection is non-blocking and efficient
- All metrics are properly documented and typed
- Testing covers both metrics collection and alerting
- Performance impact of metrics collection is minimized
- Metrics can be exported to various monitoring systems 