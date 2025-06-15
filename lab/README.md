# Local Supabase Performance Lab

This directory contains a complete local Supabase setup for performance testing the storage adapter implementation.

## 🚀 Quick Start

**One-command setup:**
```bash
./lab/scripts/setup_complete_lab.sh
```

This will:
1. Check all prerequisites (Docker, Rust, etc.)
2. Start local Supabase instance with PostgreSQL and PostgREST
3. Run initial performance tests
4. Show you all available commands and interfaces

## 📁 Directory Structure

```
lab/
├── scripts/
│   ├── setup_complete_lab.sh     # 🎯 One-command complete setup
│   ├── local_supabase.sh         # 🐳 Docker Supabase management
│   ├── supabase_perf_lab.sh      # 📊 Performance testing framework
│   └── supabase_demo.sh          # 🎮 Demo and visualization
├── docker/
│   ├── docker-compose.yml        # 🐳 Supabase services configuration
│   └── init/
│       └── 01-init-supabase.sql  # 🗄️ Database initialization
├── reports/                       # 📈 Generated performance reports
└── supabase-performance-lab.md   # 📚 Complete documentation
```

## 🎯 Usage Modes

### 1. Mock Mode (Default)
Tests the framework without real database connections:
```bash
./lab/scripts/supabase_perf_lab.sh --mock --quick
```

### 2. Local Mode (Recommended)
Tests against real local Supabase instance:
```bash
./lab/scripts/supabase_perf_lab.sh --local --full
```

### 3. Real Mode
Tests against remote Supabase project:
```bash
export SUPABASE_URL="https://your-project.supabase.co"
export SUPABASE_ANON_KEY="your-anon-key"
./lab/scripts/supabase_perf_lab.sh --real --full
```

## 🛠️ Management Commands

### Local Supabase Management
```bash
# Start local Supabase stack
./lab/scripts/local_supabase.sh start

# Check status of all services
./lab/scripts/local_supabase.sh status

# View logs
./lab/scripts/local_supabase.sh logs

# Stop services
./lab/scripts/local_supabase.sh stop

# Clean everything (removes data)
./lab/scripts/local_supabase.sh clean

# Open Supabase Studio in browser
./lab/scripts/local_supabase.sh studio

# Connect to PostgreSQL database
./lab/scripts/local_supabase.sh db
```

### Performance Testing
```bash
# Quick test (100 operations)
./lab/scripts/supabase_perf_lab.sh --local --quick

# Full test (1000 operations, all backends)
./lab/scripts/supabase_perf_lab.sh --local --full

# Stress test (10000 operations)
./lab/scripts/supabase_perf_lab.sh --local --stress
```

## 🌐 Web Interfaces

Once started, you can access:

- **Supabase Studio**: http://localhost:3001
  - Visual database management
  - SQL editor and query runner
  - Real-time data monitoring
  
- **PostgREST API**: http://localhost:3000
  - Direct REST API access
  - API documentation
  - Real-time API testing

- **PostgreSQL Database**: localhost:54322
  - Direct database connection
  - Username: `postgres`
  - Password: `postgres`

## 📊 Performance Testing Framework

The lab includes a comprehensive performance testing framework with:

### Test Types
- **Write Performance**: Measure insertion speed
- **Read Performance**: Measure query speed  
- **Delete Performance**: Measure deletion speed
- **Concurrent Performance**: Measure parallel operation handling

### Test Configurations
- **Quick**: 100 operations, 1KB data, 4 concurrent tasks
- **Full**: 1000 operations, 1KB data, 10 concurrent tasks
- **Stress**: 10,000 operations, 4KB data, 20 concurrent tasks

### Metrics Collected
- Operations per second (throughput)
- Average latency per operation
- Total test duration
- Error rates and retry statistics

## 🔧 Configuration

### Local Environment Variables
```bash
SUPABASE_URL=http://localhost:3000
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU
```

### Docker Services
- **PostgreSQL**: Port 54322 (external), 5432 (internal)
- **PostgREST**: Port 3000
- **Supabase Studio**: Port 3001
- **Supabase Auth**: Port 9999
- **Postgres Meta**: Port 8080

## 🚨 Troubleshooting

### Common Issues

**Docker not running:**
```bash
# Start Docker daemon
sudo systemctl start docker    # Linux
# or open Docker Desktop       # Windows/Mac
```

**Port conflicts:**
```bash
# Check what's using ports
lsof -i :3000,3001,54322,9999,8080

# Stop conflicting services or change ports in docker-compose.yml
```

**Database connection issues:**
```bash
# Check database health
./lab/scripts/local_supabase.sh status

# View database logs
./lab/scripts/local_supabase.sh logs supabase-lab-db

# Restart services
./lab/scripts/local_supabase.sh restart
```

**Performance test failures:**
```bash
# Check if services are running
curl http://localhost:3000/

# Test database access
./lab/scripts/local_supabase.sh test

# Run with debug output
RUST_LOG=debug cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture
```

### Clean Restart
If things get messed up, clean restart:
```bash
./lab/scripts/local_supabase.sh clean
./lab/scripts/setup_complete_lab.sh
```

## 📈 Expected Performance

### Local Supabase Performance
- **Write**: 50-200 ops/sec
- **Read**: 100-500 ops/sec  
- **Delete**: 50-200 ops/sec
- **Concurrent**: 150-600 ops/sec

Performance depends on:
- System resources (CPU, memory, disk)
- Docker overhead
- Network stack (localhost)
- Database configuration

### Comparison with Other Backends
- **LocalStorage**: 500K-900K ops/sec (in-memory)
- **Network-based**: 50-500 ops/sec (network latency)

## 🎯 Development Workflow

1. **Start Lab**: `./lab/scripts/setup_complete_lab.sh`
2. **Develop**: Make changes to storage adapter
3. **Test**: `./lab/scripts/supabase_perf_lab.sh --local --quick`
4. **Debug**: Use Supabase Studio to inspect data
5. **Benchmark**: `./lab/scripts/supabase_perf_lab.sh --local --full`
6. **Analyze**: Check reports in `lab/reports/`

## 📚 Further Reading

- [Supabase Documentation](https://supabase.com/docs)
- [PostgREST API Reference](https://postgrest.org/en/stable/)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Docker Compose Reference](https://docs.docker.com/compose/)

## 🤝 Contributing

When adding new performance tests:

1. Add test functions to `tests/storage_perf.rs`
2. Update lab scripts as needed
3. Document expected performance characteristics
4. Test in all modes (mock, local, real)
5. Update this README with new features

## 📄 License

This lab setup is part of the p2p-ai-agents project and follows the same license terms.
