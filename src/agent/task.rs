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

/// Priority level of a task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Type of task to perform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    /// Basic text processing
    TextProcessing,
    /// Vector computation
    VectorComputation,
    /// Custom task type
    Custom(String),
}

/// Payload containing task data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload {
    pub task_type: TaskType,
    pub data: serde_json::Value,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// A unit of work to be performed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique ID of the task.
    pub id: TaskId,
    /// Priority of the task.
    pub priority: TaskPriority,
    /// Task payload.
    pub payload: Option<TaskPayload>,
    /// Description of the work.
    pub description: String,
    /// Current status.
    pub status: TaskStatus,
    // Add more fields like input data, result, priority, etc.
}

impl Task {
    /// Creates a new task with a description (legacy helper).
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            priority: TaskPriority::Normal,
            payload: None,
            description: description.into(),
            status: TaskStatus::Pending,
        }
    }

    /// Creates a new task with priority and payload.
    pub fn with_payload(priority: TaskPriority, payload: TaskPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            priority,
            payload: Some(payload),
            description: String::new(), // Description optional when payload used
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

    /// Adds a fully formed task.
    pub async fn add_task(&self, task: Task) -> TaskId {
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
