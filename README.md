# Distributed Peer-to-Peer AI Agents

[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/p2p-ai-agents.svg)](https://crates.io/crates/p2p-ai-agents)
[![Build Status](https://img.shields.io/badge/Build-Passing-green.svg)](https://github.com/p2p-ai-agents/p2p-ai-agents)
[![Contributors](https://img.shields.io/badge/Contributors-Welcome-brightgreen.svg)](docs/CONTRIBUTING.md)
[![Rust Documentation](https://img.shields.io/badge/docs-rustdoc-blue.svg)](https://docs.rs/p2p-ai-agents)
[![Dependency Status](https://deps.rs/repo/github/p2p-ai-agents/p2p-ai-agents/status.svg)](https://deps.rs/repo/github/p2p-ai-agents/p2p-ai-agents)
[![Code Coverage](https://codecov.io/gh/p2p-ai-agents/p2p-ai-agents/branch/main/graph/badge.svg)](https://codecov.io/gh/p2p-ai-agents/p2p-ai-agents)

> **⚠️ WORK IN PROGRESS DISCLAIMER**
> 
> This project is currently in early development and serves as a foundational boilerplate for building distributed peer-to-peer AI agents. The current implementation provides basic structure and components, but many features are still being developed. This is not yet ready for production use.
> 
> We welcome contributions and feedback as we work toward a fully functional system!

## Vision

This project aims to democratize AI by building a distributed, peer-to-peer (P2P) network of lightweight AI agents. Anyone can contribute their idle compute (PC, server, Raspberry Pi, etc.) to help process, chunk, and retrieve data—reducing the need for centralized, energy-intensive datacenters and making AI accessible to all.

**🌟 Join the movement to build a sustainable, decentralized AI future!**

## Quick Links

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
   - [Implementation Docs](docs/implementation/)

4. [Documentation](docs/)
   - [Documentation Index](docs/INDEX.md)
   - [Quick Reference](docs/QUICK_REFERENCE.md)
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

*Let's build a greener, open, and decentralized AI future—together!*