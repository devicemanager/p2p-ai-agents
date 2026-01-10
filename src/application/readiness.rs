//! Readiness indicators for orchestration tools
//!
//! This module provides readiness indicators through file-based and
//! port-based mechanisms for Kubernetes and other orchestration tools.

use crate::application::{Application, ApplicationState};
use crate::core::services::{Service, ServiceError, ServiceHealth, ServiceId, ServiceStatus};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Configuration for readiness probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessConfig {
    /// Enable file-based readiness indicator
    pub file_enabled: bool,
    /// Path to readiness file
    pub file_path: PathBuf,
    /// Enable port-based readiness indicator
    pub port_enabled: bool,
    /// Port for readiness probe
    pub port: u16,
}

impl Default for ReadinessConfig {
    fn default() -> Self {
        Self {
            file_enabled: true,
            file_path: PathBuf::from("data/.ready"),
            port_enabled: false,
            port: 9091,
        }
    }
}

/// Readiness file content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessInfo {
    /// When the node became ready
    pub ready_at: chrono::DateTime<Utc>,
    /// Node state
    pub state: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ReadinessInfo {
    /// Create new readiness info
    pub fn new(state: String, uptime_seconds: u64) -> Self {
        Self {
            ready_at: Utc::now(),
            state,
            uptime_seconds,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Readiness manager for the application
pub struct ReadinessManager {
    config: ReadinessConfig,
    application: Application,
    ready: Arc<RwLock<bool>>,
    ready_since: Arc<RwLock<Option<Instant>>>,
    service_id: ServiceId,
}

impl ReadinessManager {
    /// Create a new readiness manager
    pub fn new(application: Application) -> Self {
        Self {
            config: ReadinessConfig::default(),
            application,
            ready: Arc::new(RwLock::new(false)),
            ready_since: Arc::new(RwLock::new(None)),
            service_id: ServiceId::new(),
        }
    }

    /// Configure the readiness manager
    pub fn with_config(mut self, config: ReadinessConfig) -> Self {
        self.config = config;
        self
    }

    /// Check if node is ready
    pub async fn is_ready(&self) -> bool {
        *self.ready.read().await
    }

    /// Get uptime since ready
    pub async fn uptime_since_ready(&self) -> Option<Duration> {
        self.ready_since
            .read()
            .await
            .as_ref()
            .map(|instant| instant.elapsed())
    }

    /// Mark node as ready
    pub async fn mark_ready(&self) -> Result<(), ServiceError> {
        let state = self.application.state().await;

        if state != ApplicationState::Active {
            return Err(ServiceError::InitializationFailed(format!(
                "Cannot mark ready in state: {:?}",
                state
            )));
        }

        // Update internal state
        let mut ready = self.ready.write().await;
        *ready = true;
        drop(ready);

        let mut ready_since = self.ready_since.write().await;
        *ready_since = Some(Instant::now());
        drop(ready_since);

        // Create readiness file if enabled
        if self.config.file_enabled {
            self.create_readiness_file().await?;
        }

        info!("Node marked as ready");
        Ok(())
    }

    /// Mark node as not ready
    pub async fn mark_not_ready(&self) -> Result<(), ServiceError> {
        // Update internal state
        let mut ready = self.ready.write().await;
        *ready = false;
        drop(ready);

        let mut ready_since = self.ready_since.write().await;
        *ready_since = None;
        drop(ready_since);

        // Remove readiness file if enabled
        if self.config.file_enabled {
            self.remove_readiness_file().await?;
        }

        info!("Node marked as not ready");
        Ok(())
    }

    /// Create readiness file
    async fn create_readiness_file(&self) -> Result<(), ServiceError> {
        let metadata = self.application.metadata().await;

        let info = ReadinessInfo::new(metadata.current_state, metadata.uptime_seconds.unwrap_or(0))
            .with_metadata("version".to_string(), metadata.version)
            .with_metadata("node_id".to_string(), metadata.node_id);

        let json = serde_json::to_string_pretty(&info).map_err(|e| {
            ServiceError::InitializationFailed(format!("Failed to serialize readiness info: {}", e))
        })?;

        // Ensure parent directory exists
        if let Some(parent) = self.config.file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    ServiceError::InitializationFailed(format!(
                        "Failed to create readiness file directory: {}",
                        e
                    ))
                })?;
            }
        }

        fs::write(&self.config.file_path, json).await.map_err(|e| {
            ServiceError::InitializationFailed(format!("Failed to write readiness file: {}", e))
        })?;

        info!(
            "Created readiness file: {}",
            self.config.file_path.display()
        );
        Ok(())
    }

    /// Remove readiness file
    async fn remove_readiness_file(&self) -> Result<(), ServiceError> {
        if self.config.file_path.exists() {
            fs::remove_file(&self.config.file_path).await.map_err(|e| {
                ServiceError::InitializationFailed(format!(
                    "Failed to remove readiness file: {}",
                    e
                ))
            })?;
            info!(
                "Removed readiness file: {}",
                self.config.file_path.display()
            );
        }
        Ok(())
    }

    /// Check if readiness file exists
    pub fn readiness_file_exists(&self) -> bool {
        self.config.file_path.exists()
    }

    /// Read readiness info from file
    pub async fn read_readiness_file(&self) -> Result<ReadinessInfo, ServiceError> {
        let content = fs::read_to_string(&self.config.file_path)
            .await
            .map_err(|e| {
                ServiceError::InitializationFailed(format!("Failed to read readiness file: {}", e))
            })?;

        let info: ReadinessInfo = serde_json::from_str(&content).map_err(|e| {
            ServiceError::InitializationFailed(format!("Failed to parse readiness file: {}", e))
        })?;

        Ok(info)
    }

    /// Liveness probe - checks if process is alive
    pub async fn liveness_probe(&self) -> bool {
        let state = self.application.state().await;
        // The process is alive unless it's completely stopped or failed
        // For liveness, we want to return true even during initialization
        !matches!(state, ApplicationState::Stopped)
    }

    /// Readiness probe - checks if ready for traffic
    pub async fn readiness_probe(&self) -> bool {
        let state = self.application.state().await;
        state == ApplicationState::Active && self.is_ready().await
    }

    /// Startup probe - checks if startup is complete
    pub async fn startup_probe(&self) -> bool {
        let state = self.application.state().await;
        matches!(state, ApplicationState::Active)
    }
}

/// Check if a port is available
pub fn is_port_available(port: u16) -> bool {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    TcpListener::bind(addr).is_ok()
}

/// Find an available port starting from the given port
pub fn find_available_port(start_port: u16) -> Option<u16> {
    (start_port..start_port + 100).find(|&port| is_port_available(port))
}

#[async_trait::async_trait]
impl Service for ReadinessManager {
    fn id(&self) -> ServiceId {
        self.service_id.clone()
    }

    fn name(&self) -> &str {
        "readiness-manager"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn initialize(&self) -> Result<(), ServiceError> {
        info!("Initializing readiness manager");

        // Ensure readiness file doesn't exist on startup
        if self.config.file_enabled && self.readiness_file_exists() {
            warn!("Removing stale readiness file from previous run");
            self.remove_readiness_file().await?;
        }

        // Check port availability if enabled
        if self.config.port_enabled && !is_port_available(self.config.port) {
            warn!("Readiness port {} is not available", self.config.port);
            if let Some(alt_port) = find_available_port(self.config.port) {
                warn!("Alternative port {} is available", alt_port);
            }
        }

        Ok(())
    }

    async fn start(&self) -> Result<(), ServiceError> {
        info!("Starting readiness manager");

        // Monitor application state and automatically mark ready when Active
        let app = self.application.clone();
        let ready_manager = Arc::new(self.clone());

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let state = app.state().await;
                let is_ready = ready_manager.is_ready().await;

                match state {
                    ApplicationState::Active if !is_ready => {
                        if let Err(e) = ready_manager.mark_ready().await {
                            error!("Failed to mark node as ready: {}", e);
                        }
                    }
                    ApplicationState::Stopped
                    | ApplicationState::ShuttingDown
                    | ApplicationState::Initializing
                    | ApplicationState::Registering
                        if is_ready =>
                    {
                        if let Err(e) = ready_manager.mark_not_ready().await {
                            error!("Failed to mark node as not ready: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<(), ServiceError> {
        info!("Stopping readiness manager");
        self.mark_not_ready().await?;
        Ok(())
    }

    async fn status(&self) -> ServiceStatus {
        if self.is_ready().await {
            ServiceStatus::Running
        } else {
            ServiceStatus::Starting
        }
    }

    async fn health(&self) -> ServiceHealth {
        let status = self.status().await;
        let uptime = self.uptime_since_ready().await.unwrap_or(Duration::ZERO);
        let mut metrics = HashMap::new();

        metrics.insert(
            "ready".to_string(),
            serde_json::Value::String(self.is_ready().await.to_string()),
        );
        metrics.insert(
            "liveness".to_string(),
            serde_json::Value::String(self.liveness_probe().await.to_string()),
        );
        metrics.insert(
            "readiness".to_string(),
            serde_json::Value::String(self.readiness_probe().await.to_string()),
        );
        metrics.insert(
            "startup".to_string(),
            serde_json::Value::String(self.startup_probe().await.to_string()),
        );

        ServiceHealth {
            status,
            uptime,
            last_heartbeat: Utc::now(),
            metrics,
        }
    }

    async fn handle_request(
        &self,
        request: crate::core::services::ServiceRequest,
    ) -> Result<crate::core::services::ServiceResponse, ServiceError> {
        let start = Instant::now();

        let data = match request.method.as_str() {
            "liveness" => {
                let alive = self.liveness_probe().await;
                Some(serde_json::json!({ "alive": alive }))
            }
            "readiness" => {
                let ready = self.readiness_probe().await;
                Some(serde_json::json!({ "ready": ready }))
            }
            "startup" => {
                let started = self.startup_probe().await;
                Some(serde_json::json!({ "started": started }))
            }
            _ => None,
        };

        let success = data.is_some();
        let error = if !success {
            Some(format!("Unknown action: {}", request.method))
        } else {
            None
        };

        Ok(crate::core::services::ServiceResponse {
            id: request.id,
            success,
            data,
            error,
            duration: start.elapsed(),
        })
    }
}

impl Clone for ReadinessManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            application: self.application.clone(),
            ready: self.ready.clone(),
            ready_since: self.ready_since.clone(),
            service_id: self.service_id.clone(),
        }
    }
}

/// Helper function to check if path is a readiness file
pub fn is_readiness_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == ".ready")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readiness_config_default() {
        let config = ReadinessConfig::default();
        assert!(config.file_enabled);
        assert_eq!(config.file_path, PathBuf::from("data/.ready"));
        assert!(!config.port_enabled);
        assert_eq!(config.port, 9091);
    }

    #[test]
    fn test_readiness_info_creation() {
        let info = ReadinessInfo::new("Active".to_string(), 123);
        assert_eq!(info.state, "Active");
        assert_eq!(info.uptime_seconds, 123);
        assert!(info.metadata.is_empty());
    }

    #[test]
    fn test_readiness_info_with_metadata() {
        let info = ReadinessInfo::new("Active".to_string(), 123)
            .with_metadata("key1".to_string(), "value1".to_string())
            .with_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(info.metadata.len(), 2);
        assert_eq!(info.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(info.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[tokio::test]
    async fn test_readiness_manager_creation() {
        let app = Application::new();
        let manager = ReadinessManager::new(app);
        assert!(!manager.is_ready().await);
        assert!(manager.uptime_since_ready().await.is_none());
    }

    #[tokio::test]
    async fn test_readiness_manager_ready_state() {
        let app = Application::new();
        let manager = ReadinessManager::new(app.clone());

        // Not ready initially
        assert!(!manager.is_ready().await);

        // Initialize and activate app
        app.initialize().await.unwrap();
        app.register().await.unwrap();

        // Now we can mark ready
        manager.mark_ready().await.unwrap();
        assert!(manager.is_ready().await);
        assert!(manager.uptime_since_ready().await.is_some());

        // Mark not ready
        manager.mark_not_ready().await.unwrap();
        assert!(!manager.is_ready().await);
        assert!(manager.uptime_since_ready().await.is_none());
    }

    #[tokio::test]
    async fn test_readiness_probes() {
        let app = Application::new();
        let manager = ReadinessManager::new(app.clone());

        // Before initialization - app is in Stopped state, so not live
        assert!(!manager.liveness_probe().await);
        assert!(!manager.readiness_probe().await); // Not ready
        assert!(!manager.startup_probe().await); // Not started

        // Initialize
        app.initialize().await.unwrap();
        assert!(manager.liveness_probe().await);
        assert!(!manager.readiness_probe().await);
        assert!(!manager.startup_probe().await);

        // Register (becomes Active)
        app.register().await.unwrap();
        assert!(manager.liveness_probe().await);
        assert!(manager.startup_probe().await); // Startup complete

        // Mark ready
        manager.mark_ready().await.unwrap();
        assert!(manager.liveness_probe().await);
        assert!(manager.readiness_probe().await); // Now ready
        assert!(manager.startup_probe().await);

        // Stop
        app.stop().await.unwrap();
        assert!(!manager.liveness_probe().await); // Not alive
        assert!(!manager.readiness_probe().await);
        assert!(!manager.startup_probe().await);
    }

    #[test]
    fn test_is_readiness_file() {
        assert!(is_readiness_file(".ready"));
        assert!(is_readiness_file("data/.ready"));
        assert!(is_readiness_file(PathBuf::from("/var/lib/data/.ready")));
        assert!(!is_readiness_file("ready"));
        assert!(!is_readiness_file("notready"));
        assert!(!is_readiness_file("data/status.json"));
    }

    #[test]
    fn test_port_availability() {
        // Test with a very high port that's likely available
        let high_port = 59999;
        assert!(is_port_available(high_port));

        // Find an available port
        let available = find_available_port(50000);
        assert!(available.is_some());
        assert!(available.unwrap() >= 50000);
    }

    #[tokio::test]
    async fn test_readiness_file_lifecycle() {
        use tempfile::TempDir;

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

        let manager = ReadinessManager::new(app).with_config(config);

        // File shouldn't exist yet
        assert!(!manager.readiness_file_exists());

        // Mark ready - should create file
        manager.mark_ready().await.unwrap();
        assert!(manager.readiness_file_exists());

        // Read file content
        let info = manager.read_readiness_file().await.unwrap();
        assert_eq!(info.state, "Active");

        // Mark not ready - should remove file
        manager.mark_not_ready().await.unwrap();
        assert!(!manager.readiness_file_exists());
    }

    #[tokio::test]
    async fn test_service_health_metrics() {
        let app = Application::new();
        app.initialize().await.unwrap();
        app.register().await.unwrap();

        let manager = ReadinessManager::new(app);
        manager.mark_ready().await.unwrap();

        let health = manager.health().await;
        assert_eq!(health.status, ServiceStatus::Running);
        assert!(health.metrics.contains_key("ready"));
        assert!(health.metrics.contains_key("liveness"));
        assert!(health.metrics.contains_key("readiness"));
        assert!(health.metrics.contains_key("startup"));
    }
}
