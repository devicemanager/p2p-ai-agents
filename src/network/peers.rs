//! Peer management and state tracking module.
//! Provides types and functionality for managing peer information, state, and metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{Multiaddr, PeerId};
use crate::agent::task::TaskType;

/// Capabilities supported by a peer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerCapabilities {
    /// List of tasks this peer can perform
    pub supported_tasks: Vec<TaskType>,
    /// List of AI models available on this peer (e.g., "prajjwal1/bert-tiny")
    #[serde(default)]
    pub supported_models: Vec<String>,
}

/// Connection status of a peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Peer is connected
    Connected,
    /// Peer is disconnected
    Disconnected,
}

/// Information about a network peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Unique peer identifier
    pub peer_id: PeerId,
    /// Known network addresses
    pub addresses: Vec<Multiaddr>,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Peer reputation score
    pub reputation: i32,
    /// Peer capabilities
    pub capabilities: PeerCapabilities,
    /// Connection status
    pub status: ConnectionStatus,
}

/// Metrics for a peer
#[derive(Debug, Clone, Default)]
pub struct PeerMetrics {
    /// Number of messages sent to this peer
    pub messages_sent: u64,
    /// Number of messages received from this peer
    pub messages_received: u64,
    /// Number of bytes sent to this peer
    pub bytes_sent: u64,
    /// Number of bytes received from this peer
    pub bytes_received: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
}

/// State of a peer in the network
#[derive(Debug, Clone)]
pub struct PeerState {
    /// Peer information
    pub info: PeerInfo,
    /// Peer metrics
    pub metrics: PeerMetrics,
}

/// Cache for managing peer information and state
#[derive(Debug, Clone)]
pub struct PeerCache {
    /// Internal storage for peer states
    peers: Arc<RwLock<HashMap<PeerId, PeerState>>>,
}

impl PeerCache {
    /// Create a new peer cache
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or update peer information
    pub async fn upsert_peer(&self, info: PeerInfo) {
        let peer_id = info.peer_id.clone();
        let mut peers = self.peers.write().await;

        peers
            .entry(peer_id.clone())
            .and_modify(|state| state.info = info.clone())
            .or_insert_with(|| PeerState {
                info,
                metrics: PeerMetrics::default(),
            });
    }

    /// Get peer information
    pub async fn get_peer(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        let peers = self.peers.read().await;
        peers.get(peer_id).map(|state| state.info.clone())
    }

    /// Remove peer from cache
    pub async fn remove_peer(&self, peer_id: &PeerId) -> Option<PeerState> {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id)
    }

    /// Get all connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers
            .values()
            .filter(|state| state.info.status == ConnectionStatus::Connected)
            .map(|state| state.info.clone())
            .collect()
    }

    /// Get all peers
    pub async fn get_all_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers.values().map(|state| state.info.clone()).collect()
    }

    /// Update peer metrics
    pub async fn update_metrics<F>(&self, peer_id: &PeerId, f: F)
    where
        F: FnOnce(&mut PeerMetrics),
    {
        let mut peers = self.peers.write().await;
        if let Some(state) = peers.get_mut(peer_id) {
            f(&mut state.metrics);
        }
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }
}

impl Default for PeerCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_peer_info(id: &str) -> PeerInfo {
        PeerInfo {
            peer_id: PeerId(id.to_string()),
            addresses: vec![Multiaddr(format!("/ip4/127.0.0.1/tcp/808{}", id))],
            last_seen: chrono::Utc::now(),
            reputation: 50,
            capabilities: PeerCapabilities::default(),
            status: ConnectionStatus::Connected,
        }
    }

    #[test]
    fn test_peer_info_creation() {
        let peer_id = PeerId("test-peer".to_string());
        let addresses = vec![
            Multiaddr("/ip4/127.0.0.1/tcp/8080".to_string()),
            Multiaddr("/ip4/192.168.1.100/tcp/8080".to_string()),
        ];
        let now = chrono::Utc::now();

        let peer_info = PeerInfo {
            peer_id: peer_id.clone(),
            addresses: addresses.clone(),
            last_seen: now,
            reputation: 50,
            capabilities: PeerCapabilities::default(),
            status: ConnectionStatus::Connected,
        };

        assert_eq!(peer_info.peer_id, peer_id);
        assert_eq!(peer_info.addresses.len(), 2);
        assert_eq!(peer_info.reputation, 50);
        assert_eq!(peer_info.status, ConnectionStatus::Connected);
    }

    #[test]
    fn test_connection_status_variants() {
        let connected = ConnectionStatus::Connected;
        let disconnected = ConnectionStatus::Disconnected;

        assert_eq!(connected, ConnectionStatus::Connected);
        assert_eq!(disconnected, ConnectionStatus::Disconnected);
        assert_ne!(connected, disconnected);
    }

    #[tokio::test]
    async fn test_peer_cache_upsert() {
        let cache = PeerCache::new();
        let peer_info = create_test_peer_info("1");

        cache.upsert_peer(peer_info.clone()).await;

        let retrieved = cache.get_peer(&peer_info.peer_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().peer_id, peer_info.peer_id);
    }

    #[tokio::test]
    async fn test_peer_cache_remove() {
        let cache = PeerCache::new();
        let peer_info = create_test_peer_info("2");

        cache.upsert_peer(peer_info.clone()).await;
        let removed = cache.remove_peer(&peer_info.peer_id).await;

        assert!(removed.is_some());
        assert!(cache.get_peer(&peer_info.peer_id).await.is_none());
    }

    #[tokio::test]
    async fn test_peer_cache_connected_peers() {
        let cache = PeerCache::new();
        let peer1 = create_test_peer_info("1");
        let mut peer2 = create_test_peer_info("2");
        peer2.status = ConnectionStatus::Disconnected;

        cache.upsert_peer(peer1).await;
        cache.upsert_peer(peer2).await;

        let connected = cache.get_connected_peers().await;
        assert_eq!(connected.len(), 1);
    }

    #[tokio::test]
    async fn test_peer_cache_metrics_update() {
        let cache = PeerCache::new();
        let peer_info = create_test_peer_info("3");
        let peer_id = peer_info.peer_id.clone();

        cache.upsert_peer(peer_info).await;

        cache
            .update_metrics(&peer_id, |metrics| {
                metrics.messages_sent = 10;
                metrics.bytes_sent = 1024;
            })
            .await;

        // Verify metrics were updated
        let peers = cache.peers.read().await;
        let state = peers.get(&peer_id).unwrap();
        assert_eq!(state.metrics.messages_sent, 10);
        assert_eq!(state.metrics.bytes_sent, 1024);
    }

    #[tokio::test]
    async fn test_peer_cache_count() {
        let cache = PeerCache::new();
        assert_eq!(cache.peer_count().await, 0);

        cache.upsert_peer(create_test_peer_info("1")).await;
        cache.upsert_peer(create_test_peer_info("2")).await;

        assert_eq!(cache.peer_count().await, 2);
    }
}
