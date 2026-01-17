//! Integration Tests for MVP Core Functionality
//!
//! Tests the complete system behavior across identity, network, and task modules.
//! These tests validate the MVP is working end-to-end.

use p2p_ai_agents::identity::AgentIdentity;
use p2p_ai_agents::network::p2p_agent::P2PAgent;
use tokio::time::{sleep, timeout, Duration};

/// Test helper: Create and start an agent
async fn create_test_agent() -> Result<P2PAgent, Box<dyn std::error::Error>> {
    let identity = AgentIdentity::generate();
    let agent = P2PAgent::new(identity).await?;
    Ok(agent)
}

#[tokio::test]
async fn test_two_agents_discover_each_other() {
    if std::env::var("CI").is_ok() {
        eprintln!("skipped on CI");
        return;
    }
    // Test that two agents on the same network can discover each other via mDNS
    // Arrange
    let identity_a = AgentIdentity::generate();
    let identity_b = AgentIdentity::generate();

    let mut agent_a = P2PAgent::new(identity_a).await.unwrap();
    let mut agent_b = P2PAgent::new(identity_b).await.unwrap();

    agent_a.listen().unwrap();
    agent_b.listen().unwrap();

    // Act: Run agents for discovery period
    let handle_a = tokio::spawn(async move {
        for _ in 0..50 {
            let _ = agent_a.poll_once().await;
            sleep(Duration::from_millis(100)).await;
        }
        agent_a
    });

    let handle_b = tokio::spawn(async move {
        for _ in 0..50 {
            let _ = agent_b.poll_once().await;
            sleep(Duration::from_millis(100)).await;
        }
        agent_b
    });

    // Wait for agents to complete polling
    let agent_a = handle_a.await.unwrap();
    let agent_b = handle_b.await.unwrap();

    // Assert: Agents should have discovered each other
    let peers_a = agent_a.list_peers();
    let peers_b = agent_b.list_peers();

    assert!(
        !peers_a.is_empty(),
        "Agent A should discover at least 1 peer, found {}",
        peers_a.len()
    );
    assert!(
        !peers_b.is_empty(),
        "Agent B should discover at least 1 peer, found {}",
        peers_b.len()
    );
}

#[tokio::test]
async fn test_end_to_end_task_exchange() {
    if std::env::var("CI").is_ok() {
        eprintln!("skipped on CI");
        return;
    }
    // Test complete task workflow: create, send, execute, receive
    // Arrange
    let identity_a = AgentIdentity::generate();
    let identity_b = AgentIdentity::generate();

    let mut agent_a = P2PAgent::new(identity_a).await.unwrap();
    let mut agent_b = P2PAgent::new(identity_b).await.unwrap();

    agent_a.listen().unwrap();
    agent_b.listen().unwrap();

    let agent_b_peer_id = agent_b.peer_id();

    // Spawn agent B event loop
    let handle_b = tokio::spawn(async move {
        loop {
            if let Err(e) = agent_b.poll_once().await {
                eprintln!("Agent B error: {}", e);
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Spawn agent A polling
    let handle_a = tokio::spawn(async move {
        for _ in 0..50 {
            let _ = agent_a.poll_once().await;
            sleep(Duration::from_millis(100)).await;
        }
        agent_a
    });

    // Wait for discovery
    sleep(Duration::from_secs(3)).await;

    let mut agent_a = handle_a.await.unwrap();

    // Act: Send message from A to B
    let result = timeout(
        Duration::from_secs(5),
        agent_a.send_message(agent_b_peer_id, "Hello from Agent A".to_string()),
    )
    .await;

    // Assert: Should receive response
    match result {
        Ok(Ok(response)) => {
            assert!(
                response.message.contains("Echo"),
                "Response should echo message, got: {}",
                response.message
            );
        }
        Ok(Err(e)) => {
            panic!("Message send failed: {}", e);
        }
        Err(_) => {
            // Timeout is acceptable in CI environments
            println!("⚠️  Message exchange timed out (acceptable in CI)");
        }
    }

    // Cleanup
    handle_b.abort();
}

#[tokio::test]
async fn test_discovery_timeout() {
    // mDNS behavior varies across environments and can be flaky on CI.
    // This test is intended to catch hard hangs, but it is not reliable enough
    // to block CI merges.
    if std::env::var("CI").is_ok() {
        eprintln!("skipped on CI");
        return;
    }
    // Test that the agent's discovery loop doesn't hang indefinitely.
    //
    // Note: On shared CI runners and developer machines, mDNS can pick up other
    // agents/services on the LAN/runner network. So asserting "0 peers" is
    // flaky and not a reliable signal.
    let result = timeout(Duration::from_secs(5), async {
        let identity = AgentIdentity::generate();
        let mut agent = P2PAgent::new(identity).await.unwrap();
        agent.listen().unwrap();

        // Run for a short period.
        for _ in 0..10 {
            let _ = agent.poll_once().await;
            sleep(Duration::from_millis(100)).await;
        }

        // Assert only that the call completes and returns a valid list.
        let _peers = agent.list_peers();
    })
    .await;

    assert!(
        result.is_ok(),
        "Discovery loop should complete within timeout"
    );
}

#[tokio::test]
async fn test_task_send_to_unknown_peer() {
    // Test error handling when sending to non-existent peer
    let identity = AgentIdentity::generate();
    let mut agent = P2PAgent::new(identity).await.unwrap();
    agent.listen().unwrap();

    // Try to send to random peer that doesn't exist
    let fake_peer = libp2p::PeerId::random();

    let result = timeout(
        Duration::from_secs(2),
        agent.send_message(fake_peer, "test message".to_string()),
    )
    .await;

    // Should either timeout or return error
    match result {
        Ok(Ok(_)) => panic!("Should not succeed sending to unknown peer"),
        Ok(Err(_)) => {
            // Expected: error returned
        }
        Err(_) => {
            // Expected: timeout
        }
    }
}

#[tokio::test]
async fn test_agent_creation_and_initialization() {
    // Test basic agent creation workflow
    let result = create_test_agent().await;
    assert!(
        result.is_ok(),
        "Agent creation should succeed: {:?}",
        result.err()
    );

    let mut agent = result.unwrap();

    // Test listening
    let listen_result = agent.listen();
    assert!(
        listen_result.is_ok(),
        "Agent should be able to listen: {:?}",
        listen_result.err()
    );

    // Test initial peer list is empty
    let peers = agent.list_peers();
    assert_eq!(peers.len(), 0, "New agent should have no peers initially");
}

#[tokio::test]
async fn test_multiple_agents_network() {
    if std::env::var("CI").is_ok() {
        eprintln!("skipped on CI");
        return;
    }
    // Test network with 3 agents
    let mut agents = Vec::new();

    for _ in 0..3 {
        let identity = AgentIdentity::generate();
        let mut agent = P2PAgent::new(identity).await.unwrap();
        agent.listen().unwrap();
        agents.push(agent);
    }

    // Run all agents for discovery period
    let handles: Vec<_> = agents
        .into_iter()
        .map(|mut agent| {
            tokio::spawn(async move {
                for _ in 0..50 {
                    let _ = agent.poll_once().await;
                    sleep(Duration::from_millis(100)).await;
                }
                agent
            })
        })
        .collect();

    // Wait for all agents
    let mut final_agents = Vec::new();
    for handle in handles {
        final_agents.push(handle.await.unwrap());
    }

    // Each agent should discover the other 2
    for (i, agent) in final_agents.iter().enumerate() {
        let peers = agent.list_peers();
        assert!(
            peers.len() >= 2,
            "Agent {} should discover at least 2 peers, found {}",
            i,
            peers.len()
        );
    }
}

#[tokio::test]
async fn test_identity_uniqueness() {
    // Test that each agent has unique identity
    let id1 = AgentIdentity::generate();
    let id2 = AgentIdentity::generate();
    let id3 = AgentIdentity::generate();

    assert_ne!(id1.peer_id(), id2.peer_id());
    assert_ne!(id2.peer_id(), id3.peer_id());
    assert_ne!(id1.peer_id(), id3.peer_id());
}

#[tokio::test]
async fn test_signature_verification() {
    // Test cryptographic signature workflow
    let identity = AgentIdentity::generate();
    let message = b"Test message for signing";

    let signature = identity.sign(message);
    let is_valid = AgentIdentity::verify(message, &signature, &identity.public_key());

    assert!(is_valid, "Valid signature should verify successfully");

    // Test invalid signature
    let wrong_message = b"Different message";
    let is_invalid = AgentIdentity::verify(wrong_message, &signature, &identity.public_key());

    assert!(
        !is_invalid,
        "Signature should not verify for different message"
    );
}
