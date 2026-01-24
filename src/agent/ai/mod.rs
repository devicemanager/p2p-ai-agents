//! AI Module
//!
//! This module provides capabilities for managing and running AI models locally.
//! It includes the `InferenceEngine` for running models and the `ModelManager`
//! for downloading and caching them.

/// Inference engine for running AI models.
pub mod engine;
/// Model manager for downloading and caching models.
pub mod model_manager;

pub use engine::InferenceEngine;
pub use model_manager::ModelManager;
