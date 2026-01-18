//! Task management module.
//!
//! This module defines the core data structures and logic for distributed task
//! management, including:
//! - `Task`: The fundamental unit of work
//! - `TaskResult`: The outcome of a task execution
//! - `TaskStatus`: The lifecycle state of a task
//! - `TaskManager`: The high-level service for managing task lifecycles
//! - `TaskExecutor`: The service responsible for executing tasks (mock for MVP)

/// Executor submodule for task execution logic.
pub mod executor;
/// Manager submodule for task lifecycle management.
pub mod manager;

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Represents a unit of work to be distributed and executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task.
    pub id: Uuid,
    /// The actual prompt or instruction for the AI model.
    pub prompt: String,
    /// The PeerId of the agent that created/requested this task.
    /// Note: PeerId doesn't implement Serialize/Deserialize by default in all versions,
    /// so we often convert to/from string or bytes. For MVP, we assume
    /// standard Serde support is available or handled by the transport layer.
    /// If not, we might need a wrapper or custom serializer.
    /// libp2p PeerId does implement Serialize/Deserialize if features are enabled.
    pub sender: PeerId,
    /// Unix timestamp (seconds) when the task was created.
    pub created_at: u64,
    /// Priority level (0-255, higher is more urgent). Default: 5.
    pub priority: u8,
}

impl Task {
    /// Creates a new task with a generated UUID and current timestamp.
    pub fn new(prompt: String, sender: PeerId) -> Self {
        Self {
            id: Uuid::new_v4(),
            prompt,
            sender,
            created_at: chrono::Utc::now().timestamp() as u64,
            priority: 5,
        }
    }
}

/// The result of an executed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// The ID of the task this result belongs to.
    pub task_id: Uuid,
    /// The output of the execution (e.g., the model's response).
    pub result: String,
    /// How long execution took in milliseconds.
    pub duration_ms: u64,
    /// Unix timestamp when execution finished.
    pub completed_at: u64,
}

/// The lifecycle status of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task has been created but not yet assigned or started.
    Pending,
    /// Task is currently being processed.
    Executing,
    /// Task completed successfully.
    Completed,
    /// Task failed during execution.
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Executing => write!(f, "Executing"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    #[test]
    fn test_task_creation() {
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        let task = Task::new("Hello World".to_string(), peer_id);

        assert_eq!(task.prompt, "Hello World");
        assert_eq!(task.sender, peer_id);
        assert_eq!(task.priority, 5);
        assert!(task.created_at > 0);
    }

    #[test]
    fn test_task_serialization() {
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        let task = Task::new("Serialize me".to_string(), peer_id);

        let json = serde_json::to_string(&task).expect("Failed to serialize task");
        let deserialized: Task = serde_json::from_str(&json).expect("Failed to deserialize task");

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.prompt, deserialized.prompt);
        assert_eq!(task.sender, deserialized.sender);
    }
}
