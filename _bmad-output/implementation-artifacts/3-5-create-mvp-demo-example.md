# Story 3.5: Create MVP Demo Example

**Story ID**: 3.5  
**Story Key**: 3-5-create-mvp-demo-example  
**Epic**: Week 3 - MVP Implementation  
**Owner**: Amelia (Dev)  
**Status**: not-started  
**Priority**: Critical  
**Effort**: 1 day  
**Created**: 2026-01-10  
**Dependencies**: Stories 3.2, 3.3, 3.4 (All components implemented)

---

## Story

**As a** potential user  
**I want** to run a simple demo that shows 2 agents discovering each other and exchanging a task  
**So that** I understand how the P2P AI network works

**Context**: Create `examples/mvp_demo.rs` that orchestrates all MVP components into an end-to-end demonstration.

**Blocks**: Story 4.1 (User Validation needs working demo)

---

## Acceptance Criteria

### AC1: Demo Executable
- [ ] `cargo run --example mvp_demo` runs successfully
- [ ] Demo completes in <10 seconds
- [ ] Demo works on macOS, Linux, Windows
- [ ] No panics or crashes

### AC2: Demo Flow
- [ ] Start 2 agents (Agent A and Agent B)
- [ ] Agent A and B discover each other via mDNS
- [ ] Agent A creates task with prompt "Translate 'hello' to French"
- [ ] Agent A sends task to Agent B over network
- [ ] Agent B executes task (mock inference)
- [ ] Agent B returns result to Agent A
- [ ] Agent A displays result

### AC3: Output Quality
- [ ] Clear console output with labeled sections
- [ ] Show agent PeerIDs
- [ ] Show discovery progress
- [ ] Show task creation, sending, execution, result
- [ ] Execution times displayed (discovery, task latency)
- [ ] Color output (optional but nice)

### AC4: Error Handling
- [ ] Timeout if discovery fails (5s)
- [ ] Timeout if task response fails (30s)
- [ ] Graceful shutdown on Ctrl+C
- [ ] Error messages clear and actionable

### AC5: Documentation
- [ ] Comments explain each step
- [ ] README section explains how to run demo
- [ ] Example output shown in docs/user-guides/QUICK_START.md

---

## Tasks/Subtasks

### Task 1: Design Demo Flow
- [ ] Sketch demo sequence (discovery → task → result)
- [ ] Define output format (ASCII art, emojis, labels)
- [ ] Plan error handling strategy
- [ ] Document demo structure

### Task 2: Create Demo Skeleton
- [ ] Create `examples/mvp_demo.rs`
- [ ] Set up Tokio runtime
- [ ] Add CLI argument parsing (optional: --agent-a, --agent-b)
- [ ] Add tracing initialization
- [ ] Test demo compiles

### Task 3: Implement Agent Initialization
- [ ] Create 2 AgentIdentity instances
- [ ] Create 2 P2PAgent instances
- [ ] Start both agents concurrently
- [ ] Print agent PeerIDs
- [ ] Test agents start successfully

### Task 4: Implement Discovery Phase
- [ ] Wait for agents to discover each other
- [ ] Timeout after 5 seconds if no discovery
- [ ] Print discovery success message
- [ ] Display discovery time
- [ ] Test discovery works locally

### Task 5: Implement Task Exchange
- [ ] Agent A creates task: "Translate 'hello' to French"
- [ ] Agent A sends task to Agent B
- [ ] Agent B receives task
- [ ] Agent B executes task (mock)
- [ ] Agent B sends result back to Agent A
- [ ] Agent A receives result
- [ ] Print entire flow with timestamps
- [ ] Test end-to-end task exchange

### Task 6: Polish Output
- [ ] Add section headers (=== PHASE 1: DISCOVERY ===)
- [ ] Add emojis for visual appeal (🚀 🔍 ✅)
- [ ] Format PeerIDs (truncate to 8 chars for readability)
- [ ] Add execution time summaries
- [ ] Test output readability

### Task 7: Error Handling & Graceful Shutdown
- [ ] Handle discovery timeout
- [ ] Handle task timeout
- [ ] Handle network errors
- [ ] Implement Ctrl+C handler (tokio::signal)
- [ ] Test error scenarios

### Task 8: Documentation
- [ ] Add file header comments explaining demo
- [ ] Inline comments for each phase
- [ ] Update docs/user-guides/QUICK_START.md with:
  - [ ] How to run demo
  - [ ] Expected output
  - [ ] Troubleshooting section
- [ ] Update README.md with demo quickstart

---

## Dev Notes

### Technical Context

**Architecture Reference**: `docs/architecture/mvp-architecture.md`

**From Week 3 Implementation Summary**:
```rust
// Target: examples/mvp_demo.rs (~450 LOC)
```

**Demo Pseudo-Code**:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    
    println!("🚀 P2P AI Agents MVP Demo");
    println!("=========================\n");
    
    // Phase 1: Initialize agents
    println!("=== PHASE 1: AGENT INITIALIZATION ===");
    let identity_a = AgentIdentity::generate();
    let identity_b = AgentIdentity::generate();
    println!("Agent A: {}", identity_a.peer_id());
    println!("Agent B: {}", identity_b.peer_id());
    
    let mut agent_a = P2PAgent::new(identity_a).await?;
    let mut agent_b = P2PAgent::new(identity_b).await?;
    
    tokio::spawn(async move { agent_a.start().await });
    tokio::spawn(async move { agent_b.start().await });
    
    // Phase 2: Discovery
    println!("\n=== PHASE 2: PEER DISCOVERY ===");
    let start = Instant::now();
    // Wait for discovery...
    println!("✅ Discovered peer in {:?}", start.elapsed());
    
    // Phase 3: Task Exchange
    println!("\n=== PHASE 3: TASK EXCHANGE ===");
    let task = Task::new("Translate 'hello' to French", agent_a_peer_id);
    println!("📋 Created task: {}", task.id);
    
    let result = agent_a.send_message(agent_b_peer_id, task).await?;
    println!("✅ Result: {}", result.result);
    
    println!("\n🎉 Demo complete!");
    Ok(())
}
```

### Implementation Guidance

1. **File Location**: `examples/mvp_demo.rs`
2. **Dependencies**: Uses all MVP modules (identity, network, task)
3. **Async Pattern**: Tokio runtime, spawn agents as tasks
4. **Output Style**: Clear, beginner-friendly, visual

### Performance Targets

- Total runtime: <10s (from AC1)
- Discovery: <5s (typical: 2-3s per spike)
- Task latency: <100ms (typical: 18ms per spike)

### Demo Output Example

```
🚀 P2P AI Agents MVP Demo
=========================

=== PHASE 1: AGENT INITIALIZATION ===
Agent A: 12D3KooW... (peer_id truncated)
Agent B: 12D3KooW... (peer_id truncated)

=== PHASE 2: PEER DISCOVERY ===
🔍 Searching for peers on local network...
✅ Discovered Agent B in 2.1s

=== PHASE 3: TASK EXCHANGE ===
📋 Created task: Translate 'hello' to French
📤 Sending task to Agent B...
⚙️  Agent B executing task...
📥 Received result in 145ms
✅ Result: "Bonjour (mock translation)"

=== SUMMARY ===
Discovery time: 2.1s
Task latency: 145ms
Total runtime: 3.2s

🎉 Demo complete!
```

### Code Quality

- Commented for clarity (beginner-friendly)
- No `unwrap()` - use `?` for errors
- Graceful error messages
- Clean shutdown

---

## Dev Agent Record

### Debug Log
*To be populated during implementation*

### Implementation Plan
*To be documented as tasks are executed*

### Completion Notes
*To be filled when story is complete*

---

## File List

### Files Created
*To be updated as implementation proceeds*

### Files Modified
*To be updated as implementation proceeds*

---

## Change Log

- **2026-01-10**: Story created (Amelia) - MVP demonstration bringing all components together

---

## Status

**Current**: not-started  
**Next**: ready-for-dev (after Stories 3.2-3.4 complete)  
**Goal**: review (demo runs successfully, output clear, docs updated)
