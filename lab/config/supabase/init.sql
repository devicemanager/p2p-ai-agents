-- Supabase Local Development Database Initialization
-- This script sets up the necessary tables and configurations for local testing

-- Enable necessary extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create auth schema for Supabase Auth
CREATE SCHEMA IF NOT EXISTS auth;

-- Create storage schema for Supabase Storage
CREATE SCHEMA IF NOT EXISTS storage;

-- Create realtime schema for Supabase Realtime (optional)
CREATE SCHEMA IF NOT EXISTS realtime;

-- Create necessary roles for Supabase
DO
$do$
BEGIN
   -- Anonymous role
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'anon') THEN
      CREATE ROLE anon NOLOGIN NOINHERIT;
   END IF;
   
   -- Authenticated role
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticated') THEN
      CREATE ROLE authenticated NOLOGIN NOINHERIT;
   END IF;
   
   -- Service role
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'service_role') THEN
      CREATE ROLE service_role NOLOGIN NOINHERIT BYPASSRLS;
   END IF;
   
   -- Authenticator role (used by PostgREST)
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'authenticator') THEN
      CREATE ROLE authenticator NOINHERIT LOGIN PASSWORD 'your-super-secret-jwt-token-with-at-least-32-characters-long';
   END IF;
   
   -- Supabase admin roles
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'supabase_auth_admin') THEN
      CREATE ROLE supabase_auth_admin NOINHERIT CREATEROLE LOGIN PASSWORD 'your-super-secret-jwt-token-with-at-least-32-characters-long';
   END IF;
   
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'supabase_storage_admin') THEN
      CREATE ROLE supabase_storage_admin NOINHERIT CREATEROLE LOGIN PASSWORD 'your-super-secret-jwt-token-with-at-least-32-characters-long';
   END IF;
END
$do$;

-- Grant necessary permissions
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON SCHEMA public TO service_role;

-- Create storage performance test table
CREATE TABLE IF NOT EXISTS public.storage_perf_test (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create additional test table for key-value storage
CREATE TABLE IF NOT EXISTS public.storage_kv (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index for better performance
CREATE INDEX IF NOT EXISTS idx_storage_perf_test_created_at ON public.storage_perf_test(created_at);
CREATE INDEX IF NOT EXISTS idx_storage_kv_created_at ON public.storage_kv(created_at);

-- Enable Row Level Security (RLS) for production-like environment
ALTER TABLE public.storage_perf_test ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.storage_kv ENABLE ROW LEVEL SECURITY;

-- Create policies for testing
CREATE POLICY IF NOT EXISTS "Enable read access for all users" ON public.storage_perf_test
    FOR SELECT USING (true);

CREATE POLICY IF NOT EXISTS "Enable insert access for all users" ON public.storage_perf_test
    FOR INSERT WITH CHECK (true);

CREATE POLICY IF NOT EXISTS "Enable update access for all users" ON public.storage_perf_test
    FOR UPDATE USING (true);

CREATE POLICY IF NOT EXISTS "Enable delete access for all users" ON public.storage_perf_test
    FOR DELETE USING (true);

CREATE POLICY IF NOT EXISTS "Enable read access for all users" ON public.storage_kv
    FOR SELECT USING (true);

CREATE POLICY IF NOT EXISTS "Enable insert access for all users" ON public.storage_kv
    FOR INSERT WITH CHECK (true);

CREATE POLICY IF NOT EXISTS "Enable update access for all users" ON public.storage_kv
    FOR UPDATE USING (true);

CREATE POLICY IF NOT EXISTS "Enable delete access for all users" ON public.storage_kv
    FOR DELETE USING (true);

-- Grant table permissions to roles
GRANT ALL ON public.storage_perf_test TO anon, authenticated, service_role;
GRANT ALL ON public.storage_kv TO anon, authenticated, service_role;

-- Create some sample data for testing
INSERT INTO public.storage_perf_test (id, data) VALUES 
    ('test-1', '\x48656c6c6f20576f726c64'),  -- "Hello World" in hex
    ('test-2', '\x54657374204461746120322e3020302e31'),  -- "Test Data 2.0 0.1" in hex
    ('test-3', '\x5375706162617365204c6f63616c20546573742044617461')  -- "Supabase Local Test Data" in hex
ON CONFLICT (id) DO NOTHING;

-- Create function to clean up test data
CREATE OR REPLACE FUNCTION cleanup_perf_test_data()
RETURNS void AS $$
BEGIN
    DELETE FROM public.storage_perf_test WHERE id LIKE 'perf_%';
    DELETE FROM public.storage_kv WHERE id LIKE 'perf_%';
END;
$$ LANGUAGE plpgsql;

-- Grant execute permission on the cleanup function
GRANT EXECUTE ON FUNCTION cleanup_perf_test_data() TO anon, authenticated, service_role;

-- Create function to get performance test statistics
CREATE OR REPLACE FUNCTION get_perf_test_stats()
RETURNS TABLE(
    table_name TEXT,
    row_count BIGINT,
    total_size TEXT,
    avg_data_size NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        'storage_perf_test'::TEXT,
        COUNT(*),
        pg_size_pretty(pg_total_relation_size('public.storage_perf_test')),
        AVG(LENGTH(data))
    FROM public.storage_perf_test
    UNION ALL
    SELECT 
        'storage_kv'::TEXT,
        COUNT(*),
        pg_size_pretty(pg_total_relation_size('public.storage_kv')),
        AVG(LENGTH(data))
    FROM public.storage_kv;
END;
$$ LANGUAGE plpgsql;

-- Grant execute permission on the stats function
GRANT EXECUTE ON FUNCTION get_perf_test_stats() TO anon, authenticated, service_role;

-- Log successful initialization
SELECT 'Supabase local database initialized successfully!' as status;
