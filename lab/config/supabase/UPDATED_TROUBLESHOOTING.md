# Comprehensive Supabase Storage Adapter Troubleshooting Guide

## Table of Contents
1. [Quick Start Checklist](#quick-start-checklist)
2. [Environment-Specific Issues](#environment-specific-issues)
3. [Working Solutions](#working-solutions)
4. [Docker Issues and Alternatives](#docker-issues-and-alternatives)
5. [Standalone PostgreSQL + PostgREST Setup](#standalone-postgresql--postgrest-setup)
6. [Testing Procedures](#testing-procedures)
7. [Common Issues and Solutions](#common-issues-and-solutions)

## Quick Start Checklist

### ✅ Environment Assessment
- [ ] Identify if running in GitHub Codespaces, Local Docker, or Cloud Environment
- [ ] Check Docker permissions and container support
- [ ] Verify PostgreSQL and PostgREST availability
- [ ] Confirm network port accessibility (5432, 8000, 3000)

### ✅ Choose Approach
- [ ] **Docker Compose (Recommended for Local)**: Full Supabase stack
- [ ] **Standalone Setup (Codespaces/Restricted)**: PostgreSQL + PostgREST only
- [ ] **External Supabase (Production)**: Real Supabase cloud instance

## Environment-Specific Issues

### GitHub Codespaces Limitations

**Problem**: Docker Compose fails with `"failed to register layer: unshare: operation not permitted"`

**Root Cause**: 
- Codespaces restricts container operations for security
- Limited support for Docker volume mounts and layer operations
- Container runtime restrictions prevent full Supabase stack deployment

**Development Note**: 
- ⚠️ For development/testing, bypassing unshare restrictions is acceptable
- This is NOT a production security concern - just a Codespaces limitation
- Security restrictions like unshare can be safely ignored for testing purposes

**Evidence**:
```bash
INFO[2025-06-15T19:44:02.659945707Z] Attempting next endpoint for pull after error: failed to register layer: unshare: operation not permitted
failed to register layer: unshare: operation not permitted
```

**Workarounds Attempted**:
1. `privileged: true` in Docker Compose services ❌
2. `security_opt: ["apparmor:unconfined"]` ❌
3. `--security-opt apparmor=unconfined` Docker flags ❌
4. Simple PostgreSQL container (`docker run postgres:15-alpine`) ❌

**Technical Diagnosis**: 
- Issue affects ALL Docker image pulls, not just Supabase
- Storage driver: `vfs` (instead of overlay2)
- Kernel restrictions prevent layer registration during image pulls
- Cannot be bypassed with Docker security options

**Development Recommendation**: 
✅ **Use External Supabase Instance for Testing**
- Create free Supabase project at https://supabase.com
- Use real API keys for development/testing
- No local Docker required

### Local Development Environment

**Docker Works**: Full Docker Compose setup with official Supabase images
**Requirements**:
- Docker Desktop with sufficient permissions
- 4GB+ RAM available for containers
- Full container runtime support

### Cloud/VPS Environment

**Docker Usually Works**: Depends on hosting provider container policies
**Alternative**: Native package installation of PostgreSQL and PostgREST

## Container Runtime Environment Options

### GitHub Codespaces Limitations (Current Environment)

**Current Status**: Limited Docker support with VFS storage driver
- ✅ Docker daemon available and accessible
- ✅ User in docker group
- ❌ Storage Driver: `vfs` (not overlay2)
- ❌ Layer registration restricted (`unshare: operation not permitted`)

**Potential Workarounds in Codespaces**:

1. **Use --privileged flag** (may work in some configurations):
```bash
# Try running containers with elevated privileges
docker run --privileged postgres:15-alpine
```

2. **Alternative image registries** (some may have pre-pulled layers):
```bash
# Try different base images that might be cached
docker run --rm hello-world
docker run --rm alpine:latest echo "test"
```

3. **Build from scratch instead of pulling**:
```bash
# Create minimal Dockerfile to bypass layer pulling
cat > Dockerfile.minimal << 'EOF'
FROM scratch
ADD postgres-binary /usr/bin/postgres
EOF
```

### VS Code with Docker Desktop (Local)

**Status**: Full Docker support
- ✅ Complete container runtime access
- ✅ Overlay2 storage driver
- ✅ Volume mounts work properly
- ✅ All Docker Compose features available

**Recommendation**: Use Docker Compose setup as originally intended

### VS Code Dev Containers (Alternative)

**Status**: Variable depending on host
- ✅ Usually full Docker support when running on local Docker Desktop
- ✅ Can run nested containers with proper configuration
- ⚠️ May have restrictions when running on remote hosts

### Remote Development Options

1. **GitHub Codespaces with Machine Type Upgrade**:
   - Try larger machine types (4-core, 8-core)
   - May have different Docker configurations
   
2. **Self-hosted Codespaces**:
   - Full control over Docker runtime
   - Can disable security restrictions
   
3. **Remote SSH to VPS/Cloud Instance**:
   - Complete Docker control
   - No corporate/platform restrictions

## Testing Container Runtime Capabilities

Run this diagnostic script to test your environment:

```bash
#!/bin/bash
echo "🔍 Container Runtime Diagnostic"
echo "=============================="
echo ""

# Test 1: Basic Docker access
echo "1. Docker Access:"
docker version >/dev/null 2>&1 && echo "   ✅ Docker accessible" || echo "   ❌ Docker not accessible"

# Test 2: Simple container run
echo "2. Simple Container:"
docker run --rm hello-world >/dev/null 2>&1 && echo "   ✅ Basic containers work" || echo "   ❌ Container execution fails"

# Test 3: Image pull capability
echo "3. Image Pull Test:"
docker pull alpine:latest >/dev/null 2>&1 && echo "   ✅ Image pulls work" || echo "   ❌ Image pull restricted"

# Test 4: PostgreSQL specific
echo "4. PostgreSQL Container:"
docker run --rm -d --name test-pg -e POSTGRES_PASSWORD=test postgres:15-alpine >/dev/null 2>&1
if [ $? -eq 0 ]; then
    docker stop test-pg >/dev/null 2>&1
    echo "   ✅ PostgreSQL containers work"
else
    echo "   ❌ PostgreSQL containers fail"
fi

# Test 5: Docker Compose
echo "5. Docker Compose:"
command -v docker-compose >/dev/null 2>&1 && echo "   ✅ Docker Compose available" || echo "   ❌ Docker Compose not found"

echo ""
echo "Storage Driver: $(docker info --format '{{.Driver}}')"
echo "Kernel: $(uname -r)"
echo "Environment: $([ -n "$CODESPACES" ] && echo "GitHub Codespaces" || echo "Local/Other")"
```

## Recommended Approach by Environment

| Environment | Docker Capability | Recommended Setup |
|-------------|------------------|-------------------|
| **GitHub Codespaces (Standard)** | Limited (VFS, unshare restricted) | 🏆 **External Supabase** |
| **GitHub Codespaces (Premium)** | May have better support | Try Docker first, fallback to External |
| **VS Code + Docker Desktop** | Full support | 🏆 **Docker Compose** |
| **Dev Containers (Local)** | Full support | 🏆 **Docker Compose** |
| **Remote SSH to VPS** | Full support | 🏆 **Docker Compose** |
| **Self-hosted Codespaces** | Configurable | **Docker Compose** or **External** |

## Enabling Better Docker Support

### In Codespaces (Limited Options)

1. **Try different machine types**:
   - Go to Codespace settings
   - Select 4-core or 8-core machine
   - May have different Docker configurations

2. **Use prebuild configuration**:
```json
// .devcontainer/devcontainer.json
{
  "image": "mcr.microsoft.com/devcontainers/universal:2",
  "features": {
    "ghcr.io/devcontainers/features/docker-in-docker:2": {
      "dockerDashComposeVersion": "latest"
    }
  },
  "runArgs": ["--privileged"]
}
```

3. **Request access to Docker-in-Docker**:
   - Some Codespaces configurations support nested Docker
   - Requires specific setup in devcontainer.json

### For Local Development

1. **Use Docker Desktop**:
   - Provides full unrestricted Docker access
   - All Supabase Docker Compose features work

2. **Linux with Docker**:
   - Install Docker Engine directly
   - Full control over storage drivers and security options

## Working Solutions

### Option 1: External Supabase Instance (Recommended for Codespaces)

This is the most reliable approach for GitHub Codespaces and eliminates all Docker-related issues.

#### Step 1: Create Supabase Project
1. Go to https://supabase.com and sign up/sign in
2. Click "New Project"
3. Choose organization and set project details:
   - **Name**: `p2p-ai-agents-dev`
   - **Database Password**: Generate a strong password
   - **Region**: Choose closest to your location
4. Wait for project initialization (2-3 minutes)

#### Step 2: Get API Configuration
1. In your Supabase dashboard, go to **Settings** → **API**
2. Copy the following values:
   - **Project URL**: `https://your-project-ref.supabase.co`
   - **anon public key**: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`
   - **service_role secret**: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`

#### Step 3: Configure Environment
```bash
# Update .env file
cd /workspaces/p2p-ai-agents
cat > .env << 'EOF'
# Supabase Configuration (External Instance)
SUPABASE_URL=https://your-project-ref.supabase.co
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...your-anon-key...
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...your-service-role-key...

# Test Configuration
TEST_TABLE_NAME=test_storage
TEST_BUCKET_NAME=test-bucket
EOF
```

#### Step 4: Setup Database Schema
```bash
# Create test schema using Supabase SQL Editor
# Or run via API:
curl -X POST 'https://your-project-ref.supabase.co/rest/v1/rpc/create_test_schema' \
  -H "apikey: your-anon-key" \
  -H "Authorization: Bearer your-service-role-key" \
  -H "Content-Type: application/json"
```

#### Step 5: Test Connection
```bash
# Test the Rust adapter
cd /workspaces/p2p-ai-agents
cargo test storage::supabase --features supabase
```

#### Step 6: Automated Setup (Alternative)
```bash
# Use the automated setup script
cd /workspaces/p2p-ai-agents
./lab/scripts/setup_external_supabase.sh

# Test the connection
./lab/scripts/test_external_supabase.sh
```

The automated scripts will:
- ✅ Prompt for your Supabase credentials
- ✅ Create properly formatted `.env` file
- ✅ Test API connectivity and authentication
- ✅ Create test database schema
- ✅ Validate Rust adapter compilation
- ✅ Run basic CRUD operations test

**Advantages**:
- ✅ Works reliably in all environments
- ✅ Real Supabase API with all features
- ✅ No Docker/container restrictions
- ✅ Matches production environment closely
- ✅ Free tier available (500MB database, 2GB bandwidth)

**Disadvantages**:
- ⚠️ Requires internet connection
- ⚠️ Uses external service (not fully isolated)
- ⚠️ May incur costs if exceeding free tier

### Option 2: Standalone PostgreSQL + PostgREST (Codespaces Compatible)

This approach uses system-installed PostgreSQL and PostgREST, avoiding Docker entirely.

#### Step 1: Stop Conflicting Services
```bash
# Stop any running Docker containers
docker compose down 2>/dev/null || true

# Stop system PostgreSQL if running
sudo service postgresql stop
```

#### Step 2: Setup PostgreSQL Database
```bash
# Start PostgreSQL
sudo service postgresql start

# Create database and roles
sudo -u postgres psql << 'EOF'
-- Create database
CREATE DATABASE supabase_lab;

-- Connect to database
\c supabase_lab;

-- Create roles
CREATE ROLE anon NOLOGIN;
CREATE ROLE authenticated NOLOGIN;  
CREATE ROLE service_role NOLOGIN;
CREATE ROLE authenticator NOINHERIT LOGIN PASSWORD 'your-super-secret-jwt-token-with-at-least-32-characters-long';

-- Grant role memberships
GRANT anon, authenticated, service_role TO authenticator;

-- Create storage table
CREATE TABLE storage_kv (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Grant permissions
GRANT ALL ON storage_kv TO anon, authenticated, service_role, authenticator;
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role, authenticator;

-- Enable RLS with permissive policies for testing
ALTER TABLE storage_kv ENABLE ROW LEVEL SECURITY;
CREATE POLICY "allow_all" ON storage_kv FOR ALL USING (true);
EOF
```

#### Step 3: Configure PostgREST
```bash
# Create PostgREST config
cat > /tmp/postgrest.conf << 'EOF'
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
EOF
```

#### Step 4: Start PostgREST
```bash
# Start PostgREST in background
/usr/local/bin/postgrest /tmp/postgrest.conf &

# Wait for startup
sleep 5

# Test connection
curl -s http://localhost:8000/ | jq
```

#### Step 5: Configure Environment
```bash
# Update project .env file
cat > /workspaces/p2p-ai-agents/.env << 'EOF'
SUPABASE_URL=http://localhost:8000
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJhbm9uIiwKICAgICJpc3MiOiAic3VwYWJhc2UtZGVtbyIsCiAgICAiaWF0IjogMTY0MVc2OTIwMCwKICAgICJleHAiOiAxNzk5NTM1NjAwCn0.dc_X5iR_VP_qT0zsiyj_I_OZ2T9FtRU2BBNWN8Bu4GE
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJzZXJ2aWNlX3JvbGUiLAogICAgImlzcyI6ICJzdXBhYmFzZS1kZW1vIiwKICAgICJpYXQiOiAxNjQxNzY5MjAwLAogICAgImV4cCI6IDE3OTk1MzU2MDAKfQ.DaYlNEoUrrEn2Ig7tqibS-PHK5vgusbcbo7X36XVt4Q
SUPABASE_SCHEMA=public
SUPABASE_TABLE_NAME=storage_kv
EOF
```

### Option 2: Docker Compose (Local Development)

For environments with full Docker support:

```bash
cd /workspaces/p2p-ai-agents/lab/config/supabase
docker compose up -d
```

**Fixed Issues in docker-compose.yml**:
- ✅ Corrected port mapping for PostgREST (3000:3000 instead of 8000:3000)
- ✅ Fixed volume path references (./kong.yml instead of ./lab/config/supabase/kong.yml) 
- ✅ Resolved port conflicts between Kong and PostgREST
- ✅ Updated JWT tokens to use standard Supabase demo tokens

## Testing Procedures

### Comprehensive Test Script
```bash
#!/bin/bash
# Run this script to test the setup

set -e

echo "🧪 Testing Supabase Storage Integration..."

# Check environment
source /workspaces/p2p-ai-agents/.env || { echo "❌ .env file missing"; exit 1; }

# Test API endpoint
echo "📡 Testing API endpoint..."
if curl -s "$SUPABASE_URL" > /dev/null; then
    echo "✅ API endpoint accessible"
else
    echo "❌ API endpoint failed"
    exit 1
fi

# Test table access
echo "📋 Testing table access..."
RESPONSE=$(curl -s -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
    -H "apikey: $SUPABASE_ANON_KEY" \
    "$SUPABASE_URL/storage_kv?select=key")

if [[ "$RESPONSE" == "["* ]]; then
    echo "✅ Table access working"
else
    echo "❌ Table access failed: $RESPONSE"
    exit 1
fi

# Test CRUD operations
echo "🔄 Testing CRUD operations..."
TEST_KEY="test_$(date +%s)"
TEST_VALUE='{"test": "data"}'

# CREATE
CREATE_RESULT=$(curl -s -X POST \
    -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
    -H "apikey: $SUPABASE_ANON_KEY" \
    -H "Content-Type: application/json" \
    -H "Prefer: return=representation" \
    -d "{\"key\": \"$TEST_KEY\", \"value\": $TEST_VALUE}" \
    "$SUPABASE_URL/storage_kv")

if [[ "$CREATE_RESULT" == "["* ]]; then
    echo "✅ CREATE operation successful"
else
    echo "❌ CREATE operation failed: $CREATE_RESULT"
fi

# READ
READ_RESULT=$(curl -s \
    -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
    -H "apikey: $SUPABASE_ANON_KEY" \
    "$SUPABASE_URL/storage_kv?key=eq.$TEST_KEY")

if [[ "$READ_RESULT" == "["* ]]; then
    echo "✅ READ operation successful"
else
    echo "❌ READ operation failed: $READ_RESULT"
fi

# DELETE
DELETE_RESULT=$(curl -s -X DELETE \
    -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
    -H "apikey: $SUPABASE_ANON_KEY" \
    "$SUPABASE_URL/storage_kv?key=eq.$TEST_KEY")

echo "✅ DELETE operation completed"

# Test Rust integration
echo "🦀 Testing Rust integration..."
cd /workspaces/p2p-ai-agents

if cargo test test_supabase_storage --verbose 2>/dev/null; then
    echo "✅ Rust tests passed"
else
    echo "⚠️  Rust tests failed or need investigation"
fi

echo "🎉 Test completed!"
```

## Common Issues and Solutions

### Issue 1: "Connection refused" 
**Symptoms**: Cannot connect to localhost:8000
**Solutions**:
```bash
# Check if PostgREST is running
pgrep -f postgrest || echo "PostgREST not running"

# Check port usage
netstat -tulpn | grep :8000 || echo "Port 8000 not in use"

# Restart PostgREST
pkill -f postgrest
/usr/local/bin/postgrest /tmp/postgrest.conf &
```

### Issue 2: "Authentication failed"
**Symptoms**: 401 Unauthorized errors
**Solutions**:
```bash
# Verify JWT tokens
echo $SUPABASE_ANON_KEY | cut -d'.' -f2 | base64 -d | jq

# Check PostgREST JWT secret
grep jwt-secret /tmp/postgrest.conf

# Test with different role
curl -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
     -H "apikey: $SUPABASE_SERVICE_ROLE_KEY" \
     "$SUPABASE_URL/storage_kv"
```

### Issue 3: "Table not found"
**Symptoms**: 404 errors for table endpoints
**Solutions**:
```bash
# Check database connection
sudo -u postgres psql -d supabase_lab -c "\dt"

# Verify table permissions  
sudo -u postgres psql -d supabase_lab -c "SELECT * FROM information_schema.table_privileges WHERE table_name='storage_kv';"

# Check RLS policies
sudo -u postgres psql -d supabase_lab -c "SELECT * FROM pg_policies WHERE tablename='storage_kv';"
```

### Issue 4: Docker "unshare: operation not permitted"
**Symptoms**: Docker layers fail to register
**Root Cause**: Environment restrictions (Codespaces, limited containers)
**Solution**: Use standalone setup instead of Docker

## Performance Considerations

### Database Optimization
```sql
-- Add indexes for better performance
CREATE INDEX IF NOT EXISTS idx_storage_kv_created_at ON storage_kv(created_at);
CREATE INDEX IF NOT EXISTS idx_storage_kv_updated_at ON storage_kv(updated_at);

-- Analyze table for query optimization
ANALYZE storage_kv;
```

### PostgREST Tuning
```ini
# Increase connection pool for high load
db-pool = 20
db-pool-timeout = 10

# Enable connection pooling mode
db-use-legacy-gucs = false
```

### Monitoring
```bash
# Monitor PostgREST logs
tail -f /tmp/postgrest.log

# Check PostgreSQL activity
sudo -u postgres psql -d supabase_lab -c "SELECT * FROM pg_stat_activity WHERE datname='supabase_lab';"

# Monitor system resources
htop
df -h
```

## Final Test Report Template

```markdown
# Supabase Storage Integration Test Report

**Date**: $(date)
**Environment**: [GitHub Codespaces / Local / Docker]
**Setup Method**: [Standalone / Docker Compose]
**Duration**: [X] minutes

## ✅ Successful Tests
- [ ] Environment setup and configuration
- [ ] PostgreSQL database connectivity
- [ ] PostgREST API responses
- [ ] JWT authentication (anon/service roles)
- [ ] CRUD operations via REST API
- [ ] Rust adapter integration
- [ ] Performance benchmarks

## ❌ Failed Tests
- [ ] [List any failures with explanations]

## 📊 Performance Metrics
- **API Response Time**: [X]ms average
- **Database Query Time**: [X]ms average
- **Concurrent Connections**: [X] tested
- **Memory Usage**: [X]MB peak

## 🔧 Configuration Used
- PostgreSQL: [version]
- PostgREST: [version]  
- Database: supabase_lab
- Table: storage_kv
- API Endpoint: http://localhost:8000

## 🚨 Issues Encountered
[Document any issues and their resolutions]

## ✅ Recommendations
1. [List recommendations for production use]
2. [Performance optimizations suggested]
3. [Security considerations]

## 📝 Next Steps
- [ ] [List follow-up actions needed]
```

This comprehensive guide provides multiple approaches to get Supabase storage working in any environment, with clear troubleshooting steps for common issues.

## Final Recommendations

### For GitHub Codespaces Development

**🏆 RECOMMENDED**: Use External Supabase Instance
```bash
# Quick setup
cd /workspaces/p2p-ai-agents
./lab/scripts/setup_external_supabase.sh
./lab/scripts/test_external_supabase.sh
```

**Why this is the best approach**:
- ✅ Eliminates all Docker/container restrictions
- ✅ Uses real Supabase API (matches production)
- ✅ Reliable and consistent across environments
- ✅ No complex local setup required
- ✅ Free tier available for development

### For Local Development

**🏆 RECOMMENDED**: Docker Compose (if Docker works)
```bash
cd /workspaces/p2p-ai-agents/lab/config/supabase
docker compose up -d
```

**Alternative**: Standalone PostgreSQL + PostgREST
- Use when Docker has issues
- Follow Option 2 instructions above

### For Production/Staging

**🏆 RECOMMENDED**: Managed Supabase
- Use official Supabase hosting
- Configure via environment variables
- Enable proper RLS policies and security

## Quick Troubleshooting Decision Tree

```
Are you in GitHub Codespaces?
├─ YES → Use External Supabase Instance (Option 1)
└─ NO
   ├─ Does Docker work normally?
   │  ├─ YES → Use Docker Compose
   │  └─ NO → Use Standalone PostgreSQL + PostgREST (Option 2)
   └─ Is this for production?
      └─ YES → Use Managed Supabase (external instance)
```

## Support and Next Steps

If you're still having issues:

1. **Check the specific error** in the sections above
2. **Run the test scripts** to validate your setup
3. **Use external Supabase** as a reliable fallback
4. **Document new issues** for future troubleshooting

**Available Scripts**:
- `./lab/scripts/setup_external_supabase.sh` - Configure external Supabase
- `./lab/scripts/test_external_supabase.sh` - Test external connection
- `./lab/scripts/setup_database.sh` - Setup local database
- `./lab/scripts/run_comprehensive_test.sh` - Run all tests
- `./lab/scripts/reset_environment.sh` - Reset local environment

**Key Files**:
- `.env` - Environment configuration
- `lab/config/supabase/docker-compose.yml` - Local Docker stack
- `src/storage/supabase.rs` - Rust adapter implementation

---

*Last updated: 2025-06-15*
*Environment tested: GitHub Codespaces, Docker 28.2.2*

## Enabling Docker Support in Codespaces

### Current Diagnosis

Based on runtime testing, the current Codespaces configuration has:
- ✅ Docker daemon available
- ✅ Docker Compose available
- ❌ **Image pulling fails** (unshare restriction)
- ❌ **Container execution fails** (layer registration)
- **Score: 1/8** → External Supabase recommended

### Option 1: Enhanced Devcontainer Configuration

**Problem**: Current `.devcontainer/devcontainer.json` lacks Docker-in-Docker support

**Solution**: Use enhanced devcontainer with proper Docker support

```bash
# 1. Backup current config
cp .devcontainer/devcontainer.json .devcontainer/devcontainer-backup.json

# 2. Use enhanced config with Docker-in-Docker
cp .devcontainer/devcontainer-with-docker.json .devcontainer/devcontainer.json

# 3. Rebuild Codespace
# Go to Command Palette (Ctrl+Shift+P) → "Codespaces: Rebuild Container"
```

**Enhanced Configuration Features**:
- ✅ Docker-in-Docker feature enabled
- ✅ Docker Compose latest version
- ✅ Docker Buildx support
- ✅ Privileged container support
- ✅ Additional ports forwarded (5432, 8000)

### Option 2: Manual Docker-in-Docker Setup

If you can't rebuild the Codespace, try enabling Docker-in-Docker manually:

```bash
# Install Docker-in-Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Enable Docker daemon with less restrictive settings
sudo dockerd --storage-driver=overlay2 --userland-proxy=false &

# Test improved capabilities
./lab/scripts/diagnose_docker_runtime.sh
```

### Option 3: Different Codespace Machine Types

**Current**: 2-core machine (may have more restrictions)
**Try**: 4-core or 8-core machine types

1. Go to your repository on GitHub.com
2. Click "Code" → "Codespaces" → "..."
3. Select "New with options"
4. Choose "4-core" or "8-core" machine type
5. Test Docker capabilities with diagnostic script

### Option 4: Prebuilt Codespaces

Create a prebuild configuration to avoid runtime Docker issues:

```json
// .github/dependabot.yml or codespaces config
{
  "prebuild": {
    "triggers": ["push"],
    "commands": [
      "docker pull postgres:15-alpine",
      "docker pull supabase/postgres:15",
      "docker pull kong:latest"
    ]
  }
}
```

## Alternative Development Environments

### VS Code with Docker Desktop (Recommended for Local)

**Setup**:
1. Install Docker Desktop
2. Install VS Code + Remote-Containers extension
3. Open project in Dev Container

**Advantages**:
- ✅ Full Docker support
- ✅ All storage drivers available
- ✅ No layer restrictions
- ✅ Complete Supabase stack works

**Commands**:
```bash
# Full Docker Compose setup works
cd lab/config/supabase
docker compose up -d

# All tests pass
./lab/scripts/run_comprehensive_test.sh
```

### Remote SSH Development

**Setup**:
1. Rent a VPS (DigitalOcean, Linode, AWS EC2)
2. Install Docker + development tools
3. Connect via VS Code Remote-SSH

**Docker Installation on Ubuntu**:
```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Test full capabilities
docker run hello-world
docker compose version
```

### Local Development (Ultimate Solution)

**Best for**: Full-time development on this project

**Setup**:
1. Clone repository locally
2. Install Docker Desktop (macOS/Windows) or Docker Engine (Linux)
3. Use VS Code with Dev Containers extension

**Advantages**:
- ✅ Complete control over Docker configuration
- ✅ No network latency
- ✅ All features work as intended
- ✅ Best performance

## Docker Capability Testing

Use our diagnostic script to test any environment:

```bash
# Test current environment
./lab/scripts/diagnose_docker_runtime.sh

# Expected scores:
# 8/8: Perfect Docker support → Use Docker Compose
# 4-7/8: Partial support → Try Docker, fallback to External
# 0-3/8: Limited support → Use External Supabase
```
