# Distributed Tracing Foundation

## Task Information

**Task ID**: `arch-005-distributed-tracing`  
**Component**: Observability  
**Category**: Architectural Decision  
**Priority**: **MEDIUM** (Phase 2 Foundation)  
**Status**: TODO  
**Created**: 2026-01-02  
**Source**: Architecture Document - Pre-mortem Scenario 4  

## Description

Design distributed tracing foundation to enable debugging of tasks flowing across multiple peers. Pre-mortem identified "impossible to debug production issues" as critical failure scenario.

**Context from Architecture Doc:**
- No visibility into task flows across peers
- Logs from distributed peers cannot be correlated
- "Tasks never complete" issues take weeks to debug
- No end-to-end request tracking

## Acceptance Criteria

### Design Requirements
- [ ] **Correlation IDs** - Propagate through all P2P messages
  - UUID generation for new tasks
  - Propagation in message headers
  - Logging integration (include in all log entries)
- [ ] **OpenTelemetry integration** - Industry standard tracing
  - Span creation for task operations
  - Trace context propagation
  - Exporter configuration (Jaeger, Zipkin)
- [ ] **Structured logging** - Machine-parseable logs
  - JSON format with correlation IDs
  - Log levels: trace, debug, info, warn, error
  - Dynamic verbosity control
- [ ] **Optional log aggregation** - Peers opt-in to centralization
  - Log shipping configuration
  - Privacy-preserving (sanitize sensitive data)
  - Retention policies

### Documentation Deliverables
- [ ] Tracing architecture specification
- [ ] Correlation ID propagation flow diagram
- [ ] Logging standards and format
- [ ] OpenTelemetry setup guide
- [ ] Performance impact analysis

## Implementation Notes

**What to Trace:**
- Task submission → assignment → execution → result
- Peer connections and disconnections
- Storage operations
- Network protocol messages

**Feeds Into Implementation Tasks:**
- Core: Logging infrastructure
- Network: Message header modifications
- Monitoring: OpenTelemetry exporter

## References

- Architecture Document: Pre-mortem - Scenario 4
- OpenTelemetry Rust SDK
- Distributed tracing best practices
