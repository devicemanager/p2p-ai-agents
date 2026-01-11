//! P2P Network Agent
//!
//! Provides peer-to-peer networking using libp2p with mDNS discovery.
//! This is a simplified MVP implementation for local network only.

use crate::identity::AgentIdentity;
use crate::network::protocol::{AgentCodec, AgentProtocol, AgentRequest, AgentResponse};
use libp2p::{
    futures::StreamExt,
    mdns, noise,
    request_response::{self, OutboundRequestId, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

const MESSAGE_SIZE_LIMIT: usize = 10 * 1024 * 1024; // 10MB

/// P2P agent for discovering peers and exchanging messages
pub struct P2PAgent {
    swarm: Swarm<MyBehaviour>,
    peers: HashMap<PeerId, PeerInfo>,
    _identity: AgentIdentity,
    pending_requests: HashMap<OutboundRequestId, tokio::sync::oneshot::Sender<AgentResponse>>,
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
    request_response: request_response::Behaviour<AgentCodec>,
}

impl P2PAgent {
    /// Create new P2P agent with given identity
    #[tracing::instrument(skip(identity))]
    pub async fn new(identity: AgentIdentity) -> Result<Self, Box<dyn Error>> {
        // Extract keypair from AgentIdentity
        let keypair = identity.keypair().clone();
        let peer_id = PeerId::from(keypair.public());

        // Build swarm with TCP transport + Noise + mDNS + Request-Response
        let swarm = SwarmBuilder::with_existing_identity(keypair.clone())
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, || {
                yamux::Config::default()
            })?
            .with_behaviour(|_key| {
                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

                let request_response = request_response::Behaviour::with_codec(
                    AgentCodec::default(),
                    std::iter::once((AgentProtocol, ProtocolSupport::Full)),
                    request_response::Config::default()
                        .with_request_timeout(Duration::from_secs(30)),
                );

                Ok(MyBehaviour {
                    mdns,
                    request_response,
                })
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(Self {
            swarm,
            peers: HashMap::new(),
            _identity: identity,
            pending_requests: HashMap::new(),
        })
    }

    /// Send a message to a specific peer and wait for response
    #[tracing::instrument(skip(self))]
    pub async fn send_message(
        &mut self,
        peer_id: PeerId,
        message: String,
    ) -> Result<AgentResponse, Box<dyn Error>> {
        // Check message size
        if message.len() > MESSAGE_SIZE_LIMIT {
            return Err(format!("Message exceeds size limit of {}MB", MESSAGE_SIZE_LIMIT / 1024 / 1024).into());
        }

        let request = AgentRequest { message };
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        let request_id = self
            .swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, request);
        
        self.pending_requests.insert(request_id, tx);

        // Wait for response with timeout
        match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err("Response channel closed".into()),
            Err(_) => Err("Request timed out".into()),
        }
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
            SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    tracing::info!("Received request: {}", request.message);
                    // Auto-respond with echo for MVP
                    let response = AgentResponse {
                        message: format!("Echo: {}", request.message),
                    };
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, response);
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    if let Some(tx) = self.pending_requests.remove(&request_id) {
                        let _ = tx.send(response);
                    }
                }
            },
            SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    request_id, error, ..
                },
            )) => {
                tracing::error!("Outbound request failed: {:?}", error);
                self.pending_requests.remove(&request_id);
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

    #[tokio::test]
    async fn test_create_agent() {
        // Test agent creation (fast, no network operations)
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

    #[tokio::test]
    async fn test_message_size_limit() {
        let result = timeout(Duration::from_secs(2), async {
            let identity = AgentIdentity::generate();
            let mut agent = P2PAgent::new(identity).await.unwrap();

            // Create message exceeding 10MB limit
            let large_message = "x".repeat(11 * 1024 * 1024);
            let fake_peer = PeerId::random();
            
            // Should fail due to size limit
            let result = agent.send_message(fake_peer, large_message).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("exceeds size limit"));
        })
        .await;

        assert!(result.is_ok(), "Test timed out");
    }
}
