# Dependency Injection Guide

This guide explains how to use the Dependency Injection (DI) system in the P2P AI Agents project. The DI container is a core architectural component that manages service lifecycles and dependencies, promoting loose coupling and testability.

## 1. Core Concepts

The DI system is built around the `ServiceRegistry` struct (often referred to as the "container"). It handles:

*   **Registration**: Mapping interfaces (Traits) or concrete types to their implementations.
*   **Resolution**: Retrieving instances of registered services.
*   **Lifetimes**: Managing how long a service instance lives (currently primarily Singleton).

### Key Components

*   **`ServiceRegistry`**: The central container.
*   **`Service` Trait**: The base trait that all injectable services must implement.
*   **`ServiceId`**: A unique identifier for each service (usually a UUID).

## 2. Basic Usage

### Defining a Service

All services must implement the `Service` trait. This trait requires `Send + Sync` to ensure thread safety.

```rust
use async_trait::async_trait;
use p2p_ai_agents::core::services::{Service, ServiceRequest, ServiceResponse, ServiceError};
use uuid::Uuid;

// Define your service struct
pub struct MyService {
    id: Uuid,
}

impl MyService {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[async_trait]
impl Service for MyService {
    // Return the unique ID of this service instance
    fn id(&self) -> &Uuid {
        &self.id
    }

    // Handle generic requests (optional, can return error if not used)
    async fn handle_request(&self, _request: ServiceRequest) -> Result<ServiceResponse, ServiceError> {
        Err(ServiceError::NotImplemented("handle_request".to_string()))
    }
}
```

### Registering a Service

Services are registered into the `ServiceRegistry`.

```rust
use std::sync::Arc;
use p2p_ai_agents::core::services::ServiceRegistry;

#[tokio::main]
async fn main() {
    let registry = ServiceRegistry::new();
    let my_service = Arc::new(MyService::new());

    // Register the service
    registry.register_service(my_service.clone()).await.expect("Failed to register");
}
```

### Resolving a Service

To use a service, you resolve it from the registry using its ID.

```rust
// Assuming you have the service ID
let service_id = my_service.id();

if let Some(service) = registry.get_service(service_id).await {
    // Use the service
    println!("Found service: {}", service.id());
} else {
    println!("Service not found");
}
```

## 3. Advanced Patterns

### Service Discovery by Type

(Note: The current implementation primarily supports lookup by ID. Future improvements may add type-based lookup.)

### Handling Requests

The `handle_request` method allows for a uniform interface for inter-service communication, useful for the plugin system or remote procedure calls.

```rust
#[async_trait]
impl Service for CalculatorService {
    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, ServiceError> {
        match request.method.as_str() {
            "add" => {
                // Parse parameters and perform logic
                Ok(ServiceResponse {
                    id: request.id,
                    success: true,
                    data: Some(serde_json::json!({ "result": 42 })),
                    error: None,
                })
            }
            _ => Err(ServiceError::MethodNotFound(request.method)),
        }
    }
}
```

## 4. Testing with DI

DI makes testing easier by allowing you to inject mock services.

### Mocking a Service

You can create a mock implementation of a service for testing dependent components.

```rust
struct MockDatabase {
    id: Uuid,
}

#[async_trait]
impl Service for MockDatabase {
    fn id(&self) -> &Uuid { &self.id }
    // Implement mock behavior
}

#[tokio::test]
async fn test_with_mock() {
    let registry = ServiceRegistry::new();
    let mock_db = Arc::new(MockDatabase { id: Uuid::new_v4() });
    
    registry.register_service(mock_db).await.unwrap();
    
    // Run your test using the registry with the mock
}
```

## 5. Common Pitfalls

### 1. Thread Safety (`Send + Sync`)
All services must be `Send + Sync` because they are shared across threads in the async runtime.
*   **DO**: Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for mutable internal state.
*   **DON'T**: Use `RefCell` or non-thread-safe types in service fields.

### 2. Circular Dependencies
Be careful not to create initialization loops where Service A needs Service B, and Service B needs Service A during construction.
*   **Fix**: Initialize services first, then register them. If they need references to each other, pass the `ServiceRegistry` (or a weak reference to it) to them *after* construction or via a `set_registry` method.

### 3. Service Not Found
If `get_service` returns `None`:
*   Check if `register_service` was actually awaited.
*   Ensure you are using the correct `ServiceId`.

## 6. Troubleshooting

| Error | Cause | Solution |
|-------|-------|----------|
| `the trait bound ... is not satisfied` | Service struct doesn't implement `Service` or isn't `Send + Sync`. | Implement `Service` trait and ensure thread safety. |
| `ServiceNotFound` | Service ID mismatch or registration failed. | Verify registration order and ID propagation. |
