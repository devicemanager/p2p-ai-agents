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

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
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
    use std::collections::HashMap;
    use tokio::time::Duration;

    /// Test TaskId generation uniqueness
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let id1 = TaskId::new();
    /// let id2 = TaskId::new();
    /// assert_ne!(id1, id2); // Each ID is unique
    /// ```
    #[test]
    fn test_task_id_new() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();

        // Each ID should be unique
        assert_ne!(id1, id2);
        assert_ne!(id1.as_str(), id2.as_str());

        // Should be valid UUID format
        assert!(uuid::Uuid::parse_str(id1.as_str()).is_ok());
        assert!(uuid::Uuid::parse_str(id2.as_str()).is_ok());
    }

    /// Test TaskId creation from string
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let task_id = TaskId::from_string("my-custom-id".to_string());
    /// assert_eq!(task_id.as_str(), "my-custom-id");
    /// ```
    #[test]
    fn test_task_id_from_string() {
        let test_id = "test-task-id-123";
        let task_id = TaskId::from_string(test_id.to_string());

        assert_eq!(task_id.as_str(), test_id);
        assert_eq!(task_id.to_string(), test_id);
    }

    /// Test TaskId default creation
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let task_id = TaskId::default(); // Creates a new unique ID
    /// ```
    #[test]
    fn test_task_id_default() {
        let id1 = TaskId::default();
        let id2 = TaskId::default();

        // Default should create unique IDs
        assert_ne!(id1, id2);
        assert!(uuid::Uuid::parse_str(id1.as_str()).is_ok());
    }

    /// Test TaskId display formatting
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let task_id = TaskId::from_string("example-id".to_string());
    /// println!("{}", task_id); // Prints: example-id
    /// ```
    #[test]
    fn test_task_id_display() {
        let test_str = "display-test-id";
        let task_id = TaskId::from_string(test_str.to_string());

        assert_eq!(format!("{}", task_id), test_str);
    }

    /// Test TaskId equality comparison
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let id1 = TaskId::from_string("same".to_string());
    /// let id2 = TaskId::from_string("same".to_string());
    /// let id3 = TaskId::from_string("different".to_string());
    ///
    /// assert_eq!(id1, id2);
    /// assert_ne!(id1, id3);
    /// ```
    #[test]
    fn test_task_id_equality() {
        let id1 = TaskId::from_string("same-id".to_string());
        let id2 = TaskId::from_string("same-id".to_string());
        let id3 = TaskId::from_string("different-id".to_string());

        // Same string should be equal
        assert_eq!(id1, id2);
        // Different string should not be equal
        assert_ne!(id1, id3);
    }

    /// Test TaskId cloning
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let original = TaskId::from_string("my-id".to_string());
    /// let cloned = original.clone();
    /// assert_eq!(original, cloned);
    /// ```
    #[test]
    fn test_task_id_clone() {
        let original = TaskId::from_string("clone-test".to_string());
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.as_str(), cloned.as_str());
    }

    /// Test TaskId hashing for use in collections
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    /// use std::collections::HashMap;
    ///
    /// let mut tasks = HashMap::new();
    /// let task_id = TaskId::new();
    /// tasks.insert(task_id.clone(), "task data");
    /// ```
    #[test]
    fn test_task_id_hash() {
        use std::collections::HashSet;

        let id1 = TaskId::from_string("hash-test-1".to_string());
        let id2 = TaskId::from_string("hash-test-2".to_string());
        let id1_dup = TaskId::from_string("hash-test-1".to_string());

        let mut set = HashSet::new();
        set.insert(id1.clone());
        set.insert(id2.clone());
        set.insert(id1_dup.clone());

        // Should only have 2 unique items (id1 and id2)
        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }

    /// Test TaskId serialization for storage and communication
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    /// use serde_json;
    ///
    /// let task_id = TaskId::from_string("serializable-id".to_string());
    /// let json = serde_json::to_string(&task_id).unwrap();
    /// // json contains: "\"serializable-id\""
    /// ```
    #[test]
    fn test_task_id_serialization() {
        let original = TaskId::from_string("serialize-test".to_string());

        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"serialize-test\"");

        // Deserialize from JSON
        let deserialized: TaskId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    /// Test TaskId with empty string
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let empty_id = TaskId::from_string("".to_string());
    /// assert_eq!(empty_id.as_str(), "");
    /// ```
    #[test]
    fn test_task_id_empty_string() {
        let empty_id = TaskId::from_string("".to_string());
        assert_eq!(empty_id.as_str(), "");
        assert_eq!(empty_id.to_string(), "");
    }

    /// Test TaskId with special characters
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::agent::TaskId;
    ///
    /// let special_id = TaskId::from_string("task-123_special!".to_string());
    /// // TaskId accepts any string content
    /// ```
    #[test]
    fn test_task_id_special_characters() {
        let special = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        let task_id = TaskId::from_string(special.to_string());
        assert_eq!(task_id.as_str(), special);
    }

    // TaskPriority tests
    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);

        let mut priorities = vec![
            TaskPriority::Low,
            TaskPriority::Critical,
            TaskPriority::Normal,
            TaskPriority::High,
        ];
        priorities.sort();

        assert_eq!(
            priorities,
            vec![
                TaskPriority::Low,
                TaskPriority::Normal,
                TaskPriority::High,
                TaskPriority::Critical,
            ]
        );
    }

    // TaskStatus tests
    #[test]
    fn test_task_status_failed_with_message() {
        let error_msg = "Something went wrong";
        let status = TaskStatus::Failed(error_msg.to_string());

        match status {
            TaskStatus::Failed(msg) => assert_eq!(msg, error_msg),
            _ => panic!("Expected Failed status"),
        }
    }

    // TaskType tests
    #[test]
    fn test_task_type_custom() {
        let custom_type = "CustomProcessing";
        let task_type = TaskType::Custom(custom_type.to_string());

        match task_type {
            TaskType::Custom(name) => assert_eq!(name, custom_type),
            _ => panic!("Expected Custom task type"),
        }
    }

    // Task creation and methods tests
    #[test]
    fn test_task_creation_and_getters() {
        let priority = TaskPriority::High;
        let payload = TaskPayload {
            task_type: TaskType::TextProcessing,
            data: serde_json::json!({"input": "test data"}),
            parameters: HashMap::new(),
        };

        let task = Task::new(priority, payload.clone());

        // Test getters
        assert_eq!(task.priority(), priority);
        assert_eq!(task.payload().task_type, payload.task_type);
        assert_eq!(task.payload().data, payload.data);
        assert!(matches!(task.status(), TaskStatus::Pending));
        assert!(task.result().is_none());

        // Created at should be recent
        let now = chrono::Utc::now();
        let created = task.created_at();
        assert!(now.signed_duration_since(created).num_seconds() < 1);
    }

    #[test]
    fn test_task_set_result() {
        let priority = TaskPriority::Normal;
        let payload = TaskPayload {
            task_type: TaskType::VectorComputation,
            data: serde_json::json!({"vectors": [1, 2, 3]}),
            parameters: HashMap::new(),
        };

        let mut task = Task::new(priority, payload);

        // Initially no result
        assert!(task.result().is_none());
        assert!(matches!(task.status(), TaskStatus::Pending));

        // Set result
        let result = TaskResult {
            output: serde_json::json!({"computed": "result"}),
            metadata: HashMap::new(),
        };
        task.set_result(result.clone());

        // Check result is set and status updated
        assert!(task.result().is_some());
        assert_eq!(task.result().unwrap().output, result.output);
        assert!(matches!(task.status(), TaskStatus::Completed));

        // Completed at should be set
        let now = chrono::Utc::now();
        let completed = task.completed_at.unwrap();
        assert!(now.signed_duration_since(completed).num_seconds() < 1);
    }

    // Existing tests...
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
    async fn test_task_executor() {
        let executor = MockExecutor;
        let payload = TaskPayload {
            task_type: TaskType::TextProcessing,
            data: serde_json::json!({"text": "test"}),
            parameters: HashMap::new(),
        };
        let task = Task::new(TaskPriority::Normal, payload);

        let result = executor.execute(&task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        assert_eq!(task_result.output["result"], "success");
    }

    // ...existing code...
}
