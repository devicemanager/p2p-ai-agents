//! Agent Module
//!
//! This module contains the core agent logic, including identity management,
//! messaging, and task execution.

pub mod identity;
pub mod messaging;
pub mod task;

use crate::agent::identity::AgentIdentity;
use crate::agent::task::{Task, TaskId, TaskManager, TaskStatus};
use crate::core::identity::IdentityError;

/// Unique identifier for an Agent.
pub type AgentId = String;

/// Configuration for an Agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    /// The unique name or identifier for the agent.
    pub name: String,
    // Add other config fields if necessary, currently minimal for compilation
}

/// Core Agent structure representing a node in the network.
pub struct Agent {
    /// The agent's identity and trust components.
    pub identity: AgentIdentity,
    /// The agent's configuration.
    pub config: AgentConfig,
    /// The agent's task manager.
    pub task_manager: TaskManager,
}

impl Agent {
    /// Creates a new Agent instance.
    pub fn new(identity: AgentIdentity, config: AgentConfig) -> Self {
        Self {
            identity,
            config,
            task_manager: TaskManager::new(),
        }
    }
    
    /// Returns the Agent's ID.
    pub fn id(&self) -> AgentId {
         // Fallback to name for now, should be DID
         self.config.name.clone()
    }
    
    /// Starts the Agent.
    pub async fn start(&self) -> anyhow::Result<()> {
        // Implement start logic
        Ok(())
    }
    
    /// Stops the Agent.
    pub async fn stop(&self) -> anyhow::Result<()> {
        // Implement stop logic
        Ok(())
    }

    /// Submits a task to the agent.
    pub async fn submit_task(&self, task: Task) -> TaskId {
        // For now, we just add it to the manager. 
        // In the future, this would trigger processing.
        self.task_manager.add_task(task).await
    }

    /// Gets the status of a task.
    pub async fn task_status(&self, id: &TaskId) -> anyhow::Result<TaskStatus> {
        let task = self.task_manager.get_task(*id).await;
        match task {
            Some(t) => Ok(t.status),
            None => Err(anyhow::anyhow!("Task not found")),
        }
    }
}

/// Errors specific to the Agent module.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Identity-related error.
    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),
    // Add other agent errors here
}

use std::sync::Arc;
use crate::core::services::ServiceRegistry;

/// A default implementation of an Agent for testing and examples.
pub struct DefaultAgent {
    agent: Arc<Agent>,
    // You might want to hold the ServiceRegistry here if needed for lifecycle management
    _service_registry: Option<Arc<ServiceRegistry>>,
}

impl DefaultAgent {
    /// Creates a new DefaultAgent with the given configuration.
    pub async fn new(config: AgentConfig) -> anyhow::Result<Self> {
        // Initialize identity with default depth 20 and initial root 0
        let identity = AgentIdentity::new(20, semaphore::Field::from(0)).await?;
        let agent = Arc::new(Agent::new(identity, config));
        
        Ok(Self {
            agent,
            _service_registry: None,
        })
    }

    /// Creates a new DefaultAgent with a provided ServiceRegistry.
    pub async fn with_services(config: AgentConfig, registry: Arc<ServiceRegistry>) -> anyhow::Result<Self> {
         let identity = AgentIdentity::new(20, semaphore::Field::from(0)).await?;
         let agent = Arc::new(Agent::new(identity, config));

         Ok(Self {
             agent,
             _service_registry: Some(registry),
         })
    }

    /// Delegate start to the inner Agent
    pub async fn start(&self) -> anyhow::Result<()> {
        self.agent.start().await
    }

    /// Delegate stop to the inner Agent
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.agent.stop().await
    }

    /// Delegate id to the inner Agent
    pub fn id(&self) -> AgentId {
        self.agent.id()
    }
    
    /// Get status (Mock implementation for example)
    pub async fn status(&self) -> anyhow::Result<AgentStatus> {
        Ok(AgentStatus { is_running: true })
    }

    /// Submit a task to the agent
    pub async fn submit_task(&self, task: Task) -> anyhow::Result<TaskId> {
        Ok(self.agent.submit_task(task).await)
    }

    /// Get the status of a submitted task
    pub async fn task_status(&self, id: &TaskId) -> anyhow::Result<TaskStatus> {
        self.agent.task_status(id).await
    }
}

/// Simple status struct for the example
#[derive(Debug)]
pub struct AgentStatus {
    /// Whether the agent is currently running.
    pub is_running: bool,
}

/// Resource limits placeholder for the example
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage.
    pub max_cpu: f32,
    /// Maximum memory usage in bytes.
    pub max_memory: u64,
    /// Maximum storage usage in bytes.
    pub max_storage: u64,
    /// Maximum bandwidth usage in bytes/sec.
    pub max_bandwidth: u64,
    /// Maximum number of concurrent connections.
    pub max_connections: u32,
}
