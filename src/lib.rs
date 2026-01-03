//! P2P AI Agents - A distributed peer-to-peer network of AI agents
//!
//! This library provides the core functionality for creating and managing
//! a network of AI agents that can collaborate on processing tasks in a
//! decentralized manner.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod agent;
/// Application layer
pub mod application;
/// Core architectural components
pub mod core;
/// Metrics collection and monitoring
pub mod metrics;
pub mod network;
/// The storage module provides the pluggable storage layer for the system.
pub mod storage;
// pub mod cli;

/// Re-exports of commonly used types
pub mod prelude {
    pub use crate::agent::{Agent, AgentConfig, AgentId};
    pub use crate::application::{Application, ApplicationError, ApplicationState};
    pub use crate::core::{
        config::{Config, ConfigError, ConfigManager},
        container::Container,
        events::{Event, EventBus, EventHandler, EventResult},
        services::{Service, ServiceError, ServiceRegistry},
    };
    pub use crate::network::{
        discovery::{DiscoveryManager, PeerInfo},
        transport::{TransportError, TransportType},
        NetworkConfig, NetworkError, NetworkManager, NetworkMessage, NetworkResult, ProtocolConfig,
        ResourceLimits, SecurityConfig,
    };
    // Removed broken storage re-exports
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
    Network(#[from] network::NetworkError),

    // Storage-related errors (commented out until implemented)
    // #[error("Storage error: {0}")]
    // Storage(#[from] storage::Error),
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Basic test to ensure the library compiles and loads correctly
        assert_eq!(2 + 2, 4);
    }
}
