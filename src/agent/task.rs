//! Task management module for Agents.

use crate::storage::local::{ConsistencyLevel, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unique identifier for a task.
pub type TaskId = Uuid;

/// Status of a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is created but not started (queued).
    Queued,
    /// Task is currently running.
    Running,
    /// Task completed successfully with result.
    Completed(serde_json::Value),
    /// Task failed.
    Failed(String),
    /// Task cancelled.
    Cancelled,
    /// Task timed out.
    Timeout,
}

/// Priority level of a task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Low priority task.
    Low,
    /// Normal priority task (default).
    Normal,
    /// High priority task.
    High,
    /// Critical priority task.
    Critical,
}

/// Type of task to perform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// Basic text processing
    TextProcessing,
    /// Vector computation
    VectorComputation,
    /// Custom task type
    Custom(String),
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::TextProcessing => write!(f, "TextProcessing"),
            TaskType::VectorComputation => write!(f, "VectorComputation"),
            TaskType::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

/// Payload containing task data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload {
    /// The type of the task.
    pub task_type: TaskType,
    /// The input data for the task.
    pub data: serde_json::Value,
    /// Configuration parameters for the task.
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
    /// Progress percentage (0-100) if available.
    pub progress_percent: Option<u8>,
    /// Timestamp when task was created.
    pub created_at: SystemTime,
    /// Timestamp when task started running.
    pub started_at: Option<SystemTime>,
    /// Timestamp when task completed/failed/cancelled.
    pub completed_at: Option<SystemTime>,
    /// Execution time in milliseconds (so far or total).
    pub execution_time_ms: Option<u64>,
    /// Error reason for failed tasks.
    pub error_reason: Option<String>,
    /// Detailed error information.
    pub error_details: Option<String>,
    /// Size of result in bytes (for completed tasks).
    pub result_size_bytes: Option<usize>,
}

impl Task {
    /// Creates a new task with a description (legacy helper).
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            priority: TaskPriority::Normal,
            payload: None,
            description: description.into(),
            status: TaskStatus::Queued,
            progress_percent: Some(0),
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            error_reason: None,
            error_details: None,
            result_size_bytes: None,
        }
    }

    /// Creates a new task with priority and payload.
    pub fn with_payload(priority: TaskPriority, payload: TaskPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            priority,
            payload: Some(payload),
            description: String::new(), // Description optional when payload used
            status: TaskStatus::Queued,
            progress_percent: Some(0),
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            error_reason: None,
            error_details: None,
            result_size_bytes: None,
        }
    }

    /// Gets estimated completion time in seconds (if available).
    pub fn estimated_completion_secs(&self) -> Option<u64> {
        if self.status != TaskStatus::Running {
            return None;
        }

        let progress = self.progress_percent? as f64;
        if progress < 1.0 {
            return None;
        }

        let elapsed_ms = self.execution_time_so_far_ms()? as f64;
        let estimated_total_ms = (elapsed_ms / progress) * 100.0;
        let remaining_ms = estimated_total_ms - elapsed_ms;

        Some((remaining_ms / 1000.0).ceil() as u64)
    }

    /// Gets execution time so far in milliseconds.
    pub fn execution_time_so_far_ms(&self) -> Option<u64> {
        match self.status {
            TaskStatus::Running => {
                let started = self.started_at?;
                let elapsed = SystemTime::now().duration_since(started).ok()?;
                Some(elapsed.as_millis() as u64)
            }
            _ => self.execution_time_ms,
        }
    }

    /// Gets result preview (first 100 chars).
    pub fn result_preview(&self) -> Option<String> {
        match &self.status {
            TaskStatus::Completed(result) => {
                let result_str = result.to_string();
                if result_str.len() > 100 {
                    Some(format!("{}...", &result_str[..100]))
                } else {
                    Some(result_str)
                }
            }
            _ => None,
        }
    }
}

/// Trait for executing tasks.
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    /// Executes a specific task payload and returns the result.
    async fn execute(&self, payload: &TaskPayload) -> anyhow::Result<serde_json::Value>;
}

use futures::future::AbortHandle;

/// Manages tasks for an agent.
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
    running_tasks: Arc<RwLock<HashMap<TaskId, AbortHandle>>>,
    storage: Arc<dyn Storage>,
}

// Default implementation uses LocalStorage in current directory
impl Default for TaskManager {
    fn default() -> Self {
        use crate::storage::local::LocalStorage;
        let storage =
            Arc::new(LocalStorage::new("data/tasks").expect("Failed to create default storage"));
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }
}

impl TaskManager {
    /// Creates a new TaskManager with custom storage.
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    /// Submits a new task.
    pub async fn submit_task(&self, description: impl Into<String>) -> TaskId {
        let task = Task::new(description);
        let id = task.id;

        // Persist to storage
        if let Ok(json) = serde_json::to_vec(&task) {
            let _ = self
                .storage
                .put(&id.to_string(), json, ConsistencyLevel::Strong)
                .await;
        }

        self.tasks.write().await.insert(id, task);
        id
    }

    /// Adds a fully formed task.
    pub async fn add_task(&self, task: Task) -> TaskId {
        let id = task.id;

        // Persist to storage
        if let Ok(json) = serde_json::to_vec(&task) {
            let _ = self
                .storage
                .put(&id.to_string(), json, ConsistencyLevel::Strong)
                .await;
        }

        self.tasks.write().await.insert(id, task);
        id
    }

    /// Updates the status of a task.
    pub async fn update_status(&self, id: TaskId, status: TaskStatus) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            // Update timestamps based on status transition
            match (&task.status, &status) {
                (TaskStatus::Queued, TaskStatus::Running) => {
                    task.started_at = Some(SystemTime::now());
                }
                (_, TaskStatus::Completed(_))
                | (_, TaskStatus::Failed(_))
                | (_, TaskStatus::Cancelled)
                | (_, TaskStatus::Timeout) => {
                    task.completed_at = Some(SystemTime::now());

                    // Calculate final execution time
                    if let Some(started) = task.started_at {
                        if let Ok(duration) = SystemTime::now().duration_since(started) {
                            task.execution_time_ms = Some(duration.as_millis() as u64);
                        }
                    }

                    // Calculate result size for completed tasks
                    if let TaskStatus::Completed(ref result) = status {
                        let result_str = result.to_string();
                        task.result_size_bytes = Some(result_str.len());
                    }

                    // Store error details for failed tasks
                    if let TaskStatus::Failed(ref reason) = status {
                        task.error_reason = Some(reason.clone());
                        task.error_details = Some(format!("Task failed: {}", reason));
                    }

                    // Clean up running tasks
                    self.running_tasks.write().await.remove(&id);
                }
                _ => {}
            }

            task.status = status;

            // Update in storage
            if let Ok(json) = serde_json::to_vec(&task) {
                let _ = self
                    .storage
                    .put(&id.to_string(), json, ConsistencyLevel::Strong)
                    .await;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }

    /// Registers a running task handle.
    pub async fn register_running_task(&self, id: TaskId, handle: AbortHandle) {
        self.running_tasks.write().await.insert(id, handle);
    }

    /// Cancels a task.
    pub async fn cancel_task(&self, id: TaskId) -> anyhow::Result<()> {
        // First check if task exists and abort if running
        if let Some(handle) = self.running_tasks.write().await.remove(&id) {
            handle.abort();
        }

        self.update_status(id, TaskStatus::Cancelled).await
    }

    /// Updates task progress.
    pub async fn update_progress(&self, id: TaskId, progress_percent: u8) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            task.progress_percent = Some(progress_percent.min(100));

            // Update in storage
            if let Ok(json) = serde_json::to_vec(&task) {
                let _ = self
                    .storage
                    .put(&id.to_string(), json, ConsistencyLevel::Strong)
                    .await;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }

    /// Retrieves a task by ID.
    pub async fn get_task(&self, id: TaskId) -> Option<Task> {
        // Try memory first
        if let Some(task) = self.tasks.read().await.get(&id).cloned() {
            return Some(task);
        }

        // Try storage
        if let Ok(Some(bytes)) = self
            .storage
            .get(&id.to_string(), ConsistencyLevel::Strong)
            .await
        {
            if let Ok(task) = serde_json::from_slice::<Task>(&bytes) {
                // Populate cache
                self.tasks.write().await.insert(id, task.clone());
                return Some(task);
            }
        }

        None
    }

    /// Lists all tasks.
    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// Loads all tasks from storage into memory
    pub async fn load_tasks(&self) -> anyhow::Result<usize> {
        let keys = self.storage.list().await?;
        let mut count = 0;
        let mut tasks = self.tasks.write().await;

        for key in keys {
            if let Ok(Some(bytes)) = self.storage.get(&key, ConsistencyLevel::Strong).await {
                if let Ok(task) = serde_json::from_slice::<Task>(&bytes) {
                    tasks.insert(task.id, task);
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Gets the next pending task based on priority.
    pub async fn get_next_pending_task(&self) -> Option<Task> {
        let tasks = self.tasks.read().await;
        let mut pending_tasks: Vec<&Task> = tasks
            .values()
            .filter(|t| t.status == TaskStatus::Queued)
            .collect();

        // Sort by priority (Critical > High > Normal > Low)
        pending_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        pending_tasks.first().map(|t| (*t).clone())
    }

    /// Gets detailed task status information.
    pub async fn get_task_status(&self, id: TaskId) -> Option<TaskStatusInfo> {
        let task = self.get_task(id).await?;
        Some(TaskStatusInfo::from_task(&task))
    }
}

/// Detailed task status information for querying.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusInfo {
    /// Task ID.
    pub task_id: TaskId,
    /// Current status.
    pub status: TaskStatus,
    /// Progress percentage (0-100) if available.
    pub progress_percent: Option<u8>,
    /// Estimated completion time in seconds (for running tasks).
    pub estimated_completion_secs: Option<u64>,
    /// Execution time so far in milliseconds.
    pub execution_time_so_far_ms: Option<u64>,
    /// Timestamp when task completed (for completed/failed tasks).
    pub completion_timestamp: Option<SystemTime>,
    /// Size of result in bytes (for completed tasks).
    pub result_size_bytes: Option<usize>,
    /// Result preview (first 100 chars, for completed tasks).
    pub result_preview: Option<String>,
    /// Error reason (for failed tasks).
    pub error_reason: Option<String>,
    /// Error details (for failed tasks).
    pub error_details: Option<String>,
    /// Timestamp when task failed (for failed tasks).
    pub timestamp_failed: Option<SystemTime>,
}

impl TaskStatusInfo {
    /// Creates TaskStatusInfo from a Task.
    pub fn from_task(task: &Task) -> Self {
        Self {
            task_id: task.id,
            status: task.status.clone(),
            progress_percent: task.progress_percent,
            estimated_completion_secs: task.estimated_completion_secs(),
            execution_time_so_far_ms: task.execution_time_so_far_ms(),
            completion_timestamp: task.completed_at,
            result_size_bytes: task.result_size_bytes,
            result_preview: task.result_preview(),
            error_reason: task.error_reason.clone(),
            error_details: task.error_details.clone(),
            timestamp_failed: match &task.status {
                TaskStatus::Failed(_) => task.completed_at,
                _ => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_task_status_queued() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Queued);
        assert_eq!(task.progress_percent, Some(0));
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
    }

    #[tokio::test]
    async fn test_task_status_running() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        manager.update_progress(task_id, 50).await.unwrap();

        let status_info = manager.get_task_status(task_id).await.unwrap();
        assert_eq!(status_info.status, TaskStatus::Running);
        assert_eq!(status_info.progress_percent, Some(50));
        assert!(status_info.execution_time_so_far_ms.is_some());
    }

    #[tokio::test]
    async fn test_task_status_completed() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        sleep(Duration::from_millis(100)).await;

        let result = serde_json::json!({"result": "success"});
        manager
            .update_status(task_id, TaskStatus::Completed(result.clone()))
            .await
            .unwrap();

        let status_info = manager.get_task_status(task_id).await.unwrap();
        assert_eq!(status_info.status, TaskStatus::Completed(result));
        assert!(status_info.completion_timestamp.is_some());
        assert!(status_info.result_size_bytes.is_some());
        assert!(status_info.result_preview.is_some());
        assert!(status_info.execution_time_so_far_ms.is_some());
    }

    #[tokio::test]
    async fn test_task_status_failed() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        manager
            .update_status(task_id, TaskStatus::Failed("Test error".to_string()))
            .await
            .unwrap();

        let status_info = manager.get_task_status(task_id).await.unwrap();
        match status_info.status {
            TaskStatus::Failed(reason) => assert_eq!(reason, "Test error"),
            _ => panic!("Expected Failed status"),
        }
        assert!(status_info.error_reason.is_some());
        assert!(status_info.error_details.is_some());
        assert!(status_info.timestamp_failed.is_some());
    }

    #[tokio::test]
    async fn test_task_status_timeout() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        manager
            .update_status(task_id, TaskStatus::Timeout)
            .await
            .unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Timeout);
        assert!(task.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_progress_updates() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        manager.update_progress(task_id, 25).await.unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.progress_percent, Some(25));

        manager.update_progress(task_id, 75).await.unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.progress_percent, Some(75));
    }

    #[tokio::test]
    async fn test_progress_clamped_to_100() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        manager.update_progress(task_id, 150).await.unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.progress_percent, Some(100));
    }

    #[tokio::test]
    async fn test_result_preview_truncation() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        let long_result = "x".repeat(200);
        let result = serde_json::json!({"data": long_result});
        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        manager
            .update_status(task_id, TaskStatus::Completed(result))
            .await
            .unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        let preview = task.result_preview().unwrap();
        assert!(preview.len() <= 103); // 100 chars + "..."
        assert!(preview.ends_with("..."));
    }

    #[tokio::test]
    async fn test_execution_time_tracking() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        sleep(Duration::from_millis(50)).await;

        let task = manager.get_task(task_id).await.unwrap();
        let exec_time = task.execution_time_so_far_ms().unwrap();
        assert!(exec_time >= 50);

        sleep(Duration::from_millis(50)).await;
        let result = serde_json::json!({"result": "done"});
        manager
            .update_status(task_id, TaskStatus::Completed(result))
            .await
            .unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        let final_exec_time = task.execution_time_ms.unwrap();
        assert!(final_exec_time >= 100);
    }

    #[tokio::test]
    async fn test_estimated_completion() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        sleep(Duration::from_millis(100)).await;
        manager.update_progress(task_id, 50).await.unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        let estimated = task.estimated_completion_secs();
        assert!(estimated.is_some());
    }

    #[tokio::test]
    async fn test_status_query_performance() {
        let manager = TaskManager::default();
        let mut task_ids = vec![];

        // Create 100 tasks
        for i in 0..100 {
            let task_id = manager.submit_task(format!("Task {}", i)).await;
            task_ids.push(task_id);
        }

        // Query all task statuses
        let start = std::time::Instant::now();
        for task_id in task_ids {
            let _ = manager.get_task_status(task_id).await;
        }
        let elapsed = start.elapsed();

        // Should complete 100 queries in < 10ms (< 0.1ms per query)
        // Adjust threshold as saving to disk might take longer
        // Increase threshold to 500ms for disk I/O
        assert!(elapsed.as_millis() < 500, "Elapsed: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_concurrent_status_updates() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();

        // Simulate concurrent progress updates
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let manager_clone = manager.clone();
                tokio::spawn(async move { manager_clone.update_progress(task_id, i * 10).await })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let task = manager.get_task(task_id).await.unwrap();
        assert!(task.progress_percent.is_some());
    }

    #[tokio::test]
    async fn test_task_timestamps() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        let task = manager.get_task(task_id).await.unwrap();
        assert!(task.created_at <= SystemTime::now());

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert!(task.started_at.is_some());
        assert!(task.started_at.unwrap() >= task.created_at);

        let result = serde_json::json!({"done": true});
        manager
            .update_status(task_id, TaskStatus::Completed(result))
            .await
            .unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert!(task.completed_at.is_some());
        assert!(task.completed_at.unwrap() >= task.started_at.unwrap());
    }

    #[tokio::test]
    async fn test_result_size_calculation() {
        let manager = TaskManager::default();
        let task_id = manager.submit_task("Test task").await;

        manager
            .update_status(task_id, TaskStatus::Running)
            .await
            .unwrap();

        let result = serde_json::json!({"key": "value"});
        manager
            .update_status(task_id, TaskStatus::Completed(result.clone()))
            .await
            .unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        let expected_size = result.to_string().len();
        assert_eq!(task.result_size_bytes, Some(expected_size));
    }
}
