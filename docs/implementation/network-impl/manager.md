# Network Manager Implementation

*Part of Network Implementation Guide*

## Network Manager Implementation

```rust
pub struct NetworkManager {
    swarm: Swarm<NetworkProtocol>,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    config: NetworkConfig,
    event_sender: mpsc::Sender<NetworkEvent>,
    event_receiver: mpsc::Receiver<NetworkEvent>,
    metrics: Arc<NetworkMetricsCollector>,
}

impl NetworkManager {
    pub async fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        // Generate or load identity
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        // Create transport
        let transport = libp2p::tokio::Transport::new(
            libp2p::tls::Config::new(&local_key),
            libp2p::noise::Config::new(&local_key),
        )
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(libp2p::noise::Config::new(&local_key))
        .multiplex(libp2p::yamux::Config::default())
        .boxed();
        
        // Create protocols
        let (task_sender, task_receiver) = mpsc::channel(32);
        let (discovery_sender, discovery_receiver) = mpsc::channel(32);
        let (resource_sender, resource_receiver) = mpsc::channel(32);
        let (health_sender, health_receiver) = mpsc::channel(32);
        
        let protocols = NetworkProtocol::new(
            task_sender,
            discovery_sender,
            resource_sender,
            health_sender,
        );
        
        // Create swarm
        let mut swarm = Swarm::new(transport, protocols, local_peer_id);
        
        // Listen on configured address
        swarm.listen_on(config.listen_addr.clone())?;
        
        // Bootstrap with known peers
        for addr in &config.bootstrap_peers {
            swarm.dial(addr.clone())?;
        }
        
        // Create event channels
        let (event_sender, event_receiver) = mpsc::channel(64);
        
        Ok(Self {
            swarm,
            peers: Arc::new(RwLock::new(HashMap::new())),
            config,
            event_sender,
            event_receiver,
            metrics: Arc::new(NetworkMetricsCollector::new()),
        })
    }
    
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        // Start network event loop
        let mut swarm = self.swarm.clone();
        let peers = self.peers.clone();
        let event_sender = self.event_sender.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            Self::event_loop(swarm, peers, event_sender, metrics).await
        });
        
        Ok(())
    }
    
    async fn event_loop(
        mut swarm: Swarm<NetworkProtocol>,
        peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        event_sender: mpsc::Sender<NetworkEvent>,
        metrics: Arc<NetworkMetricsCollector>,
    ) {
        loop {
            match swarm.next().await {
                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                    info!("Listening on {}", address);
                    metrics.record_listen_addr(&address);
                }
                
                Some(SwarmEvent::ConnectionEstablished { peer_id, .. }) => {
                    info!("Connected to peer {}", peer_id);
                    metrics.record_connection(&peer_id);
                    
                    // Update peer info
                    let mut peers = peers.write().await;
                    peers.entry(peer_id).or_insert_with(|| PeerInfo {
                        peer_id,
                        addresses: Vec::new(),
                        protocols: HashSet::new(),
                        last_seen: Utc::now(),
                        latency: None,
                        reputation: 0,
                    });
                }
                
                Some(SwarmEvent::ConnectionClosed { peer_id, .. }) => {
                    info!("Disconnected from peer {}", peer_id);
                    metrics.record_disconnection(&peer_id);
                    
                    // Update peer info
                    let mut peers = peers.write().await;
                    if let Some(peer) = peers.get_mut(&peer_id) {
                        peer.last_seen = Utc::now();
                    }
                }
                
                Some(SwarmEvent::Behaviour(protocol_event)) => {
                    match protocol_event {
                        NetworkProtocol::TaskProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_task_event(event, &event_sender).await;
                            }
                        }
                        
                        NetworkProtocol::DiscoveryProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_discovery_event(event, &event_sender).await;
                            }
                        }
                        
                        NetworkProtocol::ResourceProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_resource_event(event, &event_sender).await;
                            }
                        }
                        
                        NetworkProtocol::HealthProtocol { events, .. } => {
                            if let Ok(event) = events.recv().await {
                                Self::handle_health_event(event, &event_sender).await;
                            }
                        }
                    }
                }
                
                _ => {}
            }
        }
    }
}
```

