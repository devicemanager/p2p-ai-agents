//! Integration tests for startup diagnostics and readiness probes

use p2p_ai_agents::application::diagnostics::{ComponentStatus, StartupDiagnostics};
use p2p_ai_agents::application::readiness::{ReadinessConfig, ReadinessManager};
use p2p_ai_agents::application::{lifecycle::LifecycleManager, Application};
use p2p_ai_agents::core::services::Service;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

#[tokio::test]
async fn test_startup_diagnostics_full_lifecycle() {
    // Create diagnostics tracker
    let diagnostics = StartupDiagnostics::new(true);

    // Register all components
    diagnostics.register_component("config").await;
    diagnostics.register_component("identity").await;
    diagnostics.register_component("storage").await;
    diagnostics.register_component("network").await;

    // Simulate initialization sequence
    diagnostics.start_component("config").await;
    sleep(Duration::from_millis(10)).await;
    diagnostics.component_success("config").await;

    diagnostics.start_component("identity").await;
    sleep(Duration::from_millis(15)).await;
    diagnostics.component_success("identity").await;

    diagnostics.start_component("storage").await;
    sleep(Duration::from_millis(20)).await;
    diagnostics.component_success("storage").await;

    diagnostics.start_component("network").await;
    sleep(Duration::from_millis(25)).await;
    diagnostics.component_success("network").await;

    // Verify all components succeeded
    let components = diagnostics.get_all_components().await;
    assert_eq!(components.len(), 4);

    for (name, component) in &components {
        assert_eq!(component.status, ComponentStatus::Success);
        assert!(component.duration.is_some());
        println!("Component {}: {}ms", name, component.duration_ms().unwrap());
    }

    // Verify total duration
    let total_ms = diagnostics.total_duration_ms();
    assert!(total_ms >= 70); // At least sum of all sleeps

    // Print summary
    diagnostics.print_summary().await;
}

#[tokio::test]
async fn test_startup_diagnostics_with_failure() {
    let diagnostics = StartupDiagnostics::new(false);

    diagnostics.register_component("component1").await;
    diagnostics.register_component("component2").await;
    diagnostics.register_component("component3").await;

    // First component succeeds
    diagnostics.start_component("component1").await;
    sleep(Duration::from_millis(5)).await;
    diagnostics.component_success("component1").await;

    // Second component fails
    diagnostics.start_component("component2").await;
    sleep(Duration::from_millis(5)).await;
    diagnostics
        .component_failed("component2", "Connection timeout".to_string())
        .await;

    // Third component shouldn't start due to failure
    // (in real scenario, we'd abort)

    let components = diagnostics.get_all_components().await;
    let comp1 = components.get("component1").unwrap();
    let comp2 = components.get("component2").unwrap();
    let comp3 = components.get("component3").unwrap();

    assert_eq!(comp1.status, ComponentStatus::Success);
    assert_eq!(comp2.status, ComponentStatus::Failed);
    assert_eq!(comp3.status, ComponentStatus::Pending);
    assert_eq!(comp2.error, Some("Connection timeout".to_string()));
}

#[tokio::test]
async fn test_readiness_file_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let ready_path = temp_dir.path().join(".ready");

    let config = ReadinessConfig {
        file_enabled: true,
        file_path: ready_path.clone(),
        port_enabled: false,
        port: 9091,
    };

    let app = Application::new();
    app.initialize().await.unwrap();
    app.register().await.unwrap();

    let manager = ReadinessManager::new(app.clone()).with_config(config);

    // Initialize manager
    manager.initialize().await.unwrap();

    // File shouldn't exist yet
    assert!(!manager.readiness_file_exists());
    assert!(!manager.is_ready().await);

    // Mark ready
    manager.mark_ready().await.unwrap();
    assert!(manager.readiness_file_exists());
    assert!(manager.is_ready().await);

    // Verify file content
    let info = manager.read_readiness_file().await.unwrap();
    assert_eq!(info.state, "Active");
    assert!(info.ready_at <= chrono::Utc::now());

    // Check uptime
    sleep(Duration::from_millis(100)).await;
    let uptime = manager.uptime_since_ready().await;
    assert!(uptime.is_some());
    assert!(uptime.unwrap() >= Duration::from_millis(100));

    // Mark not ready
    manager.mark_not_ready().await.unwrap();
    assert!(!manager.readiness_file_exists());
    assert!(!manager.is_ready().await);
    assert!(manager.uptime_since_ready().await.is_none());
}

#[tokio::test]
async fn test_readiness_probes() {
    let app = Application::new();
    let manager = ReadinessManager::new(app.clone());

    // Initialize service
    manager.initialize().await.unwrap();

    // Before startup - app is in Stopped state, so not live
    assert!(!manager.liveness_probe().await); // Changed to assert False
    assert!(!manager.readiness_probe().await); // Not ready
    assert!(!manager.startup_probe().await); // Not started

    // Initialize and activate
    app.initialize().await.unwrap();
    assert!(manager.liveness_probe().await);
    assert!(!manager.readiness_probe().await);
    assert!(!manager.startup_probe().await);

    app.register().await.unwrap();
    assert!(manager.liveness_probe().await);
    assert!(manager.startup_probe().await); // Startup complete
    assert!(!manager.readiness_probe().await); // But not marked ready yet

    // Mark ready
    manager.mark_ready().await.unwrap();
    assert!(manager.liveness_probe().await);
    assert!(manager.readiness_probe().await); // Now ready
    assert!(manager.startup_probe().await);

    // Shutdown
    app.stop().await.unwrap();
    assert!(!manager.liveness_probe().await);
    assert!(!manager.readiness_probe().await);
    assert!(!manager.startup_probe().await);
}

#[tokio::test]
async fn test_lifecycle_with_diagnostics() {
    let app = Application::new();
    let lifecycle = LifecycleManager::new(app.clone()).with_diagnostics(true);

    // Start with diagnostics
    lifecycle.startup().await.unwrap();

    // Verify app is active
    let state = app.state().await;
    assert_eq!(state, p2p_ai_agents::application::ApplicationState::Active);

    // Graceful shutdown
    lifecycle.shutdown().await.unwrap();

    let state = app.state().await;
    assert_eq!(state, p2p_ai_agents::application::ApplicationState::Stopped);
}

#[tokio::test]
async fn test_readiness_service_health() {
    let app = Application::new();
    app.initialize().await.unwrap();
    app.register().await.unwrap();

    let manager = ReadinessManager::new(app);
    manager.mark_ready().await.unwrap();

    // Get health status
    let health = manager.health().await;
    assert_eq!(
        health.status,
        p2p_ai_agents::core::services::ServiceStatus::Running
    );

    // Check metrics
    assert!(health.metrics.contains_key("ready"));
    assert!(health.metrics.contains_key("liveness"));
    assert!(health.metrics.contains_key("readiness"));
    assert!(health.metrics.contains_key("startup"));

    assert_eq!(
        health.metrics.get("ready"),
        Some(&serde_json::Value::String("true".to_string()))
    );
    assert_eq!(
        health.metrics.get("liveness"),
        Some(&serde_json::Value::String("true".to_string()))
    );
    assert_eq!(
        health.metrics.get("readiness"),
        Some(&serde_json::Value::String("true".to_string()))
    );
    assert_eq!(
        health.metrics.get("startup"),
        Some(&serde_json::Value::String("true".to_string()))
    );
}

#[tokio::test]
async fn test_diagnostics_performance_overhead() {
    let diagnostics = StartupDiagnostics::new(false);

    // Register many components
    for i in 0..100 {
        diagnostics
            .register_component(&format!("component_{}", i))
            .await;
    }

    // Measure overhead
    let start = std::time::Instant::now();
    let overhead = diagnostics.diagnostics_overhead().await;
    let measurement_time = start.elapsed();

    println!("Diagnostics overhead: {}ms", overhead.as_millis());
    println!("Measurement time: {}ms", measurement_time.as_millis());

    // Overhead should be minimal
    assert!(
        overhead.as_millis() < 50,
        "Overhead too high: {}ms",
        overhead.as_millis()
    );
}

#[tokio::test]
async fn test_readiness_check_performance() {
    let app = Application::new();
    app.initialize().await.unwrap();
    app.register().await.unwrap();

    let manager = ReadinessManager::new(app);
    manager.mark_ready().await.unwrap();

    // Measure readiness check time
    let iterations = 1000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = manager.readiness_probe().await;
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

    println!(
        "Average readiness check time: {:.3}ms over {} iterations",
        avg_ms, iterations
    );

    // Should be very fast (< 1ms average)
    assert!(avg_ms < 1.0, "Readiness check too slow: {:.3}ms", avg_ms);
}

#[tokio::test]
async fn test_readiness_file_cleanup_on_shutdown() {
    let temp_dir = TempDir::new().unwrap();
    let ready_path = temp_dir.path().join(".ready");

    let config = ReadinessConfig {
        file_enabled: true,
        file_path: ready_path.clone(),
        port_enabled: false,
        port: 9091,
    };

    let app = Application::new();
    app.initialize().await.unwrap();
    app.register().await.unwrap();

    let manager = ReadinessManager::new(app.clone()).with_config(config);
    manager.initialize().await.unwrap();
    manager.mark_ready().await.unwrap();

    // File should exist
    assert!(manager.readiness_file_exists());

    // Stop the service (simulates shutdown)
    manager.stop().await.unwrap();

    // File should be removed
    assert!(!manager.readiness_file_exists());
}

#[tokio::test]
async fn test_stale_readiness_file_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    let ready_path = temp_dir.path().join(".ready");

    // Create a stale readiness file
    tokio::fs::write(
        &ready_path,
        r#"{"ready_at":"2024-01-01T00:00:00Z","state":"Active","uptime_seconds":0,"metadata":{}}"#,
    )
    .await
    .unwrap();

    let config = ReadinessConfig {
        file_enabled: true,
        file_path: ready_path.clone(),
        port_enabled: false,
        port: 9091,
    };

    let app = Application::new();
    let manager = ReadinessManager::new(app).with_config(config);

    // Initialize should clean up stale file
    manager.initialize().await.unwrap();

    // Stale file should be removed
    assert!(!manager.readiness_file_exists());
}

#[tokio::test]
async fn test_diagnostics_summary_output() {
    let diagnostics = StartupDiagnostics::new(true);

    diagnostics.register_component("fast").await;
    diagnostics.register_component("medium").await;
    diagnostics.register_component("slow").await;

    diagnostics.start_component("fast").await;
    sleep(Duration::from_millis(5)).await;
    diagnostics.component_success("fast").await;

    diagnostics.start_component("medium").await;
    sleep(Duration::from_millis(15)).await;
    diagnostics.component_success("medium").await;

    diagnostics.start_component("slow").await;
    sleep(Duration::from_millis(30)).await;
    diagnostics.component_success("slow").await;

    // Print summary (this will log to console)
    diagnostics.print_summary().await;

    // Verify components are sorted by duration
    let components = diagnostics.get_all_components().await;
    let fast = components.get("fast").unwrap();
    let medium = components.get("medium").unwrap();
    let slow = components.get("slow").unwrap();

    assert!(fast.duration_ms().unwrap() < medium.duration_ms().unwrap());
    assert!(medium.duration_ms().unwrap() < slow.duration_ms().unwrap());
}
