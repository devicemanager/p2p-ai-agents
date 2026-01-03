# Event Bus Performance Architecture

## Task Information

**Task ID**: `arch-004-event-bus-performance`  
**Component**: Core / Events  
**Category**: Architectural Decision  
**Priority**: **HIGH** (Phase 2 Implementation)  
**Status**: TODO  
**Created**: 2026-01-02  
**Source**: Architecture Document - Pre-mortem Scenario 5  

## Description

Design event bus priority queues and backpressure mechanisms to prevent performance bottlenecks as network scales. Pre-mortem identified event bus consuming 60% CPU under load.

**Context from Architecture Doc:**
- Unbounded event queues allow unlimited backlog
- No event priorities - telemetry floods block critical events
- Every component subscribes to everything
- No backpressure or circuit breakers

## Acceptance Criteria

### Design Requirements
- [ ] **Event priority queues** - Three-tier priority system
  - CRITICAL: Security events, network failures, shutdown
  - NORMAL: Task operations, peer events
  - LOW: Telemetry, metrics, debug logging
- [ ] **Bounded channels** - Configurable capacity limits
  - Per-priority queue sizing
  - Backpressure policies: block, drop-oldest, circuit-break
- [ ] **Selective subscriptions** - Component event filtering
  - Subscription filtering by event type
  - Reduce cross-component chatter
- [ ] **Event batching** - Aggregate high-frequency events
  - Batch telemetry (100 events → 1 batch)
  - Flush on timeout or batch size
- [ ] **Circuit breakers** - Isolate slow consumers
  - Detect slow processing
  - Temporarily disconnect slow consumers
  - Alert on circuit break events

### Documentation Deliverables
- [ ] Event bus architecture diagram
- [ ] Priority assignment guidelines
- [ ] Configuration schema for queue sizing
- [ ] Performance benchmarks (throughput, latency)
- [ ] Monitoring requirements

## Implementation Notes

**Performance Targets:**
- Event processing: <1ms p95
- Queue depth: <100 events p95
- Dropped events: <0.1% under load

**Feeds Into Implementation Tasks:**
- Core: Event bus refactoring
- Core: Channel implementation
- Monitoring: Event bus metrics

## References

- Architecture Document: Pre-mortem - Scenario 5
- Architecture Document: Cross-Cutting Concerns - Event Bus Performance
