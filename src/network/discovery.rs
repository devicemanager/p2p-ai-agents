use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Bootstrap error: {0}")]
    BootstrapError(String),
    #[error("Peer discovery error: {0}")]
    PeerDiscoveryError(String),
    #[error("Connection timeout")]
    ConnectionTimeout,
    #[error("Invalid peer address")]
    InvalidPeerAddress,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub agent_type: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

pub struct DiscoveryManager {
    bootstrap_nodes: Arc<Mutex<HashSet<SocketAddr>>>,
    known_peers: Arc<Mutex<HashSet<PeerInfo>>>,
    is_bootstrap: bool,
}

impl DiscoveryManager {
    pub fn new(is_bootstrap: bool) -> Self {
        Self {
            bootstrap_nodes: Arc::new(Mutex::new(HashSet::new())),
            known_peers: Arc::new(Mutex::new(HashSet::new())),
            is_bootstrap,
        }
    }

    pub async fn add_bootstrap_node(&self, addr: SocketAddr) -> Result<(), DiscoveryError> {
        let mut nodes = self.bootstrap_nodes.lock().await;
        nodes.insert(addr);
        Ok(())
    }

    pub async fn remove_bootstrap_node(&self, addr: &SocketAddr) -> Result<(), DiscoveryError> {
        let mut nodes = self.bootstrap_nodes.lock().await;
        nodes.remove(addr);
        Ok(())
    }

    pub async fn get_bootstrap_nodes(&self) -> HashSet<SocketAddr> {
        self.bootstrap_nodes.lock().await.clone()
    }

    pub async fn add_peer(&self, peer: PeerInfo) -> Result<(), DiscoveryError> {
        let mut peers = self.known_peers.lock().await;
        peers.insert(peer);
        Ok(())
    }

    pub async fn remove_peer(&self, addr: &SocketAddr) -> Result<(), DiscoveryError> {
        let mut peers = self.known_peers.lock().await;
        peers.retain(|p| p.address != *addr);
        Ok(())
    }

    pub async fn get_known_peers(&self) -> HashSet<PeerInfo> {
        self.known_peers.lock().await.clone()
    }

    pub async fn discover_peers(&self) -> Result<(), DiscoveryError> {
        if self.is_bootstrap {
            // Bootstrap nodes don't need to discover peers
            return Ok(());
        }

        let bootstrap_nodes = self.get_bootstrap_nodes().await;
        if bootstrap_nodes.is_empty() {
            return Err(DiscoveryError::BootstrapError("No bootstrap nodes available".into()));
        }

        // Basic peer discovery implementation
        // In a real implementation, this would:
        // 1. Connect to bootstrap nodes
        // 2. Request peer list
        // 3. Update known peers
        Ok(())
    }

    pub async fn start_discovery(&self) -> Result<(), DiscoveryError> {
        if self.is_bootstrap {
            // Bootstrap nodes start listening for peer connections
            Ok(())
        } else {
            // Regular nodes start peer discovery
            self.discover_peers().await
        }
    }

    pub async fn stop_discovery(&self) -> Result<(), DiscoveryError> {
        // Clean up discovery resources
        Ok(())
    }
} 