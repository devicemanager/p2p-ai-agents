use p2p_ai_agents::agent::ai::ModelManager;
use p2p_ai_agents::agent::executors::TextProcessingExecutor;
use p2p_ai_agents::agent::task::{TaskExecutor, TaskPayload, TaskType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_text_processing_executor_with_ai() {
    let temp_dir = TempDir::new().unwrap();
    let model_manager = Arc::new(ModelManager::new(temp_dir.path()));

    // Create executor with real (mocked via feature flags) model manager
    let executor = TextProcessingExecutor::new(model_manager);

    // 1. Test standard text operation (no AI)
    let payload = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({
            "operation": "word_count",
            "text": "Hello world from Rust"
        }),
        parameters: HashMap::new(),
    };

    let result = executor.execute(&payload).await.unwrap();
    assert_eq!(result.get("word_count").unwrap().as_i64().unwrap(), 4);

    // 2. Test AI Embedding operation
    // This uses the mock download/embed path if features are off, or real if on.
    // Since we can't easily guarantee network in unit tests, we rely on the graceful fallback
    // or mock behavior we implemented in ModelManager and InferenceEngine.
    let ai_payload = TaskPayload {
        task_type: TaskType::TextProcessing,
        data: json!({
            "operation": "embed",
            "text": "Rust is awesome",
            "model": "test-model-mock" // Use simple name to avoid dir creation issues in some envs
        }),
        parameters: HashMap::new(),
    };

    let ai_result = executor.execute(&ai_payload).await.unwrap();

    assert!(ai_result.get("embedding").is_some());
    let embedding = ai_result.get("embedding").unwrap().as_array().unwrap();
    assert!(!embedding.is_empty());

    // Check model name echo
    assert_eq!(
        ai_result.get("model").unwrap().as_str().unwrap(),
        "test-model-mock"
    );
}
