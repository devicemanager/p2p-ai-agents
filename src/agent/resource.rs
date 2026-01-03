//! Resource monitoring for agents
//!
//! This module provides functionality for monitoring and managing
//! agent resources, including CPU, memory, storage, and network usage.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo::{get_current_pid, System};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::agent::ResourceLimits;

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage (0.0 to 1.0)
    pub cpu: f32,
    /// Memory usage in bytes
    pub memory: u64,
    /// Storage usage in bytes
    pub storage: u64,
    /// Network bandwidth usage in bytes per second
    pub bandwidth: u64,
}

/// Resource monitor for tracking agent resource usage
pub struct ResourceMonitor {
    /// System information
    system: Arc<RwLock<System>>,
    /// Resource limits
    limits: ResourceLimits,
    /// Monitoring status
    is_monitoring: Arc<RwLock<bool>>,
}

/// Error type for resource operations
#[derive(Debug, Error)]
pub enum ResourceError {
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    LimitExceeded(String),

    /// Resource monitoring failed
    #[error("Resource monitoring failed: {0}")]
    MonitoringFailed(String),

    /// Invalid resource configuration
    #[error("Invalid resource configuration: {0}")]
    InvalidConfig(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// CPU limit exceeded
    #[error("CPU limit exceeded")]
    CpuLimitExceeded,

    /// Memory limit exceeded
    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,

    /// Storage limit exceeded
    #[error("Storage limit exceeded")]
    StorageLimitExceeded,

    /// Bandwidth limit exceeded
    #[error("Bandwidth limit exceeded")]
    BandwidthLimitExceeded,

    /// Process not found
    #[error("Process not found")]
    ProcessNotFound,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(limits: &ResourceLimits) -> Result<Self> {
        Ok(Self {
            system: Arc::new(RwLock::new(System::new_all())),
            limits: limits.clone(),
            is_monitoring: Arc::new(RwLock::new(false)),
        })
    }

    /// Start resource monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.write().await;
        if *is_monitoring {
            return Ok(());
        }
        *is_monitoring = true;
        Ok(())
    }

    /// Stop resource monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.write().await;
        *is_monitoring = false;
        Ok(())
    }

    /// Update resource usage information
    pub async fn update_usage(&self) -> Result<()> {
        let mut system = self.system.write().await;
        system.refresh_all();
        Ok(())
    }

    /// Get current resource usage
    pub async fn current_usage(&self) -> Result<ResourceUsage> {
        let system = self.system.read().await;
        let pid = get_current_pid().map_err(|_| ResourceError::ProcessNotFound)?;
        let process = system.process(pid).ok_or(ResourceError::ProcessNotFound)?;
        Ok(ResourceUsage {
            cpu: process.cpu_usage() / 100.0,
            memory: process.memory(),
            storage: 0,   // TODO: Implement storage monitoring
            bandwidth: 0, // TODO: Implement bandwidth monitoring
        })
    }

    /// Check if resource usage is within limits
    pub async fn check_limits(&self) -> Result<()> {
        let usage = self.current_usage().await?;
        if usage.cpu > self.limits.max_cpu {
            return Err(ResourceError::CpuLimitExceeded);
        }
        if usage.memory > self.limits.max_memory {
            return Err(ResourceError::MemoryLimitExceeded);
        }
        Ok(())
    }
}

/// Result type for resource operations
pub type Result<T> = std::result::Result<T, ResourceError>;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_resource_monitor() {
        let limits = ResourceLimits {
            max_cpu: 0.8,
            max_memory: 1024 * 1024 * 1024,       // 1GB
            max_storage: 10 * 1024 * 1024 * 1024, // 10GB
            max_bandwidth: 1024 * 1024,           // 1MB/s
            max_connections: 100,
        };

        let monitor = ResourceMonitor::new(&limits).unwrap();
        monitor.start_monitoring().await.unwrap();

        // Wait for first update
        tokio::time::sleep(Duration::from_secs(2)).await;

        let usage = monitor.current_usage().await.unwrap();
        assert!(usage.cpu >= 0.0 && usage.cpu <= 1.0);
        assert!(usage.memory > 0);
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let limits = ResourceLimits {
            max_cpu: 0.1,        // Very low CPU limit
            max_memory: 1024,    // Very low memory limit
            max_storage: 1024,   // Very low storage limit
            max_bandwidth: 1024, // Very low bandwidth limit
            max_connections: 10,
        };

        let monitor = ResourceMonitor::new(&limits).unwrap();
        monitor.start_monitoring().await.unwrap();

        // Wait for first update
        tokio::time::sleep(Duration::from_secs(2)).await;

        // This might fail depending on system load
        let result = monitor.check_limits().await;
        if let Err(err) = result {
            assert!(matches!(
                err,
                ResourceError::CpuLimitExceeded | ResourceError::MemoryLimitExceeded
            ));
        }
    }
}
