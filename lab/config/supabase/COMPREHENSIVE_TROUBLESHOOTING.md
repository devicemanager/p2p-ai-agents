# Comprehensive Supabase Storage Adapter Troubleshooting Guide

## Table of Contents
1. [Quick Start Checklist](#quick-start-checklist)
2. [Environment Setup](#environment-setup)
3. [Database Configuration](#database-configuration)
4. [PostgREST Configuration](#postgrest-configuration)
5. [Testing Procedures](#testing-procedures)
6. [Common Issues and Solutions](#common-issues-and-solutions)
7. [Performance Testing](#performance-testing)
8. [Clean State Recovery](#clean-state-recovery)
9. [Common Mistakes and How to Avoid Them](#common-mistakes-and-how-to-avoid-them)

## Quick Start Checklist

### ✅ Pre-Test Verification
- [ ] PostgreSQL service is running
- [ ] Database `supabase_lab` exists
- [ ] Required roles (anon, authenticated, service_role) exist
- [ ] Table `storage_kv` exists with correct schema
- [ ] PostgREST is running on port 8000
- [ ] Environment variables are set correctly
- [ ] API endpoints respond to health checks

### ✅ Test Execution
- [ ] Rust tests compile without errors
- [ ] Database connection successful
- [ ] CRUD operations work via API
- [ ] JWT authentication works
- [ ] Performance benchmarks complete

## Environment Setup

### Required Dependencies
```bash
# System packages
sudo apt-get update
sudo apt-get install -y postgresql postgresql-contrib curl jq

# Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Environment Variables
Create or verify `/workspaces/p2p-ai-agents/.env`:
```bash
# Core Supabase configuration
SUPABASE_URL=http://localhost:8000
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJhbm9uIiwKICAgICJpc3MiOiAic3VwYWJhc2UtZGVtbyIsCiAgICAiaWF0IjogMTY0MVc2OTIwMCwKICAgICJleHAiOiAxNzk5NTM1NjAwCn0.dc_X5iR_VP_qT0zsiyj_I_OZ2T9FtRU2BBNWN8Bu4GE
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJzZXJ2aWNlX3JvbGUiLAogICAgImlzcyI6ICJzdXBhYmFzZS1kZW1vIiwKICAgICJpYXQiOiAxNjQxNzY5MjAwLAogICAgImV4cCI6IDE3OTk1MzU2MDAKfQ.DaYlNEoUrrEn2Ig7tqibS-PHK5vgusbcbo7X36XVt4Q
SUPABASE_SCHEMA=public
SUPABASE_TABLE_NAME=storage_kv

# JWT Secret for PostgREST
JWT_SECRET=your-super-secret-jwt-token-with-at-least-32-characters-long
```

## Database Configuration

### PostgreSQL Setup Script
```bash
#!/bin/bash
# File: setup_database.sh

set -e

echo "🔧 Setting up PostgreSQL for Supabase..."

# Start PostgreSQL service
sudo service postgresql start

# Wait for PostgreSQL to be ready
sleep 2

# Create database if it doesn't exist
sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw supabase_lab || sudo -u postgres createdb supabase_lab

# Create roles and permissions
sudo -u postgres psql -d supabase_lab << EOF
-- Create roles if they don't exist
DO \$\$ 
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'service_role') THEN
        CREATE ROLE service_role NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticator') THEN
        CREATE ROLE authenticator NOINHERIT LOGIN PASSWORD 'your-super-secret-jwt-token-with-at-least-32-characters-long';
    END IF;
END
\$\$;

-- Grant role memberships
GRANT anon, authenticated, service_role TO authenticator;

-- Create storage_kv table if it doesn't exist
CREATE TABLE IF NOT EXISTS storage_kv (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Grant permissions
GRANT ALL ON storage_kv TO anon;
GRANT ALL ON storage_kv TO authenticated;
GRANT ALL ON storage_kv TO service_role;
GRANT ALL ON storage_kv TO authenticator;

-- Create RLS policies (Row Level Security)
ALTER TABLE storage_kv ENABLE ROW LEVEL SECURITY;

-- Allow all operations for service_role
CREATE POLICY IF NOT EXISTS "service_role_all" ON storage_kv
    FOR ALL USING (true);

-- Allow read/write for authenticated users
CREATE POLICY IF NOT EXISTS "authenticated_all" ON storage_kv
    FOR ALL TO authenticated USING (true);

-- Allow read for anonymous users
CREATE POLICY IF NOT EXISTS "anon_select" ON storage_kv
    FOR SELECT TO anon USING (true);

EOF

echo "✅ Database setup complete!"
```

### PostgREST Configuration

Create `/tmp/postgrest.conf`:
```bash
db-uri = "postgres://authenticator:your-super-secret-jwt-token-with-at-least-32-characters-long@localhost:5432/supabase_lab"
db-schemas = "public"
db-anon-role = "anon"
db-pool = 10
db-pool-timeout = 10
db-extra-search-path = "public"

server-host = "127.0.0.1"
server-port = 8000

jwt-secret = "your-super-secret-jwt-token-with-at-least-32-characters-long"
jwt-secret-is-base64 = false

log-level = "info"
```

## Testing Procedures

### Automated Test Script
```bash
#!/bin/bash
# File: run_comprehensive_test.sh

set -e

echo "🧪 Starting Comprehensive Supabase Storage Test..."

# Step 1: Environment Check
echo "📋 Checking environment..."
source /workspaces/p2p-ai-agents/.env

if [[ -z "$SUPABASE_URL" || -z "$SUPABASE_ANON_KEY" ]]; then
    echo "❌ Environment variables not set correctly"
    exit 1
fi

echo "✅ Environment variables OK"

# Step 2: PostgreSQL Check
echo "🐘 Checking PostgreSQL..."
if ! sudo service postgresql status > /dev/null 2>&1; then
    echo "🔧 Starting PostgreSQL..."
    sudo service postgresql start
    sleep 3
fi

# Test database connection
if ! sudo -u postgres psql -d supabase_lab -c "SELECT 1;" > /dev/null 2>&1; then
    echo "❌ Database connection failed"
    exit 1
fi

echo "✅ PostgreSQL OK"

# Step 3: PostgREST Check
echo "🌐 Checking PostgREST..."
if ! pgrep -f postgrest > /dev/null; then
    echo "🔧 Starting PostgREST..."
    /usr/local/bin/postgrest /tmp/postgrest.conf &
    POSTGREST_PID=$!
    sleep 5
else
    echo "✅ PostgREST already running"
fi

# Wait for PostgREST to be ready
for i in {1..10}; do
    if curl -s http://localhost:8000/ > /dev/null 2>&1; then
        echo "✅ PostgREST ready"
        break
    fi
    echo "⏳ Waiting for PostgREST... ($i/10)"
    sleep 2
    if [[ $i -eq 10 ]]; then
        echo "❌ PostgREST failed to start"
        exit 1
    fi
done

# Step 4: API Health Check
echo "🔍 Testing API endpoints..."

# Test table access
RESPONSE=$(curl -s -w "%{http_code}" -o /tmp/api_test.json \
    -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
    -H "apikey: $SUPABASE_ANON_KEY" \
    "http://localhost:8000/storage_kv?select=key")

if [[ "${RESPONSE: -3}" != "200" ]]; then
    echo "❌ API test failed with status: ${RESPONSE: -3}"
    cat /tmp/api_test.json
    exit 1
fi

echo "✅ API endpoints responding"

# Step 5: Run Rust Tests
echo "🦀 Running Rust tests..."
cd /workspaces/p2p-ai-agents

# Clean and rebuild
cargo clean
cargo build --release

# Run specific storage tests
echo "Running storage_kv tests..."
if cargo test test_supabase_storage --verbose -- --nocapture; then
    echo "✅ Supabase storage tests PASSED"
else
    echo "❌ Supabase storage tests FAILED"
    exit 1
fi

# Step 6: Performance Test
echo "⚡ Running performance tests..."
if cargo test --release performance_test -- --nocapture --ignored; then
    echo "✅ Performance tests PASSED"
else
    echo "⚠️  Performance tests failed or skipped"
fi

echo "🎉 All tests completed successfully!"

# Cleanup if we started PostgREST
if [[ -n "$POSTGREST_PID" ]]; then
    kill $POSTGREST_PID 2>/dev/null || true
fi
```

## Common Issues and Solutions

### Issue 1: "Connection refused" errors

**Symptoms:**
- Tests fail with connection errors
- `curl` requests to localhost:8000 fail

**Solutions:**
```bash
# Check if PostgreSQL is running
sudo service postgresql status
sudo service postgresql start

# Check if PostgREST is running
pgrep -f postgrest
netstat -tulpn | grep :8000

# Restart PostgREST
pkill -f postgrest
/usr/local/bin/postgrest /tmp/postgrest.conf &
```

### Issue 2: "Authentication failed" errors

**Symptoms:**
- 401 Unauthorized responses
- JWT token errors

**Solutions:**
```bash
# Verify JWT tokens match
echo $SUPABASE_ANON_KEY | cut -d'.' -f2 | base64 -d 2>/dev/null | jq

# Check PostgREST configuration
grep jwt-secret /tmp/postgrest.conf

# Verify database roles
sudo -u postgres psql -d supabase_lab -c "\du"
```

### Issue 3: "Table not found" errors

**Symptoms:**
- 404 errors for table endpoints
- Schema-related errors

**Solutions:**
```bash
# Check table exists
sudo -u postgres psql -d supabase_lab -c "\dt"

# Verify permissions
sudo -u postgres psql -d supabase_lab -c "SELECT * FROM information_schema.table_privileges WHERE table_name='storage_kv';"

# Recreate table
sudo -u postgres psql -d supabase_lab < setup_database.sql
```

### Issue 4: Rust compilation errors

**Symptoms:**
- `cargo test` fails to compile
- Missing dependencies

**Solutions:**
```bash
# Update Rust
rustup update

# Clean build cache
cargo clean

# Check dependencies
cargo check

# Update dependencies
cargo update
```

## Performance Testing

### Benchmark Script
```bash
#!/bin/bash
# File: benchmark_supabase.sh

echo "⚡ Running Supabase Storage Performance Benchmarks..."

# Test parameters
ITERATIONS=1000
CONCURRENT_CONNECTIONS=10

# Simple sequential test
echo "📊 Sequential operations test..."
time cargo test --release performance_sequential -- --nocapture --ignored

# Concurrent operations test
echo "📊 Concurrent operations test..."
time cargo test --release performance_concurrent -- --nocapture --ignored

# Large payload test
echo "📊 Large payload test..."
time cargo test --release performance_large_payload -- --nocapture --ignored

echo "✅ Performance benchmarks complete!"
```

## Clean State Recovery

### Complete Reset Script
```bash
#!/bin/bash
# File: reset_environment.sh

echo "🔄 Resetting Supabase environment to clean state..."

# Stop all related processes
pkill -f postgrest || true
sudo service postgresql stop || true

# Clean up temporary files
rm -f /tmp/api_test.json
rm -f /tmp/postgrest.log

# Start fresh PostgreSQL
sudo service postgresql start
sleep 3

# Drop and recreate database
sudo -u postgres dropdb supabase_lab 2>/dev/null || true
sudo -u postgres createdb supabase_lab

# Run setup script
bash setup_database.sh

# Start PostgREST
/usr/local/bin/postgrest /tmp/postgrest.conf &
sleep 5

# Verify everything is working
bash run_comprehensive_test.sh

echo "✅ Environment reset complete!"
```

### Manual Verification Commands
```bash
# PostgreSQL status
sudo service postgresql status
sudo -u postgres psql -d supabase_lab -c "SELECT version();"

# Database schema
sudo -u postgres psql -d supabase_lab -c "\dt"
sudo -u postgres psql -d supabase_lab -c "SELECT count(*) FROM storage_kv;"

# PostgREST status
curl -s http://localhost:8000/ | jq
curl -s -H "Authorization: Bearer $SUPABASE_ANON_KEY" http://localhost:8000/storage_kv | jq

# Environment variables
env | grep SUPABASE
```

## Common Mistakes and How to Avoid Them

### Mistake 1: Using Standalone PostgreSQL Instead of Docker
**What Happened:**
- Started with system-installed PostgreSQL (port 5432)
- Manually configured PostgREST as a separate process  
- Mixed environment variables between Docker and standalone setup
- Required sudo access for database operations

**Why This is Wrong:**
- Inconsistent with official Supabase Docker setup
- Harder to reproduce across different environments
- Permission issues with system PostgreSQL
- Missing Supabase-specific extensions and configurations
- JWT configuration mismatch

**Correct Approach:**
- Use the official Supabase Docker Compose configuration
- All services run in containers with proper networking
- Consistent environment variables
- Isolated from system services
- Matches production Supabase setup

### Mistake 2: Port Configuration Errors
**What Happened:**
- PostgREST configured on port 3000 inside container
- Environment expects PostgREST on port 8000
- Mapping confusion between container and host ports

**Fix:**
```yaml
# Correct port mapping in docker-compose.yml
rest:
  ports:
    - "8000:3000"  # Host:Container - maps container port 3000 to host port 8000
```

### Mistake 3: Database Schema Path Issues
**What Happened:**
- Volume mount path incorrect: `./lab/config/supabase/init.sql`
- Docker context doesn't include parent directories

**Fix:**
```yaml
# Correct volume mount (relative to docker-compose.yml location)
volumes:
  - ./init.sql:/docker-entrypoint-initdb.d/init.sql
```

## Correct Docker-Based Setup

### Prerequisites
- Docker and Docker Compose installed
- No conflicting services on ports 5432, 8000, 9999

### Step-by-Step Docker Setup
````
<userPrompt>
Provide the fully rewritten file, incorporating the suggested code change. You must produce the complete file.
</userPrompt>
