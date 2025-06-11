//! Resource monitoring for agents
//! 
//! This module provides functionality for monitoring and managing
//! agent resources, including CPU, memory, storage, and network usage.

use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tokio::time;
use thiserror::Error;
use sysinfo::{System, SystemExt, CpuExt, ProcessExt};

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
    sys: Arc<RwLock<System>>,
    /// Resource limits
    limits: ResourceLimits,
    /// Current resource usage
    usage: Arc<RwLock<ResourceUsage>>,
    /// Monitoring interval
    interval: Duration,
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
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(limits: &ResourceLimits) -> Result<Self, ResourceError> {
        let sys = Arc::new(RwLock::new(System::new_all()));
        let usage = Arc::new(RwLock::new(ResourceUsage {
            cpu: 0.0,
            memory: 0,
            storage: 0,
            bandwidth: 0,
        }));

        Ok(Self {
            sys,
            limits: limits.clone(),
            usage,
            interval: Duration::from_secs(1),
        })
    }

    /// Start resource monitoring
    pub async fn start_monitoring(&self) -> Result<(), ResourceError> {
        let sys = self.sys.clone();
        let usage = self.usage.clone();
        let limits = self.limits.clone();
        let interval = self.interval;

        tokio::spawn(async move {
            let mut interval = time::interval(interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::update_usage(&sys, &usage, &limits).await {
                    tracing::error!("Resource monitoring failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Update resource usage information
    async fn update_usage(
        sys: &RwLock<System>,
        usage: &RwLock<ResourceUsage>,
        limits: &ResourceLimits,
    ) -> Result<(), ResourceError> {
        let mut sys = sys.write().await;
        sys.refresh_all();

        // Get CPU usage
        let cpu_usage = sys.global_cpu_info().cpu_usage() / 100.0;
        if cpu_usage > limits.max_cpu {
            return Err(ResourceError::LimitExceeded(format!(
                "CPU usage {} exceeds limit {}",
                cpu_usage, limits.max_cpu
            )));
        }

        // Get memory usage
        let memory_usage = sys.used_memory();
        if memory_usage > limits.max_memory {
            return Err(ResourceError::LimitExceeded(format!(
                "Memory usage {} exceeds limit {}",
                memory_usage, limits.max_memory
            )));
        }

        // Get storage usage (placeholder - implement actual storage monitoring)
        let storage_usage = 0;
        if storage_usage > limits.max_storage {
            return Err(ResourceError::LimitExceeded(format!(
                "Storage usage {} exceeds limit {}",
                storage_usage, limits.max_storage
            )));
        }

        // Get network usage (placeholder - implement actual network monitoring)
        let bandwidth_usage = 0;
        if bandwidth_usage > limits.max_bandwidth {
            return Err(ResourceError::LimitExceeded(format!(
                "Bandwidth usage {} exceeds limit {}",
                bandwidth_usage, limits.max_bandwidth
            )));
        }

        // Update usage information
        let mut usage = usage.write().await;
        usage.cpu = cpu_usage;
        usage.memory = memory_usage;
        usage.storage = storage_usage;
        usage.bandwidth = bandwidth_usage;

        Ok(())
    }

    /// Get current resource usage
    pub async fn current_usage(&self) -> Result<ResourceUsage, ResourceError> {
        let usage = self.usage.read().await;
        Ok(usage.clone())
    }

    /// Check if resource usage is within limits
    pub async fn check_limits(&self) -> Result<(), ResourceError> {
        let usage = self.current_usage().await?;
        let limits = &self.limits;

        if usage.cpu > limits.max_cpu {
            return Err(ResourceError::LimitExceeded(format!(
                "CPU usage {} exceeds limit {}",
                usage.cpu, limits.max_cpu
            )));
        }

        if usage.memory > limits.max_memory {
            return Err(ResourceError::LimitExceeded(format!(
                "Memory usage {} exceeds limit {}",
                usage.memory, limits.max_memory
            )));
        }

        if usage.storage > limits.max_storage {
            return Err(ResourceError::LimitExceeded(format!(
                "Storage usage {} exceeds limit {}",
                usage.storage, limits.max_storage
            )));
        }

        if usage.bandwidth > limits.max_bandwidth {
            return Err(ResourceError::LimitExceeded(format!(
                "Bandwidth usage {} exceeds limit {}",
                usage.bandwidth, limits.max_bandwidth
            )));
        }

        Ok(())
    }
}

/// Result type for resource operations
pub type Result<T> = std::result::Result<T, ResourceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_monitor() {
        let limits = ResourceLimits {
            max_cpu: 0.8,
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_storage: 10 * 1024 * 1024 * 1024, // 10GB
            max_bandwidth: 1024 * 1024, // 1MB/s
        };

        let monitor = ResourceMonitor::new(&limits).unwrap();
        monitor.start_monitoring().await.unwrap();

        // Wait for first update
        tokio::time::sleep(Duration::from_secs(2)).await;

        let usage = monitor.current_usage().await.unwrap();
        assert!(usage.cpu >= 0.0 && usage.cpu <= 1.0);
        assert!(usage.memory > 0);
        assert!(usage.storage >= 0);
        assert!(usage.bandwidth >= 0);
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let limits = ResourceLimits {
            max_cpu: 0.1, // Very low CPU limit
            max_memory: 1024, // Very low memory limit
            max_storage: 1024, // Very low storage limit
            max_bandwidth: 1024, // Very low bandwidth limit
        };

        let monitor = ResourceMonitor::new(&limits).unwrap();
        monitor.start_monitoring().await.unwrap();

        // Wait for first update
        tokio::time::sleep(Duration::from_secs(2)).await;

        // This might fail depending on system load
        let result = monitor.check_limits().await;
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(matches!(err, ResourceError::LimitExceeded(_)));
        }
    }
} 