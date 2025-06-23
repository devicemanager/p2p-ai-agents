# Supabase Storage Performance Lab

This lab provides comprehensive performance testing for the Supabase storage adapter implementation in the p2p-ai-agents project.

## Quick Start

```bash
# Run with mock configuration (demonstrates framework)
./lab/scripts/supabase_perf_lab.sh --mock --quick

# Run comprehensive test with mock configuration
./lab/scripts/supabase_perf_lab.sh --mock --full

# Set up real Supabase instance
./lab/scripts/supabase_perf_lab.sh --setup

# Run with real Supabase instance (after setup)
./lab/scripts/supabase_perf_lab.sh --real --full
```

## Lab Components

### 1. Performance Test Framework

Located in `tests/storage_perf.rs`, this framework provides:

- **Modular Testing**: Individual tests for read, write, delete, and concurrent operations
- **Configurable Load**: Adjustable operation counts, data sizes, and concurrency levels
- **Comprehensive Metrics**: Operations per second, average latency, total time
- **Multi-Backend Support**: Tests all storage adapters for comparison

### 2. Test Configurations

#### Quick Test (`--quick`)
- Operations: 100
- Data Size: 1KB
- Concurrency: 4 tasks
- Runtime: ~5-10 seconds

#### Full Test (`--full`)
- Operations: 1000
- Data Size: 1KB
- Concurrency: 10 tasks
- Runtime: ~30-60 seconds

#### Stress Test (`--stress`)
- Operations: 10,000
- Data Size: 4KB
- Concurrency: 20 tasks
- Runtime: ~5-10 minutes

### 3. Storage Adapters Tested

1. **LocalStorage**: In-memory HashMap storage
2. **DistributedStorage**: Placeholder implementation
3. **CacheStorage**: Placeholder implementation  
4. **CustomStorage**: Placeholder implementation
5. **SupabaseStorage**: Real Supabase database integration

## Environment Setup

### Mock Environment (Default)

The lab automatically configures mock environment variables to demonstrate the testing framework:

```bash
SUPABASE_URL="https://demo-project.supabase.co"
SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.demo-anon-key"
```

This allows testing the framework without requiring a real Supabase instance.

### Real Supabase Environment

For actual performance testing against Supabase:

1. **Create Supabase Project**
   - Sign up at https://supabase.com
   - Create a new project
   - Note your project URL and API keys

2. **Set Up Database Table**
   ```sql
   CREATE TABLE IF NOT EXISTS storage_perf_test (
       id TEXT PRIMARY KEY,
       data BYTEA NOT NULL,
       created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
   );
   ```

3. **Configure Environment**
   ```bash
   export SUPABASE_URL='https://your-project.supabase.co'
   export SUPABASE_ANON_KEY='your-anon-key-here'
   export SUPABASE_SERVICE_ROLE_KEY='your-service-role-key-here'
   export SUPABASE_BUCKET_NAME='storage-perf-test'
   ```

## Performance Metrics

The lab measures and reports:

### Primary Metrics
- **Operations per Second (ops/s)**: Throughput measurement
- **Average Latency (μs)**: Mean time per operation
- **Total Time (ms)**: Complete test duration
- **Operation Count**: Total operations completed

### Comparative Analysis
- Performance ranking across storage backends
- Best performer identification by operation type
- Latency distribution analysis

## Expected Results

### Mock Environment Results
When running with mock configuration:
- Tests demonstrate framework functionality
- SupabaseStorage tests will skip with "SUPABASE_URL not set" message
- Local storage will show excellent performance (in-memory)
- Framework overhead is minimal

### Real Environment Results  
When running against actual Supabase:
- Network latency significantly impacts performance
- Typical performance ranges:
  - Writes: 50-200 ops/s
  - Reads: 100-500 ops/s  
  - Deletes: 50-200 ops/s
- Performance varies by:
  - Network connectivity
  - Supabase region
  - Database load
  - Data size

## Lab Scripts

### Main Script: `supabase_perf_lab.sh`

Comprehensive lab runner with options:

```bash
# Show help
./lab/scripts/supabase_perf_lab.sh --help

# Mock testing
./lab/scripts/supabase_perf_lab.sh --mock --quick
./lab/scripts/supabase_perf_lab.sh --mock --full

# Real testing  
./lab/scripts/supabase_perf_lab.sh --real --quick
./lab/scripts/supabase_perf_lab.sh --real --full
./lab/scripts/supabase_perf_lab.sh --real --stress

# Setup help
./lab/scripts/supabase_perf_lab.sh --setup
```

### Manual Testing

You can also run tests directly:

```bash
# Build with Supabase feature
cargo build --features storage-supabase

# Run specific test
cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture

# Run all performance tests
cargo test --features storage-supabase storage_perf -- --nocapture

# Run with environment variables
SUPABASE_URL=https://your-project.supabase.co \
SUPABASE_ANON_KEY=your-key \
SUPABASE_BUCKET_NAME=storage-perf-test \
cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture
```

## Report Generation

The lab automatically generates performance reports in `lab/reports/`:

- Timestamped report files
- Configuration details
- Performance metrics summary
- Next steps recommendations

## Troubleshooting

### Common Issues

1. **"SUPABASE_URL not set" message**
   - Expected in mock mode
   - Set environment variables for real testing

2. **Build errors**
   - Ensure `storage-supabase` feature is enabled
   - Check Rust version compatibility

3. **Network timeouts**
   - Check Supabase project status
   - Verify network connectivity
   - Adjust timeout in configuration

4. **Permission errors**  
   - Verify API keys are correct
   - Check database table permissions
   - Ensure service role key if needed

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture
```

## Integration with YOLO Mode

The Supabase storage tests integrate with the YOLO mode validation pipeline:

```bash
# Run task with performance validation
./scripts/tasks.sh --yolo --strict implement-supabase-storage-adapter
```

This ensures performance tests are automatically executed as part of the development workflow.

## Continuous Integration

For CI/CD pipelines, use mock mode by default:

```yaml
- name: Run Supabase Performance Tests
  run: |
    ./lab/scripts/supabase_perf_lab.sh --mock --quick
```

For nightly or scheduled runs against real Supabase:

```yaml
- name: Run Real Supabase Performance Tests  
  env:
    SUPABASE_URL: ${{ secrets.SUPABASE_URL }}
    SUPABASE_ANON_KEY: ${{ secrets.SUPABASE_ANON_KEY }}
    SUPABASE_BUCKET_NAME: storage-perf-test
  run: |
    ./lab/scripts/supabase_perf_lab.sh --real --full
```

## Contributing

When adding new performance tests:

1. Add test functions to `tests/storage_perf.rs`
2. Update the `StoragePerfTest` struct if needed
3. Add new test configurations to lab script
4. Update this documentation
5. Ensure tests work in both mock and real modes

## Performance Optimization

Based on test results, consider:

1. **Connection Pooling**: Reuse HTTP connections
2. **Batch Operations**: Group multiple operations
3. **Compression**: Compress large data payloads
4. **Caching**: Cache frequently accessed data
5. **Async Optimization**: Tune concurrency levels

The lab provides the foundation for measuring the impact of these optimizations.
