/// Integration tests for metrics collection
/// Tests that storage operations correctly record metrics.
#[cfg(feature = "metrics-prometheus")]
mod metrics_tests {
    use p2p_ai_agents::metrics::prometheus_exporter::{MetricsCollector, MetricsConfig};
    use p2p_ai_agents::storage::local::{ConsistencyLevel, LocalStorage, Storage};
    use prometheus::{Encoder, TextEncoder};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_storage_metrics_integration() {
        // Setup
        let temp_dir = TempDir::new().unwrap();
        let metrics_config = MetricsConfig::default();
        let metrics = MetricsCollector::new(metrics_config);

        let storage = LocalStorage::with_metrics(temp_dir.path(), metrics.clone())
            .expect("Failed to create LocalStorage with metrics");

        // Get metrics baseline
        let baseline = get_metrics_text();
        let baseline_count = count_storage_operations(&baseline, "local");

        // Perform storage operations
        storage
            .put("test_key", b"test_value".to_vec(), ConsistencyLevel::Strong)
            .await
            .expect("Failed to put");

        storage
            .get("test_key", ConsistencyLevel::Strong)
            .await
            .expect("Failed to get");

        storage
            .delete("test_key", ConsistencyLevel::Strong)
            .await
            .expect("Failed to delete");

        // Get updated metrics
        let updated = get_metrics_text();
        let updated_count = count_storage_operations(&updated, "local");

        // Verify metrics increased
        assert!(
            updated_count > baseline_count,
            "Storage operations should increment metrics. Baseline: {}, Updated: {}",
            baseline_count,
            updated_count
        );

        // Verify we have metrics for each operation type
        assert!(updated.contains("storage_operations_total{backend=\"local\",operation=\"put\"}"));
        assert!(updated.contains("storage_operations_total{backend=\"local\",operation=\"get\"}"));
        assert!(
            updated.contains("storage_operations_total{backend=\"local\",operation=\"delete\"}")
        );

        // Verify histogram metrics exist
        assert!(updated.contains("storage_operation_duration_seconds"));
    }

    #[tokio::test]
    #[ignore] // Requires running Redis instance
    #[cfg(feature = "storage-redis")]
    async fn test_redis_storage_metrics_integration() {
        use p2p_ai_agents::storage::redis::{RedisConfig, RedisStorage};

        let redis_config = RedisConfig::default();
        let metrics_config = MetricsConfig::default();
        let metrics = MetricsCollector::new(metrics_config);

        let storage = RedisStorage::with_metrics(redis_config, metrics.clone())
            .await
            .expect("Failed to create RedisStorage with metrics");

        let baseline = get_metrics_text();
        let baseline_count = count_storage_operations(&baseline, "redis");

        // Perform storage operations
        storage
            .put(
                "test_key_redis",
                b"test_value".to_vec(),
                ConsistencyLevel::Strong,
            )
            .await
            .expect("Failed to put");

        storage
            .get("test_key_redis", ConsistencyLevel::Strong)
            .await
            .expect("Failed to get");

        storage
            .delete("test_key_redis", ConsistencyLevel::Strong)
            .await
            .expect("Failed to delete");

        let updated = get_metrics_text();
        let updated_count = count_storage_operations(&updated, "redis");

        assert!(
            updated_count > baseline_count,
            "Redis storage operations should increment metrics. Baseline: {}, Updated: {}",
            baseline_count,
            updated_count
        );

        assert!(updated.contains("storage_operations_total{backend=\"redis\",operation=\"put\"}"));
        assert!(updated.contains("storage_operations_total{backend=\"redis\",operation=\"get\"}"));
        assert!(
            updated.contains("storage_operations_total{backend=\"redis\",operation=\"delete\"}")
        );
    }

    fn get_metrics_text() -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    fn count_storage_operations(metrics_text: &str, backend: &str) -> usize {
        metrics_text
            .lines()
            .filter(|line| {
                line.contains("storage_operations_total")
                    && line.contains(&format!("backend=\"{}\"", backend))
                    && !line.starts_with('#')
            })
            .count()
    }

    #[tokio::test]
    async fn test_metrics_http_endpoint() {
        use p2p_ai_agents::metrics::prometheus_exporter::MetricsServer;
        use std::time::Duration;

        // Start metrics server
        let config = MetricsConfig {
            enabled: true,
            port: 8081, // Use different port to avoid conflicts
            path: "/metrics".to_string(),
        };
        let collector = MetricsCollector::new(config.clone());

        // Update some metrics before starting server
        collector.update_cpu_usage(45.5);
        collector.update_memory_usage(1024 * 1024 * 512); // 512 MB
        collector.update_peers_connected(5);

        // Start server in background task
        tokio::spawn(async move {
            let _ = MetricsServer::start(config, collector).await;
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Make HTTP request
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:8081/metrics")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .expect("Failed to send HTTP request to metrics endpoint");

        // Verify response
        assert_eq!(
            response.status(),
            200,
            "Expected HTTP 200 response from metrics endpoint"
        );

        // Verify Content-Type header
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header missing");
        assert!(
            content_type.to_str().unwrap().contains("text/plain"),
            "Expected text/plain content type, got: {:?}",
            content_type
        );

        // Verify response body contains expected metrics
        let body = response.text().await.expect("Failed to read response body");

        assert!(
            body.contains("process_cpu_usage"),
            "Metrics should include process_cpu_usage"
        );
        assert!(
            body.contains("process_memory_bytes"),
            "Metrics should include process_memory_bytes"
        );
        assert!(
            body.contains("agent_peers_connected"),
            "Metrics should include agent_peers_connected"
        );
        assert!(
            body.contains("messages_received_total"),
            "Metrics should include messages_received_total"
        );
        assert!(
            body.contains("storage_operations_total"),
            "Metrics should include storage_operations_total"
        );
        assert!(
            body.contains("message_processing_duration_seconds"),
            "Metrics should include message_processing_duration_seconds"
        );
        assert!(
            body.contains("storage_operation_duration_seconds"),
            "Metrics should include storage_operation_duration_seconds"
        );
    }

    #[tokio::test]
    async fn test_message_metrics_integration() {
        use p2p_ai_agents::network::{NetworkConfig, NetworkManager, NetworkMessage};

        // Setup
        let metrics_config = MetricsConfig::default();
        let metrics = MetricsCollector::new(metrics_config);

        let network_config = NetworkConfig {
            listen_addr: "0.0.0.0:8002".parse().unwrap(),
            bootstrap_peers: vec![],
            max_peers: 100,
            protocol_config: p2p_ai_agents::network::ProtocolConfig {},
            resource_limits: p2p_ai_agents::network::ResourceLimits {
                max_bandwidth: 1024 * 1024 * 10,
                max_memory: 512 * 1024 * 1024,
                max_connections: 100,
            },
            security_config: p2p_ai_agents::network::SecurityConfig {},
        };

        let network_manager = NetworkManager::with_metrics(network_config, metrics.clone());

        // Get metrics baseline
        let baseline = get_metrics_text();
        let baseline_msg_count = count_messages_received(&baseline);

        // Send and receive messages
        network_manager
            .send_message(NetworkMessage {
                from: "test_sender".to_string(),
                to: "test_receiver".to_string(),
                content: b"test message 1".to_vec(),
            })
            .await;

        network_manager
            .send_message(NetworkMessage {
                from: "test_sender".to_string(),
                to: "test_receiver".to_string(),
                content: b"test message 2".to_vec(),
            })
            .await;

        // Receive messages to trigger metrics
        let _msg1 = network_manager.receive_message().await;
        let _msg2 = network_manager.receive_message().await;

        // Get updated metrics
        let updated = get_metrics_text();
        let updated_msg_count = count_messages_received(&updated);

        // Verify metrics increased
        assert!(
            updated_msg_count > baseline_msg_count,
            "Message metrics should increment. Baseline: {}, Updated: {}",
            baseline_msg_count,
            updated_msg_count
        );

        // Verify specific metrics exist
        assert!(updated.contains("messages_received_total"));
        assert!(updated.contains("message_processing_duration_seconds"));
    }

    fn count_messages_received(metrics_text: &str) -> usize {
        metrics_text
            .lines()
            .filter(|line| line.contains("messages_received_total") && !line.starts_with('#'))
            .count()
    }
}
