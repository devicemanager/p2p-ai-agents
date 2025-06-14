use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;

pub mod discovery;
pub mod transport;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Not initialized")]
    NotInitialized,
    #[error("Already running")]
    AlreadyRunning,
    #[error("Not running")]
    NotRunning,
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Discovery error: {0}")]
    Discovery(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub bootstrap_nodes: Vec<SocketAddr>,
    pub agent_type: String,
}

#[derive(Debug, Clone)]
pub struct NetworkMessage {
    pub from: String,
    pub to: String,
    pub content: Vec<u8>,
}

pub struct NetworkManager {
    config: NetworkConfig,
    is_initialized: bool,
    is_running: bool,
    transport_type: String,
    messages: Arc<Mutex<Vec<NetworkMessage>>>,
    connected_peers: Arc<Mutex<Vec<SocketAddr>>>,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            is_initialized: false,
            is_running: false,
            transport_type: "tcp".to_string(),
            messages: Arc::new(Mutex::new(Vec::new())),
            connected_peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub async fn start(&mut self) -> NetworkResult<()> {
        if !self.is_initialized {
            return Err(NetworkError::NotInitialized);
        }
        if self.is_running {
            return Err(NetworkError::AlreadyRunning);
        }
        self.is_running = true;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NotRunning);
        }
        self.is_running = false;
        Ok(())
    }

    pub fn set_transport(&mut self, transport: &str) {
        self.transport_type = transport.to_string();
    }

    pub fn get_transport(&self) -> &str {
        &self.transport_type
    }

    pub async fn simulate_transport_failure(&mut self) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NotRunning);
        }
        // Simulate transport failure by clearing peers
        let mut peers = self.connected_peers.lock().await;
        peers.clear();
        Ok(())
    }

    pub async fn send_message(&self, message: NetworkMessage) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NotRunning);
        }
        let mut messages = self.messages.lock().await;
        messages.push(message);
        Ok(())
    }

    pub async fn get_messages(&self) -> Vec<NetworkMessage> {
        let messages = self.messages.lock().await;
        messages.clone()
    }

    pub async fn get_connected_peers(&self) -> Vec<SocketAddr> {
        let peers = self.connected_peers.lock().await;
        peers.clone()
    }
} 