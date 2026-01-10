//! Task management module for Agents.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Unique identifier for a task.
pub type TaskId = Uuid;

/// Status of a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is created but not started.
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed(String),
}

/// A unit of work to be performed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique ID of the task.
    pub id: TaskId,
    /// Description of the work.
    pub description: String,
    /// Current status.
    pub status: TaskStatus,
    // Add more fields like input data, result, priority, etc.
}

impl Task {
    /// Creates a new task.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.into(),
            status: TaskStatus::Pending,
        }
    }
}

/// Manages tasks for an agent.
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
}

impl TaskManager {
    /// Creates a new TaskManager.
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submits a new task.
    pub async fn submit_task(&self, description: impl Into<String>) -> TaskId {
        let task = Task::new(description);
        let id = task.id;
        self.tasks.write().await.insert(id, task);
        id
    }

    /// Updates the status of a task.
    pub async fn update_status(&self, id: TaskId, status: TaskStatus) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            task.status = status;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }

    /// Retrieves a task by ID.
    pub async fn get_task(&self, id: TaskId) -> Option<Task> {
        self.tasks.read().await.get(&id).cloned()
    }
    
    /// Lists all tasks.
    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.values().cloned().collect()
    }
}
