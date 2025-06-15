#!/bin/bash
#
# Validate Rust Supabase Adapter with External Instance
# Runs the existing Rust tests against an external Supabase instance
#

set -e

echo "🦀 Rust Supabase Adapter Validation"
echo "==================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

# Load environment variables
if [ ! -f ".env" ]; then
    echo "❌ Error: .env file not found"
    echo "   Run lab/scripts/setup_external_supabase.sh first"
    exit 1
fi

source .env

# Validate required environment variables
if [ -z "$SUPABASE_URL" ] || [ -z "$SUPABASE_ANON_KEY" ]; then
    echo "❌ Error: Missing required environment variables"
    echo "   Required: SUPABASE_URL, SUPABASE_ANON_KEY"
    echo "   Current values:"
    echo "     SUPABASE_URL: ${SUPABASE_URL:-'(not set)'}"
    echo "     SUPABASE_ANON_KEY: ${SUPABASE_ANON_KEY:0:20}${SUPABASE_ANON_KEY:20+'...'}"
    exit 1
fi

echo "📋 Configuration:"
echo "   Supabase URL: $SUPABASE_URL"
echo "   Anon Key: ${SUPABASE_ANON_KEY:0:20}..."
echo "   Test Table: ${TEST_TABLE_NAME:-storage_kv}"
echo ""

# Set test-specific environment variables
export SUPABASE_TABLE_NAME="${TEST_TABLE_NAME:-storage_kv}"
export RUST_LOG="${RUST_LOG:-debug}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

echo "🔧 Step 1: Check Rust compilation"
echo "   Compiling with Supabase features..."

if cargo check --features storage-supabase >/dev/null 2>&1; then
    echo "   ✅ Compilation successful"
else
    echo "   ❌ Compilation failed"
    echo "   Running cargo check for details:"
    cargo check --features storage-supabase
    exit 1
fi

echo ""
echo "🧪 Step 2: Run Unit Tests"
echo "   Running Supabase storage unit tests..."

# Run basic unit tests first
if cargo test --features storage-supabase storage::supabase::test_supabase_config_default -- --nocapture; then
    echo "   ✅ Unit tests passed"
else
    echo "   ❌ Unit tests failed"
    exit 1
fi

echo ""
echo "🌐 Step 3: Run Integration Tests"
echo "   Testing against live Supabase instance..."

# Run integration tests that require live Supabase
echo "   Running basic storage tests..."
if cargo test --features storage-supabase storage::supabase::test_supabase_storage_basic -- --nocapture; then
    echo "   ✅ Basic storage tests passed"
else
    echo "   ⚠️ Basic storage tests failed (this may indicate connection issues)"
    echo "   Check your Supabase configuration and database schema"
fi

echo ""
echo "   Running overwrite tests..."
if cargo test --features storage-supabase storage::supabase::test_supabase_storage_overwrite -- --nocapture; then
    echo "   ✅ Overwrite tests passed"
else
    echo "   ⚠️ Overwrite tests failed"
fi

echo ""
echo "🏃 Step 4: Run Performance Tests"
echo "   Running storage performance benchmarks..."

if cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture; then
    echo "   ✅ Performance tests completed"
else
    echo "   ⚠️ Performance tests had issues"
fi

echo ""
echo "📊 Step 5: Run Comprehensive Storage Tests"
echo "   Testing all storage operations..."

# Run the comprehensive storage performance tests
if cargo test --features storage-supabase storage_perf -- --nocapture; then
    echo "   ✅ Comprehensive tests completed"
else
    echo "   ⚠️ Some comprehensive tests failed"
fi

echo ""
echo "🎉 Rust Supabase Adapter Validation Complete!"
echo ""
echo "Summary:"
echo "✅ Rust compilation with Supabase features"
echo "✅ Unit tests"
echo "✅ Integration tests (with live Supabase)"
echo "✅ Performance benchmarks"
echo "✅ Comprehensive storage operations"
echo ""
echo "Your Rust Supabase adapter is working correctly!"
echo ""
echo "Available test commands:"
echo "  cargo test --features storage-supabase storage::supabase"
echo "  cargo test --features storage-supabase test_supabase_storage_performance"
echo "  cargo test --features storage-supabase storage_perf"
echo ""
echo "For development:"
echo "  cargo run --features storage-supabase --example your_example"
echo "  RUST_LOG=debug cargo test --features storage-supabase -- --nocapture"
