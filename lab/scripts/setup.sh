#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up lab environment...${NC}"

# Create lab directory structure
mkdir -p lab/{config,scripts,tests}
mkdir -p lab/config/keys

# Create test files
echo -e "${GREEN}Creating test files...${NC}"
touch lab/tests/mod.rs
touch lab/tests/discovery.rs
touch lab/tests/transport.rs

# Create basic configuration
echo -e "${GREEN}Creating basic configuration...${NC}"
cat > lab/config/agent.yaml << 'EOF'
# Basic agent configuration
agent:
  id: "agent1"
  type: "processing"  # One of: processing, vector, storage, coordinator, gateway
  resources:
    cpu: 1
    memory: "1GB"
    storage: "5GB"

network:
  listen_addr: "0.0.0.0:4001"
  external_addr: "127.0.0.1:4001"
  bootstrap_nodes: []  # Empty for initial node

security:
  key_path: "keys/agent1.key"  # Ed25519 keypair
  tls_enabled: true

protocols:
  discovery:
    enabled: true
    version: "0.1"
  transport:
    enabled: true
    types: ["tcp", "webrtc"]
EOF

# Create monitoring script
echo -e "${GREEN}Creating monitoring script...${NC}"
cat > lab/scripts/monitor.sh << 'EOF'
#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Monitoring network status...${NC}"

# Monitor network connections
echo -e "\n${GREEN}Active network connections:${NC}"
netstat -an | grep "400[1-3]" || echo "No active connections"

# Monitor process
echo -e "\n${GREEN}Agent process status:${NC}"
ps aux | grep "p2p-ai-agents" || echo "No agent processes running"

# Basic resource usage
echo -e "\n${GREEN}Resource usage:${NC}"
top -b -n 1 | grep "p2p-ai-agents" || echo "No resource usage data"
EOF

# Make scripts executable
chmod +x lab/scripts/*.sh

# Create a basic test runner
echo -e "${GREEN}Creating test runner...${NC}"
cat > lab/scripts/run_tests.sh << 'EOF'
#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running network tests...${NC}"

# Run all network tests
cargo test --test network -- --nocapture

# Run specific test categories
echo -e "\n${GREEN}Running discovery tests...${NC}"
cargo test --test network::discovery -- --nocapture

echo -e "\n${GREEN}Running transport tests...${NC}"
cargo test --test network::transport -- --nocapture

echo -e "\n${GREEN}All tests completed!${NC}"
EOF

chmod +x lab/scripts/run_tests.sh

# Create a basic test environment
echo -e "${GREEN}Creating test environment...${NC}"
cat > lab/tests/mod.rs << 'EOF'
use p2p_ai_agents::network;
use std::time::Duration;
use tokio::time;

mod discovery;
mod transport;

/// Common test utilities and fixtures
mod common {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Test configuration
    pub struct TestConfig {
        pub listen_addr: SocketAddr,
        pub bootstrap_nodes: Vec<SocketAddr>,
        pub agent_type: String,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                listen_addr: "127.0.0.1:0".parse().unwrap(),
                bootstrap_nodes: Vec::new(),
                agent_type: "processing".to_string(),
            }
        }
    }

    /// Test environment for network tests
    pub struct TestEnv {
        pub config: TestConfig,
        pub network_manager: Arc<Mutex<network::NetworkManager>>,
    }

    impl TestEnv {
        pub async fn new(config: TestConfig) -> Result<Self, network::NetworkError> {
            let network_manager = network::NetworkManager::new(config.clone()).await?;
            Ok(Self {
                config,
                network_manager: Arc::new(Mutex::new(network_manager)),
            })
        }

        pub async fn cleanup(&self) {
            // Cleanup resources
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{TestConfig, TestEnv};

    #[tokio::test]
    async fn test_basic_agent_setup() {
        let config = TestConfig::default();
        let env = TestEnv::new(config).await.expect("Failed to create test environment");
        
        // Verify network manager is initialized
        let manager = env.network_manager.lock().await;
        assert!(manager.is_initialized(), "Network manager should be initialized");
        
        env.cleanup().await;
    }
}
EOF

echo -e "${GREEN}Lab environment setup complete!${NC}"
echo -e "To run the tests, use: ${GREEN}./lab/scripts/run_tests.sh${NC}"
echo -e "To monitor the network, use: ${GREEN}./lab/scripts/monitor.sh${NC}" 