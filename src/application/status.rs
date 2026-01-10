//! Status monitoring and reporting
//!
//! This module provides the `StatusManager` service which collects system metrics
//! and persists them to a status file for external monitoring (e.g. by the CLI).

use crate::application::Application;
use crate::core::services::{
    Service, ServiceError, ServiceHealth, ServiceId, ServiceRequest, ServiceResponse, ServiceStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    /// Node ID
    pub node_id: String,
    /// Current application state
    pub state: String, // String representation of ApplicationState
    /// Node uptime in seconds
    pub uptime_seconds: u64,
    /// Number of connected peers
    pub connected_peers: usize,
    /// List of connected peers
    pub peers: Vec<String>,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Total memory in bytes
    pub total_memory_bytes: u64,
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f32,
    /// Number of tasks processed (cumulative)
    pub tasks_processed: u64,
    /// Number of active agents
    pub active_agents: usize,
    /// Timestamp of this status update
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Node version
    pub version: String,
}

impl NodeStatus {
    /// Create a new empty status
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            state: "Unknown".to_string(),
            uptime_seconds: 0,
            connected_peers: 0,
            peers: Vec::new(),
            memory_usage_bytes: 0,
            total_memory_bytes: 0,
            cpu_usage_percent: 0.0,
            tasks_processed: 0,
            active_agents: 0,
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Status manager service
pub struct StatusManager {
    application: Application,
    status_file_path: std::path::PathBuf,
    update_interval: Duration,
    running: Arc<RwLock<bool>>,
    monitor_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl StatusManager {
    /// Create a new status manager
    pub fn new(application: Application) -> Self {
        Self {
            application,
            status_file_path: std::path::PathBuf::from("data/node_status.json"),
            update_interval: Duration::from_secs(5),
            running: Arc::new(RwLock::new(false)),
            monitor_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the status file path
    pub fn with_status_file(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.status_file_path = path.into();
        self
    }

    /// Set the update interval
    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Collect and write metrics loop
    async fn run_loop(
        application: Application,
        status_file_path: std::path::PathBuf,
        update_interval: Duration,
        running: Arc<RwLock<bool>>,
    ) {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        info!("Status monitoring loop started");

        while *running.read().await {
            // Refresh system stats
            sys.refresh_memory();
            sys.refresh_cpu(); // This will work better in a loop

            // Collect application data
            let agents = application.agents().await;
            let node_id = if let Some(agent) = agents.first() {
                agent.id().as_str().to_string()
            } else {
                "initializing".to_string()
            };

            let app_state = application.state().await;

            // Get connected peers
            #[cfg(feature = "network")]
            let connected_peers_count = {
                let network_manager = application.network_manager.read().await;
                if let Some(nm) = network_manager.as_ref() {
                    let peers = nm.get_connected_peers().await;
                    peers.len()
                } else {
                    0
                }
            };

            #[cfg(not(feature = "network"))]
            let connected_peers_count = 0;

            // Peers list (separate to handle cfg block easier)
            #[cfg(feature = "network")]
            let peers_list = {
                let network_manager = application.network_manager.read().await;
                if let Some(nm) = network_manager.as_ref() {
                    let peers = nm.get_connected_peers().await;
                    peers.iter().map(|a| a.to_string()).collect()
                } else {
                    Vec::new()
                }
            };

            #[cfg(not(feature = "network"))]
            let peers_list = Vec::new();

            let status = NodeStatus {
                node_id,
                state: format!("{:?}", app_state), // Use Debug impl for shorter string
                uptime_seconds: 0,                 // Would need start time
                connected_peers: connected_peers_count,
                peers: peers_list,
                memory_usage_bytes: sys.used_memory(),
                total_memory_bytes: sys.total_memory(),
                cpu_usage_percent: sys.global_cpu_info().cpu_usage(),
                tasks_processed: 0,
                active_agents: agents.len(),
                timestamp: chrono::Utc::now(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            };

            // Write to file
            if let Ok(json) = serde_json::to_string_pretty(&status) {
                // Ensure directory exists
                if let Some(parent) = status_file_path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }

                if let Err(e) = tokio::fs::write(&status_file_path, json).await {
                    error!("Failed to write status file: {}", e);
                }
            }

            tokio::time::sleep(update_interval).await;
        }

        info!("Status monitoring loop stopped");
    }
}

#[async_trait::async_trait]
impl Service for StatusManager {
    fn id(&self) -> ServiceId {
        ServiceId::new()
    }

    fn name(&self) -> &str {
        "status-manager"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn initialize(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn start(&self) -> Result<(), ServiceError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let app_clone = self.application.clone();
        let path_clone = self.status_file_path.clone();
        let interval = self.update_interval;
        let running_clone = self.running.clone();

        let handle = tokio::spawn(async move {
            Self::run_loop(app_clone, path_clone, interval, running_clone).await;
        });

        let mut monitor_handle = self.monitor_handle.write().await;
        *monitor_handle = Some(handle);

        info!("StatusManager started");
        Ok(())
    }

    async fn stop(&self) -> Result<(), ServiceError> {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        let mut monitor_handle = self.monitor_handle.write().await;
        if let Some(handle) = monitor_handle.take() {
            // Wait for loop to exit (it checks running flag)
            // We can abort it to be faster
            handle.abort();
        }

        info!("StatusManager stopped");
        Ok(())
    }

    async fn status(&self) -> ServiceStatus {
        if *self.running.read().await {
            ServiceStatus::Running
        } else {
            ServiceStatus::Stopped
        }
    }

    async fn health(&self) -> ServiceHealth {
        ServiceHealth {
            status: self.status().await,
            uptime: Duration::ZERO,
            last_heartbeat: chrono::Utc::now(),
            metrics: HashMap::new(),
        }
    }

    async fn handle_request(
        &self,
        _request: ServiceRequest,
    ) -> Result<ServiceResponse, ServiceError> {
        Err(ServiceError::RequestFailed("Not implemented".to_string()))
    }
}
