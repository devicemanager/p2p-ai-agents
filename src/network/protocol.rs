//! Custom request/response protocol for P2P agent communication

use async_trait::async_trait;
use futures::{io, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::request_response::Codec;
use serde::{Deserialize, Serialize};

const MESSAGE_SIZE_LIMIT: usize = 10 * 1024 * 1024; // 10MB

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

/// Protocol identifier
#[derive(Debug, Clone)]
pub struct AgentProtocol;

impl AsRef<str> for AgentProtocol {
    fn as_ref(&self) -> &str {
        "/p2p-ai-agents/1.0.0"
    }
}

/// Codec for encoding/decoding agent messages
#[derive(Debug, Clone, Default)]
pub struct AgentCodec;

#[async_trait]
impl Codec for AgentCodec {
    type Protocol = AgentProtocol;
    type Request = AgentRequest;
    type Response = AgentResponse;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.take(MESSAGE_SIZE_LIMIT as u64)
            .read_to_end(&mut buf)
            .await?;

        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.take(MESSAGE_SIZE_LIMIT as u64)
            .read_to_end(&mut buf)
            .await?;

        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if data.len() > MESSAGE_SIZE_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Request exceeds size limit",
            ));
        }

        io.write_all(&data).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if data.len() > MESSAGE_SIZE_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Response exceeds size limit",
            ));
        }

        io.write_all(&data).await?;
        io.close().await
    }
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
