# Lab Scripts - Supabase Storage Adapter Testing

This directory contains scripts for setting up, testing, and troubleshooting the Supabase storage adapter in various environments.

## Quick Start

For **GitHub Codespaces** or any environment where Docker has restrictions:

```bash
# 1. Setup external Supabase instance
./lab/scripts/setup_external_supabase.sh

# 2. Test the connection
./lab/scripts/test_external_supabase.sh

# 3. Validate Rust adapter
./lab/scripts/validate_rust_adapter.sh
```

For **Local Development** with Docker support:

```bash
# 1. Setup local Supabase stack
cd lab/config/supabase
docker compose up -d

# 2. Run comprehensive tests
./lab/scripts/run_comprehensive_test.sh
```

## Available Scripts

### External Supabase Setup
- **`setup_external_supabase.sh`** - Interactive setup for external Supabase instance
- **`test_external_supabase.sh`** - Test external Supabase connectivity and operations
- **`validate_rust_adapter.sh`** - Validate Rust adapter against external Supabase

### Local Database Setup
- **`setup_database.sh`** - Setup local PostgreSQL database
- **`run_comprehensive_test.sh`** - Run all storage adapter tests
- **`reset_environment.sh`** - Reset local development environment

### Diagnostic Tools
- **`diagnose_docker_runtime.sh`** - Test Docker capabilities and get environment-specific recommendations

## Quick Environment Assessment

Not sure which approach to use? Run the diagnostic:

```bash
./lab/scripts/diagnose_docker_runtime.sh
```

This will test your Docker capabilities and recommend the best setup approach:
- **Score 7-8/8**: Use Docker Compose
- **Score 4-6/8**: Try Docker Compose, fallback to External Supabase  
- **Score 0-3/8**: Use External Supabase

## Environment Recommendations

| Environment Type | Docker Score | Recommended Approach |
|------------------|--------------|---------------------|
| **GitHub Codespaces (Standard)** | 1/8 | External Supabase |
| **GitHub Codespaces (Enhanced)** | 4-8/8 | Docker or External |
| **VS Code + Docker Desktop** | 8/8 | Docker Compose |
| **Remote SSH + Docker** | 8/8 | Docker Compose |
| **Local Development** | 8/8 | Docker Compose |

## Environment Support

| Environment | Recommended Approach | Scripts to Use |
|-------------|---------------------|----------------|
| **GitHub Codespaces** | External Supabase | `setup_external_supabase.sh` → `test_external_supabase.sh` → `validate_rust_adapter.sh` |
| **Local Docker** | Docker Compose | `docker compose up` → `run_comprehensive_test.sh` |
| **Local Native** | Standalone Setup | `setup_database.sh` → `run_comprehensive_test.sh` |
| **CI/CD** | External Supabase | `setup_external_supabase.sh` (with env vars) |

## Script Details

### setup_external_supabase.sh

Interactive script that:
- ✅ Prompts for Supabase project credentials
- ✅ Validates URL and key formats
- ✅ Creates properly formatted `.env` file
- ✅ Tests basic API connectivity
- ✅ Verifies Rust compilation

**Usage:**
```bash
./lab/scripts/setup_external_supabase.sh
```

**Required Information:**
- Supabase Project URL (`https://your-ref.supabase.co`)
- Supabase Anon Key (JWT token from API settings)
- Supabase Service Role Key (JWT token from API settings)

### test_external_supabase.sh

Comprehensive test suite that:
- ✅ Tests API connectivity and authentication
- ✅ Creates test database schema if needed
- ✅ Runs basic CRUD operations
- ✅ Validates Rust adapter compilation
- ✅ Provides detailed error reporting

**Usage:**
```bash
./lab/scripts/test_external_supabase.sh
```

**Requirements:**
- `.env` file with Supabase configuration
- Internet connection to Supabase instance

### validate_rust_adapter.sh

Rust-specific validation that:
- ✅ Compiles Rust code with Supabase features
- ✅ Runs unit tests
- ✅ Runs integration tests against live Supabase
- ✅ Executes performance benchmarks
- ✅ Tests comprehensive storage operations

**Usage:**
```bash
./lab/scripts/validate_rust_adapter.sh
```

**Requirements:**
- Configured external Supabase instance
- Rust toolchain with required features

## Environment Variables

### Required for External Supabase
```bash
SUPABASE_URL=https://your-project-ref.supabase.co
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Optional Configuration
```bash
TEST_TABLE_NAME=test_storage          # Default: test_storage
TEST_BUCKET_NAME=test-bucket          # Default: test-bucket
RUST_LOG=debug                        # Rust logging level
RUST_BACKTRACE=1                      # Enable Rust backtraces
```

## Troubleshooting

### Common Issues

1. **"Docker: unshare: operation not permitted"**
   - **Solution**: Use external Supabase instead of Docker
   - **Scripts**: `setup_external_supabase.sh`

2. **"Connection refused" to Supabase**
   - **Check**: Internet connectivity
   - **Check**: Supabase URL format
   - **Check**: API keys are correct

3. **"Table does not exist"**
   - **Solution**: Run `test_external_supabase.sh` to create schema
   - **Alternative**: Create table manually in Supabase SQL Editor

4. **"Unauthorized" API errors**
   - **Check**: API keys are not expired
   - **Check**: Service role key has sufficient permissions
   - **Check**: RLS policies allow access

### Debug Mode

Run any script with detailed output:
```bash
RUST_LOG=debug ./lab/scripts/validate_rust_adapter.sh
```

### Manual Testing

Test individual components:
```bash
# Test API connectivity
curl -H "apikey: $SUPABASE_ANON_KEY" "$SUPABASE_URL/rest/v1/"

# Test Rust compilation
cargo check --features storage-supabase

# Run specific Rust test
cargo test --features storage-supabase storage::supabase::test_supabase_storage_basic -- --nocapture
```

## Integration with Development Workflow

### For New Developers

1. Clone repository
2. Run `./lab/scripts/setup_external_supabase.sh`
3. Start developing with Rust adapter

### For CI/CD

Set environment variables and run:
```bash
export SUPABASE_URL="https://your-ref.supabase.co"
export SUPABASE_ANON_KEY="your-anon-key"
export SUPABASE_SERVICE_ROLE_KEY="your-service-role-key"

./lab/scripts/test_external_supabase.sh
./lab/scripts/validate_rust_adapter.sh
```

### For Testing

All tests can be run with:
```bash
cargo test --features storage-supabase
```

Or specific test suites:
```bash
cargo test --features storage-supabase storage::supabase
cargo test --features storage-supabase test_supabase_storage_performance
```

## Related Documentation

- **[UPDATED_TROUBLESHOOTING.md](../config/supabase/UPDATED_TROUBLESHOOTING.md)** - Comprehensive troubleshooting guide
- **[../config/supabase/](../config/supabase/)** - Docker Compose configuration
- **[../../src/storage/supabase.rs](../../src/storage/supabase.rs)** - Rust adapter implementation

---

*For issues not covered here, see the comprehensive troubleshooting guide or create a new issue with detailed error output.*
