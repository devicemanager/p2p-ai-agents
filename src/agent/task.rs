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
    /// Task completed successfully with result.
    Completed(serde_json::Value),
    /// Task failed.
    Failed(String),
    /// Task cancelled.
    Cancelled,
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

/// Trait for executing tasks.
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    /// Executes a specific task payload and returns the result.
    async fn execute(&self, payload: &TaskPayload) -> anyhow::Result<serde_json::Value>;
}

/// Handler for TextProcessing tasks.
pub struct TextProcessingExecutor;

#[async_trait::async_trait]
impl TaskExecutor for TextProcessingExecutor {
    async fn execute(&self, payload: &TaskPayload) -> anyhow::Result<serde_json::Value> {
        let operation = payload.data.get("operation").and_then(|v| v.as_str()).unwrap_or("unknown");
        let text = payload.data.get("text").and_then(|v| v.as_str()).unwrap_or("");
        
        match operation {
            "word_count" => {
                let count = text.split_whitespace().count();
                Ok(serde_json::json!({ "word_count": count }))
            }
            "reverse" => {
                let reversed: String = text.chars().rev().collect();
                Ok(serde_json::json!({ "reversed_text": reversed }))
            }
             "tokenize" => {
                 // Simple whitespace tokenization
                 let tokens: Vec<&str> = text.split_whitespace().collect();
                 Ok(serde_json::json!({ "tokens": tokens }))
             }
            _ => Err(anyhow::anyhow!("Unknown text operation: {}", operation)),
        }
    }
}

/// Handler for VectorComputation tasks.
pub struct VectorComputationExecutor;

#[async_trait::async_trait]
impl TaskExecutor for VectorComputationExecutor {
    async fn execute(&self, payload: &TaskPayload) -> anyhow::Result<serde_json::Value> {
        let operation = payload.data.get("operation").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        match operation {
            "cosine_similarity" => {
                let vec_a = payload.data.get("vector_a")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing vector_a"))?;
                let vec_b = payload.data.get("vector_b")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing vector_b"))?;
                
                if vec_a.len() != vec_b.len() {
                    return Err(anyhow::anyhow!("Vectors must have same length"));
                }
                
                let a: Vec<f64> = vec_a.iter().filter_map(|v| v.as_f64()).collect();
                let b: Vec<f64> = vec_b.iter().filter_map(|v| v.as_f64()).collect();
                
                if a.len() != vec_a.len() || b.len() != vec_b.len() {
                     return Err(anyhow::anyhow!("Invalid vector data"));
                }
                
                let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let magnitude_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
                let magnitude_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
                
                if magnitude_a == 0.0 || magnitude_b == 0.0 {
                    return Ok(serde_json::json!({ "similarity": 0.0 }));
                }
                
                let similarity = dot_product / (magnitude_a * magnitude_b);
                Ok(serde_json::json!({ "similarity": similarity }))
            }
            _ => Err(anyhow::anyhow!("Unknown vector operation: {}", operation)),
        }
    }
}

use futures::future::AbortHandle;

/// Manages tasks for an agent.
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
    running_tasks: Arc<RwLock<HashMap<TaskId, AbortHandle>>>,
}

impl TaskManager {
    /// Creates a new TaskManager.
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
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
            // Clean up running tasks if status is terminal
            match status {
                TaskStatus::Completed(_) | TaskStatus::Failed(_) | TaskStatus::Cancelled => {
                    self.running_tasks.write().await.remove(&id);
                }
                _ => {}
            }
            task.status = status;
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

    /// Retrieves a task by ID.
    pub async fn get_task(&self, id: TaskId) -> Option<Task> {
        self.tasks.read().await.get(&id).cloned()
    }
    
    /// Lists all tasks.
    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.values().cloned().collect()
    }
    
    /// Gets the next pending task based on priority.
    pub async fn get_next_pending_task(&self) -> Option<Task> {
        let tasks = self.tasks.read().await;
        let mut pending_tasks: Vec<&Task> = tasks.values()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect();
            
        // Sort by priority (Critical > High > Normal > Low)
        pending_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        pending_tasks.first().map(|t| (*t).clone())
    }
}
