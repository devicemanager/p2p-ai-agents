//! Startup diagnostics and component initialization tracking
//!
//! This module provides detailed tracking of component initialization
//! during startup, including timing information and failure tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Status of a component initialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentStatus {
    /// Component has not started initialization
    Pending,
    /// Component is currently initializing
    Initializing,
    /// Component initialized successfully
    Success,
    /// Component initialization failed
    Failed,
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentStatus::Pending => write!(f, "PENDING"),
            ComponentStatus::Initializing => write!(f, "INITIALIZING"),
            ComponentStatus::Success => write!(f, "✓ SUCCESS"),
            ComponentStatus::Failed => write!(f, "✗ FAILED"),
        }
    }
}

/// Timing information for a component initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTiming {
    /// Component name
    pub name: String,
    /// Start time
    #[serde(skip)]
    pub start: Option<Instant>,
    /// Duration of initialization
    #[serde(serialize_with = "serialize_duration")]
    #[serde(deserialize_with = "deserialize_duration")]
    pub duration: Option<Duration>,
    /// Current status
    pub status: ComponentStatus,
    /// Error message if failed
    pub error: Option<String>,
}

fn serialize_duration<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match duration {
        Some(d) => serializer.serialize_some(&d.as_millis()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let millis: Option<u64> = Option::deserialize(deserializer)?;
    Ok(millis.map(Duration::from_millis))
}

impl ComponentTiming {
    /// Create a new component timing tracker
    pub fn new(name: String) -> Self {
        Self {
            name,
            start: None,
            duration: None,
            status: ComponentStatus::Pending,
            error: None,
        }
    }

    /// Start timing a component
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
        self.status = ComponentStatus::Initializing;
    }

    /// Mark component as successfully initialized
    pub fn success(&mut self) {
        if let Some(start) = self.start {
            self.duration = Some(start.elapsed());
        }
        self.status = ComponentStatus::Success;
    }

    /// Mark component as failed
    pub fn failed(&mut self, error: String) {
        if let Some(start) = self.start {
            self.duration = Some(start.elapsed());
        }
        self.status = ComponentStatus::Failed;
        self.error = Some(error);
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<u128> {
        self.duration.map(|d| d.as_millis())
    }
}

/// Diagnostics for startup initialization
#[derive(Debug, Clone)]
pub struct StartupDiagnostics {
    /// Component timings
    components: Arc<RwLock<HashMap<String, ComponentTiming>>>,
    /// Overall startup start time
    startup_start: Instant,
    /// Whether verbose diagnostics are enabled
    verbose: bool,
}

impl StartupDiagnostics {
    /// Create a new startup diagnostics tracker
    pub fn new(verbose: bool) -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            startup_start: Instant::now(),
            verbose,
        }
    }

    /// Register a component for tracking
    pub async fn register_component(&self, name: &str) {
        let mut components = self.components.write().await;
        components.insert(name.to_string(), ComponentTiming::new(name.to_string()));
    }

    /// Start tracking a component
    pub async fn start_component(&self, name: &str) {
        let mut components = self.components.write().await;
        if let Some(component) = components.get_mut(name) {
            component.start();
            if self.verbose {
                info!("Starting initialization: {}", name);
            }
        }
    }

    /// Mark a component as successfully initialized
    pub async fn component_success(&self, name: &str) {
        let mut components = self.components.write().await;
        if let Some(component) = components.get_mut(name) {
            component.success();
            if self.verbose {
                if let Some(duration_ms) = component.duration_ms() {
                    info!("✓ {} initialized in {}ms", name, duration_ms);
                } else {
                    info!("✓ {} initialized", name);
                }
            }
        }
    }

    /// Mark a component as failed
    pub async fn component_failed(&self, name: &str, error: String) {
        let mut components = self.components.write().await;
        if let Some(component) = components.get_mut(name) {
            component.failed(error.clone());
            if self.verbose {
                if let Some(duration_ms) = component.duration_ms() {
                    warn!("✗ {} failed after {}ms: {}", name, duration_ms, error);
                } else {
                    warn!("✗ {} failed: {}", name, error);
                }
            }
        }
    }

    /// Get timing for a specific component
    pub async fn get_component(&self, name: &str) -> Option<ComponentTiming> {
        let components = self.components.read().await;
        components.get(name).cloned()
    }

    /// Get all component timings
    pub async fn get_all_components(&self) -> HashMap<String, ComponentTiming> {
        let components = self.components.read().await;
        components.clone()
    }

    /// Get total startup duration
    pub fn total_duration(&self) -> Duration {
        self.startup_start.elapsed()
    }

    /// Get total startup duration in milliseconds
    pub fn total_duration_ms(&self) -> u128 {
        self.total_duration().as_millis()
    }

    /// Print a summary of all components
    pub async fn print_summary(&self) {
        let components = self.components.read().await;
        let total_ms = self.total_duration_ms();

        info!("═══════════════════════════════════════");
        info!("      STARTUP DIAGNOSTICS SUMMARY");
        info!("═══════════════════════════════════════");
        info!("Total startup time: {}ms", total_ms);
        info!("");
        info!("Component initialization:");

        let mut sorted_components: Vec<_> = components.values().collect();
        sorted_components.sort_by_key(|c| c.duration_ms().unwrap_or(0));

        for component in sorted_components {
            if let Some(duration_ms) = component.duration_ms() {
                info!(
                    "  {} {} - {}ms",
                    component.status, component.name, duration_ms
                );
            } else {
                info!("  {} {} - (no timing)", component.status, component.name);
            }

            if let Some(error) = &component.error {
                info!("    Error: {}", error);
            }
        }

        info!("═══════════════════════════════════════");
    }

    /// Calculate overhead of diagnostics tracking
    pub async fn diagnostics_overhead(&self) -> Duration {
        // Measure the time it takes to perform common operations
        let start = Instant::now();

        // Simulate typical operations
        for _ in 0..10 {
            let _components = self.components.read().await;
        }

        start.elapsed()
    }
}

impl Default for StartupDiagnostics {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_timing_creation() {
        let timing = ComponentTiming::new("test".to_string());
        assert_eq!(timing.name, "test");
        assert_eq!(timing.status, ComponentStatus::Pending);
        assert!(timing.start.is_none());
        assert!(timing.duration.is_none());
        assert!(timing.error.is_none());
    }

    #[test]
    fn test_component_timing_lifecycle() {
        let mut timing = ComponentTiming::new("test".to_string());

        // Start
        timing.start();
        assert_eq!(timing.status, ComponentStatus::Initializing);
        assert!(timing.start.is_some());

        // Sleep a bit to ensure duration is measurable
        std::thread::sleep(Duration::from_millis(5));

        // Success
        timing.success();
        assert_eq!(timing.status, ComponentStatus::Success);
        assert!(timing.duration.is_some());
        assert!(timing.error.is_none());
        assert!(timing.duration_ms().unwrap() >= 5);
    }

    #[test]
    fn test_component_timing_failure() {
        let mut timing = ComponentTiming::new("test".to_string());

        timing.start();
        std::thread::sleep(Duration::from_millis(5));
        timing.failed("test error".to_string());

        assert_eq!(timing.status, ComponentStatus::Failed);
        assert!(timing.duration.is_some());
        assert_eq!(timing.error, Some("test error".to_string()));
    }

    #[tokio::test]
    async fn test_startup_diagnostics_creation() {
        let diagnostics = StartupDiagnostics::new(false);
        assert!(!diagnostics.verbose);

        let components = diagnostics.get_all_components().await;
        assert!(components.is_empty());
    }

    #[tokio::test]
    async fn test_startup_diagnostics_register() {
        let diagnostics = StartupDiagnostics::new(true);

        diagnostics.register_component("config").await;
        diagnostics.register_component("identity").await;

        let components = diagnostics.get_all_components().await;
        assert_eq!(components.len(), 2);
        assert!(components.contains_key("config"));
        assert!(components.contains_key("identity"));
    }

    #[tokio::test]
    async fn test_startup_diagnostics_lifecycle() {
        let diagnostics = StartupDiagnostics::new(false);

        diagnostics.register_component("test").await;
        diagnostics.start_component("test").await;

        let component = diagnostics.get_component("test").await.unwrap();
        assert_eq!(component.status, ComponentStatus::Initializing);

        tokio::time::sleep(Duration::from_millis(10)).await;

        diagnostics.component_success("test").await;

        let component = diagnostics.get_component("test").await.unwrap();
        assert_eq!(component.status, ComponentStatus::Success);
        assert!(component.duration.is_some());
        assert!(component.duration_ms().unwrap() >= 10);
    }

    #[tokio::test]
    async fn test_startup_diagnostics_failure() {
        let diagnostics = StartupDiagnostics::new(false);

        diagnostics.register_component("test").await;
        diagnostics.start_component("test").await;
        diagnostics
            .component_failed("test", "test error".to_string())
            .await;

        let component = diagnostics.get_component("test").await.unwrap();
        assert_eq!(component.status, ComponentStatus::Failed);
        assert_eq!(component.error, Some("test error".to_string()));
    }

    #[tokio::test]
    async fn test_total_duration() {
        let diagnostics = StartupDiagnostics::new(false);

        tokio::time::sleep(Duration::from_millis(50)).await;

        let total_ms = diagnostics.total_duration_ms();
        assert!(total_ms >= 50);
    }

    #[tokio::test]
    async fn test_diagnostics_overhead() {
        let diagnostics = StartupDiagnostics::new(false);

        // Register some components
        for i in 0..10 {
            diagnostics
                .register_component(&format!("component_{}", i))
                .await;
        }

        let overhead = diagnostics.diagnostics_overhead().await;

        // Overhead should be minimal (< 10ms for 10 read operations)
        assert!(
            overhead.as_millis() < 10,
            "Diagnostics overhead too high: {}ms",
            overhead.as_millis()
        );
    }

    #[tokio::test]
    async fn test_component_status_display() {
        assert_eq!(format!("{}", ComponentStatus::Pending), "PENDING");
        assert_eq!(format!("{}", ComponentStatus::Initializing), "INITIALIZING");
        assert_eq!(format!("{}", ComponentStatus::Success), "✓ SUCCESS");
        assert_eq!(format!("{}", ComponentStatus::Failed), "✗ FAILED");
    }
}
