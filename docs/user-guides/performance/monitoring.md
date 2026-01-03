# Monitoring and Profiling

*Part of Performance & Benchmarking Guide*

## Monitoring and Profiling

### Performance Monitoring

#### 1. Built-in Metrics
```rust
// Enable metrics collection
use p2p_ai_agents::metrics::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize metrics
    let metrics = MetricsCollector::new();
    metrics.start_collection().await?;
    
    // Your application code here
    
    // Export metrics
    metrics.export_to_prometheus().await?;
    Ok(())
}
```

#### 2. Custom Metrics
```rust
use metrics::{counter, gauge, histogram};

fn track_task_processing(task_type: &str, duration: Duration) {
    // Count tasks processed
    counter!("tasks_processed", 1, "type" => task_type);
    
    // Track processing time
    histogram!("task_duration", duration, "type" => task_type);
    
    // Monitor active tasks
    gauge!("active_tasks", get_active_task_count());
}
```

### Profiling Tools

#### 1. CPU Profiling
```bash
# Profile with perf
perf record --call-graph=dwarf ./target/release/p2p-ai-agents
perf report

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin p2p-ai-agents
```

#### 2. Memory Profiling
```bash
# Profile with valgrind
valgrind --tool=massif ./target/release/p2p-ai-agents
ms_print massif.out.* > memory-profile.txt

# Profile with heaptrack
heaptrack ./target/release/p2p-ai-agents
heaptrack_gui heaptrack.p2p-ai-agents.*
```

#### 3. Network Profiling
```bash
# Monitor network traffic
sudo netstat -i 1  # Interface statistics
sudo iftop          # Real-time bandwidth usage
sudo tcpdump -i any port 4001  # Capture P2P traffic
```

### Monitoring Dashboard

#### Grafana Setup
```yaml
# docker-compose.yml for monitoring stack
version: '3.8'
services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
  
  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

