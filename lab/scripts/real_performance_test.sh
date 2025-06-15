#!/bin/bash

# Real Supabase/PostgREST Performance Test
# This script tests the actual performance of our local PostgREST setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
BASE_URL="http://localhost:3000"
TABLE_NAME="test_storage"
NUM_OPERATIONS=500
DATA_SIZE=1024

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
    echo "🚀 Real Supabase/PostgREST Performance Test"
    echo "========================================================"
    echo -e "${NC}"
    echo "Configuration:"
    echo "  Base URL: $BASE_URL"
    echo "  Table: $TABLE_NAME"
    echo "  Operations: $NUM_OPERATIONS"
    echo "  Data Size: $DATA_SIZE bytes"
    echo ""
}

check_api() {
    log_info "Checking API availability..."
    if curl -s -f "$BASE_URL/$TABLE_NAME" > /dev/null; then
        log_success "API is responding"
    else
        log_error "API is not responding at $BASE_URL"
        exit 1
    fi
}

generate_test_data() {
    # Generate random string of specified size
    head -c $DATA_SIZE /dev/urandom | base64 | tr -d '\n'
}

test_write_performance() {
    log_info "Testing WRITE performance..."
    
    start_time=$(date +%s.%N)
    
    for i in $(seq 1 $NUM_OPERATIONS); do
        test_data=$(generate_test_data)
        key="perf_write_test_$i"
        
        curl -s -X POST "$BASE_URL/$TABLE_NAME" \
            -H "Content-Type: application/json" \
            -d "{\"key\": \"$key\", \"value\": \"$test_data\"}" > /dev/null
        
        if [ $((i % 25)) -eq 0 ]; then
            echo -n "."
        fi
    done
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    ops_per_second=$(echo "scale=2; $NUM_OPERATIONS / $duration" | bc)
    avg_latency_ms=$(echo "scale=2; $duration * 1000 / $NUM_OPERATIONS" | bc)
    
    echo ""
    log_success "WRITE Test Completed"
    echo "  Operations: $NUM_OPERATIONS"
    echo "  Duration: ${duration}s"
    echo "  Throughput: $ops_per_second ops/s"
    echo "  Avg Latency: ${avg_latency_ms}ms"
    echo ""
}

test_read_performance() {
    log_info "Testing READ performance..."
    
    start_time=$(date +%s.%N)
    
    for i in $(seq 1 $NUM_OPERATIONS); do
        key="perf_write_test_$i"
        
        curl -s "$BASE_URL/$TABLE_NAME?key=eq.$key" > /dev/null
        
        if [ $((i % 25)) -eq 0 ]; then
            echo -n "."
        fi
    done
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    ops_per_second=$(echo "scale=2; $NUM_OPERATIONS / $duration" | bc)
    avg_latency_ms=$(echo "scale=2; $duration * 1000 / $NUM_OPERATIONS" | bc)
    
    echo ""
    log_success "READ Test Completed"
    echo "  Operations: $NUM_OPERATIONS"
    echo "  Duration: ${duration}s"
    echo "  Throughput: $ops_per_second ops/s"
    echo "  Avg Latency: ${avg_latency_ms}ms"
    echo ""
}

test_delete_performance() {
    log_info "Testing DELETE performance..."
    
    start_time=$(date +%s.%N)
    
    for i in $(seq 1 $NUM_OPERATIONS); do
        key="perf_write_test_$i"
        
        curl -s -X DELETE "$BASE_URL/$TABLE_NAME?key=eq.$key" > /dev/null
        
        if [ $((i % 25)) -eq 0 ]; then
            echo -n "."
        fi
    done
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    ops_per_second=$(echo "scale=2; $NUM_OPERATIONS / $duration" | bc)
    avg_latency_ms=$(echo "scale=2; $duration * 1000 / $NUM_OPERATIONS" | bc)
    
    echo ""
    log_success "DELETE Test Completed"
    echo "  Operations: $NUM_OPERATIONS"
    echo "  Duration: ${duration}s"
    echo "  Throughput: $ops_per_second ops/s"
    echo "  Avg Latency: ${avg_latency_ms}ms"
    echo ""
}

cleanup_test_data() {
    log_info "Cleaning up test data..."
    curl -s -X DELETE "$BASE_URL/$TABLE_NAME?key=like.perf_*" > /dev/null
    log_success "Test data cleaned up"
}

print_system_info() {
    echo -e "${BLUE}System Information:${NC}"
    echo "  Hostname: $(hostname)"
    echo "  OS: $(uname -s) $(uname -r)"
    echo "  CPU: $(nproc) cores"
    echo "  Memory: $(free -h | grep Mem | awk '{print $2}')"
    echo "  Load: $(uptime | awk -F'load average:' '{print $2}')"
    echo ""
}

# Main execution
print_banner
print_system_info
check_api

echo -e "${YELLOW}Starting performance tests...${NC}"
echo ""

# Run tests
test_write_performance
test_read_performance
test_delete_performance

# Cleanup
cleanup_test_data

echo -e "${GREEN}"
echo "========================================================"
echo "✅ Real Performance Test Completed Successfully!"
echo "========================================================"
echo -e "${NC}"
echo "This was a real test against a local PostgreSQL database"
echo "served via PostgREST API, demonstrating actual measured"
echo "performance metrics (not simulated)."
echo ""
