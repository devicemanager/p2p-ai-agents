/// Prometheus metrics exporter
///
/// Provides Prometheus-compatible metrics endpoint via HTTP server.
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Encoder, Gauge,
    HistogramVec, TextEncoder,
};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info};

lazy_static! {
    // Gauges
    static ref PROCESS_CPU_USAGE: Gauge =
        register_gauge!("process_cpu_usage", "CPU usage percentage").unwrap();

    static ref PROCESS_MEMORY_BYTES: Gauge =
        register_gauge!("process_memory_bytes", "Memory usage in bytes").unwrap();

    static ref AGENT_PEERS_CONNECTED: Gauge =
        register_gauge!("agent_peers_connected", "Number of connected peers").unwrap();

    // Counters
    static ref MESSAGES_RECEIVED_TOTAL: CounterVec =
        register_counter_vec!("messages_received_total", "Total messages received", &[]).unwrap();

    static ref STORAGE_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "storage_operations_total",
        "Total storage operations",
        &["operation", "backend"]
    )
    .unwrap();

    // Histograms
    static ref MESSAGE_PROCESSING_DURATION: HistogramVec = register_histogram_vec!(
        "message_processing_duration_seconds",
        "Message processing duration in seconds",
        &[],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    )
    .unwrap();

    static ref STORAGE_OPERATION_DURATION: HistogramVec = register_histogram_vec!(
        "storage_operation_duration_seconds",
        "Storage operation duration in seconds",
        &["operation", "backend"],
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5]
    )
    .unwrap();
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Enable/disable metrics endpoint
    pub enabled: bool,
    /// Port for metrics HTTP server
    pub port: u16,
    /// Path for metrics endpoint
    pub path: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8080,
            path: "/metrics".to_string(),
        }
    }
}

/// Metrics collector for recording application metrics
#[derive(Clone)]
pub struct MetricsCollector {
    #[allow(dead_code)] // Config may be used for future runtime configuration
    config: MetricsConfig,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        debug!("Initializing metrics collector");
        Self { config }
    }

    /// Record a storage operation
    pub fn record_storage_operation(&self, operation: &str, backend: &str, duration_ms: u64) {
        STORAGE_OPERATIONS_TOTAL
            .with_label_values(&[operation, backend])
            .inc();

        let duration_seconds = duration_ms as f64 / 1000.0;
        STORAGE_OPERATION_DURATION
            .with_label_values(&[operation, backend])
            .observe(duration_seconds);

        debug!(
            "Recorded storage operation: {} on {} took {}ms",
            operation, backend, duration_ms
        );
    }

    /// Record a received message
    pub fn record_message_received(&self) {
        MESSAGES_RECEIVED_TOTAL.with_label_values(&[""; 0]).inc();
        debug!("Recorded message received");
    }

    /// Record message processing duration
    pub fn record_message_duration(&self, duration_ms: u64) {
        let duration_seconds = duration_ms as f64 / 1000.0;
        MESSAGE_PROCESSING_DURATION
            .with_label_values(&[""; 0])
            .observe(duration_seconds);
        debug!("Recorded message processing duration: {}ms", duration_ms);
    }

    /// Update number of connected peers
    pub fn update_peers_connected(&self, count: usize) {
        AGENT_PEERS_CONNECTED.set(count as f64);
        debug!("Updated peers connected: {}", count);
    }

    /// Update CPU usage
    pub fn update_cpu_usage(&self, usage: f64) {
        PROCESS_CPU_USAGE.set(usage);
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: u64) {
        PROCESS_MEMORY_BYTES.set(bytes as f64);
    }
}

/// HTTP server for Prometheus metrics endpoint
pub struct MetricsServer;

impl MetricsServer {
    /// Start the metrics HTTP server
    pub async fn start(
        config: MetricsConfig,
        _collector: MetricsCollector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !config.enabled {
            info!("Metrics endpoint disabled");
            return Ok(());
        }

        let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
        let path = Arc::new(config.path.clone());
        let path_for_log = path.clone();

        let make_svc = make_service_fn(move |_conn| {
            let path = Arc::clone(&path);
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let path = Arc::clone(&path);
                    async move { handle_request(req, &path).await }
                }))
            }
        });

        info!("Starting metrics server on http://{}{}", addr, path_for_log);

        Server::bind(&addr).serve(make_svc).await.map_err(|e| {
            error!("Metrics server error: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

        Ok(())
    }
}

async fn handle_request(
    req: Request<Body>,
    metrics_path: &str,
) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, path) if path == metrics_path => {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = vec![];

            if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                error!("Failed to encode metrics: {}", e);
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Failed to encode metrics"))
                    .unwrap());
            }

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; version=0.0.4")
                .body(Body::from(buffer))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 8080);
        assert_eq!(config.path, "/metrics");
    }

    #[test]
    fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let _collector = MetricsCollector::new(config);
    }

    #[test]
    fn test_record_storage_operation() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        // Capture metrics before
        let metrics_before = prometheus::gather();
        let storage_before = metrics_before
            .iter()
            .find(|m| m.name() == "storage_operations_total")
            .map(|m| {
                m.get_metric()
                    .iter()
                    .map(|m| m.get_counter().value.unwrap_or(0.0))
                    .sum::<f64>()
            })
            .unwrap_or(0.0);

        collector.record_storage_operation("get", "local", 10);
        collector.record_storage_operation("put", "redis", 25);

        // Capture metrics after
        let metrics_after = prometheus::gather();
        let storage_after = metrics_after
            .iter()
            .find(|m| m.name() == "storage_operations_total")
            .map(|m| {
                m.get_metric()
                    .iter()
                    .map(|m| m.get_counter().value.unwrap_or(0.0))
                    .sum::<f64>()
            })
            .unwrap_or(0.0);

        // Verify counter incremented by 2
        assert_eq!(
            storage_after - storage_before,
            2.0,
            "storage operations counter should increment"
        );

        // Verify histogram buckets populated
        let duration_metric = metrics_after
            .iter()
            .find(|m| m.name() == "storage_operation_duration_seconds");
        assert!(
            duration_metric.is_some(),
            "storage duration histogram should exist"
        );
    }

    #[test]
    fn test_record_message_metrics() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        // Capture counter before
        let metrics_before = prometheus::gather();
        let messages_before = metrics_before
            .iter()
            .find(|m| m.name() == "messages_received_total")
            .map(|m| {
                m.get_metric()
                    .first()
                    .map(|m| m.get_counter().value.unwrap_or(0.0))
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0);

        collector.record_message_received();
        collector.record_message_duration(50);

        // Capture counter after
        let metrics_after = prometheus::gather();
        let messages_after = metrics_after
            .iter()
            .find(|m| m.name() == "messages_received_total")
            .map(|m| {
                m.get_metric()
                    .first()
                    .map(|m| m.get_counter().value.unwrap_or(0.0))
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0);

        // Verify counter incremented
        assert!(
            messages_after > messages_before,
            "messages counter should increment"
        );

        // Verify histogram exists and has observations
        let duration_metric = metrics_after
            .iter()
            .find(|m| m.name() == "message_processing_duration_seconds");
        assert!(
            duration_metric.is_some(),
            "message duration histogram should exist"
        );
    }

    #[test]
    fn test_update_gauges() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        collector.update_peers_connected(5);
        collector.update_cpu_usage(25.5);
        collector.update_memory_usage(1024 * 1024 * 100);

        // Verify gauges were set correctly
        let metrics = prometheus::gather();

        let peers_gauge = metrics
            .iter()
            .find(|m| m.name() == "agent_peers_connected")
            .and_then(|m| m.get_metric().first())
            .and_then(|m| m.get_gauge().value);
        assert_eq!(peers_gauge, Some(5.0), "peers gauge should be set to 5");

        let cpu_gauge = metrics
            .iter()
            .find(|m| m.name() == "process_cpu_usage")
            .and_then(|m| m.get_metric().first())
            .and_then(|m| m.get_gauge().value);
        assert_eq!(cpu_gauge, Some(25.5), "CPU gauge should be set to 25.5");

        let memory_gauge = metrics
            .iter()
            .find(|m| m.name() == "process_memory_bytes")
            .and_then(|m| m.get_metric().first())
            .and_then(|m| m.get_gauge().value);
        assert_eq!(
            memory_gauge,
            Some(104857600.0),
            "memory gauge should be set correctly"
        );
    }

    #[tokio::test]
    async fn test_metrics_endpoint_disabled() {
        let config = MetricsConfig {
            enabled: false,
            ..Default::default()
        };
        let collector = MetricsCollector::new(config.clone());

        let result = MetricsServer::start(config, collector).await;
        assert!(result.is_ok());
    }
}
