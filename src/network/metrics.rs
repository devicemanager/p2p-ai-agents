use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Collector for network metrics.
#[derive(Debug, Default)]
pub struct MetricsCollector {
    metrics: HashMap<String, AtomicU64>,
}

impl MetricsCollector {
    /// Creates a new metrics collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Gets a metric value by key.
    #[must_use]
    pub fn get(&self, key: &str) -> u64 {
        self.metrics
            .get(key)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Updates a metric value.
    pub fn update(&mut self, key: &str, value: u64) {
        self.metrics
            .entry(key.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(value, Ordering::Relaxed);
    }

    /// Increments a metric value by 1.
    pub fn increment(&mut self, key: &str) {
        self.metrics
            .entry(key.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements a metric value by 1.
    pub fn decrement(&mut self, key: &str) {
        self.metrics
            .entry(key.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_sub(1, Ordering::Relaxed);
    }

    /// Adds a value to a metric.
    pub fn add(&mut self, key: &str, value: u64) {
        self.metrics
            .entry(key.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(value, Ordering::Relaxed);
    }
}
