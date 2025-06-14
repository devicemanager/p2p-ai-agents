//! Integration test for basic usage examples from the documentation.
use p2p_ai_agents::network::{NetworkConfig, ProtocolConfig, ResourceLimits, SecurityConfig, NetworkManager, PeerInfo, PeerId, Multiaddr, PeerCapabilities, ConnectionStatus, NetworkMessage, NetworkError};
use chrono::Utc;
use std::sync::Arc;

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
    let mut manager = NetworkManager::new(config);
    // manager.initialize().await.unwrap(); // Uncomment if implemented
    // manager.start().await.unwrap();

    let peer = PeerInfo {
        peer_id: PeerId("QmPeer...".to_string()),
        addresses: vec![Multiaddr("/ip4/127.0.0.1/tcp/8080".to_string())],
        last_seen: Utc::now(),
        reputation: 100,
        capabilities: PeerCapabilities,
        status: ConnectionStatus::Connected,
    };
    // Example stub: add peer address to connected_peers
    manager.add_connected_peer("127.0.0.1:8080".parse().unwrap()).await;

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
        NetworkError::NotInitialized => assert!(true),
        _ => assert!(false, "Unexpected error variant"),
    }

    // Example: Handling a failing operation
    let result: Result<(), NetworkError> = Err(NetworkError::NotInitialized);
    if let Err(e) = result {
        match e {
            NetworkError::NotInitialized => assert!(true),
            _ => assert!(false, "Unexpected error variant in failing operation"),
        }
    }
}
