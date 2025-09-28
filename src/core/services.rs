//! Service layer abstraction for the P2P AI Agents system
//!
//! This module provides a service layer pattern for better separation
//! of concerns and business logic encapsulation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

/// Service identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(pub Uuid);

impl ServiceId {
    /// Create a new service identifier
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ServiceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Service status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// Service is starting up
    Starting,
    /// Service is running normally
    Running,
    /// Service is stopping
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service encountered an error
    Error(String),
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Current service status
    pub status: ServiceStatus,
    /// How long the service has been running
    pub uptime: std::time::Duration,
    /// Last time the service reported its health
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    /// Service-specific metrics
    pub metrics: HashMap<String, serde_json::Value>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// List of service dependencies
    pub dependencies: Vec<String>,
    /// Service-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Base service trait
#[async_trait]
pub trait Service: Send + Sync {
    /// Get the service ID
    fn id(&self) -> ServiceId;
    
    /// Get the service name
    fn name(&self) -> &str;
    
    /// Get the service version
    fn version(&self) -> &str;
    
    /// Initialize the service
    async fn initialize(&self) -> Result<(), ServiceError>;
    
    /// Start the service
    async fn start(&self) -> Result<(), ServiceError>;
    
    /// Stop the service
    async fn stop(&self) -> Result<(), ServiceError>;
    
    /// Get the service status
    async fn status(&self) -> ServiceStatus;
    
    /// Get the service health
    async fn health(&self) -> ServiceHealth;
    
    /// Handle a service request
    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, ServiceError>;
}

/// Service request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Request identifier
    pub id: Uuid,
    /// Method name to call
    pub method: String,
    /// Method parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Request timeout
    pub timeout: Option<std::time::Duration>,
}

/// Service response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// Response identifier (matches request ID)
    pub id: Uuid,
    /// Whether the request was successful
    pub success: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Error message if unsuccessful
    pub error: Option<String>,
    /// Request processing duration
    pub duration: std::time::Duration,
}

/// Service registry for managing services
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<ServiceId, Arc<dyn Service>>>>,
    service_configs: Arc<RwLock<HashMap<String, ServiceConfig>>>,
}

/// Error types for service operations
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    /// Service already exists
    #[error("Service already exists: {0}")]
    ServiceAlreadyExists(String),
    
    /// Service initialization failed
    #[error("Service initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Service start failed
    #[error("Service start failed: {0}")]
    StartFailed(String),
    
    /// Service stop failed
    #[error("Service stop failed: {0}")]
    StopFailed(String),
    
    /// Service request failed
    #[error("Service request failed: {0}")]
    RequestFailed(String),
    
    /// Service dependency not found
    #[error("Service dependency not found: {0}")]
    DependencyNotFound(String),
    
    /// Service configuration error
    #[error("Service configuration error: {0}")]
    ConfigurationError(String),
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    pub async fn register(&self, service: Arc<dyn Service>) -> Result<(), ServiceError> {
        let service_id = service.id();
        let service_name = service.name().to_string();
        
        let mut services = self.services.write().await;
        
        if services.contains_key(&service_id) {
            return Err(ServiceError::ServiceAlreadyExists(service_name));
        }
        
        services.insert(service_id, service);
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister(&self, service_id: &ServiceId) -> Result<(), ServiceError> {
        let mut services = self.services.write().await;
        
        if let Some(service) = services.remove(service_id) {
            // Stop the service before removing
            if let Err(e) = service.stop().await {
                tracing::warn!("Failed to stop service {}: {}", service.name(), e);
            }
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound(service_id.0.to_string()))
        }
    }

    /// Get a service by ID
    pub async fn get_service(&self, service_id: &ServiceId) -> Option<Arc<dyn Service>> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// Get a service by name
    pub async fn get_service_by_name(&self, name: &str) -> Option<Arc<dyn Service>> {
        let services = self.services.read().await;
        services.values()
            .find(|service| service.name() == name)
            .cloned()
    }

    /// List all services
    pub async fn list_services(&self) -> Vec<ServiceId> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Start all services
    pub async fn start_all(&self) -> Result<(), ServiceError> {
        let services = self.services.read().await;
        let mut results = Vec::new();
        
        for service in services.values() {
            let result = service.start().await;
            results.push((service.name().to_string(), result));
        }
        
        // Check for any failures
        for (name, result) in results {
            if let Err(e) = result {
                return Err(ServiceError::StartFailed(format!("{}: {}", name, e)));
            }
        }
        
        Ok(())
    }

    /// Stop all services
    pub async fn stop_all(&self) -> Result<(), ServiceError> {
        let services = self.services.read().await;
        let mut results = Vec::new();
        
        for service in services.values() {
            let result = service.stop().await;
            results.push((service.name().to_string(), result));
        }
        
        // Check for any failures
        for (name, result) in results {
            if let Err(e) = result {
                return Err(ServiceError::StopFailed(format!("{}: {}", name, e)));
            }
        }
        
        Ok(())
    }

    /// Get health status of all services
    pub async fn health_check(&self) -> HashMap<String, ServiceHealth> {
        let services = self.services.read().await;
        let mut health_status = HashMap::new();
        
        for service in services.values() {
            let health = service.health().await;
            health_status.insert(service.name().to_string(), health);
        }
        
        health_status
    }

    /// Register service configuration
    pub async fn register_config(&self, config: ServiceConfig) {
        let mut configs = self.service_configs.write().await;
        configs.insert(config.name.clone(), config);
    }

    /// Get service configuration
    pub async fn get_config(&self, name: &str) -> Option<ServiceConfig> {
        let configs = self.service_configs.read().await;
        configs.get(name).cloned()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Base service implementation
pub struct BaseService {
    id: ServiceId,
    name: String,
    version: String,
    status: Arc<RwLock<ServiceStatus>>,
    start_time: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
    metrics: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl BaseService {
    /// Create a new base service
    pub fn new(name: String, version: String) -> Self {
        Self {
            id: ServiceId::new(),
            name,
            version,
            status: Arc::new(RwLock::new(ServiceStatus::Stopped)),
            start_time: Arc::new(RwLock::new(None)),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update service status
    pub async fn set_status(&self, status: ServiceStatus) {
        let mut current_status = self.status.write().await;
        *current_status = status;
    }

    /// Record a metric
    pub async fn record_metric(&self, key: String, value: serde_json::Value) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(key, value);
    }

    /// Get uptime
    pub async fn get_uptime(&self) -> std::time::Duration {
        let start_time = self.start_time.read().await;
        if let Some(start) = *start_time {
            chrono::Utc::now().signed_duration_since(start).to_std().unwrap_or_default()
        } else {
            std::time::Duration::ZERO
        }
    }
}

#[async_trait]
impl Service for BaseService {
    fn id(&self) -> ServiceId {
        self.id.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn initialize(&self) -> Result<(), ServiceError> {
        self.set_status(ServiceStatus::Starting).await;
        Ok(())
    }

    async fn start(&self) -> Result<(), ServiceError> {
        self.set_status(ServiceStatus::Starting).await;
        
        let mut start_time = self.start_time.write().await;
        *start_time = Some(chrono::Utc::now());
        
        self.set_status(ServiceStatus::Running).await;
        Ok(())
    }

    async fn stop(&self) -> Result<(), ServiceError> {
        self.set_status(ServiceStatus::Stopping).await;
        self.set_status(ServiceStatus::Stopped).await;
        Ok(())
    }

    async fn status(&self) -> ServiceStatus {
        let status = self.status.read().await;
        status.clone()
    }

    async fn health(&self) -> ServiceHealth {
        let status = self.status.read().await;
        let uptime = self.get_uptime().await;
        let metrics = self.metrics.read().await;
        
        ServiceHealth {
            status: status.clone(),
            uptime,
            last_heartbeat: chrono::Utc::now(),
            metrics: metrics.clone(),
        }
    }

    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, ServiceError> {
        let start = std::time::Instant::now();
        
        // Default implementation - override in derived services
        let response = ServiceResponse {
            id: request.id,
            success: false,
            data: None,
            error: Some("Method not implemented".to_string()),
            duration: start.elapsed(),
        };
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestService {
        base: BaseService,
    }

    impl TestService {
        fn new() -> Self {
            Self {
                base: BaseService::new("test-service".to_string(), "1.0.0".to_string()),
            }
        }
    }

    #[async_trait]
    impl Service for TestService {
        fn id(&self) -> ServiceId {
            self.base.id()
        }

        fn name(&self) -> &str {
            self.base.name()
        }

        fn version(&self) -> &str {
            self.base.version()
        }

        async fn initialize(&self) -> Result<(), ServiceError> {
            self.base.initialize().await
        }

        async fn start(&self) -> Result<(), ServiceError> {
            self.base.start().await
        }

        async fn stop(&self) -> Result<(), ServiceError> {
            self.base.stop().await
        }

        async fn status(&self) -> ServiceStatus {
            self.base.status().await
        }

        async fn health(&self) -> ServiceHealth {
            self.base.health().await
        }

        async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, ServiceError> {
            self.base.handle_request(request).await
        }
    }

    #[tokio::test]
    async fn test_service_registry() {
        let registry = ServiceRegistry::new();
        
        let service = Arc::new(TestService::new());
        let service_id = service.id();
        
        // Register service
        registry.register(service.clone()).await.unwrap();
        
        // Get service
        let retrieved = registry.get_service(&service_id).await.unwrap();
        assert_eq!(retrieved.name(), "test-service");
        
        // List services
        let services = registry.list_services().await;
        assert_eq!(services.len(), 1);
        assert!(services.contains(&service_id));
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        let service = TestService::new();
        
        // Initialize
        service.initialize().await.unwrap();
        assert!(matches!(service.status().await, ServiceStatus::Starting));
        
        // Start
        service.start().await.unwrap();
        assert!(matches!(service.status().await, ServiceStatus::Running));
        
        // Stop
        service.stop().await.unwrap();
        assert!(matches!(service.status().await, ServiceStatus::Stopped));
    }
}
