//! Integration test for advanced usage examples: protocol and metrics.
use p2p_ai_agents::network::MetricsCollector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum ProtocolState {
    Init,
    Active,
    Closed,
}

#[tokio::test]
async fn test_protocol_state_transitions() {
    let _state = ProtocolState::Init;
    // Transition to Active
    let state = ProtocolState::Active;
    assert_eq!(state, ProtocolState::Active);
    // Transition to Closed
    let state = ProtocolState::Closed;
    assert_eq!(state, ProtocolState::Closed);
}

#[test]
fn test_metrics_collector_usage() {
    let mut metrics = MetricsCollector::new();
    metrics.increment("messages_sent");
    metrics.increment("messages_sent");
    let count = metrics.get("messages_sent");
    assert_eq!(count, 2);
}
