//! Dependency injection container for managing service lifecycle
//!
//! This module provides a lightweight dependency injection container
//! that manages service registration, resolution, and lifecycle.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

/// Error types for dependency injection operations
#[derive(Debug, Error)]
pub enum ContainerError {
    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    /// Service already registered
    #[error("Service already registered: {0}")]
    ServiceAlreadyRegistered(String),
    
    /// Service registration failed
    #[error("Service registration failed: {0}")]
    RegistrationFailed(String),
    
    /// Service resolution failed
    #[error("Service resolution failed: {0}")]
    ResolutionFailed(String),
}

/// Service factory trait for creating service instances
pub trait ServiceFactory: Send + Sync {
    /// Create a new service instance
    fn create(&self, container: &Container) -> Result<Box<dyn Any + Send + Sync>, ContainerError>;
}

/// Service lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    /// Singleton - one instance for the entire application lifetime
    Singleton,
    /// Scoped - one instance per scope (e.g., per request)
    Scoped,
    /// Transient - new instance every time
    Transient,
}

/// Service registration information
struct ServiceRegistration {
    factory: Box<dyn ServiceFactory>,
    lifetime: ServiceLifetime,
    instance: Option<Box<dyn Any + Send + Sync>>,
}

/// Dependency injection container
pub struct Container {
    services: Arc<RwLock<HashMap<TypeId, ServiceRegistration>>>,
    scoped_instances: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl Container {
    /// Create a new container
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            scoped_instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service with the container
    pub async fn register<T, F>(&self, factory: F, lifetime: ServiceLifetime) -> Result<(), ContainerError>
    where
        T: 'static + Send + Sync,
        F: ServiceFactory + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().await;
        
        if services.contains_key(&type_id) {
            return Err(ContainerError::ServiceAlreadyRegistered(
                std::any::type_name::<T>().to_string()
            ));
        }

        let registration = ServiceRegistration {
            factory: Box::new(factory),
            lifetime,
            instance: None,
        };

        services.insert(type_id, registration);
        Ok(())
    }

    /// Register a singleton service
    pub async fn register_singleton<T, F>(&self, factory: F) -> Result<(), ContainerError>
    where
        T: 'static + Send + Sync,
        F: ServiceFactory + 'static,
    {
        self.register::<T, F>(factory, ServiceLifetime::Singleton).await
    }

    /// Register a scoped service
    pub async fn register_scoped<T, F>(&self, factory: F) -> Result<(), ContainerError>
    where
        T: 'static + Send + Sync,
        F: ServiceFactory + 'static,
    {
        self.register::<T, F>(factory, ServiceLifetime::Scoped).await
    }

    /// Register a transient service
    pub async fn register_transient<T, F>(&self, factory: F) -> Result<(), ContainerError>
    where
        T: 'static + Send + Sync,
        F: ServiceFactory + 'static,
    {
        self.register::<T, F>(factory, ServiceLifetime::Transient).await
    }

    /// Resolve a service from the container
    pub async fn resolve<T>(&self) -> Result<Arc<T>, ContainerError>
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().await;
        
        let registration = services.get_mut(&type_id)
            .ok_or_else(|| ContainerError::ServiceNotFound(
                std::any::type_name::<T>().to_string()
            ))?;

        match registration.lifetime {
            ServiceLifetime::Singleton => {
                if let Some(instance) = &registration.instance {
                    // Return existing singleton instance
                    let instance = instance.downcast_ref::<Arc<T>>()
                        .ok_or_else(|| ContainerError::ResolutionFailed(
                            "Failed to downcast singleton instance".to_string()
                        ))?;
                    Ok(instance.clone())
                } else {
                    // Create new singleton instance
                    let instance = registration.factory.create(self)?;
                    let instance = instance.downcast::<Arc<T>>()
                        .map_err(|_| ContainerError::ResolutionFailed(
                            "Failed to downcast new instance".to_string()
                        ))?;
                    let instance_clone = (*instance).clone();
                    registration.instance = Some(instance as Box<dyn Any + Send + Sync>);
                    Ok(instance_clone)
                }
            }
            ServiceLifetime::Scoped => {
                let mut scoped_instances = self.scoped_instances.write().await;
                if let Some(instance) = scoped_instances.get(&type_id) {
                    let instance = instance.downcast_ref::<Arc<T>>()
                        .ok_or_else(|| ContainerError::ResolutionFailed(
                            "Failed to downcast scoped instance".to_string()
                        ))?;
                    Ok(instance.clone())
                } else {
                    let instance = registration.factory.create(self)?;
                    let instance = instance.downcast::<Arc<T>>()
                        .map_err(|_| ContainerError::ResolutionFailed(
                            "Failed to downcast new scoped instance".to_string()
                        ))?;
                    let instance_clone = instance.clone();
                    scoped_instances.insert(type_id, Box::new(instance));
                    Ok(*instance_clone)
                }
            }
            ServiceLifetime::Transient => {
                let instance = registration.factory.create(self)?;
                let instance = instance.downcast::<Arc<T>>()
                    .map_err(|_| ContainerError::ResolutionFailed(
                        "Failed to downcast new transient instance".to_string()
                    ))?;
                Ok(*instance)
            }
        }
    }

    /// Clear scoped instances (call at the end of each scope)
    pub async fn clear_scoped(&self) {
        let mut scoped_instances = self.scoped_instances.write().await;
        scoped_instances.clear();
    }

    /// Check if a service is registered
    pub async fn is_registered<T>(&self) -> bool
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let services = self.services.read().await;
        services.contains_key(&type_id)
    }

    /// Get all registered service types
    pub async fn registered_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys()
            .map(|_| "Service".to_string()) // In a real implementation, you'd store type names
            .collect()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for easier service factory creation
#[macro_export]
macro_rules! service_factory {
    ($factory_fn:expr) => {{
        struct ServiceFactoryImpl;
        impl $crate::core::container::ServiceFactory for ServiceFactoryImpl {
            fn create(&self, container: &$crate::core::container::Container) -> Result<Box<dyn Any + Send + Sync>, $crate::core::container::ContainerError> {
                let instance = $factory_fn(container)?;
                Ok(Box::new(instance))
            }
        }
        ServiceFactoryImpl
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestService {
        value: String,
    }

    impl TestService {
        fn new(value: String) -> Self {
            Self { value }
        }
    }

    #[tokio::test]
    async fn test_container_registration_and_resolution() {
        let container = Container::new();
        
        // Register a singleton service
        container.register_singleton::<TestService, _>(service_factory!(|_| {
            Ok(Arc::new(TestService::new("test".to_string())))
        })).await.unwrap();

        // Resolve the service
        let service = container.resolve::<TestService>().await.unwrap();
        assert_eq!(service.value, "test");

        // Resolve again should return the same instance
        let service2 = container.resolve::<TestService>().await.unwrap();
        assert!(Arc::ptr_eq(&service, &service2));
    }

    #[tokio::test]
    async fn test_container_service_not_found() {
        let container = Container::new();
        
        let result = container.resolve::<TestService>().await;
        assert!(matches!(result, Err(ContainerError::ServiceNotFound(_))));
    }

    #[tokio::test]
    async fn test_container_already_registered() {
        let container = Container::new();
        
        container.register_singleton::<TestService, _>(service_factory!(|_| {
            Ok(Arc::new(TestService::new("test1".to_string())))
        })).await.unwrap();

        let result = container.register_singleton::<TestService, _>(service_factory!(|_| {
            Ok(Arc::new(TestService::new("test2".to_string())))
        })).await;

        assert!(matches!(result, Err(ContainerError::ServiceAlreadyRegistered(_))));
    }
}
