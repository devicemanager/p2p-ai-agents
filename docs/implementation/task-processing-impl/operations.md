# Testing & Operations

*Part of Task Processing Guide*

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),
    
    #[error("Task processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Dependency not met: {0}")]
    DependencyNotMet(TaskId),
    
    #[error("Task timeout: {0}")]
    TaskTimeout(TaskId),
    
    #[error("Invalid task input: {0}")]
    InvalidInput(String),
    
    #[error("Unsupported task type: {0:?}")]
    UnsupportedTaskType(TaskType),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl TaskError {
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            TaskError::ProcessingFailed(_) |
            TaskError::ResourceLimitExceeded(_) |
            TaskError::TaskTimeout(_)
        )
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_task_scheduling() {
        let config = TaskManagerConfig::default();
        let task_manager = TaskManager::new(config).await;
        let scheduler = TaskScheduler::new(task_manager.clone());
        
        let task = Task {
            id: TaskId::new(),
            task_type: TaskType::TextProcessing,
            input: TaskInput::Text("Test".to_string()),
            ..Default::default()
        };
        
        let task_id = scheduler.schedule_task(task).await.unwrap();
        let result = task_manager.get_task_result(task_id).await.unwrap();
        
        assert_eq!(result.status, TaskStatus::Completed);
    }
    
    #[test]
    async fn test_task_dependencies() {
        let config = TaskManagerConfig::default();
        let task_manager = TaskManager::new(config).await;
        let scheduler = TaskScheduler::new(task_manager.clone());
        
        // Create dependent tasks
        let task1 = Task {
            id: TaskId::new(),
            task_type: TaskType::TextProcessing,
            input: TaskInput::Text("Task 1".to_string()),
            ..Default::default()
        };
        
        let task2 = Task {
            id: TaskId::new(),
            task_type: TaskType::TextProcessing,
            input: TaskInput::Text("Task 2".to_string()),
            dependencies: vec![task1.id],
            ..Default::default()
        };
        
        // Schedule tasks
        let task1_id = scheduler.schedule_task(task1).await.unwrap();
        let task2_id = scheduler.schedule_task(task2).await.unwrap();
        
        // Verify execution order
        let result1 = task_manager.get_task_result(task1_id).await.unwrap();
        let result2 = task_manager.get_task_result(task2_id).await.unwrap();
        
        assert!(result1.timestamp <= result2.timestamp);
    }
}
```

## Performance Optimization

1. **Task Batching**
   - Group similar tasks
   - Optimize resource usage
   - Reduce overhead
   - Improve throughput

2. **Resource Management**
   - Monitor resource usage
   - Implement backpressure
   - Handle resource exhaustion
   - Optimize allocation

3. **Parallel Processing**
   - Use work stealing
   - Implement task prioritization
   - Handle task dependencies
   - Optimize scheduling

4. **Caching**
   - Cache task results
   - Reuse resources
   - Optimize memory usage
   - Reduce computation

## Metrics and Monitoring

```rust
use metrics::{counter, gauge, histogram};

impl TaskManager {
    async fn record_metrics(&self, task: &Task, result: &TaskResult) {
        // Task metrics
        counter!("tasks.processed", 1);
        counter!("tasks.by_type", 1, "type" => task.task_type.to_string());
        counter!("tasks.by_priority", 1, "priority" => task.priority.to_string());
        
        // Performance metrics
        histogram!("task.duration", result.metrics.duration.as_secs_f64());
        histogram!("task.memory", result.metrics.memory_bytes as f64);
        histogram!("task.cpu", result.metrics.cpu_usage as f64);
        
        // Queue metrics
        gauge!("task.queue.size", self.task_queue.read().await.len() as f64);
        gauge!("task.running.count", self.running_tasks.read().await.len() as f64);
    }
}
