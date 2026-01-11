//! Custom request/response protocol for P2P agent communication

use serde::{Deserialize, Serialize};

/// Request message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRequest {
    /// Message content
    pub message: String,
}

/// Response message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentResponse {
    /// Response content
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_request() {
        let req = AgentRequest {
            message: "Hello, peer!".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Hello, peer!"));
    }

    #[test]
    fn test_deserialize_request() {
        let json = r#"{"message":"Test message"}"#;
        let req: AgentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message, "Test message");
    }

    #[test]
    fn test_serialize_response() {
        let res = AgentResponse {
            message: "Response data".to_string(),
        };
        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("Response data"));
    }

    #[test]
    fn test_roundtrip() {
        let req = AgentRequest {
            message: "Test".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let req2: AgentRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, req2);
    }
}
