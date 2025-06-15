# Supabase Storage Adapter Troubleshooting Guide

## Overview

This guide provides comprehensive troubleshooting for the Supabase storage adapter in the p2p-ai-agents project, covering both local development and Docker-based setups.

## Prerequisites

### Required Software
- **Rust** (latest stable) with cargo
- **PostgreSQL** 13+ 
- **PostgREST** 12.0.2+
- **Docker & Docker Compose** (for full Supabase stack)
- **curl** and **jq** (for API testing)
- **psql** (PostgreSQL client)

### System Requirements
- **Linux/macOS** (Windows WSL2 supported)
- **2+ GB RAM** available for containers
- **Network access** to localhost ports 5432, 8000, 3000
- **File permissions** for Docker socket (if using Docker)

## Environment Setup

### 1. Environment Variables

Create `/workspaces/p2p-ai-agents/.env`:
```bash
# Core Supabase configuration
SUPABASE_URL=http://localhost:8000
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJhbm9uIiwKICAgICJpc3MiOiAic3VwYWJhc2UtZGVtbyIsCiAgICAiaWF0IjogMTY0MVc2OTIwMCwKICAgICJleHAiOiAxNzk5NTM1NjAwCn0.dc_X5iR_VP_qT0zsiyj_I_OZ2T9FtRU2BBNWN8Bu4GE
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJzZXJ2aWNlX3JvbGUiLAogICAgImlzcyI6ICJzdXBhYmFzZS1kZW1vIiwKICAgICJpYXQiOiAxNjQxNzY5MjAwLAogICAgImV4cCI6IDE3OTk1MzU2MDAKfQ.DaYlNEoUrrEn2Ig7tqibS-PHK5vgusbcbo7X36XVt4Q
SUPABASE_SCHEMA=public
SUPABASE_TABLE_NAME=storage_kv
```

### 2. JWT Token Details

The JWT tokens above decode to:

**ANON_KEY payload:**
```json
{
  "role": "anon",
  "iss": "supabase-demo", 
  "iat": 1641769200,
  "exp": 1799535600
}
```

**SERVICE_ROLE_KEY payload:**
```json
{
  "role": "service_role",
  "iss": "supabase-demo",
  "iat": 1641769200, 
  "exp": 1799535600
}
```

**JWT Secret:** `your-super-secret-jwt-token-with-at-least-32-characters-long`

## Setup Methods

### Method 1: PostgreSQL + PostgREST (Recommended for Development)

This method works best in restricted environments like GitHub Codespaces.

#### Step 1: PostgreSQL Setup
```bash
# Check if PostgreSQL is running
sudo service postgresql status

# Start if needed
sudo service postgresql start

# Create database
sudo -u postgres createdb supabase_lab

# Create required roles
sudo -u postgres psql -d supabase_lab -c "
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
END
\$\$;
"
```

#### Step 2: Database Schema
```bash
sudo -u postgres psql -d supabase_lab -c "
-- Create the storage table
CREATE TABLE IF NOT EXISTS storage_kv (
    id SERIAL PRIMARY KEY,
    key VARCHAR(255) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Enable Row Level Security
ALTER TABLE storage_kv ENABLE ROW LEVEL SECURITY;

-- Create policies for anon and authenticated access
CREATE POLICY IF NOT EXISTS \"Allow anon access to storage_kv\" ON storage_kv
    FOR ALL USING (true) WITH CHECK (true);

-- Grant permissions
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON storage_kv TO postgres, anon, authenticated, service_role;
GRANT ALL ON SEQUENCE storage_kv_id_seq TO postgres, anon, authenticated, service_role;
"
```

#### Step 3: PostgREST Configuration
Create `/tmp/supabase-postgrest.conf`:
```ini
db-uri = "postgres://postgres@localhost:5432/supabase_lab"
db-schemas = "public"
db-anon-role = "anon"
server-host = "*" 
server-port = 8000
jwt-secret = "your-super-secret-jwt-token-with-at-least-32-characters-long"
jwt-aud = "authenticated"
```

#### Step 4: Start PostgREST
```bash
# Start PostgREST
cd /tmp && postgrest supabase-postgrest.conf > supabase-postgrest.log 2>&1 &

# Check if running
ps aux | grep postgrest

# Check logs
tail -f /tmp/supabase-postgrest.log
```

### Method 2: Full Docker Compose Stack

#### Prerequisites
```bash
# Test Docker permissions
docker run --rm hello-world

# If permission denied, add user to docker group:
sudo usermod -aG docker $USER
# Then logout/login or restart terminal
```

#### Step 1: Download Supabase Docker Files
```bash
cd /workspaces/p2p-ai-agents
mkdir -p lab/config/supabase/docker
cd lab/config/supabase/docker

# Get official docker-compose.yml
curl -o docker-compose.yml https://raw.githubusercontent.com/supabase/supabase/master/docker/docker-compose.yml

# Get .env template
curl -o .env.example https://raw.githubusercontent.com/supabase/supabase/master/docker/.env.example
```

#### Step 2: Configure Environment
```bash
# Copy and customize .env
cp .env.example .env

# Edit .env with proper values (see Environment Variables section above)
```

#### Step 3: Initialize Database
```bash
mkdir -p volumes/db/init

cat > volumes/db/init/00-initial-schema.sql << 'EOF'
-- Create storage_kv table for Supabase adapter
CREATE TABLE IF NOT EXISTS storage_kv (
    id SERIAL PRIMARY KEY,
    key VARCHAR(255) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Enable Row Level Security
ALTER TABLE storage_kv ENABLE ROW LEVEL SECURITY;

-- Create policies
CREATE POLICY "Allow anon access to storage_kv" ON storage_kv
    FOR ALL USING (true) WITH CHECK (true);

-- Grant permissions
GRANT ALL ON storage_kv TO postgres, anon, authenticated, service_role;
GRANT ALL ON SEQUENCE storage_kv_id_seq TO postgres, anon, authenticated, service_role;
EOF
```

#### Step 4: Start Services
```bash
# Start all services
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs
```

## Testing & Validation

### 1. API Connectivity Test
```bash
# Test PostgREST API
curl -s http://localhost:8000/storage_kv

# Test with authentication
curl -s -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \\
     -H "apikey: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \\
     http://localhost:8000/storage_kv
```

### 2. Database Connectivity Test
```bash
# Test PostgreSQL connection
psql -h localhost -U postgres -d supabase_lab -c "SELECT current_database(), current_user;"

# Check table structure
psql -h localhost -U postgres -d supabase_lab -c "\\d storage_kv"
```

### 3. Rust Adapter Tests
```bash
cd /workspaces/p2p-ai-agents

# Load environment variables
source .env

# Run Supabase storage tests
cargo test --features storage-supabase storage::supabase::tests -- --nocapture --ignored

# Run specific test
cargo test --features storage-supabase storage::supabase::tests::test_supabase_storage_overwrite -- --nocapture --ignored
```

## Common Issues & Solutions

### Issue 1: JWT Authentication Errors
**Symptoms:**
- `Server lacks JWT secret`
- `CompactDecodeError Invalid number of parts`
- `401 Unauthorized` responses

**Solutions:**
1. Verify JWT secret matches in PostgREST config and tokens
2. Check JWT token format (should have 3 parts separated by dots)
3. Ensure `jwt-secret` in PostgREST config is at least 32 characters
4. Restart PostgREST after config changes

### Issue 2: Database Connection Issues
**Symptoms:**
- `Connection refused`
- `Role "anon" does not exist`
- `Database "supabase_lab" does not exist`

**Solutions:**
1. Check PostgreSQL is running: `sudo service postgresql status`
2. Create missing roles and database (see Step 1 above)
3. Verify pg_hba.conf allows connections from PostgREST
4. Check database credentials in PostgREST config

### Issue 3: Schema Mismatches
**Symptoms:**
- `Column 'updated_at' does not exist`
- `Column 'key' does not exist`
- JSON parsing errors

**Solutions:**
1. Recreate table with correct schema (see Step 2 above)
2. Restart PostgREST to reload schema cache
3. Verify table permissions for anon/service_role
4. Check Supabase adapter expects: `key`, `value`, `created_at`, `updated_at`

### Issue 4: Docker Permission Issues
**Symptoms:**
- `permission denied while trying to connect to Docker daemon`
- `unshare: operation not permitted`

**Solutions:**
1. Add user to docker group: `sudo usermod -aG docker $USER`
2. Restart terminal/session
3. In restricted environments (Codespaces), use Method 1 instead
4. Check Docker daemon is running: `sudo systemctl status docker`

### Issue 5: Port Conflicts
**Symptoms:**
- `Address already in use`
- `bind: address already in use`

**Solutions:**
1. Check what's using ports: `netstat -tlnp | grep -E ':(8000|5432|3000)'`
2. Kill conflicting processes: `pkill -f postgrest`
3. Change ports in configuration if needed
4. Use `docker compose down` to stop all containers

### Issue 6: Test Failures
**Symptoms:**
- `Failed to put key: 400 Bad Request`
- `The result contains 0 rows`
- `Failed to parse response`

**Solutions:**
1. Verify environment variables are loaded: `echo $SUPABASE_URL`
2. Check API is responding: `curl http://localhost:8000/storage_kv`
3. Verify table schema matches adapter expectations
4. Run tests individually to isolate issues
5. Check PostgREST logs for detailed error messages

## Debugging Commands

### Environment Check
```bash
# Check environment variables
env | grep SUPABASE

# Check processes
ps aux | grep -E '(postgres|postgrest)'

# Check ports
netstat -tlnp | grep -E ':(8000|5432|3000)'
```

### Database Debug
```bash
# Connect to database
psql -h localhost -U postgres -d supabase_lab

# In psql:
\\dt          # List tables
\\d storage_kv # Describe table
\\du          # List roles
SELECT * FROM storage_kv LIMIT 5;
```

### API Debug
```bash
# Test endpoints
curl -v http://localhost:8000/
curl -v http://localhost:8000/storage_kv
curl -v -H "Authorization: Bearer $SUPABASE_ANON_KEY" http://localhost:8000/storage_kv

# Test POST
curl -X POST http://localhost:8000/storage_kv \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer $SUPABASE_ANON_KEY" \\
  -d '{"key": "test", "value": "debug"}'
```

### Log Files
- **PostgREST:** `/tmp/supabase-postgrest.log`
- **PostgreSQL:** `/var/log/postgresql/postgresql-*-main.log`
- **Docker:** `docker compose logs [service_name]`

## Performance Testing

Once the environment is stable:

```bash
# Run performance tests
cargo test --features storage-supabase --test storage_perf -- --nocapture

# Run custom performance script
./lab/scripts/real_performance_test.sh

# Monitor during tests
watch -n 1 'ps aux | grep -E "(postgres|postgrest)"'
```

## Maintenance

### Regular Cleanup
```bash
# Clean up test data
psql -h localhost -U postgres -d supabase_lab -c "DELETE FROM storage_kv WHERE key LIKE 'test%';"

# Restart services
sudo service postgresql restart
pkill -f postgrest && cd /tmp && postgrest supabase-postgrest.conf &

# Docker cleanup
docker compose down && docker compose up -d
```

### Backup/Restore
```bash
# Backup database
pg_dump -h localhost -U postgres supabase_lab > supabase_backup.sql

# Restore
psql -h localhost -U postgres -d supabase_lab < supabase_backup.sql
```

This guide should help diagnose and resolve most issues with the Supabase storage adapter setup.
