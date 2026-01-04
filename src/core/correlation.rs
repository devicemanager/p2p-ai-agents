//! Correlation ID management for distributed tracing
//!
//! This module provides correlation ID support for tracking operations across
//! the distributed system. Correlation IDs are propagated through tracing spans
//! and included in all log entries for a given operation.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A correlation ID for tracking operations across the system
///
/// Correlation IDs are UUID v4 values that are generated at the start of an operation
/// and propagated through all related spans and logs. This enables tracing a request
/// or operation through the entire system.
///
/// # Example
/// ```
/// use p2p_ai_agents::core::CorrelationId;
/// use tracing::info;
///
/// let correlation_id = CorrelationId::new();
/// info!(correlation_id = %correlation_id, "Starting operation");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    /// Create a new random correlation ID (UUID v4)
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a correlation ID from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Get the correlation ID as a string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for CorrelationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<CorrelationId> for Uuid {
    fn from(id: CorrelationId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_valid_uuid() {
        let id = CorrelationId::new();
        assert_eq!(id.as_uuid().get_version_num(), 4);
    }

    #[test]
    fn test_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = CorrelationId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn test_display() {
        let uuid = Uuid::new_v4();
        let id = CorrelationId::from_uuid(uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn test_default() {
        let id = CorrelationId::default();
        assert_eq!(id.as_uuid().get_version_num(), 4);
    }

    #[test]
    fn test_serialization() {
        let id = CorrelationId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: CorrelationId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_from_into() {
        let uuid = Uuid::new_v4();
        let id: CorrelationId = uuid.into();
        let back: Uuid = id.into();
        assert_eq!(uuid, back);
    }

    #[test]
    fn test_as_str() {
        let id = CorrelationId::new();
        let s = id.as_str();
        assert_eq!(s, id.to_string());
    }
}
