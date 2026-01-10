# Story 3.4: Implement Task Management

**Story ID**: 3.4  
**Story Key**: 3-4-implement-task-management  
**Epic**: Week 3 - MVP Implementation  
**Owner**: Amelia (Dev)  
**Status**: not-started  
**Priority**: High  
**Effort**: 1 day  
**Created**: 2026-01-10  
**Dependencies**: Story 3.3 (Network Layer)

---

## Story

**As a** P2P agent  
**I want** to create, execute, and track tasks  
**So that** I can demonstrate distributed task processing

**Context**: Implement task creation, mock execution, and result tracking per ADR-002 (Mock Inference).

**Blocks**: Story 3.5 (MVP Demo needs task management)

---

## Acceptance Criteria

### AC1: Task Creation
- [ ] Create task with UUID, prompt text, sender PeerId
- [ ] Task serializable to JSON
- [ ] Task includes timestamp and priority (default: 5)

### AC2: Mock Execution
- [ ] Execute task returns mock result after delay
- [ ] Execution delay: 100-500ms (simulates inference)
- [ ] Result format: `{"task_id": "...", "result": "Mock result for: <prompt>", "duration_ms": 123}`
- [ ] Execution logged with tracing

### AC3: Task Tracking
- [ ] Track task status (Pending, Executing, Complete, Failed)
- [ ] Store task results in HashMap
- [ ] Query task by UUID
- [ ] List all tasks

### AC4: Testing
- [ ] Unit test: Task creation
- [ ] Unit test: Mock execution
- [ ] Unit test: Status transitions
- [ ] Integration test: Full task lifecycle
- [ ] Coverage: 95%+

---

## Tasks/Subtasks

### Task 1: Design Task API
- [ ] Define `Task` struct (id, prompt, sender, timestamp, priority)
- [ ] Define `TaskResult` struct (task_id, result, duration_ms)
- [ ] Define `TaskManager` struct
- [ ] Define public API (create_task, execute_task, get_result, list_tasks)
- [ ] Document module structure

### Task 2: Implement Task Model
- [ ] Create `src/task/mod.rs`
- [ ] Define `Task` struct with derives (Serialize, Deserialize, Clone, Debug)
- [ ] Implement `Task::new()` constructor
- [ ] Add UUID generation (uuid crate)
- [ ] Write test for task creation
- [ ] Verify test passes

### Task 3: Implement Mock Executor
- [ ] Create `src/task/executor.rs`
- [ ] Define `TaskExecutor` struct
- [ ] Implement `execute()` method (async)
  - [ ] Random delay: 100-500ms (tokio::time::sleep)
  - [ ] Generate mock result
  - [ ] Return TaskResult
- [ ] Write test for execution
- [ ] Verify test passes

### Task 4: Implement Task Manager
- [ ] Create `src/task/manager.rs`
- [ ] Define `TaskManager` struct with HashMap storage
- [ ] Implement `create_task()` method
- [ ] Implement `execute_task()` method
- [ ] Implement `get_result()` method
- [ ] Implement `list_tasks()` method
- [ ] Write tests for all methods
- [ ] Verify all tests pass

### Task 5: Integration Testing
- [ ] Test: Create task → Execute → Get result
- [ ] Test: Multiple tasks executed concurrently
- [ ] Test: Task status transitions
- [ ] Verify task tracking accuracy
- [ ] Verify coverage ≥95%

### Task 6: Documentation
- [ ] Add module-level docs
- [ ] Document all public structs/methods
- [ ] Add usage examples
- [ ] Update lib.rs exports

---

## Dev Notes

### Technical Context

**Architecture Reference**: `docs/architecture/mvp-architecture.md`

**From Week 3 Implementation Summary**:
```rust
// Target structure (~180 LOC total)
src/task/
├── mod.rs           # Public exports
├── manager.rs       # TaskManager (100 LOC)
└── executor.rs      # TaskExecutor (80 LOC)
```

**Target API**:
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: Uuid,
    pub prompt: String,
    pub sender: PeerId,
    pub timestamp: u64,
    pub priority: u8,
}

pub struct TaskManager {
    tasks: HashMap<Uuid, Task>,
    results: HashMap<Uuid, TaskResult>,
}

impl TaskManager {
    pub fn new() -> Self;
    pub fn create_task(&mut self, prompt: String, sender: PeerId) -> Task;
    pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult>;
    pub fn get_result(&self, task_id: Uuid) -> Option<&TaskResult>;
    pub fn list_tasks(&self) -> Vec<&Task>;
}
```

**Dependencies**:
```toml
uuid = { version = "1", features = ["v4"] }  # Already in Cargo.toml
serde = { version = "1", features = ["derive"] }  # Already in Cargo.toml
```

### ADR References

**ADR-002: Mock Inference**
- No real ML models for MVP
- Random delay simulates inference time
- Mock result demonstrates end-to-end flow
- Allows testing without GPU/model overhead

**ADR-004: Request/Response**
- Task sent as request
- TaskResult returned as response
- JSON serialization for simplicity

### Implementation Guidance

1. **Module Location**: `src/task/` (may already exist, verify structure)
2. **Test Location**: Inline tests in each file
3. **Async Pattern**: Use `async fn` for execute, Tokio runtime
4. **Error Handling**: Custom `TaskError` enum (optional for MVP)

### Red-Green-Refactor

**Each task**:
1. **RED**: Write failing test
2. **GREEN**: Minimal impl
3. **REFACTOR**: Improve while tests stay green

### Performance Targets

- Task creation: <1ms
- Mock execution: 100-500ms (intentional delay)
- Result retrieval: <1ms

### Code Quality

- All public APIs documented
- No `unwrap()` in production
- Use `#[tracing::instrument]` on async methods
- Follow project conventions

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

- **2026-01-10**: Story created (Amelia) - Task management for distributed processing

---

## Status

**Current**: not-started  
**Next**: ready-for-dev (after Story 3.3 complete)  
**Goal**: review (when all ACs satisfied, tests pass, coverage ≥95%)
