# Story 1-2: Configure Node

**ID**: 1-2
**Epic**: EP-001 (Node Foundation & Identity)
**Status**: In Progress
**Effort**: 3 story points
**Priority**: P0 (Foundational)

---

## User Story

**As a** node operator
**I want to** configure my node with sensible defaults and persist configuration
**So that** I can customize the node behavior without redeploying the entire system

---

## Requirements Alignment

### Functional Requirements
- **FR1.4**: Store Node Configuration with Defaults
- **FR1.6**: Implement Node Configuration Validation

### Non-Functional Requirements
- **NFR-Config**: Parsing completes in < 10ms
- **NFR-Safety**: Max config file size 10MB
- **NFR-Validation**: Validation fails for invalid ranges (ports, peers, memory)

---

## Acceptance Criteria

### Scenario 1: Default Configuration Generation
```gherkin
Given a node starts for the first time
When no configuration file exists
Then a default configuration is generated at ~/.p2p-ai-agents/config/config.yaml
And includes the following fields with defaults:
  - listen_port: 9000
  - max_peers: 32
  - storage_path: ~/.p2p-ai-agents/data
  - health_check_interval_secs: 30
  - max_memory_mb: 512
  - log_level: info
```

### Scenario 2: Loading Valid Configuration
```gherkin
Given a node finds an existing configuration file
When the file is valid YAML with required fields
Then the configuration is loaded
And overrides built-in defaults
```

### Scenario 3: Configuration Validation
```gherkin
Given a configuration file with invalid values
When the node attempts validation
Then specific validation errors are reported for:
  - listen_port not in range [1024, 65535]
  - max_peers not in range [1, 256]
  - max_memory_mb not in range [128, 16384]
And the node does not proceed with initialization
```

### Scenario 4: Bootstrap Overrides (CLI/Env)
```gherkin
Given environment variables are set (e.g., P2P_LISTEN_PORT=9001)
When the node initializes
Then environment variables override the configuration file
And CLI flags override environment variables
```

---

## Developer Guardrails

### Tech Stack
- **Language**: Rust 1.75.0+
- **Serialization**: `serde`, `serde_yaml`
- **Path Handling**: `dirs` crate for OS-agnostic paths
- **Validation**: Custom validation logic in `Config::validate()`

### Security
- **File Permissions**: Config file should be 0644 (readable by owner, readable by group/others is okay for config, unlike identity)
- **Input Sanitization**: Validate all numeric ranges and paths

---

## Technical Implementation Plan

### Phase 1: Enhance Config Struct
- Add missing fields to `Config` struct (`storage_path`, `health_check_interval_secs`, `max_memory_mb`)
- Implement `Default` trait with specified values

### Phase 2: Configuration Persistence
- Implement `Config::save_default_if_missing()`
- Ensure parent directories exist
- Use atomic write pattern (write to temp, then rename)

### Phase 3: Validation Logic
- Implement `Config::validate()` method
- Check ranges:
    - `listen_port`: 1024-65535
    - `max_peers`: 1-256
    - `max_memory_mb`: 128-16384
- Return descriptive `ConfigError::ValidationError`

### Phase 4: Integration
- Update `main.rs` to call `save_default_if_missing` before loading
- Call `validate()` after loading and merging overrides

---

## Testing Strategy
- **Unit Tests**:
    - Test default generation
    - Test validation failures (boundary testing)
    - Test override precedence (CLI > Env > File > Default)
- **Integration Tests**:
    - Full startup sequence with missing config (verifying creation)
    - Startup with invalid config (verifying failure)
