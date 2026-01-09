//! Core architectural components for the P2P AI Agents system
//!
//! This module provides the foundational architectural patterns including
//! dependency injection, event handling, service management, access control,
//! load testing, system integration testing utilities, structured logging,
//! correlation ID management, and node identity management.

pub mod access_control;
pub mod config;
pub mod container;
pub mod correlation;
pub mod events;
pub mod identity;
pub mod load_tests;
pub mod logging;
pub mod metadata;
pub mod services;
pub mod system_tests;

pub use access_control::{
    AccessControlManager, AuthResult, AuthToken, Authenticator, Authorizer, AuthzResult,
    DefaultAuthenticator, DefaultAuthorizer, Permission, Policy, PolicyEffect, Principal,
    PrincipalId, Resource, Role, RoleId,
};
pub use config::{Config, ConfigError};
pub use container::Container;
pub use correlation::CorrelationId;
pub use events::{Event, EventBus, EventHandler, EventResult};
pub use identity::{
    generator::{create_new_identity, generate_keypair, keypair_to_hex, NodeIdentity},
    storage::{default_identity_path, load_identity, load_or_create_identity, save_identity},
    IdentityError, NodeIdentityData,
};
pub use load_tests::{LoadTestConfig, LoadTestResult, LoadTestRunner};
pub use logging::{init_default_logging, init_logging, LogFormat, LoggingConfig, LoggingError};
pub use metadata::{
    build_timestamp, git_commit, version_display, version_info, NodeMetadata, UptimeTracker,
};
pub use services::{Service, ServiceError, ServiceRegistry};
