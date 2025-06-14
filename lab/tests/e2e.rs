//! End-to-end tests for the P2P AI Agents system.

use p2p_ai_agents::network;
use tokio::time;
use std::time::Duration;

#[tokio::test]
async fn test_end_to_end_agent_communication() {
    // Setup two agents with different configs
    let config1 = common::TestConfig {
        listen_addr: "127.0.0.1:4101".parse().unwrap(),
        bootstrap_nodes: vec![],
        agent_type: "processing".to_string(),
    };
    let config2 = common::TestConfig {
        listen_addr: "127.0.0.1:4102".parse().unwrap(),
        bootstrap_nodes: vec!["127.0.0.1:4101".parse().unwrap()],
        agent_type: "processing".to_string(),
    };
    let agent1 = common::TestEnv::new(config1).await.expect("Failed to create agent1");
    let agent2 = common::TestEnv::new(config2).await.expect("Failed to create agent2");

    // Start both agents
    {
        let mut m1 = agent1.network_manager.lock().await;
        m1.start().await.expect("Failed to start agent1");
    }
    {
        let mut m2 = agent2.network_manager.lock().await;
        m2.start().await.expect("Failed to start agent2");
    }

    // Wait for peer discovery
    time::sleep(Duration::from_millis(500)).await;

    // Check that agent2 discovered agent1
    {
        let m2 = agent2.network_manager.lock().await;
        let peers = m2.connected_peers().await.expect("Failed to get peers");
        assert!(peers.contains(&"127.0.0.1:4101".parse().unwrap()), "Agent2 should discover Agent1");
    }

    // Cleanup
    agent1.cleanup().await;
    agent2.cleanup().await;
}

#[tokio::test]
async fn test_end_to_end_task_processing() {
    // Setup an agent
    let config = common::TestConfig {
        listen_addr: "127.0.0.1:4201".parse().unwrap(),
        bootstrap_nodes: vec![],
        agent_type: "processing".to_string(),
    };
    let agent = common::TestEnv::new(config).await.expect("Failed to create agent");
    {
        let mut m = agent.network_manager.lock().await;
        m.start().await.expect("Failed to start agent");
    }
    // Simulate submitting a task and getting a result (stub)
    // Replace with real task submission and result check when available
    assert!(true, "Task processing E2E test placeholder");
    agent.cleanup().await;
}

#[tokio::test]
async fn test_end_to_end_multi_agent_network() {
    // Setup three agents in a mesh network
    let config1 = common::TestConfig {
        listen_addr: "127.0.0.1:4301".parse().unwrap(),
        bootstrap_nodes: vec![],
        agent_type: "processing".to_string(),
    };
    let config2 = common::TestConfig {
        listen_addr: "127.0.0.1:4302".parse().unwrap(),
        bootstrap_nodes: vec!["127.0.0.1:4301".parse().unwrap()],
        agent_type: "processing".to_string(),
    };
    let config3 = common::TestConfig {
        listen_addr: "127.0.0.1:4303".parse().unwrap(),
        bootstrap_nodes: vec!["127.0.0.1:4301".parse().unwrap(), "127.0.0.1:4302".parse().unwrap()],
        agent_type: "processing".to_string(),
    };
    let agent1 = common::TestEnv::new(config1).await.expect("Failed to create agent1");
    let agent2 = common::TestEnv::new(config2).await.expect("Failed to create agent2");
    let agent3 = common::TestEnv::new(config3).await.expect("Failed to create agent3");
    {
        let mut m1 = agent1.network_manager.lock().await;
        m1.start().await.expect("Failed to start agent1");
    }
    {
        let mut m2 = agent2.network_manager.lock().await;
        m2.start().await.expect("Failed to start agent2");
    }
    {
        let mut m3 = agent3.network_manager.lock().await;
        m3.start().await.expect("Failed to start agent3");
    }
    // Wait for full mesh discovery
    time::sleep(Duration::from_secs(1)).await;
    // Check that all agents see each other
    {
        let m1 = agent1.network_manager.lock().await;
        let peers1 = m1.connected_peers().await.expect("Failed to get peers for agent1");
        assert!(peers1.contains(&"127.0.0.1:4302".parse().unwrap()), "Agent1 should see Agent2");
        assert!(peers1.contains(&"127.0.0.1:4303".parse().unwrap()), "Agent1 should see Agent3");
    }
    {
        let m2 = agent2.network_manager.lock().await;
        let peers2 = m2.connected_peers().await.expect("Failed to get peers for agent2");
        assert!(peers2.contains(&"127.0.0.1:4301".parse().unwrap()), "Agent2 should see Agent1");
        assert!(peers2.contains(&"127.0.0.1:4303".parse().unwrap()), "Agent2 should see Agent3");
    }
    {
        let m3 = agent3.network_manager.lock().await;
        let peers3 = m3.connected_peers().await.expect("Failed to get peers for agent3");
        assert!(peers3.contains(&"127.0.0.1:4301".parse().unwrap()), "Agent3 should see Agent1");
        assert!(peers3.contains(&"127.0.0.1:4302".parse().unwrap()), "Agent3 should see Agent2");
    }
    agent1.cleanup().await;
    agent2.cleanup().await;
    agent3.cleanup().await;
}

#[tokio::test]
async fn test_end_to_end_agent_restart_and_reconnect() {
    // Setup two agents, restart one, and verify reconnection
    let config1 = common::TestConfig {
        listen_addr: "127.0.0.1:4401".parse().unwrap(),
        bootstrap_nodes: vec![],
        agent_type: "processing".to_string(),
    };
    let config2 = common::TestConfig {
        listen_addr: "127.0.0.1:4402".parse().unwrap(),
        bootstrap_nodes: vec!["127.0.0.1:4401".parse().unwrap()],
        agent_type: "processing".to_string(),
    };
    let agent1 = common::TestEnv::new(config1).await.expect("Failed to create agent1");
    let agent2 = common::TestEnv::new(config2).await.expect("Failed to create agent2");
    {
        let mut m1 = agent1.network_manager.lock().await;
        m1.start().await.expect("Failed to start agent1");
    }
    {
        let mut m2 = agent2.network_manager.lock().await;
        m2.start().await.expect("Failed to start agent2");
    }
    time::sleep(Duration::from_millis(500)).await;
    // Stop agent1
    {
        let mut m1 = agent1.network_manager.lock().await;
        m1.shutdown().await.expect("Failed to shutdown agent1");
    }
    time::sleep(Duration::from_millis(200)).await;
    // Restart agent1
    {
        let mut m1 = agent1.network_manager.lock().await;
        m1.start().await.expect("Failed to restart agent1");
    }
    time::sleep(Duration::from_millis(500)).await;
    // Check that agent2 sees agent1 again
    {
        let m2 = agent2.network_manager.lock().await;
        let peers = m2.connected_peers().await.expect("Failed to get peers");
        assert!(peers.contains(&"127.0.0.1:4401".parse().unwrap()), "Agent2 should reconnect to Agent1 after restart");
    }
    agent1.cleanup().await;
    agent2.cleanup().await;
}
