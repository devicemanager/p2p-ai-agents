use super::{Task, TaskResult};
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, instrument};

/// A mock executor that simulates AI model inference.
///
/// In the MVP, we don't load real weights. Instead, we:
/// 1. Accept a Task
/// 2. Wait for a random duration (100-500ms)
/// 3. Return a mock string result
pub struct TaskExecutor;

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskExecutor {
    /// Creates a new instance of the executor.
    pub fn new() -> Self {
        Self
    }

    /// Executes a task asynchronously.
    ///
    /// This method sleeps for a random duration to simulate work,
    /// then returns a successful TaskResult.
    #[instrument(skip(self, task), fields(task_id = %task.id))]
    pub async fn execute(&self, task: Task) -> TaskResult {
        let start_time = std::time::Instant::now();

        // Simulate inference delay (100-500ms)
        let delay_ms = rand::thread_rng().gen_range(100..500);
        sleep(Duration::from_millis(delay_ms)).await;

        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        info!("Task execution completed in {}ms", duration_ms);

        TaskResult {
            task_id: task.id,
            result: format!("Mock result for: '{}'", task.prompt),
            duration_ms,
            completed_at: chrono::Utc::now().timestamp() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;
    use libp2p::PeerId;

    #[tokio::test]
    async fn test_mock_execution() {
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        let task = Task::new("Test Prompt".to_string(), peer_id);

        let executor = TaskExecutor::new();
        let result = executor.execute(task.clone()).await;

        assert_eq!(result.task_id, task.id);
        assert!(result.result.contains("Test Prompt"));
        assert!(result.duration_ms >= 100);
        assert!(result.duration_ms < 600); // Allow some buffer
    }
}
