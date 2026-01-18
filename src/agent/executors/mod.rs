//! Agent task executors.

/// Executor registry module.
pub mod registry;
pub mod text_processing;
pub mod vector_computation;

pub use registry::ExecutorRegistry;
pub use text_processing::TextProcessingExecutor;
pub use vector_computation::VectorComputationExecutor;
