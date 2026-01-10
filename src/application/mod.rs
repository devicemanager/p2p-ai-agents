//! Application layer for the P2P AI Agents system
//!
//! This module provides the main application architecture that orchestrates
//! all system components using dependency injection and event-driven patterns.

pub mod diagnostics;
pub mod lifecycle;
pub mod readiness;
pub mod status;

use crate::agent::Agent;
use crate::core::{
    config::{Config, ConfigError},
    container::Container,
    events::{EventBus, EventHandler, EventResult},
    metadata::{NodeMetadata, UptimeTracker},
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

/// Application state representing the node lifecycle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationState {
    /// Application has stopped and is not running
    Stopped,
    /// Application is initializing (loading config, setting up components)
    Initializing,
    /// Application is registering with the network (connecting to peers, announcing presence)
    Registering,
    /// Application is active and processing tasks
    Active,
    /// Application is shutting down gracefully
    ShuttingDown,
}

impl ApplicationState {
    /// Check if a state transition is valid
    pub fn can_transition_to(&self, next: &ApplicationState) -> bool {
        use ApplicationState::*;
        matches!(
            (self, next),
            // Valid transitions
            (Stopped, Initializing) |
            (Initializing, Registering) |
            (Initializing, ShuttingDown) | // Allow shutdown during init
            (Registering, Active) |
            (Registering, ShuttingDown) | // Allow shutdown during registration
            (Active, ShuttingDown) |
            (ShuttingDown, Stopped)
        )
    }

    /// Get a human-readable description of the state
    pub fn description(&self) -> &'static str {
        match self {
            ApplicationState::Stopped => "Node is stopped",
            ApplicationState::Initializing => "Node is initializing",
            ApplicationState::Registering => "Node is registering with network",
            ApplicationState::Active => "Node is active and processing",
            ApplicationState::ShuttingDown => "Node is shutting down",
        }
    }
}

/// Main application structure
pub struct Application {
    container: Arc<Container>,
    event_bus: Arc<EventBus>,
    service_registry: Arc<ServiceRegistry>,
    config: Arc<RwLock<Config>>,
    state: Arc<RwLock<ApplicationState>>,
    agents: Arc<RwLock<Vec<Arc<dyn Agent>>>>,
    pub(crate) network_manager: Arc<RwLock<Option<NetworkManager>>>,
    storage_manager: Arc<RwLock<Option<StorageManager>>>,
    uptime_tracker: Arc<RwLock<UptimeTracker>>,
}

impl Application {
    /// Create a new application
    pub fn new() -> Self {
        Self {
            container: Arc::new(Container::new()),
            event_bus: Arc::new(EventBus::new()),
            service_registry: Arc::new(ServiceRegistry::new()),
            config: Arc::new(RwLock::new(Config::default())),
            state: Arc::new(RwLock::new(ApplicationState::Stopped)),
            agents: Arc::new(RwLock::new(Vec::new())),
            network_manager: Arc::new(RwLock::new(None)),
            storage_manager: Arc::new(RwLock::new(None)),
            uptime_tracker: Arc::new(RwLock::new(UptimeTracker::new())),
        }
    }

    /// Initialize the application
    pub async fn initialize(&self) -> Result<(), ApplicationError> {
        self.transition_state(ApplicationState::Initializing)
            .await?;

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

        self.transition_state(ApplicationState::Registering).await?;
        Ok(())
    }

    /// Register with the network and become active
    pub async fn register(&self) -> Result<(), ApplicationError> {
        let state = self.state.read().await;
        if *state != ApplicationState::Registering {
            return Err(ApplicationError::StartupFailed(format!(
                "Cannot register from state: {:?}",
                *state
            )));
        }
        drop(state);

        // Perform network registration tasks
        tracing::info!("Registering with network...");

        // Start network manager if available
        if let Some(_network_manager) = self.network_manager.read().await.as_ref() {
            // Network registration would happen here
            tracing::info!("Network manager ready");
        }

        self.transition_state(ApplicationState::Active).await?;

        // Start uptime tracking when entering Active state
        let mut uptime_tracker = self.uptime_tracker.write().await;
        uptime_tracker.start();
        drop(uptime_tracker);

        tracing::info!("Node is now Active, uptime tracking started");

        Ok(())
    }

    /// Start the application
    pub async fn start(&self) -> Result<(), ApplicationError> {
        let state = self.state.read().await;
        if *state != ApplicationState::Active {
            return Err(ApplicationError::StartupFailed(format!(
                "Application is not in active state: {:?}",
                *state
            )));
        }
        drop(state);

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

        tracing::info!("All services started successfully");
        Ok(())
    }

    /// Stop the application
    pub async fn stop(&self) -> Result<(), ApplicationError> {
        self.transition_state(ApplicationState::ShuttingDown)
            .await?;

        // Stop uptime tracking when leaving Active state
        let mut uptime_tracker = self.uptime_tracker.write().await;
        uptime_tracker.stop();
        drop(uptime_tracker);

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

        // Shutdown core components (Network, Storage)
        if let Err(e) = self.shutdown_components().await {
            tracing::warn!("Failed to shutdown components: {}", e);
        }

        self.transition_state(ApplicationState::Stopped).await?;
        Ok(())
    }

    /// Shutdown all application components
    pub async fn shutdown_components(&self) -> Result<(), ApplicationError> {
        // Shutdown network manager
        let mut network_manager = self.network_manager.write().await;
        if let Some(manager) = network_manager.as_mut() {
            tracing::info!("Shutting down network manager...");
            if let Err(e) = manager.graceful_shutdown().await {
                tracing::warn!("Failed to gracefully shutdown network manager: {}", e);
            }
        }

        // Shutdown storage manager
        let storage_manager = self.storage_manager.read().await;
        if let Some(manager) = storage_manager.as_ref() {
            tracing::info!("Shutting down storage manager...");
            use crate::storage::local::Storage; // Import Storage trait to see shutdown method
            if let Err(e) = manager.shutdown().await {
                tracing::warn!("Failed to shutdown storage manager: {}", e);
            }
        }

        Ok(())
    }

    /// Transition to a new state with validation and logging
    async fn transition_state(&self, next_state: ApplicationState) -> Result<(), ApplicationError> {
        let mut state = self.state.write().await;

        // Validate transition
        if !state.can_transition_to(&next_state) {
            return Err(ApplicationError::InitializationFailed(format!(
                "Invalid state transition from {:?} to {:?}",
                *state, next_state
            )));
        }

        tracing::info!(
            from = ?*state,
            to = ?next_state,
            "State transition: {} -> {}",
            state.description(),
            next_state.description()
        );

        *state = next_state;
        Ok(())
    }

    /// Get the application state
    pub async fn state(&self) -> ApplicationState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Get node metadata
    ///
    /// Returns current node metadata including version, uptime, and state.
    /// Uptime is only available when the node is in Active state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use p2p_ai_agents::application::Application;
    /// # tokio_test::block_on(async {
    /// let app = Application::new();
    /// let metadata = app.metadata().await;
    /// assert!(metadata.version.len() > 0);
    /// # })
    /// ```
    pub async fn metadata(&self) -> NodeMetadata {
        let state = self.state.read().await;
        let uptime_tracker = self.uptime_tracker.read().await;

        let agents = self.agents.read().await;
        let node_id = if let Some(agent) = agents.first() {
            agent.id().as_str().to_string()
        } else {
            "initializing".to_string()
        };

        let uptime_seconds = uptime_tracker.uptime_seconds();
        let current_state = format!("{:?}", *state);

        NodeMetadata::new(node_id, current_state, uptime_seconds)
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
    pub fn config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
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
        let loaded_config = Config::load().await?;
        let mut config = self.config.write().await;
        *config = loaded_config;
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
        let config_service = ConfigService::new(self.config.clone());
        self.service_registry
            .register(Arc::new(config_service))
            .await?;

        // Register status manager
        let config = self.config.read().await;
        let status_path = config.storage_path.join("node_status.json");
        drop(config);

        let status_manager = status::StatusManager::new(self.clone()).with_status_file(status_path);

        self.service_registry
            .register(Arc::new(status_manager))
            .await?;

        // Register readiness manager
        let config = self.config.read().await;
        let readiness_config = readiness::ReadinessConfig {
            file_enabled: config.readiness_file_enabled,
            file_path: config.storage_path.join(".ready"),
            port_enabled: config.readiness_port > 0,
            port: config.readiness_port,
        };
        drop(config);

        let readiness_manager =
            readiness::ReadinessManager::new(self.clone()).with_config(readiness_config);

        self.service_registry
            .register(Arc::new(readiness_manager))
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
            config: self.config.clone(),
            state: self.state.clone(),
            agents: self.agents.clone(),
            network_manager: self.network_manager.clone(),
            storage_manager: self.storage_manager.clone(),
            uptime_tracker: self.uptime_tracker.clone(),
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
    config: Arc<RwLock<Config>>,
}

impl ConfigService {
    fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { config }
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
        assert_eq!(app.state().await, ApplicationState::Registering);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        use ApplicationState::*;

        // Test valid transitions
        assert!(Stopped.can_transition_to(&Initializing));
        assert!(Initializing.can_transition_to(&Registering));
        assert!(Registering.can_transition_to(&Active));
        assert!(Active.can_transition_to(&ShuttingDown));
        assert!(ShuttingDown.can_transition_to(&Stopped));

        // Test invalid transitions
        assert!(!Stopped.can_transition_to(&Active));
        assert!(!Active.can_transition_to(&Initializing));
        assert!(!Stopped.can_transition_to(&ShuttingDown));
    }

    #[tokio::test]
    async fn test_state_descriptions() {
        assert_eq!(ApplicationState::Stopped.description(), "Node is stopped");
        assert_eq!(
            ApplicationState::Initializing.description(),
            "Node is initializing"
        );
        assert_eq!(
            ApplicationState::Registering.description(),
            "Node is registering with network"
        );
        assert_eq!(
            ApplicationState::Active.description(),
            "Node is active and processing"
        );
        assert_eq!(
            ApplicationState::ShuttingDown.description(),
            "Node is shutting down"
        );
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let app = Application::new();

        // Start from Stopped
        assert_eq!(app.state().await, ApplicationState::Stopped);

        // Initialize
        app.initialize().await.expect("Should initialize");
        assert_eq!(app.state().await, ApplicationState::Registering);

        // Register and become active
        app.register().await.expect("Should register");
        assert_eq!(app.state().await, ApplicationState::Active);

        // Start services
        app.start().await.expect("Should start");
        assert_eq!(app.state().await, ApplicationState::Active);

        // Shutdown
        app.stop().await.expect("Should stop");
        assert_eq!(app.state().await, ApplicationState::Stopped);
    }

    #[tokio::test]
    async fn test_shutdown_during_init() {
        let app = Application::new();

        // Initialize
        app.initialize().await.expect("Should initialize");
        assert_eq!(app.state().await, ApplicationState::Registering);

        // Should be able to shutdown during registration
        app.stop().await.expect("Should stop");
        assert_eq!(app.state().await, ApplicationState::Stopped);
    }
}
