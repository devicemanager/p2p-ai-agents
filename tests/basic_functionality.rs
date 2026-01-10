//! Basic integration tests that should always pass
//! These tests ensure the library compiles and basic functionality works

use p2p_ai_agents::prelude::*;

#[test]
fn test_library_compiles() {
    // Basic smoke test to ensure the library compiles
    assert_eq!(1 + 1, 2);
}

#[test]
fn test_error_types() {
    // Test that error types can be created and matched
    use p2p_ai_agents::Error;

    let io_error = std::io::Error::other("test");
    let lib_error = Error::Io(io_error);

    match lib_error {
        Error::Io(_) => {
            // Expected path - IO error was created successfully
        }
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_agent_id_creation() {
    // Test basic AgentId functionality if available
    let _agent_id = AgentId::new();
    // Test passes if AgentId can be created without panicking
}

#[cfg(feature = "network")]
#[test]
fn test_network_config_creation() {
    // Test basic NetworkConfig creation if network feature is enabled
    use std::net::SocketAddr;

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let _config = NetworkConfig {
        listen_addr: addr,
        bootstrap_peers: vec![],
        max_peers: 50,
        protocol_config: ProtocolConfig {},
        resource_limits: ResourceLimits {
            max_bandwidth: 1024 * 1024,
            max_memory: 512 * 1024 * 1024,
            max_connections: 100,
        },
        security_config: SecurityConfig {
            trusted_authorities: vec![],
            local_certificate: None,
        },
    };

    // Test passes if NetworkConfig can be created without panicking
}

#[tokio::test]
async fn test_async_functionality() {
    // Basic async test to ensure tokio integration works
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    // Test passes if async functionality works
}
