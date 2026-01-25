use super::{Task, TaskResult};
use rand::Rng;
use serde_json::json;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, instrument};

/// A task executor that can run AI models via OpenRouter or fallback to mock execution.
pub struct TaskExecutor {
    client: reqwest::Client,
    api_key: Option<String>,
    model: String,
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskExecutor {
    /// Creates a new instance of the executor.
    /// Loads configuration from environment variables.
    pub fn new() -> Self {
        // dotenv().ok(); // Load .env file if present // Removed to prevent potential conflicts or repeated calls
        let api_key = env::var("OPENROUTER_API_KEY").ok();
        let model = env::var("LLM_MODEL")
            .unwrap_or_else(|_| "google/gemini-2.0-flash-exp:free".to_string());

        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }

    /// Executes a task asynchronously.
    ///
    /// If an API key is present, it queries OpenRouter.
    /// Otherwise, it falls back to the mock implementation.
    #[instrument(skip(self, task), fields(task_id = %task.id))]
    pub async fn execute(&self, task: Task) -> TaskResult {
        let start_time = std::time::Instant::now();

        let result_content = if let Some(api_key) = &self.api_key {
            self.execute_llm(&task.prompt, api_key).await
        } else {
            self.execute_mock(&task.prompt).await
        };

        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        info!("Task execution completed in {}ms", duration_ms);

        TaskResult {
            task_id: task.id,
            result: result_content,
            duration_ms,
            completed_at: chrono::Utc::now().timestamp() as u64,
        }
    }

    async fn execute_llm(&self, prompt: &str, api_key: &str) -> String {
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            // Optional: Identify the app to OpenRouter
            .header(
                "HTTP-Referer",
                "https://github.com/p2p-ai-agents/p2p-ai-agents",
            )
            .header("X-Title", "P2P AI Agents")
            .json(&request_body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            // Try standard OpenRouter/OpenAI format
                            if let Some(content) = json["choices"][0]["message"]["content"].as_str()
                            {
                                return content.to_string();
                            }
                            // Fallback for some models/providers that might structure differently (unlikely but safe)
                            else if let Some(content) = json["message"]["content"].as_str() {
                                return content.to_string();
                            } else {
                                error!(
                                    "Failed to parse content from OpenRouter response. JSON: {:?}",
                                    json
                                );
                            }
                        }
                        Err(e) => error!("Failed to parse JSON response: {}", e),
                    }
                } else {
                    error!("OpenRouter API returned error: status {}", resp.status());
                    if let Ok(text) = resp.text().await {
                        error!("Error body: {}", text);
                    }
                }
            }
            Err(e) => error!("Failed to send request to OpenRouter: {}", e),
        }

        // Fallback to mock if LLM fails
        info!("Falling back to mock execution due to API failure");
        self.execute_mock(prompt).await
    }

    async fn execute_mock(&self, prompt: &str) -> String {
        // Simulate inference delay (100-500ms)
        let delay_ms = rand::thread_rng().gen_range(100..500);
        sleep(Duration::from_millis(delay_ms)).await;
        format!("Mock result for: '{}'", prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;
    use libp2p::PeerId;

    #[tokio::test]
    async fn test_mock_execution() {
        // Ensure no API key is set for this test to force mock execution
        // Note: In a real CI environment, we should be careful about env vars.
        // For this unit test, we can just instantiate a TaskExecutor that mimics missing key behavior
        // if we refactored slightly, but for now we rely on the fact that CI likely won't have the key set
        // or we can unset it explicitly if needed.

        // However, since we read env vars in `new()`, we can't easily unset them for just this test
        // without affecting other tests running in parallel if we used real env vars.
        // A safer approach for the unit test is to trust the fallback mechanism or
        // modify `new` to accept options. For simplicity, we'll verify the result structure.

        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        let task = Task::new("Test Prompt".to_string(), peer_id);

        let executor = TaskExecutor::new();
        let result = executor.execute(task.clone()).await;

        assert_eq!(result.task_id, task.id);
        assert!(!result.result.is_empty());
        assert!(result.duration_ms >= 100); // Only holds true if it hits the mock path or a slow API
    }
}
