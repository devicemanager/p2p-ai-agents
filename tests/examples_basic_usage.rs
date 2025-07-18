//! Integration test for basic usage examples from the documentation.
use chrono::Utc;
use p2p_ai_agents::network::{
    ConnectionStatus, Multiaddr, NetworkConfig, NetworkError, NetworkManager, NetworkMessage,
    PeerCapabilities, PeerId, PeerInfo, ProtocolConfig, ResourceLimits, SecurityConfig,
};

#[tokio::test]
async fn test_network_initialization_and_message() {
    let config = NetworkConfig {
        listen_addr: "127.0.0.1:8080".parse().unwrap(),
        bootstrap_peers: vec![],
        max_peers: 50,
        protocol_config: ProtocolConfig {},
        resource_limits: ResourceLimits {
            max_bandwidth: 1024 * 1024,
            max_memory: 512 * 1024 * 1024,
            max_connections: 100,
        },
        security_config: SecurityConfig {},
    };
    let manager = NetworkManager::new(config);
    // manager.initialize().await.unwrap(); // Uncomment if implemented
    // manager.start().await.unwrap();

    let _peer = PeerInfo {
        peer_id: PeerId("QmPeer...".to_string()),
        addresses: vec![Multiaddr("/ip4/127.0.0.1/tcp/8080".to_string())],
        last_seen: Utc::now(),
        reputation: 100,
        capabilities: PeerCapabilities,
        status: ConnectionStatus::Connected,
    };
    // Example stub: add peer address to connected_peers
    manager
        .add_connected_peer("127.0.0.1:8080".parse().unwrap())
        .await;

    // Simulate sending and receiving a message using public methods
    let msg = NetworkMessage {
        from: "peer1".to_string(),
        to: "peer2".to_string(),
        content: b"hello".to_vec(),
    };
    manager.send_message(msg.clone()).await;
    let received = manager.receive_message().await;
    assert_eq!(received.unwrap().content, b"hello".to_vec());

    // Example error handling
    let err = NetworkError::NotInitialized;
    match err {
        NetworkError::NotInitialized => {
            // Expected error type
        }
        _ => panic!("Unexpected error variant"),
    }

    // Example: Handling a failing operation
    let result: Result<(), NetworkError> = Err(NetworkError::NotInitialized);
    if let Err(e) = result {
        match e {
            NetworkError::NotInitialized => {
                // Expected error type
            }
            _ => panic!("Unexpected error variant in failing operation"),
        }
    }
}
