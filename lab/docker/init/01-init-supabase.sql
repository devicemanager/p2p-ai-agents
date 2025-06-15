-- Initialize Supabase Lab Database for Performance Testing
-- This script sets up the necessary tables and configurations for storage testing

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create necessary roles for Supabase
DO $$ 
BEGIN
    -- Create anon role if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN NOINHERIT;
    END IF;
    
    -- Create authenticated role if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN NOINHERIT;
    END IF;
    
    -- Create service_role if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'service_role') THEN
        CREATE ROLE service_role NOLOGIN NOINHERIT;
    END IF;
END
$$;

-- Grant basic permissions
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON ALL TABLES IN SCHEMA public TO service_role;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO service_role;
GRANT ALL ON ALL FUNCTIONS IN SCHEMA public TO service_role;

-- Create the storage performance test table
CREATE TABLE IF NOT EXISTS storage_perf_test (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Grant permissions on the test table
GRANT ALL ON storage_perf_test TO service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON storage_perf_test TO authenticated;
GRANT SELECT ON storage_perf_test TO anon;

-- Create index for performance
CREATE INDEX IF NOT EXISTS idx_storage_perf_test_created_at ON storage_perf_test(created_at);
CREATE INDEX IF NOT EXISTS idx_storage_perf_test_metadata ON storage_perf_test USING GIN(metadata);

-- Create the main storage table used by the adapter
CREATE TABLE IF NOT EXISTS storage_kv (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Grant permissions on the main storage table
GRANT ALL ON storage_kv TO service_role;
GRANT SELECT, INSERT, UPDATE, DELETE ON storage_kv TO authenticated;
GRANT SELECT ON storage_kv TO anon;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_storage_kv_created_at ON storage_kv(created_at);
CREATE INDEX IF NOT EXISTS idx_storage_kv_metadata ON storage_kv USING GIN(metadata);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to both tables
DROP TRIGGER IF EXISTS update_storage_perf_test_updated_at ON storage_perf_test;
CREATE TRIGGER update_storage_perf_test_updated_at
    BEFORE UPDATE ON storage_perf_test
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_storage_kv_updated_at ON storage_kv;
CREATE TRIGGER update_storage_kv_updated_at
    BEFORE UPDATE ON storage_kv
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create a view for monitoring performance
CREATE OR REPLACE VIEW storage_performance_stats AS
SELECT 
    'storage_perf_test' as table_name,
    COUNT(*) as total_records,
    AVG(octet_length(data)) as avg_data_size,
    MIN(created_at) as oldest_record,
    MAX(created_at) as newest_record,
    NOW() - MIN(created_at) as test_duration
FROM storage_perf_test
UNION ALL
SELECT 
    'storage_kv' as table_name,
    COUNT(*) as total_records,
    AVG(octet_length(data)) as avg_data_size,
    MIN(created_at) as oldest_record,
    MAX(created_at) as newest_record,
    NOW() - MIN(created_at) as test_duration
FROM storage_kv;

-- Grant access to the view
GRANT SELECT ON storage_performance_stats TO anon, authenticated, service_role;

-- Insert some test data to verify setup
INSERT INTO storage_perf_test (id, data, metadata) VALUES 
    ('test-1', '\x48656c6c6f', '{"test": true, "setup": "init"}'),
    ('test-2', '\x576f726c64', '{"test": true, "setup": "init"}')
ON CONFLICT (id) DO NOTHING;

-- Create function to clean up test data
CREATE OR REPLACE FUNCTION cleanup_test_data()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM storage_perf_test WHERE id LIKE 'perf_%' OR id LIKE 'test-%';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    DELETE FROM storage_kv WHERE id LIKE 'perf_%' OR id LIKE 'test-%';
    GET DIAGNOSTICS deleted_count = deleted_count + ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Grant execute permission on cleanup function
GRANT EXECUTE ON FUNCTION cleanup_test_data() TO service_role;

-- Log successful initialization
DO $$
BEGIN
    RAISE NOTICE 'Supabase Lab Database initialized successfully';
    RAISE NOTICE 'Tables created: storage_perf_test, storage_kv';
    RAISE NOTICE 'Roles configured: anon, authenticated, service_role';
    RAISE NOTICE 'Performance monitoring view: storage_performance_stats';
    RAISE NOTICE 'Ready for performance testing!';
END
$$;
