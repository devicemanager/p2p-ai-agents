# Lab Setup and Performance Testing

## Status: TODO
**Assigned to:** System  
**Priority:** Medium  
**Estimated effort:** 2-3 hours  
**Dependencies:** Supabase storage adapter implementation  

## Overview
Set up a comprehensive lab environment for performance testing storage adapters, with particular focus on benchmarking the new Supabase storage adapter against existing storage backends.

## Requirements
- [x] Create performance testing framework in `lab/tests/storage_perf.rs`
- [x] Add configurable performance test parameters
- [x] Support multiple storage adapter types (local, distributed, cache, custom, Supabase)
- [x] Create comprehensive benchmark suite with different workloads
- [x] Add lab configuration files
- [x] Create automated benchmark script
- [ ] Run performance tests and collect baseline metrics
- [ ] Document performance characteristics and recommendations
- [ ] Integrate with CI/CD for continuous performance monitoring

## Implementation Details

### Performance Test Framework
- **Location:** `lab/tests/storage_perf.rs`
- **Features:**
  - Sequential write/read/delete tests
  - Concurrent access testing
  - Configurable operation count, data size, and concurrency
  - Performance metrics collection (ops/sec, latency)
  - Comparative analysis between storage backends

### Test Categories
1. **Small Test:** 100 operations, 256 bytes, 4 concurrent tasks
2. **Medium Test:** 1000 operations, 1KB, 10 concurrent tasks  
3. **Large Test:** 10000 operations, 4KB, 20 concurrent tasks

### Storage Adapters Tested
- LocalStorage (in-memory HashMap)
- DistributedStorage (stub implementation)
- CacheStorage (stub implementation)
- CustomStorage (stub implementation)
- SupabaseStorage (real implementation with network I/O)

### Configuration
- **Config file:** `lab/config/storage_perf.env`
- **Environment variables:** SUPABASE_URL, SUPABASE_ANON_KEY, etc.
- **Test parameters:** operations count, data size, concurrency level

## Files Created/Modified
- `lab/tests/storage_perf.rs` - Performance testing framework
- `lab/tests/mod.rs` - Updated to include performance tests
- `lab/config/storage_perf.env` - Configuration template
- `lab/scripts/run_storage_perf.sh` - Automated benchmark script

## Test Commands
```bash
# Run the complete benchmark suite
./lab/scripts/run_storage_perf.sh

# Individual test categories
cargo test --features storage-supabase test_storage_performance_small -- --nocapture
cargo test --features storage-supabase test_storage_performance_medium -- --nocapture
cargo test --features storage-supabase test_storage_performance_large -- --ignored --nocapture

# Supabase-specific tests (requires configuration)
cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture

# Stress testing
cargo test --features storage-supabase test_local_storage_stress -- --nocapture
```

## Expected Performance Characteristics

### LocalStorage (Baseline)
- **Write:** Very fast (in-memory HashMap)
- **Read:** Very fast (in-memory HashMap)
- **Concurrent:** Good (Arc<RwLock<HashMap>>)

### SupabaseStorage
- **Write:** Network-bound, includes HTTP overhead
- **Read:** Network-bound, includes HTTP overhead
- **Concurrent:** Limited by HTTP client connection pooling
- **Retry logic:** Built-in retry with exponential backoff

## Success Criteria
- [ ] All performance tests run successfully
- [ ] Baseline metrics collected for all storage adapters
- [ ] Performance comparison report generated
- [ ] Supabase adapter shows reasonable performance for remote storage
- [ ] Concurrent access tests demonstrate thread safety
- [ ] Documentation updated with performance characteristics

## Testing
Run the lab benchmark suite to validate:
```bash
cd /workspaces/p2p-ai-agents
./lab/scripts/run_storage_perf.sh
```

## Notes
- Supabase tests require environment variables (SUPABASE_URL, SUPABASE_ANON_KEY)
- Large tests are marked as `#[ignore]` and must be explicitly run
- Performance results will vary based on network conditions for Supabase tests
- The framework is extensible for future storage adapter implementations
