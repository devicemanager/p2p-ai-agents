# Scheduling & Execution

*Part of Task Processing Guide*

## Task Scheduling

```rust
/// Task scheduler implementation
pub struct TaskScheduler {
    task_manager: Arc<TaskManager>,
    resource_monitor: Arc<ResourceMonitor>,
    config: SchedulerConfig,
}

impl TaskScheduler {
    pub async fn schedule_task(&self, task: Task) -> Result<TaskId, TaskError> {
        // Check resource availability
        let resources = self.resource_monitor.current_usage().await?;
        if !self.has_sufficient_resources(&task, &resources) {
            return Err(TaskError::InsufficientResources);
        }
        
        // Check dependencies
        if !self.check_dependencies(&task).await? {
            return Err(TaskError::DependencyNotMet);
        }
        
        // Schedule task
        let task_id = task.id;
        self.task_manager.submit_task(task).await?;
        
        Ok(task_id)
    }
    
    async fn check_dependencies(&self, task: &Task) -> Result<bool, TaskError> {
        for dep_id in &task.dependencies {
            let result = self.task_manager.get_task_result(*dep_id).await?;
            if result.status != TaskStatus::Completed {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    fn has_sufficient_resources(&self, task: &Task, resources: &ResourceUsage) -> bool {
        // Check CPU
        if resources.cpu_usage > self.config.max_cpu_usage {
            return false;
        }
        
        // Check memory
        if resources.memory_bytes > self.config.max_memory_bytes {
            return false;
        }
        
        // Check storage
        if resources.storage_bytes > self.config.max_storage_bytes {
            return false;
        }
        
        true
    }
}
```

## Task Execution

```rust
/// Task executor implementation
pub struct TaskExecutor {
    task_manager: Arc<TaskManager>,
    processor: Arc<dyn Processor>,
    model_manager: Arc<ModelManager>,
    vector_store: Arc<dyn VectorStore>,
    config: ExecutorConfig,
}

impl TaskExecutor {
    pub async fn execute_task(&self, task: Task) -> Result<TaskResult, TaskError> {
        let start_time = Instant::now();
        
        // Execute task based on type
        let output = match task.task_type {
            TaskType::TextProcessing => {
                self.process_text_task(&task).await?
            }
            TaskType::ModelInference => {
                self.process_model_task(&task).await?
            }
            TaskType::VectorSearch => {
                self.process_vector_task(&task).await?
            }
            _ => return Err(TaskError::UnsupportedTaskType(task.task_type)),
        };
        
        // Record metrics
        let duration = start_time.elapsed();
        let metrics = self.collect_metrics(&task, duration).await?;
        
        Ok(TaskResult {
            task_id: task.id,
            status: TaskStatus::Completed,
            output,
            error: None,
            metrics,
            timestamp: Utc::now(),
        })
    }
    
    async fn process_text_task(&self, task: &Task) -> Result<TaskOutput, TaskError> {
        let text = task.input.as_text()?;
        let result = self.processor.process_text(text).await?;
        Ok(TaskOutput::Text(result))
    }
    
    async fn process_model_task(&self, task: &Task) -> Result<TaskOutput, TaskError> {
        let model = self.model_manager.get_model(&task.model_id.unwrap()).await?;
        let input = task.input.as_model_input()?;
        let result = model.infer(input).await?;
        Ok(TaskOutput::ModelOutput(result))
    }
    
    async fn process_vector_task(&self, task: &Task) -> Result<TaskOutput, TaskError> {
        let query = task.input.as_query()?;
        let results = self.vector_store.search(query).await?;
        Ok(TaskOutput::SearchResults(results))
    }
    
    async fn collect_metrics(&self, task: &Task, duration: Duration) -> Result<TaskMetrics, TaskError> {
        let resources = self.task_manager.resource_monitor.current_usage().await?;
        
        Ok(TaskMetrics {
            duration,
            memory_bytes: resources.memory_bytes,
            cpu_usage: resources.cpu_usage,
            network_bytes: resources.network_bytes,
            retry_count: task.retry_count,
        })
    }
}
```

