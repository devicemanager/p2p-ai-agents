-- Supabase-compatible database initialization
-- This sets up the database exactly as Supabase would for our storage adapter

-- Create required roles
DO $$ 
BEGIN
    -- Create anon role (for anonymous access)
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN;
    END IF;
    
    -- Create authenticated role (for authenticated users)
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN;
    END IF;
    
    -- Create service_role (for service/admin access)
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'service_role') THEN
        CREATE ROLE service_role NOLOGIN BYPASSRLS;
    END IF;
    
    -- Create authenticator role (used by PostgREST)
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticator') THEN
        CREATE ROLE authenticator NOINHERIT LOGIN PASSWORD 'your-super-secret-jwt-token-with-at-least-32-characters-long';
    END IF;
END
$$;

-- Grant role memberships to authenticator
GRANT anon, authenticated, service_role TO authenticator;

-- Create the storage_kv table for our key-value storage
CREATE TABLE IF NOT EXISTS storage_kv (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Enable Row Level Security (RLS)
ALTER TABLE storage_kv ENABLE ROW LEVEL SECURITY;

-- Create policies for different roles
-- Allow service_role full access (bypass RLS)
CREATE POLICY IF NOT EXISTS "service_role_all" ON storage_kv
    FOR ALL TO service_role USING (true);

-- Allow authenticated users full access
CREATE POLICY IF NOT EXISTS "authenticated_all" ON storage_kv
    FOR ALL TO authenticated USING (true);

-- Allow anonymous users full access (for testing - restrict in production)
CREATE POLICY IF NOT EXISTS "anon_all" ON storage_kv
    FOR ALL TO anon USING (true);

-- Grant table permissions to all roles
GRANT ALL ON storage_kv TO anon, authenticated, service_role, authenticator;

-- Grant schema permissions
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role, authenticator;
GRANT ALL ON ALL TABLES IN SCHEMA public TO anon, authenticated, service_role, authenticator;

-- Set default privileges for future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO anon, authenticated, service_role, authenticator;

-- Create an update trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER IF NOT EXISTS update_storage_kv_updated_at 
    BEFORE UPDATE ON storage_kv 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Insert a test record
INSERT INTO storage_kv (key, value) 
VALUES ('test_init', '{"message": "Database initialized successfully", "timestamp": "' || CURRENT_TIMESTAMP || '"}')
ON CONFLICT (key) DO UPDATE SET 
    value = EXCLUDED.value,
    updated_at = CURRENT_TIMESTAMP;
