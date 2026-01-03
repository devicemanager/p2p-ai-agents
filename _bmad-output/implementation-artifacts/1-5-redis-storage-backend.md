# Story 1.5: Redis Storage Backend

Status: ready-for-dev

## Story

As a node operator,
I want my agent to support Redis as a storage backend,
So that I can use shared storage for multi-agent deployments.

## Acceptance Criteria

**AC1: Redis Connection Initialization**
```
Given: Redis connection URL is configured
When: the Redis storage backend initializes
Then: a connection pool is established
And: a ping command verifies connectivity
And: initialization returns Ok(())
```

**AC2: Strong Consistency Write Operations**
```
Given: Redis storage backend with Strong consistency
When: writing data with key "task_queue"
Then: the data is written to Redis with SET command
And: the operation waits for Redis acknowledgment
And: returns Ok(())
```

**AC3: Read Operations with Deserialization**
```
Given: data exists in Redis for key "task_queue"
When: reading data with key "task_queue"
Then: the data is retrieved with GET command
And: deserialized from JSON
And: returns Ok(data)
```

**AC4: Connection Failure and Retry Logic**
```
Given: Redis connection fails
When: attempting to perform any storage operation
Then: the operation retries up to 3 times with exponential backoff
And: if all retries fail, returns Err(StorageError::ConnectionFailed)
And: logs "Redis connection failed after 3 retries"
```

**AC5: Eventual Consistency Support**
```
Given: Redis storage with Eventual consistency
When: writing data
Then: the operation returns immediately after Redis acknowledgment
And: no additional consistency guarantees are enforced
```

## Implementation Plan

### Files to Create/Modify

1. **Create: `src/storage/redis.rs`**
   - RedisStorage struct
   - Connection pool management
   - Storage trait implementation
   - Retry logic with exponential backoff

2. **Modify: `src/storage/mod.rs`**
   - Add redis module
   - Export RedisStorage
   - Add redis feature flag

3. **Modify: `Cargo.toml`**
   - Add redis dependency
   - Add storage-redis feature

4. **Create: Tests in `src/storage/redis.rs`**
   - Connection initialization tests
   - CRUD operation tests
   - Retry logic tests
   - Consistency level tests

### Technical Details

**Dependencies:**
```toml
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
```

**RedisStorage Structure:**
```rust
pub struct RedisStorage {
    connection: ConnectionManager,
    config: RedisConfig,
}

pub struct RedisConfig {
    pub url: String,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}
```

**Key Methods:**
- `new(config: RedisConfig) -> Result<Self, StorageError>`
- `ping() -> Result<(), StorageError>`
- `with_retry<F, T>(&self, operation: F) -> Result<T, StorageError>`

### Testing Strategy

1. **Unit Tests:**
   - Mock Redis connection for unit tests
   - Test serialization/deserialization
   - Test retry logic

2. **Integration Tests:**
   - Require running Redis instance (docker-compose)
   - Test actual Redis operations
   - Test connection failure scenarios

3. **Test Coverage Target:** 95%

## Dependencies

- Story 1.3: Basic Local Storage Backend (done)
- Story 1.4: Storage Consistency Model (done)

## Notes

- Use `redis::aio::ConnectionManager` for automatic reconnection
- Serialize all values as JSON for consistency with LocalStorage
- Add docker-compose.yml entry for Redis testing
- Document Redis connection URL format in config/agent.example.yaml
