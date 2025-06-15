#!/bin/bash
# Comprehensive Supabase Database Setup Script

set -e

echo "🔧 Setting up PostgreSQL for Supabase Storage Testing..."

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if running as correct user
if [[ $EUID -eq 0 ]]; then
    print_error "Do not run this script as root"
    exit 1
fi

# Start PostgreSQL service
print_status "Starting PostgreSQL service..."
if sudo service postgresql start; then
    print_success "PostgreSQL service started"
else
    print_error "Failed to start PostgreSQL service"
    exit 1
fi

# Wait for PostgreSQL to be ready
print_status "Waiting for PostgreSQL to be ready..."
for i in {1..10}; do
    if sudo -u postgres psql -c "SELECT 1;" > /dev/null 2>&1; then
        print_success "PostgreSQL is ready"
        break
    fi
    print_warning "Waiting for PostgreSQL... ($i/10)"
    sleep 2
    if [[ $i -eq 10 ]]; then
        print_error "PostgreSQL failed to start properly"
        exit 1
    fi
done

# Create database if it doesn't exist
print_status "Creating supabase_lab database..."
if sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw supabase_lab; then
    print_warning "Database supabase_lab already exists"
else
    if sudo -u postgres createdb supabase_lab; then
        print_success "Database supabase_lab created"
    else
        print_error "Failed to create database"
        exit 1
    fi
fi

# Create roles and setup permissions
print_status "Setting up database roles and permissions..."
sudo -u postgres psql -d supabase_lab << 'EOF'
-- Create roles if they don't exist
DO $$ 
BEGIN
    -- Create anon role
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN;
        RAISE NOTICE 'Created role: anon';
    ELSE
        RAISE NOTICE 'Role anon already exists';
    END IF;
    
    -- Create authenticated role
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN;
        RAISE NOTICE 'Created role: authenticated';
    ELSE
        RAISE NOTICE 'Role authenticated already exists';
    END IF;
    
    -- Create service_role
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'service_role') THEN
        CREATE ROLE service_role NOLOGIN;
        RAISE NOTICE 'Created role: service_role';
    ELSE
        RAISE NOTICE 'Role service_role already exists';
    END IF;
    
    -- Create authenticator role
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticator') THEN
        CREATE ROLE authenticator NOINHERIT LOGIN PASSWORD 'your-super-secret-jwt-token-with-at-least-32-characters-long';
        RAISE NOTICE 'Created role: authenticator';
    ELSE
        RAISE NOTICE 'Role authenticator already exists';
    END IF;
END
$$;

-- Grant role memberships
GRANT anon, authenticated, service_role TO authenticator;

-- Create storage_kv table if it doesn't exist
CREATE TABLE IF NOT EXISTS storage_kv (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Grant permissions on table
GRANT ALL ON storage_kv TO anon;
GRANT ALL ON storage_kv TO authenticated;
GRANT ALL ON storage_kv TO service_role;
GRANT ALL ON storage_kv TO authenticator;

-- Enable Row Level Security
ALTER TABLE storage_kv ENABLE ROW LEVEL SECURITY;

-- Drop existing policies if they exist
DROP POLICY IF EXISTS "service_role_all" ON storage_kv;
DROP POLICY IF EXISTS "authenticated_all" ON storage_kv;
DROP POLICY IF EXISTS "anon_select" ON storage_kv;

-- Create RLS policies
-- Allow all operations for service_role
CREATE POLICY "service_role_all" ON storage_kv
    FOR ALL TO service_role USING (true);

-- Allow all operations for authenticated users
CREATE POLICY "authenticated_all" ON storage_kv
    FOR ALL TO authenticated USING (true);

-- Allow all operations for anon users (for testing)
CREATE POLICY "anon_all" ON storage_kv
    FOR ALL TO anon USING (true);

-- Grant usage on schema
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role, authenticator;
GRANT ALL ON ALL TABLES IN SCHEMA public TO anon, authenticated, service_role, authenticator;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO anon, authenticated, service_role, authenticator;

-- Set default privileges
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO anon, authenticated, service_role, authenticator;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO anon, authenticated, service_role, authenticator;

EOF

if [[ $? -eq 0 ]]; then
    print_success "Database roles and permissions configured"
else
    print_error "Failed to setup database roles and permissions"
    exit 1
fi

# Verify the setup
print_status "Verifying database setup..."

# Check roles
print_status "Checking database roles..."
ROLES=$(sudo -u postgres psql -d supabase_lab -tAc "SELECT rolname FROM pg_roles WHERE rolname IN ('anon', 'authenticated', 'service_role', 'authenticator');")
echo "Created roles: $ROLES"

# Check table
print_status "Checking storage_kv table..."
TABLE_EXISTS=$(sudo -u postgres psql -d supabase_lab -tAc "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'storage_kv');")
if [[ "$TABLE_EXISTS" == "t" ]]; then
    print_success "Table storage_kv exists"
else
    print_error "Table storage_kv does not exist"
    exit 1
fi

# Check permissions
print_status "Checking table permissions..."
PERMS=$(sudo -u postgres psql -d supabase_lab -tAc "SELECT grantee, privilege_type FROM information_schema.table_privileges WHERE table_name='storage_kv' AND grantee IN ('anon', 'authenticated', 'service_role', 'authenticator');")
echo "Table permissions:"
echo "$PERMS"

# Test basic insert/select
print_status "Testing basic database operations..."
sudo -u postgres psql -d supabase_lab << 'EOF'
-- Test insert
INSERT INTO storage_kv (key, value) VALUES ('test_key', '{"test": "value"}') ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- Test select
SELECT key, value, created_at FROM storage_kv WHERE key = 'test_key';
EOF

if [[ $? -eq 0 ]]; then
    print_success "Basic database operations working"
else
    print_error "Basic database operations failed"
    exit 1
fi

print_success "Database setup completed successfully!"
print_status "Summary:"
echo "  - Database: supabase_lab"
echo "  - Table: storage_kv (key TEXT, value JSONB, created_at TIMESTAMP, updated_at TIMESTAMP)"
echo "  - Roles: anon, authenticated, service_role, authenticator"
echo "  - RLS: Enabled with permissive policies for testing"
echo "  - Permissions: Full access granted to all roles"
print_status "Next step: Start PostgREST with the configuration file"
