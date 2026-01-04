//! Network module for peer-to-peer agent system.
//! Provides types and helpers for network management, metrics, resources, health, and security.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Discovery submodule for peer discovery and management.
pub mod discovery;
/// Service submodule for the NetworkService implementation.
pub mod service;
/// Transport submodule for network transport protocols.
pub mod transport;

/// Peer management and state tracking
pub mod peers;

// Re-export NetworkStats from service module
pub use service::NetworkStats;

// Re-export types from peers module
pub use peers::{ConnectionStatus, PeerCache, PeerCapabilities, PeerInfo, PeerMetrics, PeerState};

/// Errors that can occur in the network module.
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Not initialized error
    #[error("Not initialized")]
    NotInitialized,
    /// Already running error
    #[error("Already running")]
    AlreadyRunning,
    /// Not running error
    #[error("Not running")]
    NotRunning,
    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),
    /// Discovery error
    #[error("Discovery error: {0}")]
    Discovery(String),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for network operations.
pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

/// Unique identifier for a peer (stub, replace with real type as needed)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

/// Multi-address for peer connections (stub, replace with real type as needed)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Multiaddr(pub String);

/// Protocol-specific configuration (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration for network protocols.
pub struct ProtocolConfig {
    // Add protocol-specific fields here
}

/// Resource usage limits and thresholds (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Resource usage limits for the network.
pub struct ResourceLimits {
    /// Maximum bandwidth in bytes per second
    pub max_bandwidth: u64,
    /// Maximum memory usage in bytes
    pub max_memory: u64,
    /// Maximum number of connections
    pub max_connections: usize,
}

/// Security configuration (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Security-related configuration for the network.
pub struct SecurityConfig {
    // Add security-related fields here
}

/// Network configuration for the P2P system
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines core configuration parameters for the network implementation.
pub struct NetworkConfig {
    /// Network address to listen on
    pub listen_addr: std::net::SocketAddr,
    /// List of bootstrap peers for initial connection
    pub bootstrap_peers: Vec<PeerInfo>,
    /// Maximum number of connected peers
    pub max_peers: usize,
    /// Protocol-specific configuration
    pub protocol_config: ProtocolConfig,
    /// Resource limits and thresholds
    pub resource_limits: ResourceLimits,
    /// Security configuration
    pub security_config: SecurityConfig,
}

/// Message structure for network communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Message content as bytes
    pub content: Vec<u8>,
}

/// Represents a message related to task execution or management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Task-specific content (could be JSON, binary, etc.)
    pub content: Vec<u8>,
    /// Task type or action
    pub task_type: String,
}

/// Represents a message related to resource usage or limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Resource-specific content
    pub content: Vec<u8>,
    /// Resource type or action
    pub resource_type: String,
}

/// Represents a message related to health checks or status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Health-specific content
    pub content: Vec<u8>,
    /// Health check type or status
    pub health_type: String,
}

/// Manages the network state and operations.
pub struct NetworkManager {
    /// Network configuration
    config: NetworkConfig,
    /// Initialization state
    is_initialized: bool,
    /// Running state
    is_running: bool,
    /// Transport protocol type
    transport_type: String,
    /// Message queue
    messages: Arc<Mutex<Vec<NetworkMessage>>>,
    /// Connected peers
    connected_peers: Arc<Mutex<Vec<SocketAddr>>>,
    /// Prometheus metrics collector for recording message metrics
    #[cfg(feature = "metrics-prometheus")]
    prometheus_metrics: Option<crate::metrics::prometheus_exporter::MetricsCollector>,
}

impl NetworkManager {
    /// Create a new NetworkManager.
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            is_initialized: false,
            is_running: false,
            transport_type: "tcp".to_string(),
            messages: Arc::new(Mutex::new(Vec::new())),
            connected_peers: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "metrics-prometheus")]
            prometheus_metrics: None,
        }
    }

    /// Create a new NetworkManager with Prometheus metrics collector.
    #[cfg(feature = "metrics-prometheus")]
    pub fn with_metrics(
        config: NetworkConfig,
        metrics: crate::metrics::prometheus_exporter::MetricsCollector,
    ) -> Self {
        Self {
            config,
            is_initialized: false,
            is_running: false,
            transport_type: "tcp".to_string(),
            messages: Arc::new(Mutex::new(Vec::new())),
            connected_peers: Arc::new(Mutex::new(Vec::new())),
            prometheus_metrics: Some(metrics),
        }
    }

    /// Check if the manager is initialized.
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Check if the manager is running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Start the network manager.
    pub async fn start(&mut self) -> NetworkResult<()> {
        if !self.is_initialized {
            return Err(NetworkError::NotInitialized);
        }
        if self.is_running {
            return Err(NetworkError::AlreadyRunning);
        }
        self.is_running = true;
        Ok(())
    }

    /// Shutdown the network manager.
    pub async fn shutdown(&mut self) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NotRunning);
        }
        self.is_running = false;
        Ok(())
    }

    /// Set the transport protocol.
    pub fn set_transport(&mut self, transport: &str) {
        self.transport_type = transport.to_string();
    }

    /// Get the transport protocol.
    pub fn get_transport(&self) -> &str {
        &self.transport_type
    }

    /// Simulate a transport failure.
    pub async fn simulate_transport_failure(&mut self) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NotRunning);
        }
        // Simulate transport failure by clearing peers
        let mut peers = self.connected_peers.lock().await;
        peers.clear();
        Ok(())
    }

    /// Add a connected peer to the manager (for testing/demo purposes).
    pub async fn add_connected_peer(&self, addr: SocketAddr) {
        self.connected_peers.lock().await.push(addr);
    }

    /// Send a message by pushing it to the message queue.
    pub async fn send_message(&self, msg: NetworkMessage) {
        self.messages.lock().await.push(msg);
    }

    /// Receive a message by popping from the message queue.
    pub async fn receive_message(&self) -> Option<NetworkMessage> {
        #[cfg(feature = "metrics-prometheus")]
        let start = std::time::Instant::now();

        let result = self.messages.lock().await.pop();

        #[cfg(feature = "metrics-prometheus")]
        if result.is_some() {
            if let Some(ref metrics) = self.prometheus_metrics {
                metrics.record_message_received();
                let duration_ms = start.elapsed().as_millis() as u64;
                metrics.record_message_duration(duration_ms);
            }
        }

        result
    }

    /// Get received messages.
    pub async fn get_messages(&self) -> Vec<NetworkMessage> {
        let messages = self.messages.lock().await;
        messages.clone()
    }

    /// Get connected peers.
    pub async fn get_connected_peers(&self) -> Vec<SocketAddr> {
        let peers = self.connected_peers.lock().await;
        peers.clone()
    }

    /// Get a reference to the network configuration.
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }
}

/// Event channels for internal network communication.
pub struct EventChannels {
    // Placeholder for event channels
}

impl EventChannels {
    /// Create a new set of event channels.
    pub fn new() -> Self {
        EventChannels {}
    }
}

impl Default for EventChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring and constructing a NetworkManager.
pub struct NetworkManagerBuilder {
    config: Option<NetworkConfig>,
}

impl NetworkManagerBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        NetworkManagerBuilder { config: None }
    }

    /// Set the network configuration.
    pub fn with_config(mut self, config: NetworkConfig) -> Self {
        self.config = Some(config);
        self
    }
}

impl Default for NetworkManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Collector for network metrics.
pub struct MetricsCollector {
    metrics: HashMap<String, u64>,
}
impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        MetricsCollector {
            metrics: HashMap::new(),
        }
    }
    /// Increment a metric by 1.
    pub fn increment(&mut self, key: &str) {
        *self.metrics.entry(key.to_string()).or_insert(0) += 1;
    }
    /// Get the value of a metric.
    pub fn get(&self, key: &str) -> u64 {
        *self.metrics.get(key).unwrap_or(&0)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages network resources.
pub struct ResourceManager;
impl ResourceManager {
    /// Create a new resource manager.
    pub fn new() -> Self {
        ResourceManager
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitors network health.
pub struct HealthMonitor;
impl HealthMonitor {
    /// Create a new health monitor.
    pub fn new() -> Self {
        HealthMonitor
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Handles network security.
pub struct SecurityManager;
impl SecurityManager {
    /// Create a new security manager.
    pub fn new() -> Self {
        SecurityManager
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_channels_new() {
        let _ = EventChannels::new();
    }

    #[test]
    fn test_network_manager_builder_new_and_with_config() {
        let config = NetworkConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            bootstrap_peers: vec![],
            max_peers: 10,
            protocol_config: ProtocolConfig {},
            resource_limits: ResourceLimits {
                max_bandwidth: 1024,
                max_memory: 2048,
                max_connections: 100,
            },
            security_config: SecurityConfig {},
        };
        let _ = NetworkManagerBuilder::new().with_config(config);
    }

    #[test]
    fn test_metrics_collector_new() {
        let _ = MetricsCollector::new();
    }

    #[test]
    fn test_resource_manager_new() {
        let _ = ResourceManager::new();
    }

    #[test]
    fn test_health_monitor_new() {
        let _ = HealthMonitor::new();
    }

    #[test]
    fn test_security_manager_new() {
        let _ = SecurityManager::new();
    }

    // Test for PeerId public methods
    #[test]
    fn test_peer_id_creation() {
        let id_str = "test-peer-123";
        let peer_id = PeerId(id_str.to_string());

        assert_eq!(peer_id.0, id_str);
    }

    #[test]
    fn test_peer_id_equality() {
        let id1 = PeerId("same-id".to_string());
        let id2 = PeerId("same-id".to_string());
        let id3 = PeerId("different-id".to_string());

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    // Test for Multiaddr public methods
    #[test]
    fn test_multiaddr_creation() {
        let addr_str = "/ip4/127.0.0.1/tcp/8080";
        let multiaddr = Multiaddr(addr_str.to_string());

        assert_eq!(multiaddr.0, addr_str);
    }

    #[test]
    fn test_multiaddr_equality() {
        let addr1 = Multiaddr("/ip4/127.0.0.1/tcp/8080".to_string());
        let addr2 = Multiaddr("/ip4/127.0.0.1/tcp/8080".to_string());
        let addr3 = Multiaddr("/ip4/127.0.0.1/tcp/9090".to_string());

        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    // Test for ResourceLimits
    #[test]
    fn test_resource_limits_creation() {
        let limits = ResourceLimits {
            max_bandwidth: 1024 * 1024,    // 1MB/s
            max_memory: 512 * 1024 * 1024, // 512MB
            max_connections: 100,
        };

        assert_eq!(limits.max_bandwidth, 1024 * 1024);
        assert_eq!(limits.max_memory, 512 * 1024 * 1024);
        assert_eq!(limits.max_connections, 100);
    }
}
