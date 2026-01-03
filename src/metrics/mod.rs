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
pub struct MetricsCollector;

#[cfg(not(feature = "metrics-prometheus"))]
impl MetricsCollector {
    pub fn new(_config: MetricsConfig) -> Self {
        Self
    }

    pub fn record_storage_operation(&self, _operation: &str, _backend: &str, _duration_ms: u64) {}
    pub fn record_message_received(&self) {}
    pub fn record_message_duration(&self, _duration_ms: u64) {}
    pub fn update_peers_connected(&self, _count: usize) {}
}

#[cfg(not(feature = "metrics-prometheus"))]
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
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
pub struct MetricsServer;

#[cfg(not(feature = "metrics-prometheus"))]
impl MetricsServer {
    pub async fn start(
        _config: MetricsConfig,
        _collector: MetricsCollector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
