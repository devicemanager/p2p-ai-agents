# Distributed Expert Intelligence Network

[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/p2p-ai-agents.svg)](https://crates.io/crates/p2p-ai-agents)
[![Build Status](https://img.shields.io/badge/Build-Passing-green.svg)](https://github.com/p2p-ai-agents/p2p-ai-agents)
[![Contributors](https://img.shields.io/badge/Contributors-Welcome-brightgreen.svg)](docs/CONTRIBUTING.md)
[![Rust Documentation](https://img.shields.io/badge/docs-rustdoc-blue.svg)](https://docs.rs/p2p-ai-agents)
[![Dependency Status](https://deps.rs/repo/github/p2p-ai-agents/p2p-ai-agents/status.svg)](https://deps.rs/repo/github/p2p-ai-agents/p2p-ai-agents)
[![Code Coverage](https://codecov.io/gh/p2p-ai-agents/p2p-ai-agents/branch/main/graph/badge.svg)](https://codecov.io/gh/p2p-ai-agents/p2p-ai-agents)

> **⚠️ WORK IN PROGRESS - MAJOR VISION TRANSFORMATION**
> 
> **January 2026 Update**: This project is undergoing a strategic transformation from a distributed ML compute network to a **Distributed Expert System** architecture.
> 
> **New Vision**: Building a peer-to-peer network where domain experts contribute knowledge and reasoning rules, enabling transparent, explainable decision support across multiple domains. Unlike black-box AI, our system shows WHY and HOW it reaches conclusions.
> 
> **Recent Major Updates:**
> - ✅ **Vision Transformation**: Pivoting to expert system architecture (January 2026)
> - ✅ **Core Architecture**: Complete dependency injection, event system, service registry, and access control
> - ✅ **Security Framework**: RBAC authentication and authorization with pluggable providers
> - ✅ **P2P Foundation**: libp2p networking with peer discovery and secure messaging
> - ✅ **Structured Logging**: JSON logging with correlation IDs for distributed tracing
> - 🔲 **Inference Engine**: Rule-based reasoning system (in development)
> - 🔲 **Knowledge Base**: Expert knowledge repository (in development)
> 
> We welcome contributions, especially from domain experts interested in democratizing knowledge!

## Vision

**"Distributed Expert Intelligence Network: Peer-to-Peer Knowledge, Reasoning & Transparent Decision-Making"**

### What We're Building

This project transforms how expert knowledge is shared and applied by creating a decentralized network of expert nodes that collaboratively solve complex domain-specific problems through **transparent rule-based reasoning**.

**Key Differentiators:**
- 🧠 **Expert Knowledge, Not Just Compute**: Domain experts contribute specialized reasoning rules and knowledge
- 🔍 **Transparent Reasoning**: Every conclusion shows its reasoning path - no black boxes
- 🌐 **Decentralized Collaboration**: Multiple experts can consult on complex problems simultaneously
- ✅ **Explainable by Design**: Built-in "why" and "how" explanations for all decisions
- 🔐 **Trustworthy & Auditable**: Full reasoning traces for compliance and verification

**Target Use Cases:**
- 🏥 Medical diagnosis decision support
- ⚖️ Legal document analysis and case research
- 💰 Financial risk assessment and compliance
- 🔧 Technical troubleshooting and support
- 🎓 Intelligent tutoring and education

**🌟 Join us in democratizing expert knowledge and making AI reasoning transparent!**

## Architecture Overview

### Core Components

**Expert System Layer (New - In Development):**
- **Inference Engine**: Forward/backward chaining reasoning
- **Knowledge Base**: Domain-specific rules and facts
- **Expert Registry**: Credential verification and expertise matching
- **Query Router**: Intelligent routing to appropriate experts
- **Explanation Generator**: Transparent reasoning traces

**P2P Foundation (Complete):**
- **Network Layer**: libp2p-based peer discovery and messaging
- **Identity System**: Ed25519 cryptographic identity for nodes
- **Security**: RBAC authentication and authorization
- **Storage**: Pluggable storage backends (local, Redis, Supabase)

**Infrastructure (Complete):**
- **Event System**: Async event-driven architecture
- **Service Registry**: Dependency injection container
- **Observability**: Structured logging, metrics, tracing
- **CLI Tools**: Node management and monitoring

### How It Works

```
1. Expert Query → "Patient has fever, cough, shortness of breath"
2. Query Router → Identifies medical diagnosis domain
3. Expert Selection → Finds qualified medical expert nodes
4. Inference Engine → Applies diagnostic reasoning rules
5. Multi-Expert Consultation → Synthesizes opinions if needed
6. Result + Explanation → "Likely pneumonia because..."
   └─ Reasoning Trace: Shows which rules were applied and why
```

1. [Getting Started](docs/user-guides/getting-started.md)
   - [Agent Configuration](docs/user-guides/agent-configuration.md)
   - [Security Best Practices](docs/user-guides/security-best-practices.md)
   - [Troubleshooting](docs/user-guides/troubleshooting.md)

2. [Architecture](docs/architecture/)
   - [System Overview](docs/architecture/system-overview.md)
   - [Data Flow](docs/architecture/data-flow.md)
   - [Networking](docs/architecture/networking.md)
   - [Security](docs/architecture/security.md)

3. [Development](docs/development/)
   - [Development Guide](docs/development/README.md)
   - [Testing Guide](docs/development/testing-guide.md)
   - [Code Formatting](docs/code-formatting.md)
   - [Implementation Docs](docs/implementation/)

4. [Core Architecture](docs/core/)
   - [Access Control](docs/core/access-control.md)
   - [Load Testing](docs/core/load-testing.md)
   - [Structured Logging](docs/core/logging.md)

5. [Documentation](docs/)
   - [Documentation Index](docs/INDEX.md)
   - [Quick Reference](docs/QUICK-REFERENCE.md)
   - [Glossary](docs/GLOSSARY.md)
   - [CLI Documentation](docs/cli.md)

## Community & Support

- 📖 [Documentation](docs/)
- 💬 [Discord Community](https://discord.gg/p2p-ai-agents) *(coming soon)*
- 🐛 [Report Issues](https://github.com/p2p-ai-agents/p2p-ai-agents/issues)
- 📧 [Email Support](mailto:support@p2p-ai-agents.org) *(coming soon)*

## Development

### Getting Started with Development

1. **Prerequisites**
   ```bash
   # Install Rust (latest stable)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install development tools
   make install-tools
   ```

2. **Quick Development Commands**
   ```bash
   # Check compilation
   make check
   
   # Run tests
   make test
   
   # Format code
   make fmt
   
   # Run all CI checks locally
   make ci-check
   
   # Generate coverage report
   make coverage
   ```

3. **Task Management**
   ```bash
   # View implementation progress
   ./scripts/tasks.sh stats
   
   # Find tasks to work on
   ./scripts/tasks.sh search "network"
   ./scripts/tasks.sh list todo
   
   # Start working on a task
   ./scripts/tasks.sh start task-name.md
   
   # Complete a task
   ./scripts/tasks.sh complete task-name.md
   ```

4. **CI/CD Status**
   - ✅ **Rust CI**: Automated building, testing, and linting
   - ✅ **Documentation**: Automated quality checks and maintenance
   - ✅ **Code Coverage**: Automated coverage reporting via Codecov
   - ✅ **Issue Tracking**: Automated GitHub issue creation from implementation checklists

### Project Structure

See [development documentation](docs/development/) for detailed information about:
- Code organization
- Testing strategies
- Contributing guidelines
- Implementation tracking

## License

This project is licensed under either of:

- Apache License, Version 2.0 
- MIT License 

See [LICENSE](LICENSE) for details.

## Acknowledgments

- [libp2p](https://libp2p.io/) for the networking foundation
- [tokio](https://tokio.rs/) for the async runtime
- [ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek) for cryptographic primitives

---

*Let's democratize expert knowledge and make AI reasoning transparent—together!*

---

## 📄 Transformation Documentation

For details on the vision transformation from distributed ML compute to expert systems:
- [Vision Transformation Executive Summary](_bmad-output/vision-transformation-2026-01-09/EXECUTIVE_SUMMARY.md)
- [Impact Analysis & Roadmap](_bmad-output/vision-transformation-2026-01-09/)

