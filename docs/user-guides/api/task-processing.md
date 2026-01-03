# Task Processing API

*Part of P2P AI Agents API Reference*

---

## Task Processing API

### Task

Represents a unit of work that can be processed by agents.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: TaskId,
    
    /// Task type or name
    pub task_type: String,
    
    /// Task input data
    pub input: Vec<u8>,
    
    /// Task metadata
    pub metadata: HashMap<String, String>,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// Task timeout
    pub timeout: Option<Duration>,
    
    /// Creation timestamp
    pub created_at: SystemTime,
}

impl Task {
    /// Create a new task
    pub fn new(task_type: String, input: Vec<u8>) -> Self
    
    /// Create a task with metadata
    pub fn with_metadata(task_type: String, input: Vec<u8>, metadata: HashMap<String, String>) -> Self
    
    /// Set task priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self
    
    /// Set task timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self
}
```

### TaskStatus

Status tracking for task execution.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued for processing
    Pending,
    
    /// Task is currently being processed
    Running {
        started_at: SystemTime,
        progress: Option<f32>,
    },
    
    /// Task completed successfully
    Completed {
        started_at: SystemTime,
        completed_at: SystemTime,
        result: Vec<u8>,
    },
    
    /// Task failed with error
    Failed {
        started_at: SystemTime,
        failed_at: SystemTime,
        error: String,
    },
    
    /// Task was cancelled
    Cancelled {
        cancelled_at: SystemTime,
    },
}
```

**Example:**

```rust
use p2p_ai_agents::agent::{Task, TaskPriority};
use std::time::Duration;
use std::collections::HashMap;

// Create a simple task
let task = Task::new("image_processing".to_string(), image_data);

// Create a task with metadata and priority
let mut metadata = HashMap::new();
metadata.insert("format".to_string(), "jpeg".to_string());
metadata.insert("quality".to_string(), "high".to_string());

let task = Task::with_metadata("image_processing".to_string(), image_data, metadata)
    .with_priority(TaskPriority::High)
    .with_timeout(Duration::from_secs(300));
```
