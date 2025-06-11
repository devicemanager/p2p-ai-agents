//! Agent module for managing AI agents in the P2P network
//! 
//! This module provides the core functionality for creating and managing
//! AI agents, including identity management, task handling, and resource
//! monitoring.

mod identity;
mod task;
mod resource;

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

pub use identity::{Identity, IdentityError};
pub use task::{Task, TaskId, TaskStatus, TaskError};
pub use resource::{ResourceMonitor, ResourceError};

/// Agent identifier type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent identifier
    pub id: AgentId,
    /// Network configuration
    pub network: crate::network::NetworkConfig,
    /// Storage configuration
    pub storage: crate::storage::StorageConfig,
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
    async fn task_status(&self, task_id: &TaskId) -> Result<TaskStatus>;

    /// Get the agent's resource usage
    async fn resource_usage(&self) -> Result<ResourceUsage>;
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
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage (0.0 to 1.0)
    pub cpu: f32,
    /// Memory usage in bytes
    pub memory: u64,
    /// Storage usage in bytes
    pub storage: u64,
    /// Network bandwidth usage in bytes per second
    pub bandwidth: u64,
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
    identity: Arc<Identity>,
    resource_monitor: Arc<ResourceMonitor>,
    // TODO: Add other fields as needed
}

impl DefaultAgent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self> {
        let identity = Identity::new()?;
        let resource_monitor = ResourceMonitor::new(&config.resource_limits)?;

        Ok(Self {
            id: config.id.clone(),
            config,
            identity: Arc::new(identity),
            resource_monitor: Arc::new(resource_monitor),
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
        // TODO: Implement status reporting
        Ok(AgentStatus {
            is_running: true,
            active_tasks: 0,
            resource_usage: self.resource_monitor.current_usage().await?,
            uptime: 0,
        })
    }

    async fn submit_task(&self, task: Task) -> Result<TaskId> {
        // TODO: Implement task submission
        Ok(task.id().clone())
    }

    async fn task_status(&self, task_id: &TaskId) -> Result<TaskStatus> {
        // TODO: Implement task status check
        Ok(TaskStatus::Pending)
    }

    async fn resource_usage(&self) -> Result<ResourceUsage> {
        self.resource_monitor.current_usage().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let config = AgentConfig {
            id: AgentId("test-agent".to_string()),
            network: crate::network::NetworkConfig::default(),
            storage: crate::storage::StorageConfig::default(),
            resource_limits: ResourceLimits {
                max_cpu: 0.8,
                max_memory: 1024 * 1024 * 1024, // 1GB
                max_storage: 10 * 1024 * 1024 * 1024, // 10GB
                max_bandwidth: 1024 * 1024, // 1MB/s
            },
        };

        let agent = DefaultAgent::new(config).await.unwrap();
        assert_eq!(agent.id().0, "test-agent");
    }
} 