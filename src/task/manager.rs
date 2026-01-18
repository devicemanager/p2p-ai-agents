use super::{executor::TaskExecutor, Task, TaskResult, TaskStatus};
use libp2p::PeerId;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

/// Manages the lifecycle, storage, and retrieval of tasks.
pub struct TaskManager {
    /// In-memory storage of tasks
    tasks: HashMap<Uuid, Task>,
    /// In-memory storage of task statuses
    statuses: HashMap<Uuid, TaskStatus>,
    /// In-memory storage of execution results
    results: HashMap<Uuid, TaskResult>,
    /// The execution engine (mock for now)
    executor: TaskExecutor,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager {
    /// Creates a new, empty TaskManager.
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            statuses: HashMap::new(),
            results: HashMap::new(),
            executor: TaskExecutor::new(),
        }
    }

    /// Creates a new task and registers it in the system.
    pub fn create_task(&mut self, prompt: String, sender: PeerId) -> Task {
        let task = Task::new(prompt, sender);
        self.tasks.insert(task.id, task.clone());
        self.statuses.insert(task.id, TaskStatus::Pending);
        info!("Created task {} from peer {}", task.id, sender);
        task
    }

    /// Executes a registered task.
    ///
    /// Returns the result and updates the internal state.
    /// If the task is not found, returns None.
    pub async fn execute_task(&mut self, task_id: Uuid) -> Option<TaskResult> {
        // We clone the task to avoid holding the borrow across the await point
        let task = self.tasks.get(&task_id).cloned()?;

        // Update status to Executing
        self.statuses.insert(task_id, TaskStatus::Executing);

        // Execute (async)
        let result = self.executor.execute(task).await;

        // Update status to Completed and store result
        self.statuses.insert(task_id, TaskStatus::Completed);
        self.results.insert(task_id, result.clone());

        Some(result)
    }

    /// Retrieves the current status of a task.
    pub fn get_status(&self, task_id: &Uuid) -> Option<TaskStatus> {
        self.statuses.get(task_id).copied()
    }

    /// Retrieves the result of a completed task.
    pub fn get_result(&self, task_id: &Uuid) -> Option<&TaskResult> {
        self.results.get(task_id)
    }

    /// Lists all tasks known to this manager.
    pub fn list_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    #[tokio::test]
    async fn test_task_lifecycle() {
        let mut manager = TaskManager::new();
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        // 1. Create
        let task = manager.create_task("Run this".to_string(), peer_id);
        assert_eq!(manager.get_status(&task.id), Some(TaskStatus::Pending));

        // 2. Execute
        let result = manager.execute_task(task.id).await;
        assert!(result.is_some());

        // 3. Verify
        assert_eq!(manager.get_status(&task.id), Some(TaskStatus::Completed));
        let stored_result = manager.get_result(&task.id).unwrap();
        assert_eq!(stored_result.task_id, task.id);
        assert!(stored_result.result.contains("Run this"));
    }

    #[test]
    fn test_list_tasks() {
        let mut manager = TaskManager::new();
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        manager.create_task("Task 1".to_string(), peer_id);
        manager.create_task("Task 2".to_string(), peer_id);

        let list = manager.list_tasks();
        assert_eq!(list.len(), 2);
    }
}
