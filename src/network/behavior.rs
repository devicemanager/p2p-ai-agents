#![allow(missing_docs)]

use libp2p::{identify, kad, mdns, ping};

/// Agent network behavior combining multiple libp2p protocols
#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct AgentBehavior {
    /// Identity protocol for peer identification
    pub identify: identify::Behaviour,
    /// mDNS protocol for local peer discovery
    pub mdns: mdns::tokio::Behaviour,
    /// Ping protocol for connection health checks
    pub ping: ping::Behaviour,
    /// Kademlia DHT for distributed peer routing
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

impl AgentBehavior {
    /// Create a new AgentBehavior with the given public key
    pub fn new(
        local_public_key: libp2p::identity::PublicKey,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let peer_id = local_public_key.to_peer_id();
        let store = kad::store::MemoryStore::new(peer_id);
        let kademlia = kad::Behaviour::new(peer_id, store);

        Ok(Self {
            identify: identify::Behaviour::new(identify::Config::new(
                "/p2p-ai-agents/1.0.0".to_string(),
                local_public_key,
            )),
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?,
            ping: ping::Behaviour::new(ping::Config::new()),
            kademlia,
        })
    }
}
