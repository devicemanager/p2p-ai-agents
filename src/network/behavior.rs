use libp2p::{
    identify,
    kad,
    mdns,
    ping,
};

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct AgentBehavior {
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

impl AgentBehavior {
    pub fn new(local_public_key: libp2p::identity::PublicKey) -> Result<Self, Box<dyn std::error::Error>> {
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
