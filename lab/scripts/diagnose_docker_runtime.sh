#!/bin/bash
#
# Container Runtime Diagnostic Script
# Tests Docker capabilities and provides environment-specific recommendations
#

set -e

echo "🔍 Container Runtime Diagnostic"
echo "=============================="
echo ""

# Environment detection
ENVIRONMENT="Unknown"
if [ -n "$CODESPACES" ]; then
    ENVIRONMENT="GitHub Codespaces"
elif [ -n "$DEVCONTAINER" ]; then
    ENVIRONMENT="Dev Container"
elif [ -f "/.dockerenv" ]; then
    ENVIRONMENT="Docker Container"
else
    ENVIRONMENT="Local/VPS"
fi

echo "📋 Environment Information:"
echo "   Environment: $ENVIRONMENT"
echo "   User: $(whoami) ($(id -u):$(id -g))"
echo "   Kernel: $(uname -r)"
echo "   OS: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2 2>/dev/null || uname -s)"
echo ""

# Test 1: Docker daemon access
echo "🐳 Test 1: Docker Daemon Access"
if docker version >/dev/null 2>&1; then
    echo "   ✅ Docker daemon accessible"
    echo "   Docker Version: $(docker version --format '{{.Client.Version}}')"
    echo "   Server Version: $(docker version --format '{{.Server.Version}}')"
    
    # Get Docker info
    STORAGE_DRIVER=$(docker info --format '{{.Driver}}' 2>/dev/null || echo "unknown")
    echo "   Storage Driver: $STORAGE_DRIVER"
    
    # Check if user is in docker group
    if groups | grep -q docker; then
        echo "   ✅ User in docker group"
    else
        echo "   ⚠️ User not in docker group (may need sudo)"
    fi
else
    echo "   ❌ Docker daemon not accessible"
    echo "   This environment doesn't support Docker operations"
    exit 1
fi

echo ""

# Test 2: Simple container execution
echo "🚀 Test 2: Basic Container Execution"
if timeout 30 docker run --rm hello-world >/dev/null 2>&1; then
    echo "   ✅ Basic containers work"
    BASIC_CONTAINERS=true
else
    echo "   ❌ Basic container execution fails"
    BASIC_CONTAINERS=false
fi

echo ""

# Test 3: Image pulling capability
echo "📦 Test 3: Image Pull Capability"
echo "   Testing Alpine Linux pull..."
if timeout 60 docker pull alpine:latest >/dev/null 2>&1; then
    echo "   ✅ Image pulls work normally"
    IMAGE_PULL=true
    
    # Cleanup
    docker rmi alpine:latest >/dev/null 2>&1 || true
else
    echo "   ❌ Image pull fails (likely unshare restriction)"
    IMAGE_PULL=false
fi

echo ""

# Test 4: PostgreSQL container (specific to our use case)
echo "🐘 Test 4: PostgreSQL Container Test"
echo "   Attempting PostgreSQL container..."
PG_TEST_OUTPUT=$(mktemp)
if timeout 60 docker run --rm -d --name diagnostic-pg \
    -e POSTGRES_PASSWORD=test \
    -e POSTGRES_DB=test \
    postgres:15-alpine >$PG_TEST_OUTPUT 2>&1; then
    
    echo "   ✅ PostgreSQL container started successfully"
    POSTGRES_WORKS=true
    
    # Stop the container
    docker stop diagnostic-pg >/dev/null 2>&1 || true
else
    echo "   ❌ PostgreSQL container failed"
    echo "   Error: $(cat $PG_TEST_OUTPUT | head -3)"
    POSTGRES_WORKS=false
fi
rm -f $PG_TEST_OUTPUT

echo ""

# Test 5: Docker Compose availability
echo "🔧 Test 5: Docker Compose"
if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    echo "   ✅ Docker Compose available"
    echo "   Version: $(docker compose version --short)"
    COMPOSE_AVAILABLE=true
elif command -v docker-compose >/dev/null 2>&1; then
    echo "   ✅ Docker Compose (legacy) available"
    echo "   Version: $(docker-compose version --short)"
    COMPOSE_AVAILABLE=true
else
    echo "   ❌ Docker Compose not available"
    COMPOSE_AVAILABLE=false
fi

echo ""

# Test 6: Volume mounting
echo "💾 Test 6: Volume Mounting"
TEST_DIR=$(mktemp -d)
echo "test content" > "$TEST_DIR/test.txt"

if timeout 30 docker run --rm -v "$TEST_DIR:/test:ro" alpine:latest cat /test/test.txt >/dev/null 2>&1; then
    echo "   ✅ Volume mounting works"
    VOLUME_MOUNT=true
else
    echo "   ❌ Volume mounting fails"
    VOLUME_MOUNT=false
fi

rm -rf "$TEST_DIR"

echo ""

# Test 7: Privileged containers
echo "🔐 Test 7: Privileged Container Support"
if timeout 30 docker run --rm --privileged alpine:latest whoami >/dev/null 2>&1; then
    echo "   ✅ Privileged containers supported"
    PRIVILEGED_WORKS=true
else
    echo "   ❌ Privileged containers restricted"
    PRIVILEGED_WORKS=false
fi

echo ""

# Summary and Recommendations
echo "📊 SUMMARY AND RECOMMENDATIONS"
echo "==============================="
echo ""

# Calculate capability score
SCORE=0
$BASIC_CONTAINERS && SCORE=$((SCORE + 1))
$IMAGE_PULL && SCORE=$((SCORE + 2))
$POSTGRES_WORKS && SCORE=$((SCORE + 2))
$COMPOSE_AVAILABLE && SCORE=$((SCORE + 1))
$VOLUME_MOUNT && SCORE=$((SCORE + 1))
$PRIVILEGED_WORKS && SCORE=$((SCORE + 1))

echo "Docker Capability Score: $SCORE/8"
echo ""

if [ $SCORE -ge 7 ]; then
    echo "🏆 RECOMMENDATION: Use Docker Compose Setup"
    echo "   Your environment has excellent Docker support"
    echo "   ✅ Run: cd lab/config/supabase && docker compose up -d"
    echo ""
    echo "Next steps:"
    echo "   1. cd lab/config/supabase"
    echo "   2. docker compose up -d"
    echo "   3. ./lab/scripts/run_comprehensive_test.sh"
    
elif [ $SCORE -ge 4 ]; then
    echo "⚠️ RECOMMENDATION: Try Docker Compose, with External Fallback"
    echo "   Your environment has partial Docker support"
    echo "   Try Docker first, use external Supabase if issues occur"
    echo ""
    echo "Next steps:"
    echo "   1. Try: cd lab/config/supabase && docker compose up -d"
    echo "   2. If it fails: ./lab/scripts/setup_external_supabase.sh"
    
else
    echo "🌐 RECOMMENDATION: Use External Supabase Instance"
    echo "   Your environment has limited Docker support"
    echo "   External Supabase is more reliable for this setup"
    echo ""
    echo "Next steps:"
    echo "   1. ./lab/scripts/setup_external_supabase.sh"
    echo "   2. ./lab/scripts/test_external_supabase.sh"
    echo "   3. ./lab/scripts/validate_rust_adapter.sh"
fi

echo ""

# Environment-specific notes
if [ "$ENVIRONMENT" = "GitHub Codespaces" ]; then
    echo "📝 GitHub Codespaces Notes:"
    echo "   - Standard Codespaces have Docker limitations"
    echo "   - Premium machine types may have better support"
    echo "   - Consider upgrading machine type for better Docker support"
    echo "   - External Supabase is often more reliable"
elif [ "$ENVIRONMENT" = "Local/VPS" ]; then
    echo "📝 Local/VPS Notes:"
    echo "   - You should have full Docker support"
    echo "   - If Docker fails, check Docker installation"
    echo "   - Consider Docker Desktop on macOS/Windows"
fi

echo ""
echo "For detailed troubleshooting: lab/config/supabase/UPDATED_TROUBLESHOOTING.md"
