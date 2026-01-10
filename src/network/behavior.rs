#![allow(missing_docs)]

use libp2p::{gossipsub, identify, kad, mdns, ping};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

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
    /// Gossipsub for message propagation
    pub gossipsub: gossipsub::Behaviour,
}

impl AgentBehavior {
    /// Create a new AgentBehavior with the given keypair and agent version
    pub fn new(
        local_key: libp2p::identity::Keypair,
        agent_version: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let local_public_key = local_key.public();
        let peer_id = local_public_key.to_peer_id();
        let store = kad::store::MemoryStore::new(peer_id);
        let kademlia = kad::Behaviour::new(peer_id, store);

        // Configure Gossipsub
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn) // content-address messages
            .build()
            .map_err(std::io::Error::other)?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key),
            gossipsub_config,
        )
        .map_err(std::io::Error::other)?;

        // We will customize the agent version to include capabilities string if possible,
        // but Identify config is static here.
        // We need to pass the capabilities string to `new`.
        // Let's change the signature of `new`.

        Ok(Self {
            identify: identify::Behaviour::new(identify::Config::new(
                agent_version,
                local_public_key,
            )),
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?,
            ping: ping::Behaviour::new(ping::Config::new()),
            kademlia,
            gossipsub,
        })
    }
}
