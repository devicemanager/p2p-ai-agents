//! Network module for peer-to-peer agent system.
//! Provides types and helpers for network management, metrics, resources, health, and security.

use libp2p::{
    futures::StreamExt, gossipsub, identify, identity, mdns, multiaddr::Protocol, noise,
    request_response, swarm::SwarmEvent, tcp, yamux, Multiaddr as Libp2pMultiaddr,
    PeerId as Libp2pPeerId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info};

/// Behavior submodule for network behaviors.
pub mod behavior;
/// Discovery submodule for peer discovery and management.
pub mod discovery;

// Add identify to the behavior if not already there
// It seems behavior.rs handles it.

// We need to update NetworkManager to handle discovery events or expose discovery info.
// The previous plan mentioned "Update NetworkManager to store peer metadata (capabilities) alongside PeerIDs."
// NetworkManager currently has `connected_peers: Arc<Mutex<Vec<SocketAddr>>>`.
// We should probably upgrade this to `PeerInfo` or use `PeerCache`.
// `src/network/peers.rs` already defines `PeerCache` and `PeerInfo`.
// Let's use `PeerCache` in `NetworkManager`.

/// Service submodule for the NetworkService implementation.
pub mod service;
/// Transport submodule for network transport protocols.
pub mod transport;

/// Peer management and state tracking
pub mod peers;

/// Connection diversity enforcement
pub mod diversity;
/// Reputation system
pub mod reputation;

/// MVP P2P agent for local network (mDNS + TCP)
pub mod p2p_agent;

/// Custom protocol for agent message exchange
pub mod protocol;

// Re-export NetworkStats from service module
pub use service::NetworkStats;

// Re-export types from peers module
pub use peers::{ConnectionStatus, PeerCache, PeerCapabilities, PeerInfo, PeerMetrics, PeerState};

use crate::network::behavior::{AgentBehavior, AgentBehaviorEvent};

/// Errors that can occur in the network module.
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Not initialized error
    #[error("Not initialized")]
    NotInitialized,
    /// Already running error
    #[error("Already running")]
    AlreadyRunning,
    /// Not running error
    #[error("Not running")]
    NotRunning,
    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),
    /// Discovery error
    #[error("Discovery error: {0}")]
    Discovery(String),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Libp2p error
    #[error("Libp2p error: {0}")]
    Libp2p(String),
}

/// Result type for network operations.
pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

/// Unique identifier for a peer
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<PeerId> for String {
    fn from(id: PeerId) -> Self {
        id.0
    }
}

impl PeerId {
    /// Convert this PeerId to a libp2p PeerId
    pub fn to_libp2p(&self) -> Result<Libp2pPeerId, NetworkError> {
        Libp2pPeerId::from_str(&self.0).map_err(|e| NetworkError::Libp2p(e.to_string()))
    }
}

/// Multi-address for peer connections
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Multiaddr(pub String);

impl Multiaddr {
    /// Convert this Multiaddr to a libp2p Multiaddr
    pub fn to_libp2p(&self) -> Result<Libp2pMultiaddr, NetworkError> {
        Libp2pMultiaddr::from_str(&self.0).map_err(|e| NetworkError::Libp2p(e.to_string()))
    }
}

/// Protocol-specific configuration (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration for network protocols.
pub struct ProtocolConfig {
    // Add protocol-specific fields here
}

/// Resource usage limits and thresholds (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Resource usage limits for the network.
pub struct ResourceLimits {
    /// Maximum bandwidth in bytes per second
    pub max_bandwidth: u64,
    /// Maximum memory usage in bytes
    pub max_memory: u64,
    /// Maximum number of connections
    pub max_connections: usize,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Security-related configuration for the network.
pub struct SecurityConfig {
    /// List of trusted authorities (hex-encoded public keys)
    pub trusted_authorities: Vec<String>,
    /// The local node's Federation Certificate (optional at startup, required for full validation)
    pub local_certificate: Option<security::FederationCertificate>,
}

/// Network configuration for the P2P system
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines core configuration parameters for the network implementation.
pub struct NetworkConfig {
    /// Network address to listen on
    pub listen_addr: std::net::SocketAddr,
    /// List of bootstrap peers for initial connection
    pub bootstrap_peers: Vec<PeerInfo>,
    /// Maximum number of connected peers
    pub max_peers: usize,
    /// Protocol-specific configuration
    pub protocol_config: ProtocolConfig,
    /// Resource limits and thresholds
    pub resource_limits: ResourceLimits,
    /// Security configuration
    pub security_config: SecurityConfig,
}

/// Message structure for network communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Message content as bytes
    pub content: Vec<u8>,
}

/// Represents a message related to task execution or management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Task-specific content (could be JSON, binary, etc.)
    pub content: Vec<u8>,
    /// Task type or action
    pub task_type: String,
}

/// Represents a message related to resource usage or limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Resource-specific content
    pub content: Vec<u8>,
    /// Resource type or action
    pub resource_type: String,
}

/// Represents a message related to health checks or status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMessage {
    /// Sender identifier
    pub from: String,
    /// Receiver identifier
    pub to: String,
    /// Health-specific content
    pub content: Vec<u8>,
    /// Health check type or status
    pub health_type: String,
}

enum NetworkCommand {
    Dial {
        addr: Libp2pMultiaddr,
    },
    #[allow(dead_code)]
    SendMessage {
        peer_id: Libp2pPeerId,
        message: Vec<u8>,
    },
    /// Command to send a direct request
    SendRequest {
        peer_id: Libp2pPeerId,
        request: Vec<u8>,
    },
    Shutdown,
}

/// Manages the network state and operations.
pub struct NetworkManager {
    /// Network configuration
    config: NetworkConfig,
    /// Initialization state
    is_initialized: bool,
    /// Running state
    is_running: bool,
    /// Transport protocol type
    transport_type: String,
    /// Message queue
    messages: Arc<Mutex<Vec<NetworkMessage>>>,
    /// Connected peers (deprecated in favor of peer_cache)
    connected_peers: Arc<Mutex<Vec<SocketAddr>>>,
    /// Peer cache for advanced peer management
    pub peer_cache: Arc<PeerCache>,
    /// Listen addresses
    listen_addresses: Arc<Mutex<Vec<Libp2pMultiaddr>>>,

    /// The local PeerId (assigned upon start)
    local_peer_id: Option<Libp2pPeerId>,

    /// Agent version string for Identify protocol
    agent_version: String,

    /// Command sender for the swarm event loop
    command_sender: Option<mpsc::Sender<NetworkCommand>>,
    /// Channel for sending received messages to the Agent
    message_callback: Option<mpsc::Sender<Vec<u8>>>,
    /// Certificate manager for identity verification
    #[allow(dead_code)]
    certificate_manager: CertificateManager,
    /// Prometheus metrics collector for recording message metrics
    #[cfg(feature = "metrics-prometheus")]
    prometheus_metrics: Option<crate::metrics::prometheus_exporter::MetricsCollector>,
}

impl NetworkManager {
    /// Create a new NetworkManager.
    pub fn new(config: NetworkConfig) -> Self {
        let mut certificate_manager = CertificateManager::new();

        // Load trusted authorities from config
        for auth_hex in &config.security_config.trusted_authorities {
            if let Ok(bytes) = hex::decode(auth_hex) {
                if let Ok(array) = bytes.try_into() {
                    let _ = certificate_manager.add_authority(array);
                }
            }
        }

        Self {
            config,
            is_initialized: false,
            is_running: false,
            transport_type: "tcp".to_string(),
            messages: Arc::new(Mutex::new(Vec::new())),
            connected_peers: Arc::new(Mutex::new(Vec::new())),
            peer_cache: Arc::new(PeerCache::new()),
            listen_addresses: Arc::new(Mutex::new(Vec::new())),
            local_peer_id: None,
            agent_version: "p2p-ai-agent/1.0.0".to_string(),
            command_sender: None,
            message_callback: None,
            certificate_manager,
            #[cfg(feature = "metrics-prometheus")]
            prometheus_metrics: None,
        }
    }

    /// Create a new NetworkManager with Prometheus metrics collector.
    #[cfg(feature = "metrics-prometheus")]
    pub fn with_metrics(
        config: NetworkConfig,
        metrics: crate::metrics::prometheus_exporter::MetricsCollector,
    ) -> Self {
        let mut certificate_manager = CertificateManager::new();

        // Load trusted authorities from config
        for auth_hex in &config.security_config.trusted_authorities {
            if let Ok(bytes) = hex::decode(auth_hex) {
                if let Ok(array) = bytes.try_into() {
                    let _ = certificate_manager.add_authority(array);
                }
            }
        }

        Self {
            config,
            is_initialized: false,
            is_running: false,
            transport_type: "tcp".to_string(),
            messages: Arc::new(Mutex::new(Vec::new())),
            connected_peers: Arc::new(Mutex::new(Vec::new())),
            peer_cache: Arc::new(PeerCache::new()),
            listen_addresses: Arc::new(Mutex::new(Vec::new())),
            local_peer_id: None,
            agent_version: "p2p-ai-agent/1.0.0".to_string(),
            command_sender: None,
            message_callback: None,
            certificate_manager,
            prometheus_metrics: Some(metrics),
        }
    }

    /// Check if the manager is initialized.
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Check if the manager is running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the local PeerId if initialized
    pub fn local_peer_id(&self) -> Option<Libp2pPeerId> {
        self.local_peer_id
    }

    /// Set the callback channel for received messages
    pub fn set_message_callback(&mut self, callback: mpsc::Sender<Vec<u8>>) {
        self.message_callback = Some(callback);
    }

    /// Set the agent version string
    pub fn set_agent_version(&mut self, version: String) {
        self.agent_version = version;
    }

    /// Start the network manager.
    pub async fn start(&mut self) -> NetworkResult<()> {
        if !self.is_initialized {
            // Auto-initialize if not done yet (simplification for Agent usage)
            self.is_initialized = true;
        }
        if self.is_running {
            return Err(NetworkError::AlreadyRunning);
        }

        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = Libp2pPeerId::from(local_key.public());
        self.local_peer_id = Some(local_peer_id);
        info!("Local peer id: {:?}", local_peer_id);

        let agent_version = self.agent_version.clone();

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| NetworkError::Libp2p(e.to_string()))?
            .with_behaviour(move |key| {
                AgentBehavior::new(key.clone(), agent_version.clone())
                    .map_err(|e| NetworkError::Libp2p(e.to_string()))
                    .expect("Failed to create behavior")
            })
            .map_err(|e| NetworkError::Libp2p(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Listen on address
        let listen_addr_str = format!(
            "/ip4/{}/tcp/{}",
            self.config.listen_addr.ip(),
            self.config.listen_addr.port()
        );
        let listen_multiaddr = Libp2pMultiaddr::from_str(&listen_addr_str)
            .map_err(|e: libp2p::multiaddr::Error| NetworkError::Libp2p(e.to_string()))?;

        swarm.listen_on(listen_multiaddr).map_err(
            |e: libp2p::TransportError<std::io::Error>| NetworkError::Libp2p(e.to_string()),
        )?;

        // Dial bootstrap peers
        for peer in &self.config.bootstrap_peers {
            for addr in &peer.addresses {
                if let Ok(libp2p_addr) = addr.to_libp2p() {
                    match swarm.dial(libp2p_addr) {
                        Ok(_) => info!("Dialed bootstrap peer {:?}", addr),
                        Err(e) => error!("Failed to dial bootstrap peer {:?}: {:?}", addr, e),
                    }
                }
            }
        }

        // Create command channel
        let (tx, mut rx) = mpsc::channel::<NetworkCommand>(32);
        self.command_sender = Some(tx.clone());

        // Clone message callback for the event loop
        let message_callback = self.message_callback.clone();

        let _messages_clone = self.messages.clone();
        let connected_peers_clone = self.connected_peers.clone();
        let listen_addresses_clone = self.listen_addresses.clone();
        let peer_cache_clone = self.peer_cache.clone();

        // Subscribe to gossipsub topic
        let topic = gossipsub::IdentTopic::new("p2p-ai-agents-global");
        if let Some(agent_behavior) = swarm.behaviour_mut().gossipsub.subscribe(&topic).err() {
            error!(
                "Failed to subscribe to gossipsub topic: {:?}",
                agent_behavior
            );
        }

        // Dial bootstrap peers
        for peer in &self.config.bootstrap_peers {
            for addr in &peer.addresses {
                if let Ok(libp2p_addr) = addr.to_libp2p() {
                    match swarm.dial(libp2p_addr) {
                        Ok(_) => info!("Dialed bootstrap peer {:?}", addr),
                        Err(e) => error!("Failed to dial bootstrap peer {:?}: {:?}", addr, e),
                    }
                }
            }
        }

        // Spawn event loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {:?}", address);
                            listen_addresses_clone.lock().await.push(address);
                        }
                        SwarmEvent::Behaviour(AgentBehaviorEvent::Gossipsub(gossipsub::Event::Message {
                            propagation_source: peer_id,
                            message_id: id,
                            message,
                        })) => {
                             debug!("Got message: {:?} from peer: {:?}", id, peer_id);
                             // Forward message to agent
                             if let Some(callback) = &message_callback {
                                 let _ = callback.send(message.data).await;
                             }
                        }
                        SwarmEvent::Behaviour(AgentBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                            for (peer_id, _multiaddr) in list {
                                info!("mDNS discovered a new peer: {peer_id}");
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            }
                        }
                        SwarmEvent::Behaviour(AgentBehaviorEvent::Mdns(mdns::Event::Expired(list))) => {
                            for (peer_id, _multiaddr) in list {
                                info!("mDNS discover peer has expired: {peer_id}");
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            }
                        }
                        SwarmEvent::Behaviour(AgentBehaviorEvent::Identify(identify::Event::Received { peer_id, info })) => {
                            info!("Received Identify from {peer_id}: {:?}", info);
                            for addr in info.listen_addrs {
                                swarm.add_external_address(addr);
                            }
                            // Assuming protocol version or agent version implies capabilities for now
                            // Or in a real scenario, we parse extra fields or use a custom protocol.
                            // For this MVP, if protocol is "/p2p-ai-agents/1.0.0", we assume basic capabilities.
                            // If we want to support specific capability discovery, we need to extend Identify
                            // or use the Kademlia DHT to store provider records.

                            // Let's just register them in the cache so `find_peers_with_capability` sees them.
                            // We need to know what capabilities they have.
                            // Hack: For now, assume all discovered peers support "TextProcessing" for the demo.
                            // Real impl: Use Kademlia to advertise capabilities or custom handshake.

                            let mut supported_tasks = vec![];
                            // If the agent version string contains "TextProcessing", add it.
                            // (We need to update AgentBehavior to send this in agent_version)

                            // CHECKING protocol_version INSTEAD of agent_version as that's what we modified?
                            // No, we modified agent_version in AgentBehavior::new.
                            // BUT Identify info has both.
                            // The output shows: protocol_version: "/p2p-ai-agents/1.0.0/TextProcessing", agent_version: "rust-libp2p/0.44.2"
                            // AHA! libp2p's Identify struct fields:
                            // protocol_version: String, agent_version: String.
                            // We passed our version string as the first arg to Identify::Config::new.
                            // The first arg is `protocol_version`!
                            // So we should check `info.protocol_version`.

                            if info.protocol_version.contains("TextProcessing") {
                                supported_tasks.push(crate::agent::task::TaskType::TextProcessing);
                            }
                             if info.protocol_version.contains("VectorComputation") {
                                supported_tasks.push(crate::agent::task::TaskType::VectorComputation);
                            }

                            use crate::network::peers::{PeerInfo, ConnectionStatus, PeerCapabilities};
                            let peer_info = PeerInfo {
                                peer_id: PeerId(peer_id.to_string()),
                                addresses: vec![], // We might want to store observed addr
                                last_seen: chrono::Utc::now(),
                                reputation: 50,
                                capabilities: PeerCapabilities {
                                    supported_tasks,
                                },
                                status: ConnectionStatus::Connected,
                            };
                            peer_cache_clone.upsert_peer(peer_info).await;
                        }
                        SwarmEvent::Behaviour(AgentBehaviorEvent::RequestResponse(request_response::Event::Message {
                            message: request_response::Message::Request { request, channel, .. },
                            peer: _,
                        })) => {
                            debug!("Got direct request: {:?}", request);
                            // Handle request - we need to send it to the agent, and ideally send a response back.
                            // For now, let's pass it up to the agent message loop.
                            if let Some(callback) = &message_callback {
                                // We might need to handle responses later.
                                // Currently our callback just takes Vec<u8> (Message content)
                                // We don't have a way to reply to this specific request via the channel in this MVP structure
                                // without modifying the callback signature.
                                // For now, we just assume the request is a standard Message.
                                let _ = callback.send(request.message.clone()).await;

                                // Send a simple acknowledgement back
                                let response = crate::network::protocol::AgentResponse {
                                    message: "ACK".as_bytes().to_vec(),
                                };
                                let _ = swarm.behaviour_mut().request_response.send_response(channel, response);
                            }
                        }
                        SwarmEvent::Behaviour(AgentBehaviorEvent::RequestResponse(request_response::Event::Message {
                            message: request_response::Message::Response { response, .. },
                            peer: _,
                        })) => {
                            debug!("Got direct response: {:?}", response);
                            // Currently we don't have a way to route responses back to the specific caller request.
                            // But if the response contains a TaskResponse, the Agent message loop will handle it.
                            if let Some(callback) = &message_callback {
                                let _ = callback.send(response.message).await;
                            }
                        }
                        SwarmEvent::Behaviour(behavior_event) => {
                             // Handle other behavior events
                             debug!("Other behavior event: {:?}", behavior_event);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            info!("Connection established with {:?}", peer_id);
                            // Store connected peer (simplification, storing just IP)
                            let addr = endpoint.get_remote_address();
                            let mut ip = None;
                            let mut port = None;
                            for proto in addr.iter() {
                                match proto {
                                    Protocol::Ip4(i) => ip = Some(std::net::IpAddr::V4(i)),
                                    Protocol::Ip6(i) => ip = Some(std::net::IpAddr::V6(i)),
                                    Protocol::Tcp(p) => port = Some(p),
                                    _ => {}
                                }
                            }
                            if let (Some(i), Some(p)) = (ip, port) {
                                let socket_addr = SocketAddr::new(i, p);
                                connected_peers_clone.lock().await.push(socket_addr);
                            }
                        }
                        SwarmEvent::ConnectionClosed { peer_id, endpoint, .. } => {
                            info!("Connection closed with {:?}", peer_id);
                            let addr = endpoint.get_remote_address();
                            let mut ip = None;
                            let mut port = None;
                            for proto in addr.iter() {
                                match proto {
                                    Protocol::Ip4(i) => ip = Some(std::net::IpAddr::V4(i)),
                                    Protocol::Ip6(i) => ip = Some(std::net::IpAddr::V6(i)),
                                    Protocol::Tcp(p) => port = Some(p),
                                    _ => {}
                                }
                            }
                            if let (Some(i), Some(p)) = (ip, port) {
                                let socket_addr = SocketAddr::new(i, p);
                                let mut peers = connected_peers_clone.lock().await;
                                if let Some(pos) = peers.iter().position(|x| *x == socket_addr) {
                                    peers.remove(pos);
                                }
                            }
                        }
                         _ => {}
                    },
                    command = rx.recv() => match command {
                        Some(NetworkCommand::Dial { addr }) => {
                            if let Err(e) = swarm.dial(addr) {
                                error!("Failed to dial: {:?}", e);
                            }
                        }
                        Some(NetworkCommand::SendMessage { peer_id: _, message }) => {
                             // Broadcast via Gossipsub
                             let topic = gossipsub::IdentTopic::new("p2p-ai-agents-global");
                             if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, message) {
                                 error!("Failed to publish message: {:?}", e);
                             }
                        }
                        Some(NetworkCommand::SendRequest { peer_id, request }) => {
                            let request_data = crate::network::protocol::AgentRequest {
                                message: request,
                            };
                            swarm.behaviour_mut().request_response.send_request(&peer_id, request_data);
                        }
                        Some(NetworkCommand::Shutdown) => {
                            break;
                        }
                        None => break,
                    }
                }
            }
        });

        self.is_running = true;
        Ok(())
    }

    /// Shutdown the network manager.
    pub async fn shutdown(&mut self) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NotRunning);
        }

        if let Some(tx) = &self.command_sender {
            let _ = tx.send(NetworkCommand::Shutdown).await;
        }

        self.is_running = false;
        Ok(())
    }

    /// Perform a graceful shutdown of the network manager.
    pub async fn graceful_shutdown(&mut self) -> NetworkResult<()> {
        self.shutdown().await
    }

    /// Set the transport protocol.
    pub fn set_transport(&mut self, transport: &str) {
        self.transport_type = transport.to_string();
    }

    /// Get the transport protocol.
    pub fn get_transport(&self) -> &str {
        &self.transport_type
    }

    /// Dial a peer at the given address.
    pub async fn dial(&self, addr: Multiaddr) -> NetworkResult<()> {
        let libp2p_addr = addr.to_libp2p()?;
        if let Some(tx) = &self.command_sender {
            tx.send(NetworkCommand::Dial { addr: libp2p_addr })
                .await
                .map_err(|_| NetworkError::NotRunning)?;
            Ok(())
        } else {
            Err(NetworkError::NotRunning)
        }
    }

    /// Simulate a transport failure.
    pub async fn simulate_transport_failure(&mut self) -> NetworkResult<()> {
        if !self.is_running {
            return Err(NetworkError::NotRunning);
        }
        // Simulate transport failure by clearing peers
        let mut peers = self.connected_peers.lock().await;
        peers.clear();
        Ok(())
    }

    /// Add a connected peer to the manager (for testing/demo purposes).
    pub async fn add_connected_peer(&self, addr: SocketAddr) {
        self.connected_peers.lock().await.push(addr);
    }

    /// Send a message by pushing it to the message queue and broadcasting via swarm.
    pub async fn send_message(&self, msg: NetworkMessage) {
        // 1. Push to internal queue for testing/debug visibility
        self.messages.lock().await.push(msg.clone());

        // 2. Send command to swarm to publish
        if let Some(tx) = &self.command_sender {
            // Note: Gossipsub ignores the specific peer_id in SendMessage when we blindly publish to topic,
            // but we pass a dummy one or the intended target if we were doing direct unicast.
            // For now, our SendMessage handler in the loop just publishes to the global topic.
            // We need to convert our internal peer ID string to libp2p PeerId if we wanted direct messaging,
            // but for broadcast we just trigger the command.

            // We'll create a dummy PeerId since the handler ignores it for now
            let dummy_peer = Libp2pPeerId::random();

            let _ = tx
                .send(NetworkCommand::SendMessage {
                    peer_id: dummy_peer,
                    message: msg.content,
                })
                .await;
        } else {
            error!("Cannot send message: Network not running (command_sender missing)");
        }
    }

    /// Send a direct request to a specific peer.
    pub async fn send_request(&self, peer_id: &str, request: Vec<u8>) -> NetworkResult<()> {
        let libp2p_peer_id =
            Libp2pPeerId::from_str(peer_id).map_err(|e| NetworkError::Libp2p(e.to_string()))?;

        if let Some(tx) = &self.command_sender {
            tx.send(NetworkCommand::SendRequest {
                peer_id: libp2p_peer_id,
                request,
            })
            .await
            .map_err(|_| NetworkError::NotRunning)?;
            Ok(())
        } else {
            Err(NetworkError::NotRunning)
        }
    }

    /// Receive a message by popping from the message queue.
    pub async fn receive_message(&self) -> Option<NetworkMessage> {
        #[cfg(feature = "metrics-prometheus")]
        let start = std::time::Instant::now();

        let result = self.messages.lock().await.pop();

        #[cfg(feature = "metrics-prometheus")]
        if result.is_some() {
            if let Some(ref metrics) = self.prometheus_metrics {
                metrics.record_message_received();
                let duration_ms = start.elapsed().as_millis() as u64;
                metrics.record_message_duration(duration_ms);
            }
        }

        result
    }

    /// Get received messages.
    pub async fn get_messages(&self) -> Vec<NetworkMessage> {
        let messages = self.messages.lock().await;
        messages.clone()
    }

    /// Get connected peers.
    pub async fn get_connected_peers(&self) -> Vec<SocketAddr> {
        let peers = self.connected_peers.lock().await;
        peers.clone()
    }

    /// Get listen addresses
    pub async fn get_listen_addresses(&self) -> Vec<Libp2pMultiaddr> {
        self.listen_addresses.lock().await.clone()
    }

    /// Get a reference to the network configuration.
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }
}

/// Event channels for internal network communication.
pub struct EventChannels {
    // Placeholder for event channels
}

impl EventChannels {
    /// Create a new set of event channels.
    pub fn new() -> Self {
        EventChannels {}
    }
}

impl Default for EventChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring and constructing a NetworkManager.
pub struct NetworkManagerBuilder {
    config: Option<NetworkConfig>,
}

impl NetworkManagerBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        NetworkManagerBuilder { config: None }
    }

    /// Set the network configuration.
    pub fn with_config(mut self, config: NetworkConfig) -> Self {
        self.config = Some(config);
        self
    }
}

impl Default for NetworkManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Collector for network metrics.
pub struct MetricsCollector {
    metrics: HashMap<String, u64>,
}
impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        MetricsCollector {
            metrics: HashMap::new(),
        }
    }
    /// Increment a metric by 1.
    pub fn increment(&mut self, key: &str) {
        *self.metrics.entry(key.to_string()).or_insert(0) += 1;
    }
    /// Get the value of a metric.
    pub fn get(&self, key: &str) -> u64 {
        *self.metrics.get(key).unwrap_or(&0)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages network resources.
pub struct ResourceManager;
impl ResourceManager {
    /// Create a new resource manager.
    pub fn new() -> Self {
        ResourceManager
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitors network health.
pub struct HealthMonitor;
impl HealthMonitor {
    /// Create a new health monitor.
    pub fn new() -> Self {
        HealthMonitor
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Handles network security.
pub mod security;
pub use security::CertificateManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_channels_new() {
        let _ = EventChannels::new();
    }

    #[test]
    fn test_network_manager_builder_new_and_with_config() {
        let config = NetworkConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            bootstrap_peers: vec![],
            max_peers: 10,
            protocol_config: ProtocolConfig {},
            resource_limits: ResourceLimits {
                max_bandwidth: 1024,
                max_memory: 2048,
                max_connections: 100,
            },
            security_config: SecurityConfig {
                trusted_authorities: vec![],
                local_certificate: None,
            },
        };
        let _ = NetworkManagerBuilder::new().with_config(config);
    }

    // Other tests...
}
