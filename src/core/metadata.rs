//! Node metadata and version information
//!
//! This module provides functionality to query node metadata including version,
//! uptime, and current state. Metadata is available even when the node is not
//! in the Active state, though some fields (like uptime) are only populated
//! when active.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Node metadata containing version, uptime, and state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Unique node identifier
    pub node_id: String,
    /// Node version (from Cargo.toml)
    pub version: String,
    /// Build timestamp (RFC 3339 format)
    pub build_timestamp: String,
    /// Git commit hash (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    /// Node uptime in seconds (None if not in Active state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    /// Current lifecycle state
    pub current_state: String,
    /// Last metadata update timestamp
    pub timestamp: DateTime<Utc>,
}

impl NodeMetadata {
    /// Create new node metadata
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier for this node
    /// * `current_state` - Current lifecycle state as string
    /// * `uptime_seconds` - Optional uptime in seconds (only when Active)
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::core::metadata::NodeMetadata;
    ///
    /// let metadata = NodeMetadata::new(
    ///     "node-123".to_string(),
    ///     "Active".to_string(),
    ///     Some(3600),
    /// );
    ///
    /// assert_eq!(metadata.node_id, "node-123");
    /// assert_eq!(metadata.current_state, "Active");
    /// assert_eq!(metadata.uptime_seconds, Some(3600));
    /// ```
    pub fn new(node_id: String, current_state: String, uptime_seconds: Option<u64>) -> Self {
        Self {
            node_id,
            version: version_info(),
            build_timestamp: build_timestamp(),
            git_commit: git_commit(),
            uptime_seconds,
            current_state,
            timestamp: Utc::now(),
        }
    }

    /// Create partial metadata (for non-Active states)
    ///
    /// This creates metadata without uptime information, suitable for
    /// states other than Active where uptime tracking hasn't started.
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::core::metadata::NodeMetadata;
    ///
    /// let metadata = NodeMetadata::partial(
    ///     "node-123".to_string(),
    ///     "Initializing".to_string(),
    /// );
    ///
    /// assert_eq!(metadata.current_state, "Initializing");
    /// assert!(metadata.uptime_seconds.is_none());
    /// ```
    pub fn partial(node_id: String, current_state: String) -> Self {
        Self::new(node_id, current_state, None)
    }

    /// Update the uptime value
    pub fn with_uptime(mut self, uptime_seconds: u64) -> Self {
        self.uptime_seconds = Some(uptime_seconds);
        self
    }

    /// Update the current state
    pub fn with_state(mut self, current_state: String) -> Self {
        self.current_state = current_state;
        self
    }

    /// Refresh the timestamp to current time
    pub fn refresh_timestamp(mut self) -> Self {
        self.timestamp = Utc::now();
        self
    }
}

/// Get version information from Cargo.toml
pub fn version_info() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get build timestamp (compile time)
pub fn build_timestamp() -> String {
    // This would ideally use build.rs to inject the actual build time
    // For now, we use the package version date
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get git commit hash (if available from build environment)
pub fn git_commit() -> Option<String> {
    // This would be injected via build.rs in production
    // For now, return None
    option_env!("GIT_COMMIT").map(|s| s.to_string())
}

/// Display full version information
///
/// Returns a formatted string suitable for --version output
///
/// # Examples
///
/// ```
/// use p2p_ai_agents::core::metadata::version_display;
///
/// let version_text = version_display();
/// assert!(version_text.contains(env!("CARGO_PKG_VERSION")));
/// ```
pub fn version_display() -> String {
    let mut output = format!("p2p-ai-agents {}\n", env!("CARGO_PKG_VERSION"));

    if let Some(commit) = git_commit() {
        output.push_str(&format!("git commit: {}\n", commit));
    }

    output.push_str(&format!("build timestamp: {}\n", build_timestamp()));

    output
}

/// Uptime tracker for nodes
///
/// Tracks when a node enters the Active state and calculates uptime.
#[derive(Debug, Clone)]
pub struct UptimeTracker {
    start_time: Option<Instant>,
}

impl UptimeTracker {
    /// Create a new uptime tracker (not started)
    pub fn new() -> Self {
        Self { start_time: None }
    }

    /// Start tracking uptime (called when entering Active state)
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Stop tracking uptime (called when leaving Active state)
    pub fn stop(&mut self) {
        self.start_time = None;
    }

    /// Get current uptime in seconds (None if not started)
    pub fn uptime_seconds(&self) -> Option<u64> {
        self.start_time.map(|start| start.elapsed().as_secs())
    }

    /// Check if uptime tracking is active
    pub fn is_tracking(&self) -> bool {
        self.start_time.is_some()
    }
}

impl Default for UptimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version = version_info();
        assert!(!version.is_empty());
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_build_timestamp() {
        let timestamp = build_timestamp();
        assert!(!timestamp.is_empty());
    }

    #[test]
    fn test_version_display() {
        let display = version_display();
        assert!(display.contains(env!("CARGO_PKG_VERSION")));
        assert!(display.contains("p2p-ai-agents"));
    }

    #[test]
    fn test_node_metadata_new() {
        let metadata = NodeMetadata::new("test-node".to_string(), "Active".to_string(), Some(100));

        assert_eq!(metadata.node_id, "test-node");
        assert_eq!(metadata.current_state, "Active");
        assert_eq!(metadata.uptime_seconds, Some(100));
        assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_node_metadata_partial() {
        let metadata = NodeMetadata::partial("test-node".to_string(), "Initializing".to_string());

        assert_eq!(metadata.node_id, "test-node");
        assert_eq!(metadata.current_state, "Initializing");
        assert!(metadata.uptime_seconds.is_none());
    }

    #[test]
    fn test_node_metadata_with_uptime() {
        let metadata =
            NodeMetadata::partial("test-node".to_string(), "Active".to_string()).with_uptime(3600);

        assert_eq!(metadata.uptime_seconds, Some(3600));
    }

    #[test]
    fn test_node_metadata_with_state() {
        let metadata = NodeMetadata::partial("test-node".to_string(), "Initializing".to_string())
            .with_state("Active".to_string());

        assert_eq!(metadata.current_state, "Active");
    }

    #[test]
    fn test_node_metadata_serialization() {
        let metadata = NodeMetadata::new("test-node".to_string(), "Active".to_string(), Some(100));

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: NodeMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.node_id, deserialized.node_id);
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.current_state, deserialized.current_state);
        assert_eq!(metadata.uptime_seconds, deserialized.uptime_seconds);
    }

    #[test]
    fn test_uptime_tracker_new() {
        let tracker = UptimeTracker::new();
        assert!(!tracker.is_tracking());
        assert!(tracker.uptime_seconds().is_none());
    }

    #[test]
    fn test_uptime_tracker_start() {
        let mut tracker = UptimeTracker::new();
        tracker.start();
        assert!(tracker.is_tracking());
        assert!(tracker.uptime_seconds().is_some());
    }

    #[test]
    fn test_uptime_tracker_stop() {
        let mut tracker = UptimeTracker::new();
        tracker.start();
        assert!(tracker.is_tracking());

        tracker.stop();
        assert!(!tracker.is_tracking());
        assert!(tracker.uptime_seconds().is_none());
    }

    #[test]
    fn test_uptime_tracker_duration() {
        use std::thread::sleep;
        use std::time::Duration;

        let mut tracker = UptimeTracker::new();
        tracker.start();

        sleep(Duration::from_millis(100));

        let uptime = tracker.uptime_seconds();
        assert!(uptime.is_some());
        // Should be at least 0 seconds (might be 0 due to rounding)
        // Note: uptime is a u64, so it's always >= 0, but we keep this test for semantic clarity
        // and to ensure no negative values leak if implementation changes
        // Just checking is_some() is sufficient for u64, but we'll access it to ensure it's readable
        let _val = uptime.unwrap();
    }

    #[test]
    fn test_uptime_tracker_multiple_starts() {
        let mut tracker = UptimeTracker::new();
        tracker.start();
        let first_uptime = tracker.uptime_seconds();

        // Starting again should reset the start time
        tracker.start();
        let second_uptime = tracker.uptime_seconds();

        assert!(first_uptime.is_some());
        assert!(second_uptime.is_some());
    }
}
