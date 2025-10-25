# Load Testing Framework

This document describes the load testing framework implemented for the P2P AI Agents core components.

## Overview

The load testing framework provides comprehensive performance testing capabilities for:

- **Access Control**: Authentication and authorization operations
- **Event System**: Event publishing and handling
- **Service Management**: Service registration and health checks
- **Container Operations**: Dependency injection resolution

## Load Test Results

Load tests produce detailed metrics including:

- **Operations per Second**: Throughput measurement
- **Latency Statistics**: Average, 95th percentile, and 99th percentile
- **Error Rate**: Percentage of failed operations
- **Duration**: Total test execution time

## Usage Examples

### Basic Load Test

```rust
use p2p_ai_agents::core::{LoadTestConfig, LoadTestRunner};

let config = LoadTestConfig {
    concurrency: 10,
    duration: std::time::Duration::from_secs(30),
    ..Default::default()
};

let runner = LoadTestRunner::new(config);

// Define the operation to test
let operation = || async {
    // Your operation here
    my_async_function().await?;
    Ok(())
};

let result = runner.run("my_load_test", operation).await?;
println!("Ops/sec: {:.2}", result.ops_per_second);
println!("Avg latency: {:.2}ms", result.avg_latency_ms);
```

### Access Control Load Testing

```rust
use p2p_ai_agents::core::load_tests::access_control_load_tests::*;

// Test authentication performance
let config = LoadTestConfig {
    concurrency: 50,
    duration: std::time::Duration::from_secs(60),
    collect_latency: true,
    ..Default::default()
};

let auth_result = test_authentication_load(config).await?;
println!("Authentication: {:.0} ops/sec", auth_result.ops_per_second);

// Test authorization performance
let authz_result = test_authorization_load(config).await?;
println!("Authorization: {:.0} ops/sec", authz_result.ops_per_second);
```

### Event System Load Testing

```rust
use p2p_ai_agents::core::load_tests::event_load_tests::*;

// Test event publishing performance
let config = LoadTestConfig {
    concurrency: 20,
    duration: std::time::Duration::from_secs(30),
    ..Default::default()
};

let event_result = test_event_publishing_load(config).await?;
println!("Event publishing: {:.0} ops/sec", event_result.ops_per_second);
```

### Service Management Load Testing

```rust
use p2p_ai_agents::core::load_tests::service_load_tests::*;

// Test service health check performance
let config = LoadTestConfig {
    concurrency: 10,
    duration: std::time::Duration::from_secs(20),
    ..Default::default()
};

let service_result = test_service_health_load(config).await?;
println!("Service health checks: {:.0} ops/sec", service_result.ops_per_second);
```

### Container Operations Load Testing

```rust
use p2p_ai_agents::core::load_tests::container_load_tests::*;

// Test dependency injection performance
let config = LoadTestConfig {
    concurrency: 25,
    duration: std::time::Duration::from_secs(15),
    ..Default::default()
};

let container_result = test_container_operations_load(config).await?;
println!("Container resolution: {:.0} ops/sec", container_result.ops_per_second);
```

## Configuration Options

### LoadTestConfig

```rust
pub struct LoadTestConfig {
    /// Number of concurrent workers
    pub concurrency: usize,
    /// Test duration
    pub duration: Duration,
    /// Operations per second target (0 = unlimited)
    pub target_ops_per_second: u64,
    /// Whether to collect latency statistics
    pub collect_latency: bool,
    /// Whether to monitor memory usage
    pub monitor_memory: bool,
}
```

### Default Configuration

```rust
let config = LoadTestConfig::default();
// concurrency: 10
// duration: 30 seconds
// target_ops_per_second: 0 (unlimited)
// collect_latency: true
// monitor_memory: false
```

## Performance Metrics

### Interpreting Results

- **Operations per Second**: Higher is better, indicates system throughput
- **Average Latency**: Lower is better, indicates responsiveness
- **95th/99th Percentile Latency**: Should be reasonable for user experience
- **Error Rate**: Should be 0% for stable systems

### Benchmarking Guidelines

1. **Warm-up Period**: Run tests for 5-10 seconds before measuring
2. **Steady State**: Ensure system is in steady state during measurement
3. **Multiple Runs**: Run tests multiple times and average results
4. **Resource Monitoring**: Monitor CPU, memory, and I/O during tests

## Custom Load Tests

### Creating Custom Test Operations

```rust
use p2p_ai_agents::core::{LoadTestConfig, LoadTestRunner};

// Define your custom operation
async fn custom_operation() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Your test logic here
    perform_business_operation().await?;
    Ok(())
}

let config = LoadTestConfig {
    concurrency: 100,
    duration: std::time::Duration::from_secs(120),
    collect_latency: true,
    ..Default::default()
};

let runner = LoadTestRunner::new(config);
let operation = || async { custom_operation().await };

let result = runner.run("custom_test", operation).await?;
```

### Stateful Operations

```rust
// For operations that need state between calls
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = Arc::new(AtomicUsize::new(0));

let operation = {
    let counter = counter.clone();
    move || {
        let counter = counter.clone();
        async move {
            let value = counter.fetch_add(1, Ordering::Relaxed);
            process_item(value).await?;
            Ok(())
        }
    }
};
```

## Integration with CI/CD

### Automated Load Testing

```bash
# Run load tests as part of CI
cargo test --test load_tests

# Run specific load test
cargo test test_access_control_authentication_load

# Run with custom configuration
export LOAD_TEST_CONCURRENCY=50
export LOAD_TEST_DURATION=60
cargo test load_tests
```

### Performance Regression Detection

```rust
// Store baseline performance metrics
const BASELINE_AUTH_OPS_PER_SEC: f64 = 1000.0;
const BASELINE_AUTH_LATENCY_MS: f64 = 5.0;

// In tests
#[test]
fn test_performance_regression() {
    let result = test_authentication_load(default_config()).await.unwrap();

    assert!(
        result.ops_per_second >= BASELINE_AUTH_OPS_PER_SEC * 0.9,
        "Performance regression: {:.0} ops/sec < {:.0} baseline",
        result.ops_per_second, BASELINE_AUTH_OPS_PER_SEC
    );

    assert!(
        result.avg_latency_ms <= BASELINE_AUTH_LATENCY_MS * 1.5,
        "Latency regression: {:.2}ms > {:.2}ms baseline",
        result.avg_latency_ms, BASELINE_AUTH_LATENCY_MS
    );
}
```

## Troubleshooting Load Tests

### Common Issues

#### High Error Rates
**Symptoms**: Many operations failing during load test
**Solutions**:
- Check system resources (CPU, memory)
- Verify operation logic handles concurrency
- Check for race conditions

#### Inconsistent Results
**Symptoms**: Test results vary significantly between runs
**Solutions**:
- Ensure consistent test environment
- Warm up the system before measuring
- Use longer test durations for stability

#### Memory Leaks
**Symptoms**: Memory usage grows during test
**Solutions**:
- Check for proper resource cleanup
- Use scoped operations where possible
- Monitor memory usage patterns

### Debugging Load Tests

```rust
// Enable detailed logging
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

// Add logging to operations
let operation = || async {
    let start = std::time::Instant::now();
    let result = my_operation().await;
    let duration = start.elapsed();

    match &result {
        Ok(_) => tracing::debug!("Operation succeeded in {:?}", duration),
        Err(e) => tracing::error!("Operation failed in {:?}: {}", duration, e),
    }

    result
};
```

## Best Practices

### Test Design
- **Realistic Scenarios**: Test with realistic data and operation patterns
- **Gradual Load Increase**: Start with low concurrency and increase gradually
- **Mixed Operations**: Include different operation types in tests
- **Failure Simulation**: Test system behavior under failure conditions

### Environment Setup
- **Isolated Testing**: Run tests in isolated environments
- **Resource Allocation**: Ensure adequate resources for test load
- **Network Conditions**: Test under various network conditions
- **Data Setup**: Use appropriate test data sizes

### Result Analysis
- **Statistical Significance**: Run tests multiple times for statistical validity
- **Comparative Analysis**: Compare results across different configurations
- **Trend Analysis**: Track performance over time
- **Bottleneck Identification**: Use profiling tools to identify bottlenecks

## Performance Benchmarks

### Expected Performance Ranges

| Component | Operations/sec | Avg Latency | Notes |
|-----------|----------------|-------------|-------|
| Authentication | 1,000 - 10,000 | < 10ms | Depends on auth complexity |
| Authorization | 5,000 - 50,000 | < 5ms | Fast permission checks |
| Event Publishing | 10,000 - 100,000 | < 2ms | Async event handling |
| Service Health | 1,000 - 5,000 | < 50ms | I/O bound operations |
| Container Resolution | 50,000 - 500,000 | < 1ms | In-memory operations |

*Note: Actual performance depends on hardware, configuration, and operation complexity*

## Future Enhancements

### Advanced Features
- **Distributed Load Testing**: Coordinate tests across multiple nodes
- **Real-time Monitoring**: Live performance dashboards during tests
- **Load Pattern Simulation**: Complex load patterns (spikes, gradual increases)
- **Resource Monitoring**: Detailed system resource tracking
- **Performance Profiling**: Integrated profiling during load tests

### Integration Points
- **CI/CD Integration**: Automated performance gates
- **Metrics Export**: Export results to monitoring systems
- **Comparative Reporting**: Compare performance across versions
- **Alerting**: Automatic alerts for performance regressions

This load testing framework provides a solid foundation for performance testing and optimization of the P2P AI Agents system, ensuring it can handle production workloads effectively.