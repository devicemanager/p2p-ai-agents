//! P2P Network Agent
//!
//! Provides peer-to-peer networking using libp2p with mDNS discovery.
//! This is a simplified MVP implementation for local network only.

use crate::identity::AgentIdentity;
use crate::network::protocol::AgentRequest;
use libp2p::{
    futures::StreamExt,
    gossipsub::{self, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

const TOPIC_NAME: &str = "p2p-agents";

/// P2P agent for discovering peers and exchanging messages
pub struct P2PAgent {
    swarm: Swarm<MyBehaviour>,
    peers: HashMap<PeerId, PeerInfo>,
    _identity: AgentIdentity,
    topic: IdentTopic,
}

/// Information about a discovered peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer's unique identifier
    pub peer_id: PeerId,
    /// Known multiaddresses for this peer
    pub addresses: Vec<Multiaddr>,
}

/// Combined network behavior
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    mdns: mdns::tokio::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

impl P2PAgent {
    /// Create new P2P agent with given identity
    #[tracing::instrument(skip(identity))]
    pub async fn new(identity: AgentIdentity) -> Result<Self, Box<dyn Error>> {
        // Convert AgentIdentity to libp2p Keypair
        let keypair = Keypair::ed25519_from_bytes(vec![0u8; 32]).unwrap(); // TODO: Extract from identity
        let peer_id = PeerId::from(keypair.public());

        // Create topic for messaging
        let topic = IdentTopic::new(TOPIC_NAME);

        // Build swarm with TCP transport + Noise + mDNS + Gossipsub
        let swarm = SwarmBuilder::with_existing_identity(keypair.clone())
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, || {
                yamux::Config::default()
            })?
            .with_behaviour(|key| {
                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(1))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .map_err(std::io::Error::other)?;

                let mut gossipsub = gossipsub::Behaviour::new(
                    MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .map_err(std::io::Error::other)?;

                gossipsub.subscribe(&topic).map_err(std::io::Error::other)?;

                Ok(MyBehaviour { mdns, gossipsub })
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(Self {
            swarm,
            peers: HashMap::new(),
            _identity: identity,
            topic,
        })
    }

    /// Send a message to all connected peers
    #[tracing::instrument(skip(self))]
    pub fn send_message(&mut self, message: String) -> Result<(), Box<dyn Error>> {
        let request = AgentRequest { message };
        let data = serde_json::to_vec(&request)?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data)?;
        Ok(())
    }

    /// Start listening on a network address
    #[tracing::instrument(skip(self))]
    pub fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        Ok(())
    }

    /// Process one network event (for testing and manual control)
    #[tracing::instrument(skip(self))]
    pub async fn poll_once(&mut self) -> Result<(), Box<dyn Error>> {
        match self.swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!("Listening on {}", address);
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    tracing::info!("Discovered peer: {} at {}", peer_id, addr);
                    self.peers
                        .entry(peer_id)
                        .or_insert_with(|| PeerInfo {
                            peer_id,
                            addresses: Vec::new(),
                        })
                        .addresses
                        .push(addr);
                }
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, _addr) in peers {
                    tracing::info!("Peer expired: {}", peer_id);
                    self.peers.remove(&peer_id);
                }
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                message,
                ..
            })) => {
                if let Ok(request) = serde_json::from_slice::<AgentRequest>(&message.data) {
                    tracing::info!("Received message: {}", request.message);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Start the agent and run event loop until error (production use)
    #[tracing::instrument(skip(self))]
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.listen()?;
        loop {
            self.poll_once().await?;
        }
    }

    /// List currently discovered peers
    pub fn list_peers(&self) -> Vec<PeerInfo> {
        self.peers.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    // NOTE: These tests are currently ignored because mdns spawns background
    // tasks that don't complete. Integration tests in tests/ dir work properly.
    #[tokio::test]
    #[ignore]
    async fn test_create_agent() {
        // Use timeout to prevent hanging
        let result = timeout(Duration::from_secs(2), async {
            let identity = AgentIdentity::generate();
            let agent = P2PAgent::new(identity).await.unwrap();

            // Agent should be created successfully
            assert_eq!(agent.peers.len(), 0);
        })
        .await;

        assert!(result.is_ok(), "Test timed out");
    }

    #[tokio::test]
    #[ignore]
    async fn test_send_message() {
        let result = timeout(Duration::from_secs(2), async {
            let identity = AgentIdentity::generate();
            let mut agent = P2PAgent::new(identity).await.unwrap();

            // Message send will fail if not subscribed to any peers, but that's expected
            // Just test that the function doesn't panic
            let result = agent.send_message("Hello, world!".to_string());
            // Gossipsub publish may fail if no peers, which is OK for unit test
            let _ = result;
        })
        .await;

        assert!(result.is_ok(), "Test timed out");
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_peers_empty() {
        let result = timeout(Duration::from_secs(2), async {
            let identity = AgentIdentity::generate();
            let agent = P2PAgent::new(identity).await.unwrap();

            let peers = agent.list_peers();
            assert_eq!(peers.len(), 0);
        })
        .await;

        assert!(result.is_ok(), "Test timed out");
    }
}
