# Task Manager Implementation

*Part of Task Processing Guide*

## Task Manager Implementation

```rust
use tokio::sync::{mpsc, RwLock};
use std::collections::{HashMap, BinaryHeap};
use std::sync::Arc;

pub struct TaskManager {
    task_queue: Arc<RwLock<BinaryHeap<PrioritizedTask>>>,
    running_tasks: Arc<RwLock<HashMap<TaskId, RunningTask>>>,
    task_results: Arc<RwLock<HashMap<TaskId, TaskResult>>>,
    task_sender: mpsc::Sender<Task>,
    task_receiver: mpsc::Receiver<Task>,
    config: TaskManagerConfig,
    metrics: Arc<TaskMetricsCollector>,
}

impl TaskManager {
    pub async fn new(config: TaskManagerConfig) -> Self {
        let (task_sender, task_receiver) = mpsc::channel(config.queue_size);
        
        Self {
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_results: Arc::new(RwLock::new(HashMap::new())),
            task_sender,
            task_receiver,
            config,
            metrics: Arc::new(TaskMetricsCollector::new()),
        }
    }
    
    pub async fn start(&mut self) -> Result<(), TaskError> {
        // Start task processing loop
        let task_queue = self.task_queue.clone();
        let running_tasks = self.running_tasks.clone();
        let task_results = self.task_results.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            Self::process_tasks(
                task_queue,
                running_tasks,
                task_results,
                metrics,
                config,
            ).await
        });
        
        Ok(())
    }
    
    async fn process_tasks(
        task_queue: Arc<RwLock<BinaryHeap<PrioritizedTask>>>,
        running_tasks: Arc<RwLock<HashMap<TaskId, RunningTask>>>,
        task_results: Arc<RwLock<HashMap<TaskId, TaskResult>>>,
        metrics: Arc<TaskMetricsCollector>,
        config: TaskManagerConfig,
    ) {
        let mut interval = tokio::time::interval(config.processing_interval);
        
        loop {
            interval.tick().await;
            
            // Process next batch of tasks
            let tasks = {
                let mut queue = task_queue.write().await;
                let mut tasks = Vec::new();
                
                while tasks.len() < config.batch_size {
                    if let Some(task) = queue.pop() {
                        tasks.push(task);
                    } else {
                        break;
                    }
                }
                
                tasks
            };
            
            // Process tasks in parallel
            let mut handles = Vec::new();
            for task in tasks {
                let running_tasks = running_tasks.clone();
                let task_results = task_results.clone();
                let metrics = metrics.clone();
                let config = config.clone();
                
                let handle = tokio::spawn(async move {
                    Self::process_single_task(
                        task,
                        running_tasks,
                        task_results,
                        metrics,
                        config,
                    ).await
                });
                
                handles.push(handle);
            }
            
            // Wait for all tasks to complete
            for handle in handles {
                if let Err(e) = handle.await {
                    error!("Task processing error: {}", e);
                }
            }
        }
    }
    
    async fn process_single_task(
        task: PrioritizedTask,
        running_tasks: Arc<RwLock<HashMap<TaskId, RunningTask>>>,
        task_results: Arc<RwLock<HashMap<TaskId, TaskResult>>>,
        metrics: Arc<TaskMetricsCollector>,
        config: TaskManagerConfig,
    ) -> Result<(), TaskError> {
        let task_id = task.id;
        let start_time = Instant::now();
        
        // Update task status
        {
            let mut tasks = running_tasks.write().await;
            tasks.insert(task_id, RunningTask {
                start_time,
                priority: task.priority,
                worker_id: None,
            });
        }
        
        // Process task
        let result = match task.task_type {
            TaskType::TextProcessing => {
                process_text_task(task).await
            }
            TaskType::ModelInference => {
                process_model_task(task).await
            }
            TaskType::VectorSearch => {
                process_vector_task(task).await
            }
            _ => Err(TaskError::UnsupportedTaskType(task.task_type)),
        };
        
        // Record metrics
        let duration = start_time.elapsed();
        metrics.record_task_metrics(&task_id, duration);
        
        // Update task status
        {
            let mut tasks = running_tasks.write().await;
            tasks.remove(&task_id);
        }
        
        // Store result
        {
            let mut results = task_results.write().await;
            results.insert(task_id, result?);
        }
        
        Ok(())
    }
}
```

