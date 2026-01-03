//! Network service implementation for the P2P AI Agent system
//!
//! This module defines the NetworkService trait and implementation
//! following the core service pattern.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::core::services::{
    BaseService, Service, ServiceError, ServiceHealth, ServiceId, ServiceRegistry, ServiceRequest,
    ServiceResponse, ServiceStatus,
};
use crate::network::{
    Multiaddr, NetworkConfig, NetworkError, NetworkManager, NetworkMessage, PeerId,
};

/// Configuration for the NetworkService
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkServiceConfig {
    /// Network configuration
    pub network_config: NetworkConfig,
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service dependencies
    pub dependencies: Vec<String>,
}

impl Default for NetworkServiceConfig {
    fn default() -> Self {
        Self {
            network_config: NetworkConfig {
                listen_addr: "0.0.0.0:8000".parse().unwrap(),
                bootstrap_peers: vec![],
                max_peers: 100,
                protocol_config: crate::network::ProtocolConfig {},
                resource_limits: crate::network::ResourceLimits {
                    max_bandwidth: 1024 * 1024 * 10, // 10 MB/s
                    max_memory: 512 * 1024 * 1024,   // 512 MB
                    max_connections: 100,
                },
                security_config: crate::network::SecurityConfig {},
            },
            name: "network".to_string(),
            version: "0.1.0".to_string(),
            dependencies: vec![],
        }
    }
}

/// Requests that can be sent to the NetworkService
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkServiceRequest {
    /// Send a message to a specific peer
    SendMessage {
        /// Target peer ID
        peer_id: PeerId,
        /// Message to send
        message: NetworkMessage,
    },
    /// Broadcast a message to all connected peers
    BroadcastMessage(NetworkMessage),
    /// Get list of connected peers
    GetConnectedPeers,
    /// Connect to a peer
    ConnectToPeer(Multiaddr),
    /// Disconnect from a peer
    DisconnectFromPeer(PeerId),
    /// Get network statistics
    GetNetworkStats,
    /// Get peer information
    GetPeerInfo(PeerId),
}

/// Responses from the NetworkService
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkServiceResponse {
    /// Response to SendMessage
    MessageSent(Result<(), String>),
    /// Response to BroadcastMessage
    MessageBroadcast(Result<(), String>),
    /// Response to GetConnectedPeers
    ConnectedPeers(Vec<PeerId>),
    /// Response to ConnectToPeer
    Connected(Result<(), String>),
    /// Response to DisconnectFromPeer
    Disconnected(Result<(), String>),
    /// Response to GetNetworkStats
    NetworkStats(NetworkStats),
    /// Response to GetPeerInfo
    PeerInfo(Option<crate::network::PeerInfo>),
}

/// Error conversion from NetworkError to ServiceError
impl From<NetworkError> for ServiceError {
    fn from(error: NetworkError) -> Self {
        match error {
            NetworkError::NotInitialized => {
                ServiceError::InitializationFailed("Network not initialized".to_string())
            }
            NetworkError::AlreadyRunning => {
                ServiceError::StartFailed("Network already running".to_string())
            }
            NetworkError::NotRunning => ServiceError::StopFailed("Network not running".to_string()),
            NetworkError::Transport(msg) => {
                ServiceError::RequestFailed(format!("Transport error: {}", msg))
            }
            NetworkError::Discovery(msg) => {
                ServiceError::RequestFailed(format!("Discovery error: {}", msg))
            }
            NetworkError::Io(err) => ServiceError::RequestFailed(format!("IO error: {}", err)),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Number of connected peers
    pub connected_peers: usize,
    /// Number of pending connections
    pub pending_connections: usize,
    /// Total messages sent
    pub total_messages_sent: u64,
    /// Total messages received
    pub total_messages_received: u64,
    /// Network uptime in seconds
    pub uptime_seconds: u64,
    /// Bandwidth usage (bytes per second)
    pub bandwidth_usage: u64,
}

/// Implementation of the NetworkService
pub struct NetworkServiceImpl {
    /// Base service functionality
    base: BaseService,
    /// Network manager
    network_manager: Arc<RwLock<NetworkManager>>,
    /// Service configuration
    config: Arc<RwLock<NetworkServiceConfig>>,
    /// Network statistics
    stats: Arc<RwLock<NetworkStats>>,
    /// Start time for uptime calculation
    start_time: Arc<RwLock<Option<Instant>>>,
}

impl NetworkServiceImpl {
    /// Create a new NetworkServiceImpl
    pub fn new(config: NetworkServiceConfig) -> Self {
        let network_config = config.network_config.clone();

        Self {
            base: BaseService::new(config.name.clone(), config.version.clone()),
            network_manager: Arc::new(RwLock::new(NetworkManager::new(network_config))),
            config: Arc::new(RwLock::new(config)),
            stats: Arc::new(RwLock::new(NetworkStats {
                connected_peers: 0,
                pending_connections: 0,
                total_messages_sent: 0,
                total_messages_received: 0,
                uptime_seconds: 0,
                bandwidth_usage: 0,
            })),
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the service name
    pub fn name(&self) -> String {
        self.base.name().to_string()
    }

    /// Get the service version
    pub fn version(&self) -> String {
        self.base.version().to_string()
    }

    /// Record a metric
    async fn record_network_metric(&self, key: String, value: serde_json::Value) {
        self.base.record_metric(key, value).await;
    }

    /// Update network statistics
    async fn update_stats<F>(&self, f: F)
    where
        F: FnOnce(&mut NetworkStats),
    {
        let mut stats = self.stats.write().await;
        f(&mut stats);

        // Update metrics
        self.record_network_metric(
            "connected_peers".to_string(),
            serde_json::json!(stats.connected_peers),
        )
        .await;
        self.record_network_metric(
            "total_messages_sent".to_string(),
            serde_json::json!(stats.total_messages_sent),
        )
        .await;
        self.record_network_metric(
            "total_messages_received".to_string(),
            serde_json::json!(stats.total_messages_received),
        )
        .await;
    }

    /// Convert NetworkServiceRequest to ServiceRequest
    fn network_request_to_service_request(&self, request: NetworkServiceRequest) -> ServiceRequest {
        let mut parameters = HashMap::new();
        parameters.insert(
            "request".to_string(),
            serde_json::to_value(&request).unwrap_or_default(),
        );

        ServiceRequest {
            id: Uuid::new_v4(),
            method: match &request {
                NetworkServiceRequest::SendMessage { .. } => "send_message".to_string(),
                NetworkServiceRequest::BroadcastMessage(_) => "broadcast_message".to_string(),
                NetworkServiceRequest::GetConnectedPeers => "get_connected_peers".to_string(),
                NetworkServiceRequest::ConnectToPeer(_) => "connect_to_peer".to_string(),
                NetworkServiceRequest::DisconnectFromPeer(_) => "disconnect_from_peer".to_string(),
                NetworkServiceRequest::GetNetworkStats => "get_network_stats".to_string(),
                NetworkServiceRequest::GetPeerInfo(_) => "get_peer_info".to_string(),
            },
            parameters,
            timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Convert NetworkServiceResponse to ServiceResponse
    fn network_response_to_service_response(
        &self,
        request_id: Uuid,
        response: Result<NetworkServiceResponse, ServiceError>,
        start_time: Instant,
    ) -> ServiceResponse {
        match response {
            Ok(net_response) => ServiceResponse {
                id: request_id,
                success: true,
                data: Some(serde_json::to_value(net_response).unwrap_or_default()),
                error: None,
                duration: start_time.elapsed(),
            },
            Err(e) => ServiceResponse {
                id: request_id,
                success: false,
                data: None,
                error: Some(e.to_string()),
                duration: start_time.elapsed(),
            },
        }
    }
}

#[async_trait]
impl Service for NetworkServiceImpl {
    fn id(&self) -> ServiceId {
        self.base.id()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn version(&self) -> &str {
        self.base.version()
    }

    async fn initialize(&self) -> Result<(), ServiceError> {
        info!("Initializing NetworkService: {}", self.name());

        // Initialize base service
        self.base.initialize().await?;

        let config = self.config.read().await;
        let network_config = config.network_config.clone();

        // Initialize network manager
        let mut network_manager = self.network_manager.write().await;
        // Initialize the NetworkManager
        network_manager.is_initialized = true;
        drop(network_manager);

        // Register service configuration
        self.record_network_metric(
            "max_peers".to_string(),
            serde_json::json!(network_config.max_peers),
        )
        .await;
        self.record_network_metric(
            "listen_addr".to_string(),
            serde_json::json!(network_config.listen_addr.to_string()),
        )
        .await;

        info!("NetworkService initialized successfully");
        Ok(())
    }

    async fn start(&self) -> Result<(), ServiceError> {
        info!("Starting NetworkService: {}", self.name());

        let mut start_time = self.start_time.write().await;
        *start_time = Some(Instant::now());
        drop(start_time);

        // Start base service
        self.base.start().await?;

        // Start network manager
        let mut network_manager = self.network_manager.write().await;
        network_manager.start().await.map_err(ServiceError::from)?;

        info!("NetworkService started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<(), ServiceError> {
        info!("Stopping NetworkService: {}", self.name());

        // Stop network manager
        let mut network_manager = self.network_manager.write().await;
        network_manager
            .shutdown()
            .await
            .map_err(ServiceError::from)?;

        // Clear start time
        let mut start_time = self.start_time.write().await;
        *start_time = None;
        drop(start_time);

        // Stop base service
        self.base.stop().await?;

        info!("NetworkService stopped successfully");
        Ok(())
    }

    async fn status(&self) -> ServiceStatus {
        let base_status = self.base.status().await;

        // Augment with network-specific status
        let network_manager = self.network_manager.read().await;
        if network_manager.is_running() {
            ServiceStatus::Running
        } else {
            base_status
        }
    }

    async fn health(&self) -> ServiceHealth {
        let mut health = self.base.health().await;

        // Add network-specific health metrics
        let stats = self.stats.read().await;
        health.metrics.insert(
            "connected_peers".to_string(),
            serde_json::json!(stats.connected_peers),
        );
        health.metrics.insert(
            "total_messages_sent".to_string(),
            serde_json::json!(stats.total_messages_sent),
        );
        health.metrics.insert(
            "total_messages_received".to_string(),
            serde_json::json!(stats.total_messages_received),
        );
        health.metrics.insert(
            "bandwidth_usage".to_string(),
            serde_json::json!(stats.bandwidth_usage),
        );

        // Calculate uptime
        let start_time = self.start_time.read().await;
        if let Some(start) = *start_time {
            health.uptime = start.elapsed();
        }

        health
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> Result<ServiceResponse, ServiceError> {
        let start_time = Instant::now();

        // Try to parse parameters back to NetworkServiceRequest
        // The parameters should contain a "request" key with the serialized NetworkServiceRequest
        let network_request: Result<NetworkServiceRequest, _> = request
            .parameters
            .get("request")
            .ok_or_else(|| ServiceError::RequestFailed("Missing request parameter".to_string()))
            .and_then(|val| {
                serde_json::from_value(val.clone()).map_err(|e| {
                    ServiceError::RequestFailed(format!("Failed to parse request: {}", e))
                })
            });

        match network_request {
            Ok(net_req) => {
                let net_result = match net_req {
                    NetworkServiceRequest::SendMessage { peer_id, message } => {
                        let network_manager = self.network_manager.read().await;
                        network_manager
                            .send_message(NetworkMessage {
                                from: "self".to_string(),
                                to: peer_id.0,
                                content: message.content,
                            })
                            .await;

                        // Update stats
                        self.update_stats(|stats| stats.total_messages_sent += 1)
                            .await;

                        Ok(NetworkServiceResponse::MessageSent(Ok(())))
                    }
                    NetworkServiceRequest::BroadcastMessage(message) => {
                        // For now, just send to all connected peers
                        let network_manager = self.network_manager.read().await;
                        let peers = network_manager.get_connected_peers().await;
                        let peer_count = peers.len();

                        for addr in peers {
                            network_manager
                                .send_message(NetworkMessage {
                                    from: "self".to_string(),
                                    to: addr.to_string(),
                                    content: message.content.clone(),
                                })
                                .await;
                        }

                        // Update stats
                        self.update_stats(|stats| stats.total_messages_sent += peer_count as u64)
                            .await;

                        Ok(NetworkServiceResponse::MessageBroadcast(Ok(())))
                    }
                    NetworkServiceRequest::GetConnectedPeers => {
                        let network_manager = self.network_manager.read().await;
                        let peers = network_manager.get_connected_peers().await;
                        let peer_ids = peers.iter().map(|addr| PeerId(addr.to_string())).collect();

                        Ok(NetworkServiceResponse::ConnectedPeers(peer_ids))
                    }
                    NetworkServiceRequest::ConnectToPeer(addr) => {
                        let network_manager = self.network_manager.read().await;
                        match addr.0.parse() {
                            Ok(socket_addr) => {
                                network_manager.add_connected_peer(socket_addr).await;
                                self.update_stats(|stats| stats.connected_peers += 1).await;
                                Ok(NetworkServiceResponse::Connected(Ok(())))
                            }
                            Err(e) => Ok(NetworkServiceResponse::Connected(Err(format!(
                                "Invalid address: {}",
                                e
                            )))),
                        }
                    }
                    NetworkServiceRequest::DisconnectFromPeer(_peer_id) => {
                        // For simplicity, just decrement counter
                        self.update_stats(|stats| {
                            if stats.connected_peers > 0 {
                                stats.connected_peers -= 1;
                            }
                        })
                        .await;
                        Ok(NetworkServiceResponse::Disconnected(Ok(())))
                    }
                    NetworkServiceRequest::GetNetworkStats => {
                        let stats = self.stats.read().await;
                        Ok(NetworkServiceResponse::NetworkStats(stats.clone()))
                    }
                    NetworkServiceRequest::GetPeerInfo(peer_id) => {
                        // For now, return None - would need to integrate with Discovery
                        let _ = peer_id;
                        Ok(NetworkServiceResponse::PeerInfo(None))
                    }
                };

                Ok(self.network_response_to_service_response(request.id, net_result, start_time))
            }
            Err(_) => {
                // Fallback to base service handler
                self.base.handle_request(request).await
            }
        }
    }
}

/// Extension trait for NetworkService-specific operations
#[async_trait]
pub trait NetworkServiceExt: Service {
    /// Get the network statistics
    async fn get_network_stats(&self) -> Result<NetworkStats, ServiceError>;

    /// Broadcast a message to all peers
    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), ServiceError>;

    /// Send a message to a specific peer
    async fn send_message(
        &self,
        peer_id: PeerId,
        message: NetworkMessage,
    ) -> Result<(), ServiceError>;
}

#[async_trait]
impl NetworkServiceExt for NetworkServiceImpl {
    async fn get_network_stats(&self) -> Result<NetworkStats, ServiceError> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), ServiceError> {
        let request = NetworkServiceRequest::BroadcastMessage(message);
        let service_request = self.network_request_to_service_request(request);
        let response = self.handle_request(service_request).await?;

        if response.success {
            Ok(())
        } else {
            Err(ServiceError::RequestFailed(
                response.error.unwrap_or_default(),
            ))
        }
    }

    async fn send_message(
        &self,
        peer_id: PeerId,
        message: NetworkMessage,
    ) -> Result<(), ServiceError> {
        let request = NetworkServiceRequest::SendMessage { peer_id, message };
        let service_request = self.network_request_to_service_request(request);
        let response = self.handle_request(service_request).await?;

        if response.success {
            Ok(())
        } else {
            Err(ServiceError::RequestFailed(
                response.error.unwrap_or_default(),
            ))
        }
    }
}

/// Constructor for NetworkService
pub struct NetworkServiceConstructor {
    config: NetworkServiceConfig,
}

impl NetworkServiceConstructor {
    /// Create a new constructor
    pub fn new(config: NetworkServiceConfig) -> Self {
        Self { config }
    }

    /// Build the service
    pub fn build(self) -> NetworkServiceImpl {
        NetworkServiceImpl::new(self.config)
    }

    /// Create and register the service
    pub async fn create_and_register(
        self,
        registry: &ServiceRegistry,
    ) -> Result<ServiceId, ServiceError> {
        let service = Arc::new(self.build());
        let service_id = service.id();

        // Initialize the service
        service.initialize().await?;

        // Register with the registry
        registry.register(service).await?;

        Ok(service_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{ProtocolConfig, ResourceLimits, SecurityConfig};

    fn create_test_config() -> NetworkServiceConfig {
        let network_config = NetworkConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            bootstrap_peers: vec![],
            max_peers: 10,
            protocol_config: ProtocolConfig {},
            resource_limits: ResourceLimits {
                max_bandwidth: 1024 * 1024,
                max_memory: 512 * 1024 * 1024,
                max_connections: 10,
            },
            security_config: SecurityConfig {},
        };

        NetworkServiceConfig {
            network_config,
            name: "test-network".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
        }
    }

    #[tokio::test]
    async fn test_network_service_lifecycle() {
        let config = create_test_config();
        let service = NetworkServiceImpl::new(config);

        // Initialize
        service.initialize().await.expect("Failed to initialize");
        assert!(matches!(service.status().await, ServiceStatus::Starting));

        // Start
        service.start().await.expect("Failed to start");
        assert!(matches!(service.status().await, ServiceStatus::Running));

        // Get health
        let health = service.health().await;
        assert!(matches!(health.status, ServiceStatus::Running));
        assert!(health.uptime > Duration::ZERO);

        // Stop
        service.stop().await.expect("Failed to stop");
        assert!(matches!(service.status().await, ServiceStatus::Stopped));
    }

    #[tokio::test]
    async fn test_network_service_request_handling() {
        let config = create_test_config();
        let service = Arc::new(NetworkServiceImpl::new(config));

        // Initialize and start
        service.initialize().await.expect("Failed to initialize");
        service.start().await.expect("Failed to start");

        // Test GetConnectedPeers
        let request = NetworkServiceRequest::GetConnectedPeers;
        let service_request = service.network_request_to_service_request(request);
        let response = service
            .handle_request(service_request)
            .await
            .expect("Failed to handle request");

        if !response.success {
            eprintln!("Response failed: {:?}", response.error);
        }
        assert!(response.success);

        // Test GetNetworkStats
        let request = NetworkServiceRequest::GetNetworkStats;
        let service_request = service.network_request_to_service_request(request);
        let response = service
            .handle_request(service_request)
            .await
            .expect("Failed to handle request");

        if !response.success {
            eprintln!("Response failed: {:?}", response.error);
        }
        assert!(response.success);

        service.stop().await.expect("Failed to stop");
    }

    #[tokio::test]
    async fn test_network_service_send_message() {
        let config = create_test_config();
        let service = Arc::new(NetworkServiceImpl::new(config));

        service.initialize().await.expect("Failed to initialize");
        service.start().await.expect("Failed to start");

        let message = NetworkMessage {
            from: "test".to_string(),
            to: "peer".to_string(),
            content: b"test message".to_vec(),
        };

        let peer_id = PeerId("test-peer".to_string());
        let result = service.send_message(peer_id, message).await;
        assert!(result.is_ok());

        service.stop().await.expect("Failed to stop");
    }

    #[tokio::test]
    async fn test_network_service_broadcast() {
        let config = create_test_config();
        let service = Arc::new(NetworkServiceImpl::new(config));

        service.initialize().await.expect("Failed to initialize");
        service.start().await.expect("Failed to start");

        let message = NetworkMessage {
            from: "test".to_string(),
            to: "all".to_string(),
            content: b"broadcast message".to_vec(),
        };

        let result = service.broadcast_message(message).await;
        assert!(result.is_ok());

        service.stop().await.expect("Failed to stop");
    }

    #[tokio::test]
    async fn test_network_service_constructor() {
        let config = create_test_config();
        let constructor = NetworkServiceConstructor::new(config);
        let service = constructor.build();

        assert_eq!(service.name(), "test-network");
        assert_eq!(service.version(), "1.0.0");
    }

    #[tokio::test]
    async fn test_service_registration() {
        let config = create_test_config();
        let constructor = NetworkServiceConstructor::new(config);
        let registry = ServiceRegistry::new();

        let service_id = constructor
            .create_and_register(&registry)
            .await
            .expect("Failed to create and register");

        let retrieved_service = registry
            .get_service(&service_id)
            .await
            .expect("Service not found");
        assert_eq!(retrieved_service.name(), "test-network");

        // Cleanup
        let _ = registry.unregister(&service_id).await;
    }
}
