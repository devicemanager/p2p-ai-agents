use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("Receive error: {0}")]
    ReceiveError(String),
    #[error("Transport not started")]
    NotStarted,
    #[error("Transport already started")]
    AlreadyStarted,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransportType {
    TCP,
    WebRTC,
}

impl TransportType {
    pub async fn start(&mut self, _addr: SocketAddr) -> Result<(), TransportError> {
        match self {
            TransportType::TCP => {
                // Basic TCP transport implementation
                Ok(())
            }
            TransportType::WebRTC => {
                // Basic WebRTC transport implementation
                Ok(())
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), TransportError> {
        match self {
            TransportType::TCP => {
                // Stop TCP transport
                Ok(())
            }
            TransportType::WebRTC => {
                // Stop WebRTC transport
                Ok(())
            }
        }
    }

    pub async fn send(&self, _to: SocketAddr, _data: Vec<u8>) -> Result<(), TransportError> {
        match self {
            TransportType::TCP => {
                // Send over TCP
                Ok(())
            }
            TransportType::WebRTC => {
                // Send over WebRTC
                Ok(())
            }
        }
    }

    pub async fn simulate_failure(&mut self) -> Result<(), TransportError> {
        // Simulate transport failure for testing
        Ok(())
    }
} 