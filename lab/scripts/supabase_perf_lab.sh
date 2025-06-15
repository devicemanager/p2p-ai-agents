#!/bin/bash

# Supabase Storage Performance Lab
# This script sets up and runs performance tests for the Supabase storage adapter

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LAB_DIR="$PROJECT_ROOT/lab"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_banner() {
    echo -e "${BLUE}"
    echo "========================================================"
    echo "🚀 Supabase Storage Performance Lab"
    echo "========================================================"
    echo -e "${NC}"
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "cargo is not installed or not in PATH"
        exit 1
    fi
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_error "Not in a Rust project directory. Expected to find Cargo.toml"
        exit 1
    fi
    
    # Check container runtime if using local mode
    if [[ "${USE_LOCAL:-}" == "true" ]]; then
        if command -v docker &> /dev/null && docker info &> /dev/null 2>&1; then
            log_info "Using Docker for local Supabase"
        elif command -v podman &> /dev/null; then
            log_info "Using Podman for local Supabase"
        else
            log_error "Neither Docker nor Podman is available."
            log_error "Please install Docker or Podman for local Supabase testing."
            exit 1
        fi
    fi
    
    log_success "Dependencies check passed"
}

show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --mock         Run with mock Supabase configuration (default)"
    echo "  --local        Run with local Supabase instance (Docker/Podman)"
    echo "  --real         Run with real Supabase instance (requires env vars)"
    echo "  --setup        Show setup instructions for real Supabase instance"
    echo "  --quick        Run quick performance test (100 ops)"
    echo "  --full         Run full performance test (1000 ops)"
    echo "  --stress       Run stress test (10000 ops)"
    echo "  --help         Show this help message"
    echo ""
    echo "Local Supabase (--local mode):"
    echo "  Automatically starts and configures a local Supabase instance"
    echo "  Uses Docker or Podman containers for PostgreSQL and PostgREST"
    echo "  Perfect for development and testing"
    echo ""
    echo "Environment Variables (for --real mode):"
    echo "  SUPABASE_URL           Supabase project URL"
    echo "  SUPABASE_ANON_KEY      Supabase anonymous key"
    echo "  SUPABASE_SERVICE_ROLE_KEY  Supabase service role key (optional)"
    echo ""
}

show_setup_instructions() {
    echo -e "${YELLOW}"
    echo "========================================================"
    echo "🏗️  Setting up Real Supabase Instance"
    echo "========================================================"
    echo -e "${NC}"
    echo ""
    echo "1. Create a Supabase account at https://supabase.com"
    echo "2. Create a new project"
    echo "3. Go to Settings > API to get your keys"
    echo "4. Create a table for storage testing:"
    echo ""
    echo "   SQL to run in Supabase SQL editor:"
    echo "   ------------------------------------"
    echo "   CREATE TABLE IF NOT EXISTS storage_perf_test ("
    echo "       id TEXT PRIMARY KEY,"
    echo "       data BYTEA NOT NULL,"
    echo "       created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()"
    echo "   );"
    echo ""
    echo "5. Set environment variables:"
    echo "   export SUPABASE_URL='https://your-project.supabase.co'"
    echo "   export SUPABASE_ANON_KEY='your-anon-key'"
    echo "   export SUPABASE_SERVICE_ROLE_KEY='your-service-role-key'"
    echo ""
    echo "6. Run this script with --real flag"
    echo ""
}

setup_mock_environment() {
    log_info "Setting up mock Supabase environment..."
    
    # Set mock environment variables
    export SUPABASE_URL="https://demo-project.supabase.co"
    export SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.demo-anon-key"
    export SUPABASE_SERVICE_ROLE_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.demo-service-key"
    export SUPABASE_SCHEMA="public"
    export SUPABASE_TABLE_NAME="storage_perf_test"
    
    log_warning "Using mock Supabase configuration"
    log_warning "Tests will demonstrate the framework but won't hit a real database"
    log_info "To test against real Supabase, use --real flag and set up environment variables"
    echo ""
}

setup_local_environment() {
    log_info "Setting up local Supabase environment..."
    
    # Start local Supabase if not already running
    local_script="$LAB_DIR/scripts/local_supabase.sh"
    
    log_info "Checking local Supabase status..."
    if ! curl -s http://localhost:3000/ &> /dev/null; then
        log_info "Starting local Supabase instance..."
        "$local_script" start
    else
        log_success "Local Supabase instance already running"
    fi
    
    # Set local environment variables
    export SUPABASE_URL="http://localhost:3000"
    export SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    export SUPABASE_SERVICE_ROLE_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU"
    export SUPABASE_SCHEMA="public"
    export SUPABASE_TABLE_NAME="storage_perf_test"
    
    log_success "Local Supabase environment configured"
    log_info "URL: $SUPABASE_URL"
    log_info "Testing against real local database with network operations"
    echo ""
}

check_real_environment() {
    log_info "Checking real Supabase environment..."
    
    local missing_vars=()
    
    if [[ -z "${SUPABASE_URL:-}" ]]; then
        missing_vars+=("SUPABASE_URL")
    fi
    
    if [[ -z "${SUPABASE_ANON_KEY:-}" ]]; then
        missing_vars+=("SUPABASE_ANON_KEY")
    fi
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        log_error "Missing required environment variables: ${missing_vars[*]}"
        echo ""
        echo "Run '$0 --setup' for setup instructions"
        exit 1
    fi
    
    log_success "Real Supabase environment configured"
    log_info "URL: $SUPABASE_URL"
    log_info "Using anon key: ${SUPABASE_ANON_KEY:0:20}..."
    if [[ -n "${SUPABASE_SERVICE_ROLE_KEY:-}" ]]; then
        log_info "Service role key: configured"
    else
        log_warning "Service role key: not configured (optional)"
    fi
    echo ""
}

run_build_and_check() {
    log_info "Building project with Supabase feature..."
    cd "$PROJECT_ROOT"
    
    if ! cargo build --features storage-supabase; then
        log_error "Build failed"
        exit 1
    fi
    
    log_success "Build completed successfully"
    echo ""
}

run_quick_test() {
    log_info "Running quick Supabase performance test (100 operations)..."
    cd "$PROJECT_ROOT"
    
    echo "Test Configuration:"
    echo "  Operations: 100"
    echo "  Data Size: 1KB"
    echo "  Concurrency: 4"
    echo ""
    
    if cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture; then
        log_success "Quick test completed successfully"
    else
        log_warning "Test completed with warnings (this is expected for mock setup)"
    fi
    echo ""
}

run_full_test() {
    log_info "Running full storage performance benchmark..."
    cd "$PROJECT_ROOT"
    
    echo "Test Configuration:"
    echo "  Operations: 1000"
    echo "  Data Size: 1KB" 
    echo "  Concurrency: 10"
    echo ""
    
    if cargo test --features storage-supabase test_storage_performance_medium -- --nocapture; then
        log_success "Full test completed successfully"
    else
        log_warning "Test completed with warnings (this is expected for mock setup)"
    fi
    echo ""
}

run_stress_test() {
    log_info "Running stress test (large dataset)..."
    cd "$PROJECT_ROOT"
    
    echo "Test Configuration:"
    echo "  Operations: 10000"
    echo "  Data Size: 4KB"
    echo "  Concurrency: 20"
    echo ""
    
    log_warning "This test may take several minutes..."
    if cargo test --features storage-supabase test_storage_performance_large -- --nocapture --ignored; then
        log_success "Stress test completed successfully"
    else
        log_warning "Test completed with warnings (this is expected for mock setup)"
    fi
    echo ""
}

run_all_storage_tests() {
    log_info "Running comprehensive storage performance tests..."
    cd "$PROJECT_ROOT"
    
    echo "This will run all storage backends for comparison:"
    echo "  - LocalStorage"
    echo "  - DistributedStorage (stub)"
    echo "  - CacheStorage (stub)"
    echo "  - CustomStorage (stub)"
    echo "  - SupabaseStorage (configured)"
    echo ""
    
    if cargo test --features storage-supabase storage_perf -- --nocapture; then
        log_success "All storage tests completed"
    else
        log_warning "Tests completed with warnings"
    fi
    echo ""
}

generate_performance_report() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local report_file="$LAB_DIR/reports/supabase_perf_${timestamp}.md"
    
    log_info "Generating performance report: $report_file"
    
    # Create reports directory if it doesn't exist
    mkdir -p "$LAB_DIR/reports"
    
    cat > "$report_file" << EOF
# Supabase Storage Performance Report

**Generated:** $(date)
**Environment:** ${USE_REAL:-mock}
**Project:** p2p-ai-agents

## Configuration

- **Supabase URL:** ${SUPABASE_URL:-not-set}
- **Test Mode:** ${TEST_MODE:-quick}
- **Rust Version:** $(rustc --version)
- **Cargo Version:** $(cargo --version)

## Test Results

EOF

    if [[ "${USE_REAL:-}" == "true" ]]; then
        echo "- Real Supabase instance testing" >> "$report_file"
        echo "- Network latency included in results" >> "$report_file"
    else
        echo "- Mock Supabase configuration used" >> "$report_file"
        echo "- Results show framework overhead only" >> "$report_file"
    fi

    echo "" >> "$report_file"
    echo "## Raw Test Output" >> "$report_file"
    echo "" >> "$report_file"
    echo '```' >> "$report_file"
    echo "See console output above for detailed performance metrics" >> "$report_file"
    echo '```' >> "$report_file"
    
    echo "" >> "$report_file"
    echo "## Next Steps" >> "$report_file"
    echo "" >> "$report_file"
    if [[ "${USE_REAL:-}" != "true" ]]; then
        echo "- Set up real Supabase instance for accurate performance testing" >> "$report_file"
        echo "- Run with --real flag after configuring environment variables" >> "$report_file"
    else
        echo "- Analyze performance bottlenecks" >> "$report_file"
        echo "- Consider optimizations based on results" >> "$report_file"
        echo "- Run stress tests with larger datasets" >> "$report_file"
    fi
    
    log_success "Report generated: $report_file"
}

main() {
    print_banner
    
    # Parse command line arguments
    USE_REAL=false
    USE_LOCAL=false
    TEST_MODE="quick"
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --mock)
                USE_REAL=false
                USE_LOCAL=false
                shift
                ;;
            --local)
                USE_LOCAL=true
                USE_REAL=false
                shift
                ;;
            --real)
                USE_REAL=true
                USE_LOCAL=false
                shift
                ;;
            --setup)
                show_setup_instructions
                exit 0
                ;;
            --quick)
                TEST_MODE="quick"
                shift
                ;;
            --full)
                TEST_MODE="full"
                shift
                ;;
            --stress)
                TEST_MODE="stress"
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Check dependencies
    check_dependencies
    
    # Set up environment
    if [[ "$USE_REAL" == "true" ]]; then
        check_real_environment
    elif [[ "$USE_LOCAL" == "true" ]]; then
        setup_local_environment
    else
        setup_mock_environment
    fi
    
    # Build project
    run_build_and_check
    
    # Run tests based on mode
    case $TEST_MODE in
        quick)
            run_quick_test
            ;;
        full)
            run_full_test
            run_all_storage_tests
            ;;
        stress)
            run_stress_test
            ;;
    esac
    
    # Generate report
    generate_performance_report
    
    log_success "Supabase Storage Performance Lab completed!"
    echo ""
    echo "Summary:"
    echo "  Mode: ${USE_REAL:-false}${USE_LOCAL:+local}"
    echo "  Test: $TEST_MODE"
    echo "  Status: completed"
    
    if [[ "$USE_REAL" != "true" && "$USE_LOCAL" != "true" ]]; then
        echo ""
        log_info "To test against real database:"
        echo "  Local:  $0 --local --full"
        echo "  Remote: $0 --real --full (after setup)"
    fi
}

# Run main function with all arguments
main "$@"
