//! Vector computation executor.

use crate::agent::task::{TaskExecutor, TaskPayload};
use anyhow::Result;
use serde_json::json;

/// Executor for vector computation tasks.
pub struct VectorComputationExecutor;

#[async_trait::async_trait]
impl TaskExecutor for VectorComputationExecutor {
    async fn execute(&self, payload: &TaskPayload) -> Result<serde_json::Value> {
        let operation = payload
            .data
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match operation {
            "cosine_similarity" => {
                let vec_a = payload
                    .data
                    .get("vector_a")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing vector_a"))?;
                let vec_b = payload
                    .data
                    .get("vector_b")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing vector_b"))?;

                if vec_a.len() != vec_b.len() {
                    return Err(anyhow::anyhow!("Vectors must have same length"));
                }

                let a: Vec<f64> = vec_a.iter().filter_map(|v| v.as_f64()).collect();
                let b: Vec<f64> = vec_b.iter().filter_map(|v| v.as_f64()).collect();

                if a.len() != vec_a.len() || b.len() != vec_b.len() {
                    return Err(anyhow::anyhow!("Invalid vector data"));
                }

                let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let magnitude_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
                let magnitude_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

                if magnitude_a == 0.0 || magnitude_b == 0.0 {
                    return Ok(json!({ "similarity": 0.0 }));
                }

                let similarity = dot_product / (magnitude_a * magnitude_b);
                Ok(json!({ "similarity": similarity }))
            }
            _ => Err(anyhow::anyhow!("Unknown vector operation: {}", operation)),
        }
    }
}
