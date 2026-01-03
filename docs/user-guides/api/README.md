# API Reference Modules

This directory contains the modular API reference documentation for P2P AI Agents.

## Structure

The API documentation has been split into focused modules for better maintainability and AI model compatibility (all files under 500 lines).

### API Modules

1. **[core-agent.md](core-agent.md)** (173 lines)
   - Agent, AgentId, AgentConfig
   - Agent lifecycle and management
   - Identity and authentication

2. **[network.md](network.md)** (172 lines)
   - NetworkManager, NetworkConfig
   - Peer discovery and connection
   - Message protocols and transport

3. **[storage.md](storage.md)** (84 lines)
   - Storage trait and implementations
   - Backend configurations
   - Data persistence patterns

4. **[task-processing.md](task-processing.md)** (108 lines)
   - Task definition and scheduling
   - Distribution and execution
   - Result aggregation

5. **[configuration.md](configuration.md)** (68 lines)
   - Configuration loading
   - Environment variables
   - Validation

6. **[error-handling.md](error-handling.md)** (90 lines)
   - Error types and codes
   - Handling patterns
   - Debugging strategies

7. **[examples.md](examples.md)** (180 lines)
   - Complete working examples
   - Common use cases
   - Integration scenarios

8. **[integration-patterns.md](integration-patterns.md)** (133 lines)
   - Application patterns
   - Deployment strategies
   - Performance optimization

## Navigation

**Main Index:** [../api-reference.md](../api-reference.md)

## File Organization

- Each module focuses on a specific API area
- All files comply with the 500-line policy
- Cross-references maintained between modules
- Consistent code examples throughout

---

*Last updated: 2026-01-02*
