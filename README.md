# Distributed Peer-to-Peer AI Agents

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/Python-3.8%2B-blue.svg)](https://python.org)
[![Build Status](https://img.shields.io/badge/Build-Passing-green.svg)](https://github.com/yourusername/p2p-agent)
[![Contributors](https://img.shields.io/badge/Contributors-Welcome-brightgreen.svg)](CONTRIBUTING.md)

## Vision

This project aims to democratize AI by building a distributed, peer-to-peer (P2P) network of lightweight AI agents. Anyone can contribute their idle compute (PC, server, Raspberry Pi, etc.) to help process, chunk, and retrieve data—reducing the need for centralized, energy-intensive datacenters and making AI accessible to all.

**🌟 Join the movement to build a sustainable, decentralized AI future!**

## Why P2P AI Agents?

### 🌱 **Sustainability**
- **Energy Efficient**: Utilize existing, underutilized hardware instead of building new energy-intensive datacenters
- **Carbon Footprint**: Reduce CO2 emissions by leveraging distributed computing resources
- **Renewable Integration**: Smart scheduling to use renewable energy when available

### 🔒 **Privacy & Security**
- **Data Sovereignty**: Data stays local wherever possible, with federated and secure processing
- **End-to-End Encryption**: All communications are cryptographically secured
- **Zero-Knowledge Processing**: Agents can process data without accessing raw content

### 🌐 **Resilience & Decentralization**
- **No Single Point of Failure**: Network grows stronger as more people join
- **Fault Tolerance**: Automatic failover and task redistribution
- **Censorship Resistance**: Decentralized architecture prevents shutdown

### 🤝 **Accessibility & Inclusion**
- **Anyone Can Participate**: From Raspberry Pi to high-end servers
- **Economic Incentives**: Contributors can earn rewards for sharing resources
- **Global Collaboration**: Connect AI enthusiasts worldwide

## High-Level Architecture

```mermaid
flowchart TD
    subgraph User
        X[User Query]
    end
    subgraph LocalAgent["Local Agent (on your device)"]
        A1[Task Queue]
        A2[Chunking/NLP]
        A3[Vectorization]
        A4[Result Aggregation]
    end
    subgraph P2PNetwork["P2P AI Agent Network"]
        B1[Peer Agent 1]
        B2[Peer Agent 2]
        B3[Peer Agent N]
    end
    subgraph Storage["Local/Shared Storage"]
        S1[(Chunks & Metadata)]
    end
    X --> A1
    A1 -->|Distribute Tasks| A2
    A2 --> A3
    A3 --> S1
    A1 -- Collaborate --> B1
    A1 -- Collaborate --> B2
    A1 -- Collaborate --> B3
    B1 -- Share Results --> A4
    B2 -- Share Results --> A4
    B3 -- Share Results --> A4
    S1 --> A4
    A4 --> X
```

## Use Cases

### 🔍 **Distributed Document Processing**
Process large document collections across multiple devices for search, summarization, and analysis.

### 🧠 **Collaborative Machine Learning**
Train models collectively while keeping data private using federated learning techniques.

### 📊 **Distributed Data Analysis**
Analyze datasets too large for single machines by distributing computation across the network.

### 🌐 **Content Moderation**
Decentralized content filtering and moderation for social platforms and forums.

### 📚 **Knowledge Graph Construction**
Build comprehensive knowledge graphs from distributed data sources.

### 🔄 **Real-time Stream Processing**
Process continuous data streams from IoT devices and sensors.

## Architecture Examples

### Example 1: Document Search System
```
User Query → Local Agent → [Chunk, Vectorize, Search] → Peer Network → Results
```

### Example 2: Federated Learning
```
Local Data → Training Agent → Model Updates → Aggregator → Global Model
```

### Example 3: Distributed Storage
```
Large File → Chunking Agent → Storage Peers → Redundant Copies → Retrieval
```

## Features

### 🚀 **Core Capabilities**

- **Lightweight Agents**: Run efficiently on laptops, desktops, Raspberry Pis, and edge devices
- **Intelligent Peer Discovery**: Agents find each other automatically using libp2p/IPFS protocols
- **Dynamic Task Distribution**: Smart load balancing across available peers based on capabilities
- **Federated Learning**: Share model updates without exposing raw data
- **Energy-Aware Scheduling**: Optimize processing during low-cost or renewable energy periods

### 🔧 **Technical Features**

- **Multi-Protocol Support**: WebSocket, libp2p, IPFS integration
- **Cryptographic Security**: Ed25519 signatures and encrypted communications
- **Fault Tolerance**: Automatic peer failure detection and task redistribution
- **Reputation System**: Trust-based peer scoring for reliable collaboration
- **Resource Monitoring**: Real-time CPU, memory, and network usage tracking
- **Modular Architecture**: Plugin-based system for easy extension

### 🎯 **Specialized Agent Types**

- **Processing Agents**: Handle chunking, NLP, and data transformation
- **Vector Agents**: Specialized in embeddings and similarity search
- **Storage Agents**: Provide distributed data storage and retrieval
- **Coordinator Agents**: Manage task orchestration and result aggregation
- **Gateway Agents**: Bridge between P2P network and external APIs

## Quick Start

### Prerequisites

- Rust 1.70 or later
- Cargo (Rust's package manager)
- Git

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/p2p-ai-agents/p2p-ai-agents.git
   cd p2p-ai-agents
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the tests:
   ```bash
   cargo test
   ```

### Quick Start

1. Create a new agent:
   ```rust
   use p2p_ai_agents::prelude::*;

   #[tokio::main]
   async fn main() -> Result<()> {
       // Create agent configuration
       let config = AgentConfig {
           id: AgentId("my-agent".to_string()),
           network: NetworkConfig::default(),
           storage: StorageConfig::default(),
           resource_limits: ResourceLimits {
               max_cpu: 0.8,
               max_memory: 1024 * 1024 * 1024, // 1GB
               max_storage: 10 * 1024 * 1024 * 1024, // 10GB
               max_bandwidth: 1024 * 1024, // 1MB/s
           },
       };

       // Create and start agent
       let agent = DefaultAgent::new(config).await?;
       agent.start().await?;

       // Keep agent running
       tokio::signal::ctrl_c().await?;
       agent.stop().await?;
       Ok(())
   }
   ```

2. Submit a task:
   ```rust
   // Create task
   let task = Task::new(
       TaskPriority::Normal,
       TaskPayload {
           task_type: TaskType::TextProcessing,
           data: serde_json::json!({"text": "Hello, world!"}),
           parameters: HashMap::new(),
       },
   );

   // Submit task
   let task_id = agent.submit_task(task).await?;

   // Get task status
   let status = agent.task_status(&task_id).await?;
   println!("Task status: {:?}", status);
   ```

## Documentation

- [Architecture Overview](docs/architecture/system-overview.md)
- [Agent Protocol](AGENT_PROTOCOL.md)
- [API Reference](docs/api/README.md)
- [Contributing Guide](CONTRIBUTING.md)

## Project Structure

```
p2p-ai-agents/
├── src/
│   ├── agent/          # Agent implementation
│   ├── network/        # P2P networking
│   ├── storage/        # Storage management
│   └── cli/            # Command-line interface
├── examples/           # Example usage
├── tests/              # Integration tests
└── docs/              # Documentation
```

## Development Status

The project is currently in active development. See [ROADMAP.md](ROADMAP.md) for details on planned features and milestones.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [libp2p](https://libp2p.io/) for the networking foundation
- [tokio](https://tokio.rs/) for the async runtime
- [ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek) for cryptographic primitives

## Community & Support

### 💬 **Get Help**
- 📖 [Documentation](https://p2p-agent.readthedocs.io/)
- 💬 [Discord Community](https://discord.gg/p2p-agent)
- 🐛 [Report Issues](https://github.com/yourusername/p2p-agent/issues)
- 📧 [Email Support](mailto:support@p2p-agent.org)

### 🤝 **Community Resources**
- [Weekly Community Calls](https://calendar.google.com/p2p-agent-calls)
- [Developer Blog](https://blog.p2p-agent.org/)
- [Research Papers](https://research.p2p-agent.org/)
- [Twitter Updates](https://twitter.com/P2PAgent)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Open Source Commitment**: We believe in open, collaborative development. All core components will remain open source forever.

## Acknowledgments

- Thanks to the [libp2p](https://libp2p.io/) team for networking protocols
- Inspired by [IPFS](https://ipfs.io/) for distributed storage concepts  
- Built with amazing open source AI tools from [Hugging Face](https://huggingface.co/)
- Special thanks to all our [contributors](CONTRIBUTORS.md) and community members

## Security

For security vulnerabilities, please email [security@p2p-agent.org](mailto:security@p2p-agent.org) instead of using public issues.

## Citation

If you use this project in your research, please cite:

```bibtex
@software{p2p_agent,
  title={Distributed Peer-to-Peer AI Agents},
  author={P2P Agent Contributors},
  year={2025},
  url={https://github.com/yourusername/p2p-agent}
}
```

## MVP Peer-to-Peer Considerations

- **Security & Authentication:** Agents use cryptographic keypairs for identity and sign all messages. Communication is encrypted.
- **Fault Tolerance:** Agents detect failures and reassign or retry tasks as needed.
- **Message/Task Format:** All agent messages use a simple JSON structure with type, task_id, payload, sender_id, and signature fields.
- **Network Bootstrap:** New agents join via a known bootstrap node or multicast discovery, then find more peers.
- **Monitoring & Logging:** Agents log key events and errors for transparency and debugging.

---

*Let's build a greener, open, and decentralized AI future—together!*

---

## 📏 500-Line Limit Policy

**P2P Agent enforces a strict 500-line maximum for all files** to ensure compatibility with small language models and resource-constrained environments.

### Why 500 Lines?

- **🤖 AI Model Compatibility**: Small models can process files under 500 lines efficiently
- **💻 Resource Efficiency**: Lower memory footprint and faster processing
- **👥 Developer Experience**: Easier code review and better maintainability
- **🔄 Modularity**: Forces good architectural decisions and separation of concerns

### Enforcement

- **Automated checks** in pre-commit hooks and CI/CD
- **Code review requirements** for all pull requests
- **Refactoring triggers** when files approach 400 lines
- **Documentation splits** with table of contents approach

See the complete **[500-Line Limit Policy](docs/500_LINE_LIMIT_POLICY.md)** for detailed guidelines and implementation strategies.

---

## 📖 Related Documentation

- **[Documentation Index](docs/INDEX.md)** - Complete documentation overview
- **[Quick Reference](docs/QUICK_REFERENCE.md)** - Commands and configuration quick reference
- **[High Level Design](HIGH_LEVEL_DESIGN.md)** - Technical architecture
- **[Agent Protocol](AGENT_PROTOCOL.md)** - Communication protocol specification
- **[Contributing Guide](CONTRIBUTING.md)** - Development and contribution guidelines

---
