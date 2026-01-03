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
}
