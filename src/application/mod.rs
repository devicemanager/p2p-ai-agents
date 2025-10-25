//! Application layer for the P2P AI Agents system
//!
//! This module provides the main application architecture that orchestrates
//! all system components using dependency injection and event-driven patterns.

use crate::agent::Agent;
use crate::core::{
    config::{ConfigError, ConfigManager},
    container::Container,
    events::{EventBus, EventHandler, EventResult},
    services::{Service, ServiceError, ServiceRegistry},
};
use crate::network::{NetworkConfig, NetworkManager};
use crate::storage::{StorageManager, StoragePolicy};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Application error types
#[derive(Debug, Error)]
pub enum ApplicationError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Service error
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    /// Agent error
    #[error("Agent error: {0}")]
    Agent(#[from] crate::agent::Error),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] crate::network::NetworkError),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::ManagerError),

    /// Event error
    #[error("Event error: {0}")]
    Event(#[from] crate::core::events::EventError),

    /// Application initialization failed
    #[error("Application initialization failed: {0}")]
    InitializationFailed(String),

    /// Application startup failed
    #[error("Application startup failed: {0}")]
    StartupFailed(String),

    /// Application shutdown failed
    #[error("Application shutdown failed: {0}")]
    ShutdownFailed(String),
}

/// Application state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationState {
    /// Application is initializing
    Initializing,
    /// Application is running
    Running,
    /// Application is stopping
    Stopping,
    /// Application has stopped
    Stopped,
    /// Application encountered an error
    Error(String),
}

/// Main application structure
pub struct Application {
    container: Arc<Container>,
    event_bus: Arc<EventBus>,
    service_registry: Arc<ServiceRegistry>,
    config_manager: Arc<ConfigManager>,
    state: Arc<RwLock<ApplicationState>>,
    agents: Arc<RwLock<Vec<Arc<dyn Agent>>>>,
    network_manager: Arc<RwLock<Option<NetworkManager>>>,
    storage_manager: Arc<RwLock<Option<StorageManager>>>,
}

impl Application {
    /// Create a new application
    pub fn new() -> Self {
        Self {
            container: Arc::new(Container::new()),
            event_bus: Arc::new(EventBus::new()),
            service_registry: Arc::new(ServiceRegistry::new()),
            config_manager: Arc::new(ConfigManager::new()),
            state: Arc::new(RwLock::new(ApplicationState::Stopped)),
            agents: Arc::new(RwLock::new(Vec::new())),
            network_manager: Arc::new(RwLock::new(None)),
            storage_manager: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the application
    pub async fn initialize(&self) -> Result<(), ApplicationError> {
        let mut state = self.state.write().await;
        *state = ApplicationState::Initializing;

        // Load configuration
        self.load_configuration().await?;

        // Register core services
        self.register_core_services().await?;

        // Initialize event handlers
        self.initialize_event_handlers().await?;

        // Initialize storage
        self.initialize_storage().await?;

        // Initialize network
        self.initialize_network().await?;

        *state = ApplicationState::Running;
        Ok(())
    }

    /// Start the application
    pub async fn start(&self) -> Result<(), ApplicationError> {
        let state = self.state.read().await;
        if *state != ApplicationState::Running {
            return Err(ApplicationError::StartupFailed(format!(
                "Application is not in running state: {:?}",
                *state
            )));
        }

        // Start all services
        self.service_registry.start_all().await?;

        // Start network manager
        if let Some(_network_manager) = self.network_manager.read().await.as_ref() {
            // Note: NetworkManager doesn't have a start method in the current implementation
            // This would need to be added
        }

        // Start agents
        let agents = self.agents.read().await;
        for agent in agents.iter() {
            agent.start().await?;
        }

        Ok(())
    }

    /// Stop the application
    pub async fn stop(&self) -> Result<(), ApplicationError> {
        let mut state = self.state.write().await;
        *state = ApplicationState::Stopping;

        // Stop agents
        let agents = self.agents.read().await;
        for agent in agents.iter() {
            if let Err(e) = agent.stop().await {
                tracing::warn!("Failed to stop agent {}: {}", agent.id(), e);
            }
        }

        // Stop services
        if let Err(e) = self.service_registry.stop_all().await {
            tracing::warn!("Failed to stop some services: {}", e);
        }

        *state = ApplicationState::Stopped;
        Ok(())
    }

    /// Get the application state
    pub async fn state(&self) -> ApplicationState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Get the event bus
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    /// Get the service registry
    pub fn service_registry(&self) -> Arc<ServiceRegistry> {
        self.service_registry.clone()
    }

    /// Get the configuration manager
    pub fn config_manager(&self) -> Arc<ConfigManager> {
        self.config_manager.clone()
    }

    /// Add an agent to the application
    pub async fn add_agent(&self, agent: Arc<dyn Agent>) -> Result<(), ApplicationError> {
        let mut agents = self.agents.write().await;
        agents.push(agent);
        Ok(())
    }

    /// Get all agents
    pub async fn agents(&self) -> Vec<Arc<dyn Agent>> {
        let agents = self.agents.read().await;
        agents.clone()
    }

    /// Load configuration
    async fn load_configuration(&self) -> Result<(), ApplicationError> {
        // Load from environment variables
        self.config_manager.load_from_env("P2P_AI_").await?;

        // Try to load from config file
        if let Err(e) = self.config_manager.load_from_file("config.yaml").await {
            tracing::info!("No config file found, using defaults: {}", e);
        }

        // Validate configuration
        self.config_manager.validate().await?;

        Ok(())
    }

    /// Register core services
    async fn register_core_services(&self) -> Result<(), ApplicationError> {
        // Register event bus as a service
        let event_bus_service = EventBusService::new(self.event_bus.clone());
        self.service_registry
            .register(Arc::new(event_bus_service))
            .await?;

        // Register config manager as a service
        let config_service = ConfigService::new(self.config_manager.clone());
        self.service_registry
            .register(Arc::new(config_service))
            .await?;

        Ok(())
    }

    /// Initialize event handlers
    async fn initialize_event_handlers(&self) -> Result<(), ApplicationError> {
        let app_handler = ApplicationEventHandler::new(self.clone());
        self.event_bus
            .subscribe::<crate::core::events::AgentStarted, _>(app_handler.clone())
            .await?;
        self.event_bus
            .subscribe::<crate::core::events::AgentStopped, _>(app_handler.clone())
            .await?;
        self.event_bus
            .subscribe::<crate::core::events::TaskCompleted, _>(app_handler.clone())
            .await?;
        self.event_bus
            .subscribe::<crate::core::events::TaskFailed, _>(app_handler)
            .await?;

        Ok(())
    }

    /// Initialize storage
    async fn initialize_storage(&self) -> Result<(), ApplicationError> {
        let mut storage_manager = StorageManager::new();

        // Configure storage based on configuration
        let storage_policy = self.get_storage_policy().await?;
        storage_manager.set_policy(storage_policy);

        let mut storage = self.storage_manager.write().await;
        *storage = Some(storage_manager);

        Ok(())
    }

    /// Initialize network
    async fn initialize_network(&self) -> Result<(), ApplicationError> {
        let network_config = self.get_network_config().await?;
        let network_manager = NetworkManager::new(network_config);

        let mut network = self.network_manager.write().await;
        *network = Some(network_manager);

        Ok(())
    }

    /// Get storage policy from configuration
    async fn get_storage_policy(&self) -> Result<StoragePolicy, ApplicationError> {
        // Default to first available policy
        Ok(StoragePolicy::FirstAvailable(vec!["local".to_string()]))
    }

    /// Get network configuration
    async fn get_network_config(&self) -> Result<NetworkConfig, ApplicationError> {
        // Create default network configuration
        // In a real implementation, this would read from the config manager
        Ok(NetworkConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            bootstrap_peers: vec![],
            max_peers: 100,
            protocol_config: crate::network::ProtocolConfig {},
            resource_limits: crate::network::ResourceLimits {
                max_bandwidth: 1024 * 1024,     // 1MB/s
                max_memory: 1024 * 1024 * 1024, // 1GB
                max_connections: 100,
            },
            security_config: crate::network::SecurityConfig {},
        })
    }
}

impl Clone for Application {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            event_bus: self.event_bus.clone(),
            service_registry: self.service_registry.clone(),
            config_manager: self.config_manager.clone(),
            state: self.state.clone(),
            agents: self.agents.clone(),
            network_manager: self.network_manager.clone(),
            storage_manager: self.storage_manager.clone(),
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

/// Event bus service wrapper
struct EventBusService {
    #[allow(dead_code)]
    event_bus: Arc<EventBus>,
}

impl EventBusService {
    fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }
}

#[async_trait::async_trait]
impl Service for EventBusService {
    fn id(&self) -> crate::core::services::ServiceId {
        crate::core::services::ServiceId::new()
    }

    fn name(&self) -> &str {
        "event-bus"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn initialize(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn start(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn status(&self) -> crate::core::services::ServiceStatus {
        crate::core::services::ServiceStatus::Running
    }

    async fn health(&self) -> crate::core::services::ServiceHealth {
        crate::core::services::ServiceHealth {
            status: crate::core::services::ServiceStatus::Running,
            uptime: std::time::Duration::ZERO,
            last_heartbeat: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        }
    }

    async fn handle_request(
        &self,
        request: crate::core::services::ServiceRequest,
    ) -> Result<crate::core::services::ServiceResponse, ServiceError> {
        Ok(crate::core::services::ServiceResponse {
            id: request.id,
            success: true,
            data: Some(serde_json::json!({"message": "Event bus service"})),
            error: None,
            duration: std::time::Duration::ZERO,
        })
    }
}

/// Config service wrapper
struct ConfigService {
    #[allow(dead_code)]
    config_manager: Arc<ConfigManager>,
}

impl ConfigService {
    fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self { config_manager }
    }
}

#[async_trait::async_trait]
impl Service for ConfigService {
    fn id(&self) -> crate::core::services::ServiceId {
        crate::core::services::ServiceId::new()
    }

    fn name(&self) -> &str {
        "config"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn initialize(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn start(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn stop(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn status(&self) -> crate::core::services::ServiceStatus {
        crate::core::services::ServiceStatus::Running
    }

    async fn health(&self) -> crate::core::services::ServiceHealth {
        crate::core::services::ServiceHealth {
            status: crate::core::services::ServiceStatus::Running,
            uptime: std::time::Duration::ZERO,
            last_heartbeat: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        }
    }

    async fn handle_request(
        &self,
        request: crate::core::services::ServiceRequest,
    ) -> Result<crate::core::services::ServiceResponse, ServiceError> {
        Ok(crate::core::services::ServiceResponse {
            id: request.id,
            success: true,
            data: Some(serde_json::json!({"message": "Config service"})),
            error: None,
            duration: std::time::Duration::ZERO,
        })
    }
}

/// Application event handler
#[derive(Clone)]
struct ApplicationEventHandler {
    #[allow(dead_code)]
    application: Application,
}

impl ApplicationEventHandler {
    fn new(application: Application) -> Self {
        Self { application }
    }
}

#[async_trait::async_trait]
impl EventHandler<crate::core::events::AgentStarted> for ApplicationEventHandler {
    async fn handle(&self, event: &crate::core::events::AgentStarted) -> EventResult {
        tracing::info!("Agent started: {}", event.payload);
        EventResult::Success
    }

    fn name(&self) -> &'static str {
        "ApplicationEventHandler"
    }
}

#[async_trait::async_trait]
impl EventHandler<crate::core::events::AgentStopped> for ApplicationEventHandler {
    async fn handle(&self, event: &crate::core::events::AgentStopped) -> EventResult {
        tracing::info!("Agent stopped: {}", event.payload);
        EventResult::Success
    }

    fn name(&self) -> &'static str {
        "ApplicationEventHandler"
    }
}

#[async_trait::async_trait]
impl EventHandler<crate::core::events::TaskCompleted> for ApplicationEventHandler {
    async fn handle(&self, event: &crate::core::events::TaskCompleted) -> EventResult {
        tracing::info!("Task completed: {}", event.payload);
        EventResult::Success
    }

    fn name(&self) -> &'static str {
        "ApplicationEventHandler"
    }
}

#[async_trait::async_trait]
impl EventHandler<crate::core::events::TaskFailed> for ApplicationEventHandler {
    async fn handle(&self, event: &crate::core::events::TaskFailed) -> EventResult {
        tracing::warn!("Task failed: {}", event.payload);
        EventResult::Success
    }

    fn name(&self) -> &'static str {
        "ApplicationEventHandler"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_application_creation() {
        let app = Application::new();
        assert_eq!(app.state().await, ApplicationState::Stopped);
    }

    #[tokio::test]
    async fn test_application_initialization() {
        let app = Application::new();
        let result = app.initialize().await;
        assert!(result.is_ok());
        assert_eq!(app.state().await, ApplicationState::Running);
    }
}
