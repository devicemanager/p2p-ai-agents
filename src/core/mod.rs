//! Core architectural components for the P2P AI Agents system
//!
//! This module provides the foundational architectural patterns including
//! dependency injection, event handling, and service management.

pub mod container;
pub mod events;
pub mod services;
pub mod config;

pub use container::Container;
pub use events::{Event, EventBus, EventHandler, EventResult};
pub use services::{Service, ServiceRegistry, ServiceError};
pub use config::{Config, ConfigError, ConfigManager};


