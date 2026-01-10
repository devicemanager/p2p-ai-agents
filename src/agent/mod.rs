//! Agent module for managing AI agents in the P2P network
//!
//! This module provides the core functionality for creating and managing
//! AI agents, including identity management, task handling, and resource
//! monitoring.

pub mod identity;
mod resource;
mod task;

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::core::services::{ServiceId, ServiceRegistry, ServiceRequest};

pub use identity::{Identity, IdentityError};
pub use resource::{ResourceError, ResourceMonitor, ResourceUsage};
pub use task::{
    Task, TaskError, TaskId, TaskPayload, TaskPriority, TaskResult, TaskStatus, TaskType,
};

/// Agent identifier type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    /// Create a new agent ID with a randomly generated UUID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create an agent ID from a string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent identifier
    pub id: AgentId,
    /// Network port to listen on
    pub network_port: u16,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Resource limits for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage (0.0 to 1.0)
    pub max_cpu: f32,
    /// Maximum memory usage in bytes
    pub max_memory: u64,
    /// Maximum storage usage in bytes
    pub max_storage: u64,
    /// Maximum network bandwidth in bytes per second
    pub max_bandwidth: u64,
    /// Maximum network connections
    pub max_connections: usize,
}

impl ResourceLimits {
    /// Validate resource limit values
    pub fn validate(&self) -> Result<()> {
        if self.max_cpu <= 0.0 || self.max_cpu > 1.0 {
            return Err(Error::InvalidConfig(format!(
                "max_cpu must be between 0.0 and 1.0, got {}",
                self.max_cpu
            )));
        }
        if self.max_memory == 0 {
            return Err(Error::InvalidConfig(
                "max_memory must be greater than 0".into(),
            ));
        }
        if self.max_storage == 0 {
            return Err(Error::InvalidConfig(
                "max_storage must be greater than 0".into(),
            ));
        }
        if self.max_bandwidth == 0 {
            return Err(Error::InvalidConfig(
                "max_bandwidth must be greater than 0".into(),
            ));
        }
        if self.max_connections == 0 {
            return Err(Error::InvalidConfig(
                "max_connections must be greater than 0".into(),
            ));
        }
        Ok(())
    }
}

/// Agent trait defining the core functionality
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get the agent's identifier
    fn id(&self) -> &AgentId;

    /// Get the agent's configuration
    fn config(&self) -> &AgentConfig;

    /// Start the agent
    async fn start(&self) -> Result<()>;

    /// Stop the agent
    async fn stop(&self) -> Result<()>;

    /// Get the agent's current status
    async fn status(&self) -> Result<AgentStatus>;

    /// Submit a task to the agent
    async fn submit_task(&self, task: Task) -> Result<TaskId>;

    /// Get the status of a task
    async fn task_status(&self, _task_id: &TaskId) -> Result<TaskStatus>;

    /// Get the agent's resource usage
    async fn resource_usage(&self) -> Result<ResourceUsage>;

    /// Connect to the P2P network
    async fn connect_to_network(&self, bootstrap_nodes: Vec<String>) -> Result<()>;

    /// Disconnect from the P2P network
    async fn disconnect_from_network(&self) -> Result<()>;

    /// Get the network status
    async fn network_status(&self) -> Result<NetworkStatus>;

    /// Broadcast a task to the network
    async fn broadcast_task(&self, task: Task) -> Result<()>;

    /// Discover available peers
    async fn discover_peers(&self) -> Result<Vec<PeerInfo>>;
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Whether the agent is running
    pub is_running: bool,
    /// Number of active tasks
    pub active_tasks: usize,
    /// Current resource usage
    pub resource_usage: ResourceUsage,
    /// Uptime in seconds
    pub uptime: u64,
    /// Network status
    pub network_status: NetworkStatus,
}

/// Network status information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkStatus {
    /// Whether connected to the network
    pub is_connected: bool,
    /// Number of connected peers
    pub peer_count: usize,
    /// Last connection error (if any)
    pub last_error: Option<String>,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: String,
    /// Peer address
    pub address: String,
    /// Peer capabilities
    pub capabilities: Vec<String>,
}

/// Result type for agent operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for agent operations
#[derive(Debug, Error)]
pub enum Error {
    /// Identity-related errors
    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),

    /// Task-related errors
    #[error("Task error: {0}")]
    Task(#[from] TaskError),

    /// Resource-related errors
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),

    /// Agent is not running
    #[error("Agent is not running")]
    NotRunning,

    /// Agent is already running
    #[error("Agent is already running")]
    AlreadyRunning,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Default implementation of the Agent trait
pub struct DefaultAgent {
    id: AgentId,
    config: AgentConfig,
    resource_monitor: Arc<ResourceMonitor>,
    service_registry: Arc<ServiceRegistry>,
    network_service_id: Arc<RwLock<Option<ServiceId>>>,
}

impl DefaultAgent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig, service_registry: Arc<ServiceRegistry>) -> Result<Self> {
        let resource_monitor = ResourceMonitor::new(&config.resource_limits)?;

        Ok(Self {
            id: config.id.clone(),
            config: config.clone(),
            resource_monitor: Arc::new(resource_monitor),
            service_registry: service_registry.clone(),
            network_service_id: Arc::new(RwLock::new(None)),
        })
    }
}

#[async_trait]
impl Agent for DefaultAgent {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn start(&self) -> Result<()> {
        // TODO: Implement agent startup
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // TODO: Implement agent shutdown
        Ok(())
    }

    async fn status(&self) -> Result<AgentStatus> {
        let network_status = self.network_status().await?;

        Ok(AgentStatus {
            is_running: true,
            active_tasks: 0,
            resource_usage: self.resource_monitor.current_usage().await?,
            uptime: 0,
            network_status,
        })
    }

    async fn submit_task(&self, task: Task) -> Result<TaskId> {
        // TODO: Implement task submission
        Ok(task.id().clone())
    }

    async fn task_status(&self, _task_id: &TaskId) -> Result<TaskStatus> {
        // TODO: Implement task status check
        Ok(TaskStatus::Pending)
    }

    async fn resource_usage(&self) -> Result<ResourceUsage> {
        self.resource_monitor
            .current_usage()
            .await
            .map_err(Error::Resource)
    }

    async fn connect_to_network(&self, bootstrap_nodes: Vec<String>) -> Result<()> {
        let mut peer_infos = Vec::new();
        for node in bootstrap_nodes {
            peer_infos.push(crate::network::PeerInfo {
                peer_id: crate::network::PeerId(node.clone()),
                addresses: vec![],
                last_seen: chrono::Utc::now(),
                reputation: 0,
                capabilities: crate::network::PeerCapabilities {},
                status: crate::network::ConnectionStatus::Disconnected,
            });
        }

        let network_config = crate::network::service::NetworkServiceConfig {
            network_config: crate::network::NetworkConfig {
                listen_addr: "0.0.0.0:8000".parse().unwrap(),
                bootstrap_peers: peer_infos,
                max_peers: self.config.resource_limits.max_connections,
                protocol_config: crate::network::ProtocolConfig {},
                resource_limits: crate::network::ResourceLimits {
                    max_bandwidth: self.config.resource_limits.max_bandwidth,
                    max_memory: self.config.resource_limits.max_memory,
                    max_connections: self.config.resource_limits.max_connections,
                },
                security_config: crate::network::SecurityConfig {
                    trusted_authorities: vec![],
                    local_certificate: None,
                },
            },
            name: format!("network-{}-{}", self.id().as_str(), Uuid::new_v4()),
            version: "0.1.0".to_string(),
            dependencies: vec![],
        };

        let constructor = crate::network::service::NetworkServiceConstructor::new(network_config);
        let service_id = constructor
            .create_and_register(&self.service_registry)
            .await
            .map_err(|e| Error::Internal(format!("Failed to create network service: {}", e)))?;

        // Store the service ID
        let mut service_id_guard = self.network_service_id.write().await;
        *service_id_guard = Some(service_id.clone());
        drop(service_id_guard);

        // Start the network service
        if let Some(service) = self.service_registry.get_service(&service_id).await {
            service
                .start()
                .await
                .map_err(|e| Error::Internal(format!("Failed to start network service: {}", e)))?;
        } else {
            return Err(Error::Internal(
                "Network service not found after registration".to_string(),
            ));
        }

        info!("Agent {} connected to network", self.id().as_str());
        Ok(())
    }

    async fn disconnect_from_network(&self) -> Result<()> {
        let service_id_guard = self.network_service_id.read().await;
        if let Some(service_id) = service_id_guard.as_ref() {
            if let Some(service) = self.service_registry.get_service(service_id).await {
                service.stop().await.map_err(|e| {
                    Error::Internal(format!("Failed to stop network service: {}", e))
                })?;
            }

            // Reset service ID
            drop(service_id_guard);
            let mut service_id_guard = self.network_service_id.write().await;
            *service_id_guard = None;

            info!("Agent {} disconnected from network", self.id().as_str());
        }
        Ok(())
    }

    async fn network_status(&self) -> Result<NetworkStatus> {
        let service_id_guard = self.network_service_id.read().await;
        if let Some(service_id) = service_id_guard.as_ref() {
            if let Some(service) = self.service_registry.get_service(service_id).await {
                let health = service.health().await;
                let status = service.status().await;

                let mut network_status = NetworkStatus::default();

                // Extract network-specific metrics from health
                if let Some(peer_count) = health.metrics.get("connected_peers") {
                    if let Some(count) = peer_count.as_u64() {
                        network_status.peer_count = count as usize;
                    }
                }

                network_status.is_connected =
                    matches!(status, crate::core::services::ServiceStatus::Running);

                if let Some(error_msg) = health.metrics.get("error") {
                    network_status.last_error = Some(error_msg.to_string());
                }

                Ok(network_status)
            } else {
                Ok(NetworkStatus::default())
            }
        } else {
            Ok(NetworkStatus::default())
        }
    }

    async fn broadcast_task(&self, task: Task) -> Result<()> {
        let service_id_guard = self.network_service_id.read().await;
        if let Some(service_id) = service_id_guard.as_ref() {
            if let Some(service) = self.service_registry.get_service(service_id).await {
                // Convert task to network message
                let message = crate::network::NetworkMessage {
                    from: self.id().as_str().to_string(),
                    to: "broadcast".to_string(),
                    content: serde_json::to_vec(&task)
                        .map_err(|e| Error::Internal(format!("Failed to serialize task: {}", e)))?,
                };

                let mut params = std::collections::HashMap::new();
                params.insert(
                    "message".to_string(),
                    serde_json::to_value(message).map_err(|e| {
                        Error::Internal(format!("Failed to serialize message: {}", e))
                    })?,
                );

                let response = service
                    .handle_request(ServiceRequest {
                        id: Uuid::new_v4(),
                        method: "broadcast_message".to_string(),
                        parameters: params,
                        timeout: Some(std::time::Duration::from_secs(30)),
                    })
                    .await
                    .map_err(|e| Error::Internal(format!("Failed to broadcast task: {}", e)))?;

                if response.success {
                    info!(
                        "Agent {} broadcast task {} to network",
                        self.id().as_str(),
                        task.id().as_str()
                    );
                    Ok(())
                } else {
                    Err(Error::Internal(format!(
                        "Broadcast failed: {}",
                        response.error.unwrap_or_default()
                    )))
                }
            } else {
                Err(Error::Internal("Network service not found".to_string()))
            }
        } else {
            Err(Error::NotRunning)
        }
    }

    async fn discover_peers(&self) -> Result<Vec<PeerInfo>> {
        let service_id_guard = self.network_service_id.read().await;
        if let Some(service_id) = service_id_guard.as_ref() {
            if let Some(service) = self.service_registry.get_service(service_id).await {
                let response = service
                    .handle_request(ServiceRequest {
                        id: Uuid::new_v4(),
                        method: "get_connected_peers".to_string(),
                        parameters: std::collections::HashMap::new(),
                        timeout: Some(std::time::Duration::from_secs(30)),
                    })
                    .await
                    .map_err(|e| Error::Internal(format!("Failed to discover peers: {}", e)))?;

                if response.success {
                    if let Some(data) = response.data {
                        let peer_ids: Vec<crate::network::PeerId> = serde_json::from_value(data)
                            .map_err(|e| {
                                Error::Internal(format!("Failed to parse peer IDs: {}", e))
                            })?;

                        let peers = peer_ids
                            .iter()
                            .map(|peer_id| PeerInfo {
                                peer_id: peer_id.0.clone(),
                                address: "unknown".to_string(),
                                capabilities: vec!["task_execution".to_string()],
                            })
                            .collect();

                        Ok(peers)
                    } else {
                        Ok(vec![])
                    }
                } else {
                    Err(Error::Internal(format!(
                        "Peer discovery failed: {}",
                        response.error.unwrap_or_default()
                    )))
                }
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AgentId tests
    #[test]
    fn test_agent_id_new() {
        let id1 = AgentId::new();
        let id2 = AgentId::new();

        // Each ID should be unique
        assert_ne!(id1, id2);
        assert_ne!(id1.as_str(), id2.as_str());

        // Should be valid UUID format
        assert!(uuid::Uuid::parse_str(id1.as_str()).is_ok());
        assert!(uuid::Uuid::parse_str(id2.as_str()).is_ok());
    }

    #[test]
    fn test_agent_id_from_string() {
        let test_id = "test-agent-id-456";
        let agent_id = AgentId::from_string(test_id.to_string());

        assert_eq!(agent_id.as_str(), test_id);
        assert_eq!(agent_id.to_string(), test_id);
    }

    #[test]
    fn test_agent_id_default() {
        let id1 = AgentId::default();
        let id2 = AgentId::default();

        // Default should create unique IDs
        assert_ne!(id1, id2);
        assert!(uuid::Uuid::parse_str(id1.as_str()).is_ok());
    }

    #[test]
    fn test_agent_id_display() {
        let test_str = "display-agent-test";
        let agent_id = AgentId::from_string(test_str.to_string());

        assert_eq!(format!("{}", agent_id), test_str);
    }

    #[test]
    fn test_agent_id_serialization() {
        let id = AgentId::from_string("test-123".to_string());

        // Test JSON serialization
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""test-123""#);

        // Test deserialization
        let deserialized: AgentId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_agent_id_serialization_roundtrip() {
        let original = AgentId::new();

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: AgentId = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    // AgentConfig tests
    #[test]
    fn test_agent_config_creation() {
        let agent_id = AgentId::from_string("config-test-agent".to_string());
        let limits = ResourceLimits {
            max_cpu: 0.5,
            max_memory: 512 * 1024 * 1024,       // 512MB
            max_storage: 5 * 1024 * 1024 * 1024, // 5GB
            max_bandwidth: 512 * 1024,           // 512KB/s
            max_connections: 10,
        };

        let config = AgentConfig {
            id: agent_id.clone(),
            network_port: 8080,
            resource_limits: limits.clone(),
        };

        assert_eq!(config.id, agent_id);
        assert_eq!(config.resource_limits.max_cpu, limits.max_cpu);
        assert_eq!(config.resource_limits.max_memory, limits.max_memory);
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            id: AgentId::from_string("agent-1".to_string()),
            network_port: 8080,
            resource_limits: ResourceLimits {
                max_cpu: 0.8,
                max_memory: 1024 * 1024 * 1024,
                max_storage: 10 * 1024 * 1024 * 1024,
                max_bandwidth: 1024 * 1024,
                max_connections: 100,
            },
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"agent-1\""));
        assert!(json.contains("max_cpu"));

        // Deserialize back
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.id, deserialized.id);
        assert_eq!(
            config.resource_limits.max_cpu,
            deserialized.resource_limits.max_cpu
        );
    }

    #[test]
    fn test_resource_limits_serialization() {
        let limits = ResourceLimits {
            max_cpu: 0.75,
            max_memory: 2048 * 1024 * 1024,
            max_storage: 50 * 1024 * 1024 * 1024,
            max_bandwidth: 5 * 1024 * 1024,
            max_connections: 200,
        };

        let json = serde_json::to_string(&limits).unwrap();
        let deserialized: ResourceLimits = serde_json::from_str(&json).unwrap();

        assert_eq!(limits.max_cpu, deserialized.max_cpu);
        assert_eq!(limits.max_memory, deserialized.max_memory);
        assert_eq!(limits.max_storage, deserialized.max_storage);
        assert_eq!(limits.max_bandwidth, deserialized.max_bandwidth);
        assert_eq!(limits.max_connections, deserialized.max_connections);
    }

    #[test]
    fn test_resource_limits_validation_valid() {
        let limits = ResourceLimits {
            max_cpu: 0.8,
            max_memory: 1024 * 1024 * 1024,
            max_storage: 10 * 1024 * 1024 * 1024,
            max_bandwidth: 1024 * 1024,
            max_connections: 100,
        };

        assert!(limits.validate().is_ok());
    }

    #[test]
    fn test_resource_limits_validation_invalid_cpu_too_high() {
        let limits = ResourceLimits {
            max_cpu: 1.5,
            max_memory: 1024,
            max_storage: 1024,
            max_bandwidth: 1024,
            max_connections: 10,
        };

        assert!(limits.validate().is_err());
    }

    #[test]
    fn test_resource_limits_validation_invalid_cpu_zero() {
        let limits = ResourceLimits {
            max_cpu: 0.0,
            max_memory: 1024,
            max_storage: 1024,
            max_bandwidth: 1024,
            max_connections: 10,
        };

        assert!(limits.validate().is_err());
    }

    #[test]
    fn test_resource_limits_validation_invalid_memory_zero() {
        let limits = ResourceLimits {
            max_cpu: 0.5,
            max_memory: 0,
            max_storage: 1024,
            max_bandwidth: 1024,
            max_connections: 10,
        };

        assert!(limits.validate().is_err());
    }

    #[test]
    fn test_resource_limits_validation_invalid_connections_zero() {
        let limits = ResourceLimits {
            max_cpu: 0.5,
            max_memory: 1024,
            max_storage: 1024,
            max_bandwidth: 1024,
            max_connections: 0,
        };

        assert!(limits.validate().is_err());
    }

    // Existing test
    #[tokio::test]
    async fn test_agent_creation() {
        let config = AgentConfig {
            id: AgentId("test-agent".to_string()),
            network_port: 8080,
            resource_limits: ResourceLimits {
                max_cpu: 0.8,
                max_memory: 1024 * 1024 * 1024,       // 1GB
                max_storage: 10 * 1024 * 1024 * 1024, // 10GB
                max_bandwidth: 1024 * 1024,           // 1MB/s
                max_connections: 100,
            },
        };

        let registry = Arc::new(ServiceRegistry::new());
        let agent = DefaultAgent::new(config, registry).await.unwrap();
        assert_eq!(agent.id().0, "test-agent");
    }

    #[tokio::test]
    async fn test_agent_network_integration() {
        let config = AgentConfig {
            id: AgentId::new(),
            network_port: 8080,
            resource_limits: ResourceLimits {
                max_cpu: 0.5,
                max_memory: 512 * 1024 * 1024,
                max_storage: 5 * 1024 * 1024 * 1024,
                max_bandwidth: 512 * 1024,
                max_connections: 10,
            },
        };

        let registry = Arc::new(ServiceRegistry::new());
        let agent = DefaultAgent::new(config, registry).await.unwrap();

        // Test initial network status
        let status = agent.network_status().await.unwrap();
        assert!(!status.is_connected);
        assert_eq!(status.peer_count, 0);

        // Test overall agent status
        let agent_status = agent.status().await.unwrap();
        assert!(agent_status.is_running);
        assert!(!agent_status.network_status.is_connected);
    }
}
