//! Integration tests for FR1.7: Node Metadata & Version Info
//!
//! Tests metadata query API, version display, and uptime tracking.

use p2p_ai_agents::application::{Application, ApplicationState};
use p2p_ai_agents::core::metadata::{version_display, version_info, NodeMetadata, UptimeTracker};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_version_info() {
    let version = version_info();
    assert!(!version.is_empty());
    assert!(version.contains('.'));
}

#[tokio::test]
async fn test_version_display() {
    let display = version_display();
    assert!(display.contains("p2p-ai-agents"));
    assert!(display.contains(env!("CARGO_PKG_VERSION")));
    assert!(display.contains("build timestamp"));
}

#[tokio::test]
async fn test_metadata_in_stopped_state() {
    let app = Application::new();
    let metadata = app.metadata().await;

    assert_eq!(metadata.node_id, "initializing");
    assert_eq!(metadata.current_state, "Stopped");
    assert!(
        metadata.uptime_seconds.is_none(),
        "Uptime should be None when not Active"
    );
    assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_metadata_in_active_state() {
    let app = Application::new();

    // Initialize and register to reach Active state
    app.initialize().await.expect("Should initialize");
    app.register().await.expect("Should register");

    let state = app.state().await;
    assert_eq!(state, ApplicationState::Active);

    // Small delay to ensure uptime is measurable
    sleep(Duration::from_millis(100)).await;

    let metadata = app.metadata().await;

    assert_eq!(metadata.current_state, "Active");
    assert!(
        metadata.uptime_seconds.is_some(),
        "Uptime should be present when Active"
    );

    let uptime = metadata.uptime_seconds.unwrap();
    assert!(uptime >= 0, "Uptime should be non-negative");
}

#[tokio::test]
async fn test_uptime_tracking_lifecycle() {
    let app = Application::new();

    // Initially stopped - no uptime
    let metadata = app.metadata().await;
    assert!(metadata.uptime_seconds.is_none());

    // Start application
    app.initialize().await.expect("Should initialize");
    app.register().await.expect("Should register");

    // Wait to accumulate uptime
    sleep(Duration::from_millis(200)).await;

    let metadata_active = app.metadata().await;
    assert!(metadata_active.uptime_seconds.is_some());
    let uptime1 = metadata_active.uptime_seconds.unwrap();

    // Wait more and check uptime increases
    sleep(Duration::from_millis(100)).await;

    let metadata_active2 = app.metadata().await;
    let uptime2 = metadata_active2.uptime_seconds.unwrap();
    assert!(uptime2 >= uptime1, "Uptime should increase over time");

    // Stop application
    app.stop().await.expect("Should stop");

    let metadata_stopped = app.metadata().await;
    assert!(
        metadata_stopped.uptime_seconds.is_none(),
        "Uptime should be None after stopping"
    );
}

#[tokio::test]
async fn test_metadata_query_performance() {
    let app = Application::new();
    app.initialize().await.expect("Should initialize");
    app.register().await.expect("Should register");

    let start = std::time::Instant::now();

    // Query metadata 100 times
    for _ in 0..100 {
        let _metadata = app.metadata().await;
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / 100;

    // Should be much faster than 1ms per query (p99 requirement)
    assert!(
        avg_time < Duration::from_millis(1),
        "Average query time should be < 1ms, got {:?}",
        avg_time
    );
}

#[tokio::test]
async fn test_metadata_partial_vs_full() {
    // Test partial metadata (no uptime)
    let partial = NodeMetadata::partial("node-1".to_string(), "Initializing".to_string());
    assert_eq!(partial.node_id, "node-1");
    assert_eq!(partial.current_state, "Initializing");
    assert!(partial.uptime_seconds.is_none());

    // Test full metadata (with uptime)
    let full = NodeMetadata::new("node-2".to_string(), "Active".to_string(), Some(100));
    assert_eq!(full.node_id, "node-2");
    assert_eq!(full.current_state, "Active");
    assert_eq!(full.uptime_seconds, Some(100));
}

#[tokio::test]
async fn test_uptime_tracker() {
    let mut tracker = UptimeTracker::new();

    // Initially not tracking
    assert!(!tracker.is_tracking());
    assert!(tracker.uptime_seconds().is_none());

    // Start tracking
    tracker.start();
    assert!(tracker.is_tracking());
    assert!(tracker.uptime_seconds().is_some());

    // Wait and verify uptime increases
    sleep(Duration::from_millis(100)).await;
    let uptime1 = tracker.uptime_seconds().unwrap();

    sleep(Duration::from_millis(100)).await;
    let uptime2 = tracker.uptime_seconds().unwrap();
    assert!(uptime2 >= uptime1);

    // Stop tracking
    tracker.stop();
    assert!(!tracker.is_tracking());
    assert!(tracker.uptime_seconds().is_none());
}

#[tokio::test]
async fn test_uptime_tracker_restart() {
    let mut tracker = UptimeTracker::new();

    // First session
    tracker.start();
    sleep(Duration::from_millis(100)).await;
    let uptime1 = tracker.uptime_seconds().unwrap();
    tracker.stop();

    // Second session - should start fresh
    tracker.start();
    sleep(Duration::from_millis(50)).await;
    let uptime2 = tracker.uptime_seconds().unwrap();

    // Second session uptime should be less than first (fresh start)
    assert!(
        uptime2 < uptime1 || uptime2 == 0,
        "Second session should start fresh, got uptime2={} vs uptime1={}",
        uptime2,
        uptime1
    );
}

#[tokio::test]
async fn test_metadata_serialization() {
    let metadata = NodeMetadata::new("test-node".to_string(), "Active".to_string(), Some(3600));

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).expect("Should serialize");
    assert!(json.contains("test-node"));
    assert!(json.contains("Active"));
    assert!(json.contains("3600"));

    // Deserialize back
    let deserialized: NodeMetadata = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.node_id, metadata.node_id);
    assert_eq!(deserialized.current_state, metadata.current_state);
    assert_eq!(deserialized.uptime_seconds, metadata.uptime_seconds);
}

#[tokio::test]
async fn test_metadata_builder_pattern() {
    let metadata = NodeMetadata::partial("node-1".to_string(), "Initializing".to_string())
        .with_state("Active".to_string())
        .with_uptime(1000)
        .refresh_timestamp();

    assert_eq!(metadata.current_state, "Active");
    assert_eq!(metadata.uptime_seconds, Some(1000));
}

#[tokio::test]
async fn test_metadata_through_multiple_state_transitions() {
    let app = Application::new();

    // Stopped state
    let meta1 = app.metadata().await;
    assert_eq!(meta1.current_state, "Stopped");
    assert!(meta1.uptime_seconds.is_none());

    // Initializing state
    app.initialize().await.expect("Should initialize");
    let meta2 = app.metadata().await;
    assert_eq!(meta2.current_state, "Registering");
    assert!(meta2.uptime_seconds.is_none());

    // Active state
    app.register().await.expect("Should register");
    sleep(Duration::from_millis(100)).await;
    let meta3 = app.metadata().await;
    assert_eq!(meta3.current_state, "Active");
    assert!(meta3.uptime_seconds.is_some());

    // Shutting down state
    app.stop().await.expect("Should stop");
    let meta4 = app.metadata().await;
    assert_eq!(meta4.current_state, "Stopped");
    assert!(meta4.uptime_seconds.is_none());
}

#[test]
fn test_git_commit_optional() {
    use p2p_ai_agents::core::metadata::git_commit;

    // git_commit should return None if not set via build environment
    let commit = git_commit();
    // This is fine - it's optional
    if let Some(c) = commit {
        assert!(!c.is_empty());
    }
}
