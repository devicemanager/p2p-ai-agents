//! Core architectural components for the P2P AI Agents system
//!
//! This module provides the foundational architectural patterns including
//! dependency injection, event handling, service management, access control,
//! load testing, system integration testing utilities, structured logging,
//! and correlation ID management.

pub mod access_control;
pub mod config;
pub mod container;
pub mod correlation;
pub mod events;
pub mod load_tests;
pub mod logging;
pub mod services;
pub mod system_tests;

pub use access_control::{
    AccessControlManager, AuthResult, AuthToken, Authenticator, Authorizer, AuthzResult,
    DefaultAuthenticator, DefaultAuthorizer, Permission, Policy, PolicyEffect, Principal,
    PrincipalId, Resource, Role, RoleId,
};
pub use config::{Config, ConfigError, ConfigManager};
pub use container::Container;
pub use correlation::CorrelationId;
pub use events::{Event, EventBus, EventHandler, EventResult};
pub use load_tests::{LoadTestConfig, LoadTestResult, LoadTestRunner};
pub use logging::{init_default_logging, init_logging, LogFormat, LoggingConfig, LoggingError};
pub use services::{Service, ServiceError, ServiceRegistry};
