/// Health monitor for network operations.
#[derive(Debug, Default)]
pub struct HealthMonitor;

impl HealthMonitor {
    /// Creates a new health monitor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
