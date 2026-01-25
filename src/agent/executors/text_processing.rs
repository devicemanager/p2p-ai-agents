//! Task executors module.
//!
//! This module contains implementations of `TaskExecutor` for different `TaskType`s.

use crate::agent::ai::{InferenceEngine, ModelManager};
use crate::agent::task::{TaskExecutor, TaskPayload};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

/// Executor for text processing tasks.
pub struct TextProcessingExecutor {
    /// The manager responsible for downloading and caching AI models.
    pub model_manager: Arc<ModelManager>,
}

impl TextProcessingExecutor {
    /// Creates a new `TextProcessingExecutor`.
    ///
    /// # Arguments
    ///
    /// * `model_manager` - Shared instance of `ModelManager`.
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

#[async_trait::async_trait]
impl TaskExecutor for TextProcessingExecutor {
    async fn execute(&self, payload: &TaskPayload) -> Result<serde_json::Value> {
        let operation = payload
            .data
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let text = payload
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match operation {
            "word_count" => {
                let count = text.split_whitespace().count();
                Ok(json!({ "word_count": count }))
            }
            "reverse" => {
                let reversed: String = text.chars().rev().collect();
                Ok(json!({ "reversed_text": reversed }))
            }
            "tokenize" => {
                let tokens: Vec<&str> = text.split_whitespace().collect();
                Ok(json!({ "tokens": tokens }))
            }
            "embed" => {
                // AI Task!
                let model_name = payload
                    .data
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("prajjwal1/bert-tiny"); // Default tiny model

                let model_path = self.model_manager.ensure_model(model_name).await?;

                let engine = InferenceEngine::new();
                let embedding = engine.embed(&model_path, text).await?;

                Ok(json!({ "embedding": embedding, "model": model_name }))
            }
            _ => Err(anyhow::anyhow!("Unknown text operation: {}", operation)),
        }
    }
}
