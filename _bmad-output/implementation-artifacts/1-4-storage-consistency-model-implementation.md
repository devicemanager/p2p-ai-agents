# Story 1.4: Storage Consistency Model Implementation

Status: done

## Story

As a developer,
I want a storage trait with ConsistencyLevel enum (Strong, Eventual, ReadYourWrites, Causal),
So that I can choose appropriate consistency guarantees for different data types.

## Acceptance Criteria

**Given** the Storage trait is defined
**When** reviewing the trait definition
**Then** it includes async methods: get(), put(), delete()
**And** each method accepts a ConsistencyLevel parameter
**And** ConsistencyLevel enum has variants: Strong, Eventual, ReadYourWrites, Causal

**Given** local storage backend with Strong consistency
**When** writing data and immediately reading it
**Then** the read returns the written data
**And** no stale data is returned

**Given** a storage operation with Eventual consistency
**When** the operation is called
**Then** the backend acknowledges immediately after local write
**And** consistency is guaranteed only after async replication (if applicable)

**Given** local storage backend is used
**When** any ConsistencyLevel is specified
**Then** Strong consistency semantics are applied (local storage is always strongly consistent)
**And** the operation completes successfully

## Tasks / Subtasks

- [ ] Define ConsistencyLevel enum (AC: 1)
  - [ ] Create `ConsistencyLevel` enum with variants: Strong, Eventual, ReadYourWrites, Causal
  - [ ] Add documentation for each variant
  - [ ] Derive Clone, Copy, Debug, PartialEq, Eq
  - [ ] Add serde Serialize/Deserialize for configuration

- [ ] Update Storage trait with consistency parameter (AC: 1)
  - [ ] Add `consistency: ConsistencyLevel` parameter to `get()`
  - [ ] Add `consistency: ConsistencyLevel` parameter to `put()`
  - [ ] Add `consistency: ConsistencyLevel` parameter to `delete()`
  - [ ] Update trait documentation

- [ ] Update LocalStorage implementation (AC: 2, 4)
  - [ ] Update `get()` signature with consistency parameter
  - [ ] Update `put()` signature with consistency parameter
  - [ ] Update `delete()` signature with consistency parameter
  - [ ] Ignore consistency parameter (local storage always Strong)
  - [ ] Add comment explaining local storage semantics

- [ ] Update all storage backend implementations (AC: 4)
  - [ ] Update DistributedStorage stub
  - [ ] Update CacheStorage stub
  - [ ] Update CustomStorage stub
  - [ ] Update plugin system integration

- [ ] Update all tests (AC: 2, 4)
  - [ ] Update existing tests to pass ConsistencyLevel::Strong
  - [ ] Add test for Strong consistency semantics
  - [ ] Add test for all consistency levels accepted by LocalStorage
  - [ ] Update integration tests

- [ ] Add consistency level tests (AC: 2, 3)
  - [ ] Test Strong: write then immediate read returns same data
  - [ ] Test Eventual: operation completes (local storage always strong)
  - [ ] Test ReadYourWrites: verify semantics
  - [ ] Test Causal: verify semantics

## Dev Notes

### Architecture Compliance

**ADR-3: Storage Layer Architecture**
- Consistency levels for different storage backends
- Local storage always provides Strong consistency
- Distributed backends may support Eventual consistency
- Clear semantics for each consistency level

**Source:** `_bmad-output/planning-artifacts/architecture.md` Section ADR-003

### Current Implementation Analysis

**Existing Code:** `src/storage/local.rs` (Lines 19-29)

**What Exists:**
- ✅ Storage trait with `get()`, `put()`, `delete()` methods
- ✅ LocalStorage implementation with file-based persistence
- ✅ No consistency level parameters

**What Needs Implementation:**
1. **ConsistencyLevel Enum** - Define variants and documentation
2. **Storage Trait Update** - Add consistency parameter to all methods
3. **Backend Updates** - Update all storage backend implementations
4. **Test Updates** - Pass consistency levels in all tests

### Library & Framework Requirements

**Dependencies (Already in Cargo.toml):**
- ✅ `async-trait` - Async trait support
- ✅ `serde` - Serialization for consistency levels

**New Dependencies:**
- None required

### ConsistencyLevel Semantics

**Strong:**
- Read returns most recent write
- Write waits for full acknowledgment
- Linearizable operations
- Use case: Critical data (identity, configuration)

**Eventual:**
- Read may return stale data
- Write returns immediately after local write
- Consistency achieved eventually
- Use case: Metrics, logs, non-critical data

**ReadYourWrites:**
- Session-level consistency
- Read after write in same session returns latest
- May not see other sessions' writes immediately
- Use case: User-specific data

**Causal:**
- Respects causal relationships between operations
- If A causes B, all nodes see A before B
- Use case: Message ordering, event logs

### Implementation Guidance

**Step 1: Define ConsistencyLevel enum**
```rust
/// Consistency level for storage operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Strong consistency - read returns most recent write
    /// Linearizable operations, highest latency
    Strong,
    
    /// Eventual consistency - read may return stale data
    /// Write returns immediately, lowest latency
    Eventual,
    
    /// Read-your-writes consistency
    /// Session-level consistency guarantee
    ReadYourWrites,
    
    /// Causal consistency - respects causal relationships
    /// If A causes B, all nodes see A before B
    Causal,
}

impl Default for ConsistencyLevel {
    fn default() -> Self {
        ConsistencyLevel::Strong
    }
}
```

**Step 2: Update Storage trait**
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a value by key with specified consistency level
    async fn get(
        &self,
        key: &str,
        consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError>;
    
    /// Put a value by key with specified consistency level
    async fn put(
        &self,
        key: &str,
        value: Vec<u8>,
        consistency: ConsistencyLevel,
    ) -> Result<(), StorageError>;
    
    /// Delete a value by key with specified consistency level
    async fn delete(
        &self,
        key: &str,
        consistency: ConsistencyLevel,
    ) -> Result<(), StorageError>;
}
```

**Step 3: Update LocalStorage implementation**
```rust
#[async_trait]
impl Storage for LocalStorage {
    async fn get(
        &self,
        key: &str,
        _consistency: ConsistencyLevel,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        // Local storage always provides Strong consistency
        // Consistency parameter ignored
        let file_path = self.key_to_path(key)?;
        
        match fs::read(&file_path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e)),
        }
    }
    
    // Similar for put() and delete()
}
```

**Step 4: Update tests**
```rust
#[tokio::test]
async fn test_local_storage_strong_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let storage = LocalStorage::new(temp_dir.path()).unwrap();
    
    // Write with Strong consistency
    storage
        .put("key", b"value".to_vec(), ConsistencyLevel::Strong)
        .await
        .unwrap();
    
    // Read immediately with Strong consistency
    let value = storage
        .get("key", ConsistencyLevel::Strong)
        .await
        .unwrap();
    
    assert_eq!(value, Some(b"value".to_vec()));
}

#[tokio::test]
async fn test_local_storage_accepts_all_consistency_levels() {
    let temp_dir = TempDir::new().unwrap();
    let storage = LocalStorage::new(temp_dir.path()).unwrap();
    
    // Local storage accepts all consistency levels
    for level in &[
        ConsistencyLevel::Strong,
        ConsistencyLevel::Eventual,
        ConsistencyLevel::ReadYourWrites,
        ConsistencyLevel::Causal,
    ] {
        storage
            .put("key", b"value".to_vec(), *level)
            .await
            .unwrap();
        
        let value = storage.get("key", *level).await.unwrap();
        assert_eq!(value, Some(b"value".to_vec()));
    }
}
```

### Testing Requirements

**Unit Tests (in `src/storage/mod.rs`):**
1. Update all existing tests to pass `ConsistencyLevel::Strong`
2. NEW: `test_consistency_level_enum()` - Test enum properties
3. NEW: `test_local_storage_strong_consistency()` - Verify Strong semantics
4. NEW: `test_local_storage_accepts_all_levels()` - All levels work

**Integration Tests (tests/storage_perf.rs):**
1. Update all tests to use consistency levels
2. Test performance with different consistency levels (should be same for local)

**Coverage Target:** 90%+ (critical infrastructure)

### Security Notes

⚠️ **IMPORTANT:**
- Consistency levels affect data guarantees
- Strong should be default for critical data
- Document consistency guarantees clearly
- Future distributed backends must respect consistency levels

### References

- **PRD Requirements:** FR-3.2 (Storage Consistency) - `_bmad-output/planning-artifacts/prd.md`
- **Architecture:** ADR-3 (Storage Architecture) - `_bmad-output/planning-artifacts/architecture.md`
- **Epic Details:** Epic 1, Story 1.4 - `_bmad-output/planning-artifacts/epics.md`
- **Existing Implementation:** `src/storage/local.rs` Lines 19-29
- **Project Context:** `project-context.md` Storage Layer section

### Dependency on Previous Stories

**Required from Story 1.3:**
- ✅ Storage trait definition
- ✅ LocalStorage implementation
- ✅ Test infrastructure

**This story provides:**
- 📦 Consistency level abstraction for all storage backends
- 📦 Foundation for distributed storage (Story 1.5)
- 📦 Clear semantics for storage operations

## Dev Agent Record

### Agent Model Used

_To be filled by Dev agent_

### Debug Log References

_To be filled by Dev agent_

### Completion Notes List

_To be filled by Dev agent_

### File List

**Files to Modify:**
- `src/storage/local.rs` - Add ConsistencyLevel enum, update Storage trait and implementations
- `src/storage/mod.rs` - Update all tests to use consistency levels
- `tests/storage_perf.rs` - Update integration tests

**Files Created:**
- None (enhancements to existing modules)
