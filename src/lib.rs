//! P2P AI Agents - A distributed peer-to-peer network of AI agents
//! 
//! This library provides the core functionality for creating and managing
//! a network of AI agents that can collaborate on processing tasks in a
//! decentralized manner.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod agent;
pub mod network;
pub mod storage;
pub mod cli;

/// Re-exports of commonly used types
pub mod prelude {
    pub use crate::agent::{Agent, AgentId, AgentConfig};
    pub use crate::network::{Network, NetworkConfig};
    pub use crate::storage::{Storage, StorageConfig};
}

/// Result type for the library
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Agent-related errors
    #[error("Agent error: {0}")]
    Agent(#[from] agent::Error),

    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] network::Error),

    /// Storage-related errors
    #[error("Storage error: {0}")]
    Storage(#[from] storage::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
