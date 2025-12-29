//! Agent module for managing AI agents in the P2P network
//!
//! This module provides the core functionality for creating and managing
//! AI agents, including identity management, task handling, and resource
//! monitoring.

mod identity;
mod resource;
mod task;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

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
    async fn task_status(&self, _task_id: &TaskId) -> Result<TaskStatus>;

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
    // TODO: Add other fields as needed
}

impl DefaultAgent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self> {
        let resource_monitor = ResourceMonitor::new(&config.resource_limits)?;

        Ok(Self {
            id: config.id.clone(),
            config,
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

    // AgentConfig tests
    #[test]
    fn test_agent_config_creation() {
        let agent_id = AgentId::from_string("config-test-agent".to_string());
        let limits = ResourceLimits {
            max_cpu: 0.5,
            max_memory: 512 * 1024 * 1024,       // 512MB
            max_storage: 5 * 1024 * 1024 * 1024, // 5GB
            max_bandwidth: 512 * 1024,           // 512KB/s
        };

        let config = AgentConfig {
            id: agent_id.clone(),
            resource_limits: limits.clone(),
        };

        assert_eq!(config.id, agent_id);
        assert_eq!(config.resource_limits.max_cpu, limits.max_cpu);
        assert_eq!(config.resource_limits.max_memory, limits.max_memory);
    }

    // Existing test
    #[tokio::test]
    async fn test_agent_creation() {
        let config = AgentConfig {
            id: AgentId("test-agent".to_string()),
            resource_limits: ResourceLimits {
                max_cpu: 0.8,
                max_memory: 1024 * 1024 * 1024,       // 1GB
                max_storage: 10 * 1024 * 1024 * 1024, // 10GB
                max_bandwidth: 1024 * 1024,           // 1MB/s
            },
        };

        let agent = DefaultAgent::new(config).await.unwrap();
        assert_eq!(agent.id().0, "test-agent");
    }
}
