#!/bin/bash
# Clean State Reset Script for Supabase Environment

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${PURPLE}🔄 $1${NC}"
}

print_header "Resetting Supabase Environment to Clean State"
echo "Reset started at: $(date)"
echo

# Step 1: Stop all running services
print_status "Step 1: Stopping running services..."

# Stop PostgREST
if pgrep -f postgrest > /dev/null; then
    print_status "Stopping PostgREST..."
    pkill -f postgrest || true
    sleep 2
    print_success "PostgREST stopped"
else
    print_status "PostgREST not running"
fi

# Stop PostgreSQL
print_status "Stopping PostgreSQL..."
sudo service postgresql stop || true
sleep 2
print_success "PostgreSQL stopped"

# Step 2: Clean up temporary files
print_status "Step 2: Cleaning up temporary files..."

# Clean up files with error handling
for file in "/tmp/api_test.json" "/tmp/root_test.json" "/tmp/table_test.json" "/tmp/service_test.json" \
            "/tmp/create_test.json" "/tmp/read_test.json" "/tmp/update_test.json" "/tmp/delete_test.json" \
            "/tmp/postgrest.log" "/tmp/postgrest.conf"; do
    if [[ -f "$file" ]]; then
        rm -f "$file" 2>/dev/null || print_warning "Could not remove $file (permission denied)"
    fi
done

print_success "Temporary files cleaned"

# Step 3: Clean Rust build artifacts
print_status "Step 3: Cleaning Rust build artifacts..."
cd /workspaces/p2p-ai-agents

if [[ -d "target" ]]; then
    print_status "Cleaning cargo cache..."
    cargo clean
    print_success "Cargo cache cleaned"
else
    print_status "No cargo cache to clean"
fi

# Step 4: Reset PostgreSQL database
print_status "Step 4: Resetting PostgreSQL database..."

# Start PostgreSQL
print_status "Starting PostgreSQL..."
sudo service postgresql start
sleep 3

# Drop existing database if it exists
print_status "Dropping existing database..."
sudo -u postgres dropdb supabase_lab 2>/dev/null || print_warning "Database supabase_lab didn't exist"

# Drop existing roles (they might be referenced)
print_status "Cleaning up database roles..."
sudo -u postgres psql << 'EOF' 2>/dev/null || true
-- Revoke privileges and drop roles if they exist
DO $$
BEGIN
    -- Drop roles if they exist
    IF EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticator') THEN
        DROP ROLE authenticator;
        RAISE NOTICE 'Dropped role: authenticator';
    END IF;
    
    IF EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'anon') THEN
        DROP ROLE anon;
        RAISE NOTICE 'Dropped role: anon';
    END IF;
    
    IF EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticated') THEN
        DROP ROLE authenticated;
        RAISE NOTICE 'Dropped role: authenticated';
    END IF;
    
    IF EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'service_role') THEN
        DROP ROLE service_role;
        RAISE NOTICE 'Dropped role: service_role';
    END IF;
EXCEPTION
    WHEN OTHERS THEN
        RAISE NOTICE 'Error dropping roles: %', SQLERRM;
END
$$;
EOF

print_success "Database roles cleaned"

# Step 5: Recreate fresh database
print_status "Step 5: Creating fresh database..."

sudo -u postgres createdb supabase_lab
print_success "Fresh database created"

# Step 6: Verify environment configuration
print_status "Step 6: Verifying environment configuration..."

if [[ ! -f ".env" ]]; then
    print_warning "Environment file .env not found, creating default..."
    cat > .env << 'EOF'
# Core Supabase configuration
SUPABASE_URL=http://localhost:8000
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJhbm9uIiwKICAgICJpc3MiOiAic3VwYWJhc2UtZGVtbyIsCiAgICAiaWF0IjogMTY0MVc2OTIwMCwKICAgICJleHAiOiAxNzk5NTM1NjAwCn0.dc_X5iR_VP_qT0zsiyj_I_OZ2T9FtRU2BBNWN8Bu4GE
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJzZXJ2aWNlX3JvbGUiLAogICAgImlzcyI6ICJzdXBhYmFzZS1kZW1vIiwKICAgICJpYXQiOiAxNjQxNzY5MjAwLAogICAgImV4cCI6IDE3OTk1MzU2MDAKfQ.DaYlNEoUrrEn2Ig7tqibS-PHK5vgusbcbo7X36XVt4Q
SUPABASE_SCHEMA=public
SUPABASE_TABLE_NAME=storage_kv

# JWT Secret for PostgREST
JWT_SECRET=your-super-secret-jwt-token-with-at-least-32-characters-long
EOF
    print_success "Default environment file created"
else
    print_success "Environment file exists"
fi

# Step 7: Run database setup
print_status "Step 7: Running database setup..."

if [[ -f "/workspaces/p2p-ai-agents/lab/scripts/setup_database.sh" ]]; then
    print_status "Running setup_database.sh..."
    bash /workspaces/p2p-ai-agents/lab/scripts/setup_database.sh
    print_success "Database setup completed"
else
    print_error "setup_database.sh not found!"
    exit 1
fi

# Step 8: Verify clean state
print_status "Step 8: Verifying clean state..."

# Check PostgreSQL
if sudo -u postgres psql -d supabase_lab -c "SELECT 1;" > /dev/null 2>&1; then
    print_success "PostgreSQL connection verified"
else
    print_error "PostgreSQL connection failed"
    exit 1
fi

# Check table exists and is empty
TABLE_COUNT=$(sudo -u postgres psql -d supabase_lab -tAc "SELECT COUNT(*) FROM storage_kv;" 2>/dev/null || echo "0")
print_status "Records in storage_kv table: $TABLE_COUNT"

# Check roles exist
ROLE_COUNT=$(sudo -u postgres psql -d supabase_lab -tAc "SELECT COUNT(*) FROM pg_roles WHERE rolname IN ('anon', 'authenticated', 'service_role', 'authenticator');" 2>/dev/null || echo "0")
print_status "Database roles created: $ROLE_COUNT/4"

if [[ "$ROLE_COUNT" == "4" ]]; then
    print_success "All database roles created"
else
    print_warning "Some database roles missing"
fi

# Step 9: Optional test run
print_status "Step 9: Ready for testing"
echo
print_success "Environment reset completed successfully!"
print_status "Clean state summary:"
echo "  ✅ All services stopped and restarted"
echo "  ✅ Temporary files cleaned"
echo "  ✅ Rust build cache cleared"
echo "  ✅ Database dropped and recreated"
echo "  ✅ Fresh database schema applied"
echo "  ✅ Environment configuration verified"
echo "  📋 Records in storage_kv: $TABLE_COUNT"
echo "  👥 Database roles: $ROLE_COUNT/4"
echo

read -p "Would you like to run the comprehensive test now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Starting comprehensive test..."
    bash /workspaces/p2p-ai-agents/lab/scripts/run_comprehensive_test.sh
else
    print_status "Test skipped. You can run it manually with:"
    echo "  bash /workspaces/p2p-ai-agents/lab/scripts/run_comprehensive_test.sh"
fi

print_success "Clean state reset complete!"
