# Agent Protocol Specification

## Overview

This document defines the comprehensive communication protocol used by agents in the p2p-agent network. The protocol enables secure, efficient, and reliable peer-to-peer communication for distributed AI processing tasks.

### Protocol Design Goals

- **Security**: All communications are authenticated and encrypted
- **Efficiency**: Minimal overhead for high-performance processing
- **Reliability**: Built-in fault tolerance and message delivery guarantees
- **Extensibility**: Forward-compatible design for future enhancements
- **Interoperability**: Compatible with multiple transport layers and data formats

## 1. Agent Discovery and Registration

### 1.1 Network Bootstrap

**Initial Connection**
Agents join the network through one of these methods:

1. **Bootstrap Nodes**: Connect to known reliable peers
2. **Multicast Discovery**: Local network peer discovery
3. **DHT Lookup**: Distributed hash table peer resolution
4. **Peer Introduction**: Existing peers introduce new connections

**Bootstrap Message Format**
```json
{
  "type": "bootstrap_request",
  "version": "1.0",
  "agent_id": "ed25519:ABCDEFabcdef1234567890",
  "capabilities": ["processing", "storage", "vectorization"],
  "public_key": "base64_encoded_public_key",
  "network_info": {
    "listen_addresses": ["/ip4/192.168.1.100/tcp/8000"],
    "supported_protocols": ["libp2p", "websocket", "quic"]
  },
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

### 1.2 Capability Advertisement

Agents advertise their capabilities and current status to the network:

```json
{
  "type": "capability_advertisement",
  "agent_id": "ed25519:ABCDEFabcdef1234567890",
  "capabilities": {
    "processing": {
      "types": ["nlp", "chunking", "classification"],
      "max_concurrent_tasks": 4,
      "supported_models": ["spacy_sm", "bert_base"]
    },
    "storage": {
      "available_space_gb": 100,
      "access_patterns": ["read", "write", "cache"]
    },
    "compute": {
      "cpu_cores": 8,
      "memory_gb": 16,
      "gpu_available": false
    }
  },
  "reputation_score": 0.95,
  "uptime_percentage": 99.2,
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

## 2. Task Distribution Protocol

### 2.1 Task Request Format

**Standard Task Request**
```json
{
  "type": "task_request",
  "task_id": "uuid4_task_identifier", 
  "requester_id": "ed25519:requesting_agent_id",
  "task_definition": {
    "operation": "text_chunking",
    "priority": "normal",
    "deadline": 1640995800,
    "resource_requirements": {
      "cpu_cores": 2,
      "memory_mb": 512,
      "execution_time_estimate": 30
    },
    "input_data": {
      "type": "text",
      "size_bytes": 1048576,
      "checksum": "sha256:abc123...",
      "location": "ipfs://QmHash... or direct_payload"
    },
    "parameters": {
      "chunk_size": 1000,
      "overlap": 200,
      "language": "en"
    }
  },
  "callback_info": {
    "result_endpoint": "/ip4/192.168.1.100/tcp/8000/results",
    "progress_updates": true
  },
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

### 2.2 Task Response Protocol

**Task Acceptance**
```json
{
  "type": "task_response",
  "task_id": "uuid4_task_identifier",
  "responder_id": "ed25519:responding_agent_id",
  "status": "accepted",
  "estimated_completion": 1640995500,
  "allocated_resources": {
    "cpu_cores": 2,
    "memory_mb": 512
  },
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

**Task Rejection**
```json
{
  "type": "task_response", 
  "task_id": "uuid4_task_identifier",
  "responder_id": "ed25519:responding_agent_id",
  "status": "rejected",
  "reason": "insufficient_resources",
  "alternative_agents": ["ed25519:other_agent_1", "ed25519:other_agent_2"],
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

### 2.3 Task Execution Updates

**Progress Updates**
```json
{
  "type": "task_progress",
  "task_id": "uuid4_task_identifier",
  "executor_id": "ed25519:executing_agent_id",
  "progress_percentage": 45.5,
  "stage": "processing",
  "estimated_completion": 1640995450,
  "resource_usage": {
    "cpu_percent": 75,
    "memory_mb": 384
  },
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

### 2.4 Result Delivery

**Task Completion**
```json
{
  "type": "task_result",
  "task_id": "uuid4_task_identifier", 
  "executor_id": "ed25519:executing_agent_id",
  "status": "completed",
  "result_data": {
    "type": "chunked_text",
    "chunks_count": 42,
    "output_location": "ipfs://QmResultHash...",
    "checksum": "sha256:def456...",
    "metadata": {
      "processing_time_ms": 28500,
      "chunks_created": 42,
      "average_chunk_size": 985
    }
  },
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

## 3. Data Exchange Protocol

### 3.1 Data Transfer Methods

**Small Data (< 1MB)**: Direct inclusion in message payload
**Medium Data (1MB - 100MB)**: IPFS content addressing
**Large Data (> 100MB)**: Chunked transfer with progress tracking

### 3.2 Data Integrity

All data transfers include:
- SHA-256 checksums for integrity verification
- Digital signatures for authenticity
- Redundant storage for fault tolerance

## 4. Security and Authentication

### 4.1 Message Authentication

Every protocol message must include:
```json
{
  "signature": "base64_encoded_ed25519_signature",
  "timestamp": 1640995200,
  "nonce": "random_32_byte_string"
}
```

### 4.2 Encryption

**Transport Layer**: TLS 1.3 or libp2p security protocols
**Message Layer**: Optional end-to-end encryption for sensitive data
**Key Exchange**: Elliptic Curve Diffie-Hellman (ECDH)

## 5. Error Handling and Fault Tolerance

### 5.1 Error Response Format

```json
{
  "type": "error_response",
  "error_code": "TASK_EXECUTION_FAILED",
  "error_message": "Out of memory during processing",
  "original_message_id": "uuid4_original_message",
  "retry_possible": true,
  "suggested_action": "reduce_task_size",
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

### 5.2 Heartbeat and Health Monitoring

**Periodic Heartbeat**
```json
{
  "type": "heartbeat",
  "agent_id": "ed25519:agent_identifier",
  "status": "healthy",
  "active_tasks": 3,
  "available_capacity": 0.6,
  "uptime_seconds": 86400,
  "signature": "base64_encoded_signature",
  "timestamp": 1640995200
}
```

## 6. Protocol Extensions

### 6.1 Version Negotiation

Agents negotiate protocol versions during initial handshake:
```json
{
  "supported_versions": ["1.0", "1.1"],
  "preferred_version": "1.1",
  "required_features": ["encryption", "compression"],
  "optional_features": ["streaming", "batch_processing"]
}
```

### 6.2 Custom Message Types 

The protocol supports custom message types through extension mechanism:
```json
{
  "type": "ext:federated_learning_update",
  "extension_version": "1.0",
  "data": { /* extension-specific data */ }
}
```

## 7. Quality of Service

### 7.1 Priority Levels
- **Critical**: System health, security alerts
- **High**: Time-sensitive processing tasks
- **Normal**: Standard processing requests
- **Low**: Background maintenance, bulk operations

### 7.2 Rate Limiting
- Maximum messages per second: 100
- Maximum concurrent tasks: Based on agent capacity
- Burst allowance: 2x sustained rate for 10 seconds

---

## 📖 Related Documentation

- **[Documentation Index](index.md)** - Complete documentation overview
- **[README](readme.md)** - Project overview and getting started
- **[Quick Reference](quick-reference.md)** - Commands and configuration
- **[High Level Design](high-level-design.md)** - Technical architecture
- **[Contributing Guide](contributing.md)** - Development guidelines

---

## Implementation Notes

### Transport Layer Support
- **Primary**: libp2p with multiple transports
- **Fallback**: WebSocket with custom framing
- **Future**: QUIC for improved performance

### Message Serialization
- **Format**: JSON for human readability
- **Alternative**: Protocol Buffers for efficiency
- **Compression**: gzip for large messages

### Backward Compatibility
- Protocol versions are negotiated during handshake
- Older versions supported for transition period
- Feature flags enable gradual rollout of capabilities

---

*This protocol specification is versioned and will evolve. All changes will maintain backward compatibility within major versions.*

*Note: This protocol specification is continuously updated. Check the [documentation](https://p2p-agent.readthedocs.io/) for the latest protocol details.*
