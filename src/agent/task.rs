//! Task management for agents
//!
//! This module provides functionality for creating, managing, and
//! executing tasks in the agent network.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Task identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(String);

impl TaskId {
    /// Create a new task ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a task ID from a string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get the task ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is pending execution
    Pending,
    /// Task is currently running
    Running,
    /// Task has completed successfully
    Completed,
    /// Task has failed
    Failed(String),
    /// Task has been cancelled
    Cancelled,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Low priority task
    Low = 0,
    /// Normal priority task
    Normal = 1,
    /// High priority task
    High = 2,
    /// Critical priority task
    Critical = 3,
}

/// Task type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// Text processing task
    TextProcessing,
    /// Vector computation task
    VectorComputation,
    /// Model inference task
    ModelInference,
    /// Data storage task
    Storage,
    /// Custom task type
    Custom(String),
}

/// Task payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload {
    /// Task type
    pub task_type: TaskType,
    /// Task data
    pub data: serde_json::Value,
    /// Task parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task output
    pub output: serde_json::Value,
    /// Task metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task identifier
    id: TaskId,
    /// Task priority
    priority: TaskPriority,
    /// Task payload
    payload: TaskPayload,
    /// Task status
    status: TaskStatus,
    /// Task result
    result: Option<TaskResult>,
    /// Task creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Task completion timestamp
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Error type for task operations
#[derive(Debug, Error)]
pub enum TaskError {
    /// Task not found
    #[error("Task not found: {0}")]
    NotFound(TaskId),

    /// Task already exists
    #[error("Task already exists: {0}")]
    AlreadyExists(TaskId),

    /// Invalid task state
    #[error("Invalid task state: {0}")]
    InvalidState(String),

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    /// Task cancelled
    #[error("Task cancelled: {0}")]
    Cancelled(TaskId),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for task operations
#[allow(dead_code)]
pub type TaskResultType<T> = std::result::Result<T, TaskError>;

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, _task: &Task) -> TaskResultType<TaskResult>;
}

#[allow(dead_code)]
pub struct TaskManager<E: TaskExecutor> {
    executor: std::sync::Arc<E>,
    tasks: tokio::sync::Mutex<std::collections::HashMap<TaskId, Task>>,
}

#[allow(dead_code)]
impl<E: TaskExecutor> TaskManager<E> {
    pub fn new(executor: std::sync::Arc<E>) -> Self {
        TaskManager {
            executor,
            tasks: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
    pub async fn submit_task(&self, task: Task) -> TaskResultType<TaskId> {
        let mut tasks = self.tasks.lock().await;
        let id = TaskId::new();
        tasks.insert(id.clone(), task);
        Ok(id)
    }
    pub async fn get_task(&self, task_id: &TaskId) -> TaskResultType<Task> {
        let tasks = self.tasks.lock().await;
        tasks
            .get(task_id)
            .cloned()
            .ok_or(TaskError::NotFound(task_id.clone()))
    }
    pub async fn execute_task(&self, task_id: &TaskId) -> TaskResultType<TaskResult> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or(TaskError::NotFound(task_id.clone()))?;
        let result = self.executor.execute(task).await?;
        task.set_result(result.clone());
        Ok(result)
    }
    pub async fn cancel_task(&self, task_id: &TaskId) -> TaskResultType<()> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or(TaskError::NotFound(task_id.clone()))?;
        task.set_status(TaskStatus::Cancelled);
        Ok(())
    }
}

impl Task {
    /// Create a new task
    pub fn new(priority: TaskPriority, payload: TaskPayload) -> Self {
        Self {
            id: TaskId::new(),
            priority,
            payload,
            status: TaskStatus::Pending,
            result: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }

    /// Get the task ID
    pub fn id(&self) -> &TaskId {
        &self.id
    }

    /// Get the task priority
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    /// Get the task payload
    pub fn payload(&self) -> &TaskPayload {
        &self.payload
    }

    /// Get the task status
    pub fn status(&self) -> &TaskStatus {
        &self.status
    }

    /// Get the task result
    pub fn result(&self) -> Option<&TaskResult> {
        self.result.as_ref()
    }

    /// Get the task creation timestamp
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    /// Get the task completion timestamp
    pub fn completed_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.completed_at
    }

    /// Set the task status
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status.clone();
        if matches!(
            status,
            TaskStatus::Completed | TaskStatus::Failed(_) | TaskStatus::Cancelled
        ) {
            self.completed_at = Some(chrono::Utc::now());
        }
    }

    /// Set the task result
    pub fn set_result(&mut self, result: TaskResult) {
        self.result = Some(result);
        self.status = TaskStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::Duration;

    struct MockExecutor;

    #[async_trait::async_trait]
    impl TaskExecutor for MockExecutor {
        async fn execute(&self, _task: &Task) -> TaskResultType<TaskResult> {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(TaskResult {
                output: serde_json::json!({"result": "success"}),
                metadata: std::collections::HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_task_lifecycle() {
        let executor = Arc::new(MockExecutor);
        let manager = TaskManager::new(executor);

        // Create and submit a task
        let task = Task::new(
            TaskPriority::Normal,
            TaskPayload {
                task_type: TaskType::TextProcessing,
                data: serde_json::json!({"text": "Hello, world!"}),
                parameters: HashMap::new(),
            },
        );
        let task_id = manager.submit_task(task).await.unwrap();

        // Verify task is pending
        let task = manager.get_task(&task_id).await.unwrap();
        assert!(matches!(task.status, TaskStatus::Pending));

        // Execute task
        let result = manager.execute_task(&task_id).await.unwrap();
        assert_eq!(result.output["result"], "success");

        // Verify task is completed
        let task = manager.get_task(&task_id).await.unwrap();
        assert!(matches!(task.status, TaskStatus::Completed));
        assert!(task.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let executor = Arc::new(MockExecutor);
        let manager = TaskManager::new(executor);

        // Create and submit a task
        let task = Task::new(
            TaskPriority::Normal,
            TaskPayload {
                task_type: TaskType::TextProcessing,
                data: serde_json::json!({"text": "Hello, world!"}),
                parameters: HashMap::new(),
            },
        );
        let task_id = manager.submit_task(task).await.unwrap();

        // Cancel task
        manager.cancel_task(&task_id).await.unwrap();

        // Verify task is cancelled
        let task = manager.get_task(&task_id).await.unwrap();
        assert!(matches!(task.status, TaskStatus::Cancelled));
    }
}
