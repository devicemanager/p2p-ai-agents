/// Metrics collection and Prometheus endpoint
///
/// This module provides Prometheus metrics collection for monitoring agent health,
/// performance, and resource usage.
#[cfg(feature = "metrics-prometheus")]
pub mod prometheus_exporter;

#[cfg(feature = "metrics-prometheus")]
pub use prometheus_exporter::{MetricsCollector, MetricsConfig, MetricsServer};

// Re-export for convenience when feature is disabled
#[cfg(not(feature = "metrics-prometheus"))]
/// A no-op metrics collector when Prometheus is disabled.
pub struct MetricsCollector;

#[cfg(not(feature = "metrics-prometheus"))]
impl MetricsCollector {
    /// Creates a new no-op collector.
    pub fn new(_config: MetricsConfig) -> Self {
        Self
    }

    /// Records a storage operation (no-op).
    pub fn record_storage_operation(&self, _operation: &str, _backend: &str, _duration_ms: u64) {}
    /// Records a received message (no-op).
    pub fn record_message_received(&self) {}
    /// Records message processing duration (no-op).
    pub fn record_message_duration(&self, _duration_ms: u64) {}
    /// Updates the count of connected peers (no-op).
    pub fn update_peers_connected(&self, _count: usize) {}
}

#[cfg(not(feature = "metrics-prometheus"))]
#[derive(Debug, Clone)]
/// Configuration for metrics (stub).
pub struct MetricsConfig {
    /// Whether metrics are enabled.
    pub enabled: bool,
    /// Port for the metrics server.
    pub port: u16,
    /// Path for the metrics endpoint.
    pub path: String,
}

#[cfg(not(feature = "metrics-prometheus"))]
impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            path: "/metrics".to_string(),
        }
    }
}

#[cfg(not(feature = "metrics-prometheus"))]
/// A no-op metrics server when Prometheus is disabled.
pub struct MetricsServer;

#[cfg(not(feature = "metrics-prometheus"))]
impl MetricsServer {
    /// Starts the metrics server (no-op).
    pub async fn start(
        _config: MetricsConfig,
        _collector: MetricsCollector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
