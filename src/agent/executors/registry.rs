use crate::agent::task::{TaskExecutor, TaskType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for task executors.
#[derive(Default, Clone)]
pub struct ExecutorRegistry {
    executors: Arc<RwLock<HashMap<TaskType, Arc<dyn TaskExecutor>>>>,
}

impl ExecutorRegistry {
    /// Creates a new, empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an executor for a specific task type.
    pub fn register<E>(&self, task_type: TaskType, executor: E)
    where
        E: TaskExecutor + 'static,
    {
        let mut executors = self.executors.write().unwrap();
        executors.insert(task_type, Arc::new(executor));
    }

    /// Retrieves an executor for a specific task type.
    pub fn get(&self, task_type: &TaskType) -> Option<Arc<dyn TaskExecutor>> {
        let executors = self.executors.read().unwrap();
        executors.get(task_type).cloned()
    }
}
