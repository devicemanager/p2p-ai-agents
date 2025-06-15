#!/bin/bash

# Complete Local Supabase Lab Setup
# This script provides end-to-end setup for local Supabase performance testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LAB_DIR="$PROJECT_ROOT/lab"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

log_header() {
    echo -e "${PURPLE}"
    echo "========================================================"
    echo "$1"
    echo "========================================================"
    echo -e "${NC}"
}

print_banner() {
    echo -e "${CYAN}"
    cat << 'EOF'
 ____                       _                    _          _
/ ___| _   _ _ __   __ _ ___ | |__   __ _ ___  ___| |    __ _| |__
\___ \| | | | '_ \ / _` / _ \| '_ \ / _` / __|/ _ \ |   / _` | '_ \
 ___) | |_| | |_) | (_| \__/ | |_) | (_| \__ \  __/ |__| (_| | |_) |
|____/ \__,_| .__/ \__,_|___/|_.__/ \__,_|___/\___|_____\__,_|_.__/
             |_|
EOF
    echo "      🚀 Complete Local Performance Testing Setup"
    echo -e "${NC}"
}

check_prerequisites() {
    log_header "🔍 Checking Prerequisites"
    
    local missing_deps=()
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        missing_deps+=("Docker")
    elif ! docker info &> /dev/null; then
        log_error "Docker is installed but not running"
        missing_deps+=("Docker (running)")
    else
        log_success "Docker: Available and running"
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        missing_deps+=("Docker Compose")
    else
        log_success "Docker Compose: Available"
    fi
    
    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("Rust/Cargo")
    else
        log_success "Rust/Cargo: Available"
    fi
    
    # Check curl
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    else
        log_success "curl: Available"
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing prerequisites: ${missing_deps[*]}"
        echo ""
        echo "Please install the missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            case $dep in
                "Docker")
                    echo "  - Docker: https://docs.docker.com/get-docker/"
                    ;;
                "Docker Compose")
                    echo "  - Docker Compose: https://docs.docker.com/compose/install/"
                    ;;
                "Rust/Cargo")
                    echo "  - Rust: https://rustup.rs/"
                    ;;
                "curl")
                    echo "  - curl: Available in most package managers"
                    ;;
            esac
        done
        exit 1
    fi
    
    log_success "All prerequisites satisfied!"
    echo ""
}

setup_local_supabase() {
    log_header "🏗️  Setting up Local Supabase Instance"
    
    local_script="$LAB_DIR/scripts/local_supabase.sh"
    
    if [[ ! -f "$local_script" ]]; then
        log_error "Local Supabase script not found: $local_script"
        exit 1
    fi
    
    log_info "Setting up local Supabase instance..."
    "$local_script" setup
    
    log_success "Local Supabase setup completed!"
    echo ""
}

run_quick_demo() {
    log_header "🧪 Running Quick Performance Demo"
    
    perf_script="$LAB_DIR/scripts/supabase_perf_lab.sh"
    
    if [[ ! -f "$perf_script" ]]; then
        log_error "Performance lab script not found: $perf_script"
        exit 1
    fi
    
    log_info "Running quick performance test with local Supabase..."
    "$perf_script" --local --quick
    
    echo ""
}

run_comprehensive_tests() {
    log_header "🚀 Running Comprehensive Performance Tests"
    
    perf_script="$LAB_DIR/scripts/supabase_perf_lab.sh"
    
    log_info "Running full performance test suite..."
    "$perf_script" --local --full
    
    echo ""
}

show_next_steps() {
    log_header "🎯 Next Steps and Usage"
    
    echo "Your local Supabase lab is now ready! Here's what you can do:"
    echo ""
    
    echo -e "${GREEN}🔧 Management Commands:${NC}"
    echo "  ./lab/scripts/local_supabase.sh status    # Check service status"
    echo "  ./lab/scripts/local_supabase.sh studio    # Open Supabase Studio"
    echo "  ./lab/scripts/local_supabase.sh logs      # View service logs"
    echo "  ./lab/scripts/local_supabase.sh stop      # Stop services"
    echo "  ./lab/scripts/local_supabase.sh start     # Start services"
    echo ""
    
    echo -e "${GREEN}📊 Performance Testing:${NC}"
    echo "  ./lab/scripts/supabase_perf_lab.sh --local --quick   # Quick test"
    echo "  ./lab/scripts/supabase_perf_lab.sh --local --full    # Full test"
    echo "  ./lab/scripts/supabase_perf_lab.sh --local --stress  # Stress test"
    echo ""
    
    echo -e "${GREEN}🌐 Web Interfaces:${NC}"
    echo "  Supabase Studio:  http://localhost:3001"
    echo "  PostgREST API:    http://localhost:3000"
    echo "  Database (psql):  localhost:54322"
    echo ""
    
    echo -e "${GREEN}🔑 Environment Variables:${NC}"
    echo "  SUPABASE_URL=http://localhost:3000"
    echo "  SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    echo ""
    
    echo -e "${GREEN}📁 Lab Resources:${NC}"
    echo "  Documentation:  lab/supabase-performance-lab.md"
    echo "  Reports:        lab/reports/"
    echo "  Configuration:  lab/docker/"
    echo ""
    
    echo -e "${YELLOW}💡 Tips:${NC}"
    echo "  - Use Supabase Studio to inspect data and performance"
    echo "  - Check lab/reports/ for detailed performance reports"
    echo "  - Customize test parameters in the performance scripts"
    echo "  - Run 'docker stats' to monitor resource usage"
    echo ""
}

open_studio() {
    log_info "Opening Supabase Studio..."
    
    # Check if Studio is running
    if ! curl -s http://localhost:3001/ &> /dev/null; then
        log_warning "Supabase Studio is not running. Starting services first..."
        "$LAB_DIR/scripts/local_supabase.sh" start
        sleep 5
    fi
    
    # Try to open in browser
    if command -v xdg-open &> /dev/null; then
        xdg-open http://localhost:3001
    elif command -v open &> /dev/null; then
        open http://localhost:3001
    elif command -v start &> /dev/null; then
        start http://localhost:3001
    else
        log_info "Please open http://localhost:3001 in your browser"
    fi
}

show_help() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  setup          Complete setup (default)"
    echo "  quick          Quick demo after setup"
    echo "  full           Full performance testing"
    echo "  studio         Open Supabase Studio"
    echo "  status         Show system status"
    echo "  help           Show this help"
    echo ""
}

show_status() {
    log_header "📊 System Status"
    
    # Check local Supabase
    local_script="$LAB_DIR/scripts/local_supabase.sh"
    if [[ -f "$local_script" ]]; then
        "$local_script" status
    else
        log_warning "Local Supabase script not found"
    fi
    
    echo ""
    log_info "Lab Resources:"
    echo "  📁 Docker config:   $LAB_DIR/docker/"
    echo "  📊 Reports:         $LAB_DIR/reports/"
    echo "  📚 Documentation:   $LAB_DIR/supabase-performance-lab.md"
}

main() {
    print_banner
    
    case ${1:-setup} in
        setup)
            check_prerequisites
            setup_local_supabase
            run_quick_demo
            show_next_steps
            ;;
        quick)
            check_prerequisites
            setup_local_supabase
            run_quick_demo
            ;;
        full)
            check_prerequisites
            setup_local_supabase
            run_comprehensive_tests
            show_next_steps
            ;;
        studio)
            open_studio
            ;;
        status)
            show_status
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
