//! Core architectural components for the P2P AI Agents system
//!
//! This module provides the foundational architectural patterns including
//! dependency injection, event handling, service management, access control,
//! and load testing utilities.

pub mod access_control;
pub mod config;
pub mod container;
pub mod events;
pub mod load_tests;
pub mod services;

pub use access_control::{
    AccessControlManager, AuthResult, AuthToken, Authenticator, Authorizer, AuthzResult,
    DefaultAuthenticator, DefaultAuthorizer, Permission, Policy, PolicyEffect, Principal,
    PrincipalId, Resource, Role, RoleId,
};
pub use config::{Config, ConfigError, ConfigManager};
pub use container::Container;
pub use events::{Event, EventBus, EventHandler, EventResult};
pub use load_tests::{LoadTestConfig, LoadTestResult, LoadTestRunner};
pub use services::{Service, ServiceError, ServiceRegistry};
