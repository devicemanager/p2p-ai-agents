//! Load testing for core components
//!
//! This module provides load testing utilities for the P2P AI Agents core components
//! including access control, event system, services, and container management.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Barrier;
use tokio::task;

use crate::core::services::{ServiceError, ServiceHealth, ServiceStatus};
use crate::core::{
    Event, EventBus, EventHandler, EventResult, Permission, Principal, PrincipalId, Resource, Role,
    RoleId, Service, ServiceRegistry,
};
use crate::service_factory;

/// Load test results
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    /// Test name
    pub test_name: String,
    /// Total operations performed
    pub total_operations: u64,
    /// Duration of the test
    pub duration: Duration,
    /// Operations per second
    pub ops_per_second: f64,
    /// Average latency per operation
    pub avg_latency_ms: f64,
    /// 95th percentile latency
    pub p95_latency_ms: f64,
    /// 99th percentile latency
    pub p99_latency_ms: f64,
    /// Number of errors encountered
    pub errors: u64,
    /// Peak memory usage (if available)
    pub peak_memory_mb: Option<f64>,
}

/// Load test configuration
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Number of concurrent workers
    pub concurrency: usize,
    /// Test duration
    pub duration: Duration,
    /// Operations per second target (0 = unlimited)
    pub target_ops_per_second: u64,
    /// Whether to collect latency statistics
    pub collect_latency: bool,
    /// Whether to monitor memory usage
    pub monitor_memory: bool,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            duration: Duration::from_secs(30),
            target_ops_per_second: 0,
            collect_latency: true,
            monitor_memory: false,
        }
    }
}

/// Load test runner
pub struct LoadTestRunner {
    config: LoadTestConfig,
}

impl LoadTestRunner {
    /// Create a new load test runner
    pub fn new(config: LoadTestConfig) -> Self {
        Self { config }
    }

    /// Run a load test with the given operation
    pub async fn run<F, Fut>(
        &self,
        test_name: &str,
        operation: F,
    ) -> Result<LoadTestResult, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
            + Send,
    {
        let start_time = Instant::now();
        let barrier = Arc::new(Barrier::new(self.config.concurrency + 1));

        // Spawn worker tasks
        let mut handles = Vec::new();
        let mut result_collectors = Vec::new();

        for _ in 0..self.config.concurrency {
            let barrier = barrier.clone();
            let operation = operation.clone();
            let collect_latency = self.config.collect_latency;
            let test_duration = self.config.duration;

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            result_collectors.push(rx);

            let handle = task::spawn(async move {
                barrier.wait().await;

                let mut local_results = WorkerResults::default();
                if collect_latency {
                    local_results.latencies = Some(Vec::new());
                }

                let test_start = Instant::now();
                loop {
                    let op_start = Instant::now();
                    match operation().await {
                        Ok(()) => {
                            let latency = op_start.elapsed();
                            local_results.operations += 1;
                            if let Some(ref mut latencies) = local_results.latencies {
                                latencies.push(latency.as_millis() as u64);
                            }
                        }
                        Err(_) => {
                            local_results.errors += 1;
                        }
                    }

                    // Check if we should continue (time-based termination)
                    if test_start.elapsed() >= test_duration {
                        break;
                    }
                }

                let _ = tx.send(local_results);
            });

            handles.push(handle);
        }

        // Start the test
        barrier.wait().await;

        // Wait for test duration
        tokio::time::sleep(self.config.duration).await;

        // Collect results
        let mut total_operations = 0u64;
        let mut total_errors = 0u64;
        let mut all_latencies = Vec::new();

        for mut rx in result_collectors {
            if let Some(results) = rx.recv().await {
                total_operations += results.operations;
                total_errors += results.errors;
                if let Some(latencies) = results.latencies {
                    all_latencies.extend(latencies);
                }
            }
        }

        // Wait for all workers to finish
        for handle in handles {
            let _ = handle.await;
        }

        let duration = start_time.elapsed();

        // Calculate statistics
        let ops_per_second = total_operations as f64 / duration.as_secs_f64();

        let (avg_latency_ms, p95_latency_ms, p99_latency_ms) = if !all_latencies.is_empty() {
            all_latencies.sort();

            let avg = all_latencies.iter().sum::<u64>() as f64 / all_latencies.len() as f64;

            let p95_idx = (all_latencies.len() as f64 * 0.95) as usize;
            let p99_idx = (all_latencies.len() as f64 * 0.99) as usize;

            let p95 = all_latencies.get(p95_idx).copied().unwrap_or(0);
            let p99 = all_latencies.get(p99_idx).copied().unwrap_or(0);

            (avg, p95 as f64, p99 as f64)
        } else {
            (0.0, 0.0, 0.0)
        };

        Ok(LoadTestResult {
            test_name: test_name.to_string(),
            total_operations,
            duration,
            ops_per_second,
            avg_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            errors: total_errors,
            peak_memory_mb: None, // TODO: Implement memory monitoring
        })
    }
}

/// Worker results for collecting per-worker statistics
#[derive(Debug, Default)]
struct WorkerResults {
    operations: u64,
    errors: u64,
    latencies: Option<Vec<u64>>,
}

/// Access control load tests
pub mod access_control_load_tests {
    use super::*;
    use crate::core::{AccessControlManager, DefaultAuthenticator, DefaultAuthorizer};

    /// Load test for authentication operations
    pub async fn test_authentication_load(
        config: LoadTestConfig,
    ) -> Result<LoadTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let authenticator = Arc::new(DefaultAuthenticator);
        let acm = AccessControlManager::new(authenticator, Arc::new(DefaultAuthorizer::new()));

        // Pre-populate with test principals
        for i in 0..100 {
            let principal = Principal {
                id: PrincipalId::new(format!("load-test-principal-{}", i)),
                name: format!("Load Test Principal {}", i),
                roles: std::collections::HashSet::new(),
                attributes: HashMap::new(),
            };
            acm.add_principal(principal).await;
        }

        let runner = LoadTestRunner::new(config);

        let operation = {
            let acm = acm.clone();
            move || {
                let acm = acm.clone();
                let principal_id = PrincipalId::new(format!(
                    "load-test-principal-{}",
                    rand::random::<u32>() % 100
                ));
                let credentials = HashMap::new();

                async move {
                    let _result = acm.authenticate(&principal_id, &credentials).await;
                    Ok(())
                }
            }
        };

        runner.run("authentication_load_test", operation).await
    }

    /// Load test for authorization operations
    pub async fn test_authorization_load(
        config: LoadTestConfig,
    ) -> Result<LoadTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let authenticator = Arc::new(DefaultAuthenticator);
        let authorizer = Arc::new(DefaultAuthorizer::new());
        let acm = AccessControlManager::new(authenticator, authorizer.clone());

        // Set up test roles and permissions
        let role = Role {
            id: RoleId::new("load-test-role".to_string()),
            name: "Load Test Role".to_string(),
            description: "Role for load testing".to_string(),
            permissions: {
                let mut perms = HashMap::new();
                let mut res_perms = std::collections::HashSet::new();
                res_perms.insert(Permission::new("read".to_string()));
                perms.insert(Resource::new("test-resource".to_string()), res_perms);
                perms
            },
        };
        acm.add_role(role).await;

        // Pre-populate with test principals
        for i in 0..100 {
            let mut roles = std::collections::HashSet::new();
            roles.insert(RoleId::new("load-test-role".to_string()));

            let principal = Principal {
                id: PrincipalId::new(format!("load-test-principal-{}", i)),
                name: format!("Load Test Principal {}", i),
                roles,
                attributes: HashMap::new(),
            };
            acm.add_principal(principal).await;
        }

        let runner = LoadTestRunner::new(config);

        let operation = {
            let acm = acm.clone();
            move || {
                let acm = acm.clone();
                let principal_id = PrincipalId::new(format!(
                    "load-test-principal-{}",
                    rand::random::<u32>() % 100
                ));

                async move {
                    let _result = acm
                        .is_allowed(
                            &principal_id,
                            &Resource::new("test-resource".to_string()),
                            &Permission::new("read".to_string()),
                        )
                        .await;
                    Ok(())
                }
            }
        };

        runner.run("authorization_load_test", operation).await
    }
}

/// Event system load tests
pub mod event_load_tests {
    use super::*;
    use crate::core::events::EventId;

    /// Mock event for testing
    #[derive(Debug, Clone)]
    struct TestEvent {
        id: EventId,
        timestamp: chrono::DateTime<chrono::Utc>,
        source: Option<String>,
    }

    impl TestEvent {
        fn new(source: Option<String>) -> Self {
            Self {
                id: EventId::new(),
                timestamp: chrono::Utc::now(),
                source,
            }
        }
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "test_event"
        }

        fn id(&self) -> EventId {
            self.id.clone()
        }

        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            self.timestamp
        }

        fn source(&self) -> Option<String> {
            self.source.clone()
        }

        fn clone_event(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
    }

    /// Mock event handler
    struct TestEventHandler {
        counter: Arc<std::sync::atomic::AtomicU64>,
    }

    #[async_trait::async_trait]
    impl EventHandler<TestEvent> for TestEventHandler {
        async fn handle(&self, _event: &TestEvent) -> EventResult {
            self.counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            EventResult::Success
        }

        fn name(&self) -> &'static str {
            "TestEventHandler"
        }
    }

    /// Load test for event publishing
    pub async fn test_event_publishing_load(
        config: LoadTestConfig,
    ) -> Result<LoadTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let event_bus = Arc::new(EventBus::new());

        // Register event handlers
        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
        for _ in 0..5 {
            let handler = TestEventHandler {
                counter: counter.clone(),
            };
            event_bus.subscribe::<TestEvent, _>(handler).await?;
        }

        let runner = LoadTestRunner::new(config);

        let event_bus_clone = event_bus.clone();
        let operation = move || {
            let event_bus = event_bus_clone.clone();

            async move {
                let event = TestEvent::new(None);
                event_bus.publish(event).await?;
                Ok(())
            }
        };

        runner.run("event_publishing_load_test", operation).await
    }
}

/// Service management load tests
pub mod service_load_tests {
    use super::*;
    use crate::core::services::{ServiceId, ServiceRequest, ServiceResponse};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Mock service for testing
    struct MockService {
        id: ServiceId,
        name: String,
        operation_count: Arc<AtomicU64>,
    }

    impl MockService {
        fn new(name: String) -> Self {
            Self {
                id: ServiceId::new(),
                name,
                operation_count: Arc::new(AtomicU64::new(0)),
            }
        }
    }

    #[async_trait]
    impl Service for MockService {
        fn id(&self) -> ServiceId {
            self.id.clone()
        }

        fn name(&self) -> &str {
            &self.name
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

        async fn status(&self) -> ServiceStatus {
            ServiceStatus::Running
        }

        async fn health(&self) -> ServiceHealth {
            ServiceHealth {
                status: ServiceStatus::Running,
                uptime: std::time::Duration::from_secs(60),
                last_heartbeat: chrono::Utc::now(),
                metrics: HashMap::new(),
            }
        }

        async fn handle_request(
            &self,
            request: ServiceRequest,
        ) -> Result<ServiceResponse, ServiceError> {
            let start = std::time::Instant::now();
            self.operation_count.fetch_add(1, Ordering::Relaxed);

            let response = ServiceResponse {
                id: request.id,
                success: true,
                data: Some(serde_json::json!({"result": "success"})),
                error: None,
                duration: start.elapsed(),
            };

            Ok(response)
        }
    }

    /// Load test for service health checks
    pub async fn test_service_health_load(
        config: LoadTestConfig,
    ) -> Result<LoadTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let registry = Arc::new(ServiceRegistry::new());

        // Register multiple mock services
        for i in 0..10 {
            let service = Arc::new(MockService::new(format!("mock-service-{}", i)));
            registry.register(service).await?;
        }

        let runner = LoadTestRunner::new(config);

        let registry_clone = registry.clone();
        let operation = move || {
            let registry = registry_clone.clone();

            async move {
                let services = registry.list_services().await;
                for service_id in services {
                    if let Some(service) = registry.get_service(&service_id).await {
                        let _health = service.health().await;
                    }
                }
                Ok(())
            }
        };

        runner.run("service_health_load_test", operation).await
    }
}

/// Container management load tests
pub mod container_load_tests {
    use super::*;
    use crate::core::Container;

    /// Load test for container operations
    pub async fn test_container_operations_load(
        config: LoadTestConfig,
    ) -> Result<LoadTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let container = Arc::new(Container::new());

        // Register a test service
        container
            .register_singleton::<String, _>(service_factory!(|_| Ok(Arc::new(
                "test-service-value".to_string()
            ))))
            .await?;

        let runner = LoadTestRunner::new(config);

        let container_clone = container.clone();
        let operation = move || {
            let container = container_clone.clone();

            async move {
                // Test resolve operations multiple times
                for _ in 0..10 {
                    let _service: Arc<String> = container.resolve().await?;
                }
                Ok(())
            }
        };

        runner
            .run("container_operations_load_test", operation)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_access_control_authentication_load() {
        let config = LoadTestConfig {
            concurrency: 2,
            duration: Duration::from_secs(1),
            ..Default::default()
        };

        let result = access_control_load_tests::test_authentication_load(config).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.total_operations > 0);
        assert!(result.ops_per_second > 0.0);
        assert_eq!(result.test_name, "authentication_load_test");
    }

    #[tokio::test]
    async fn test_access_control_authorization_load() {
        let config = LoadTestConfig {
            concurrency: 2,
            duration: Duration::from_secs(1),
            ..Default::default()
        };

        let result = access_control_load_tests::test_authorization_load(config).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.total_operations > 0);
        assert!(result.ops_per_second > 0.0);
        assert_eq!(result.test_name, "authorization_load_test");
    }

    #[tokio::test]
    async fn test_event_publishing_load() {
        let config = LoadTestConfig {
            concurrency: 2,
            duration: Duration::from_secs(1),
            ..Default::default()
        };

        let result = event_load_tests::test_event_publishing_load(config).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.total_operations > 0);
        assert!(result.ops_per_second > 0.0);
        assert_eq!(result.test_name, "event_publishing_load_test");
    }

    #[tokio::test]
    async fn test_service_health_load() {
        let config = LoadTestConfig {
            concurrency: 2,
            duration: Duration::from_secs(1),
            ..Default::default()
        };

        let result = service_load_tests::test_service_health_load(config).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.total_operations > 0);
        assert!(result.ops_per_second > 0.0);
        assert_eq!(result.test_name, "service_health_load_test");
    }

    #[tokio::test]
    async fn test_container_operations_load() {
        let config = LoadTestConfig {
            concurrency: 2,
            duration: Duration::from_secs(1),
            ..Default::default()
        };

        let result = container_load_tests::test_container_operations_load(config).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.total_operations > 0);
        assert!(result.ops_per_second > 0.0);
        assert_eq!(result.test_name, "container_operations_load_test");
    }

    #[tokio::test]
    async fn test_load_test_runner() {
        let config = LoadTestConfig {
            concurrency: 2,
            duration: Duration::from_millis(100),
            ..Default::default()
        };

        let runner = LoadTestRunner::new(config);

        let operation = || async {
            // Simple operation that just succeeds
            Ok(())
        };

        let result = runner.run("simple_test", operation).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.total_operations > 0);
        assert!(result.duration.as_millis() >= 100);
        assert_eq!(result.errors, 0);
    }
}
