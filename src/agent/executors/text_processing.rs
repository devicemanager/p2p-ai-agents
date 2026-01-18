//! Task executors module.
//!
//! This module contains implementations of `TaskExecutor` for different `TaskType`s.

use crate::agent::task::{TaskExecutor, TaskPayload};
use anyhow::Result;
use serde_json::json;

/// Executor for text processing tasks.
pub struct TextProcessingExecutor;

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
            _ => Err(anyhow::anyhow!("Unknown text operation: {}", operation)),
        }
    }
}
