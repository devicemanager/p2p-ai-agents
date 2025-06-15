#!/bin/bash

# Local Supabase Setup and Management Script
# This script manages a local Supabase instance for performance testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SUPABASE_CONFIG_DIR="$PROJECT_ROOT/lab/config/supabase"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

print_banner() {
    echo -e "${BLUE}"
    echo "========================================================"
    echo "🐘 Local Supabase Setup for Performance Testing"
    echo "========================================================"
    echo -e "${NC}"
}

show_help() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  setup          Set up local Supabase instance"
    echo "  start          Start Supabase services"
    echo "  stop           Stop Supabase services"
    echo "  restart        Restart Supabase services"
    echo "  status         Show status of Supabase services"
    echo "  logs [service] Show logs (all services or specific service)"
    echo "  test           Run connection and performance tests"
    echo "  clean          Clean up all data and containers"
    echo "  studio         Open Supabase Studio in browser"
    echo "  db-shell       Connect to PostgreSQL database"
    echo "  reset-data     Reset test data in database"
    echo "  health         Check health of all services"
    echo ""
    echo "Options:"
    echo "  --force        Force operation (for clean, setup)"
    echo "  --verbose      Verbose output"
    echo "  --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 setup                    # Initial setup"
    echo "  $0 start                    # Start all services"
    echo "  $0 test                     # Run performance tests"
    echo "  $0 logs rest                # Show PostgREST logs"
    echo "  $0 clean --force            # Force clean everything"
    echo ""
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi
    
    # Check Docker Compose
    if ! docker compose version &> /dev/null && ! docker-compose --version &> /dev/null; then
        missing_deps+=("docker-compose")
    fi
    
    # Check curl
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    # Check jq (for JSON processing)
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        echo ""
        echo "To install missing dependencies:"
        echo "  Docker: https://docs.docker.com/get-docker/"
        echo "  curl: apt-get install curl (or your package manager)"
        echo "  jq: apt-get install jq (or your package manager)"
        exit 1
    fi
    
    log_success "All dependencies are available"
}

setup_supabase() {
    log_step "Setting up local Supabase instance..."
    
    # Create necessary directories
    mkdir -p "$SUPABASE_CONFIG_DIR"
    
    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker first."
        exit 1
    fi
    
    log_info "Pulling Docker images..."
    cd "$SUPABASE_CONFIG_DIR"
    docker compose pull
    
    log_success "Local Supabase setup completed"
    echo ""
    echo "Next steps:"
    echo "  1. Run: $0 start"
    echo "  2. Wait for services to be healthy"
    echo "  3. Run: $0 test"
}

start_services() {
    log_step "Starting Supabase services..."
    
    cd "$SUPABASE_CONFIG_DIR"
    
    # Start services in detached mode
    docker compose up -d
    
    log_info "Waiting for services to start..."
    sleep 5
    
    # Wait for database to be ready
    log_info "Waiting for PostgreSQL to be ready..."
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if docker compose exec -T db pg_isready -U postgres &> /dev/null; then
            log_success "PostgreSQL is ready"
            break
        fi
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    if [[ $attempt -gt $max_attempts ]]; then
        log_error "PostgreSQL failed to start within timeout"
        exit 1
    fi
    
    # Wait for PostgREST to be ready
    log_info "Waiting for PostgREST to be ready..."
    attempt=1
    while [[ $attempt -le $max_attempts ]]; do
        if curl -s http://localhost:3000 &> /dev/null; then
            log_success "PostgREST is ready"
            break
        fi
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    # Wait for Kong API Gateway
    log_info "Waiting for Kong API Gateway to be ready..."
    attempt=1
    while [[ $attempt -le $max_attempts ]]; do
        if curl -s http://localhost:8000 &> /dev/null; then
            log_success "Kong API Gateway is ready"
            break
        fi
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    log_success "All Supabase services are running!"
    echo ""
    show_service_info
}

stop_services() {
    log_step "Stopping Supabase services..."
    
    cd "$SUPABASE_CONFIG_DIR"
    docker compose down
    
    log_success "Supabase services stopped"
}

restart_services() {
    log_step "Restarting Supabase services..."
    stop_services
    sleep 2
    start_services
}

show_status() {
    log_step "Checking Supabase service status..."
    
    cd "$SUPABASE_CONFIG_DIR"
    docker compose ps
    echo ""
    
    # Check service health
    log_info "Service Health Checks:"
    
    # PostgreSQL
    if docker compose exec -T db pg_isready -U postgres &> /dev/null; then
        echo -e "  PostgreSQL:   ${GREEN}✓ Healthy${NC}"
    else
        echo -e "  PostgreSQL:   ${RED}✗ Unhealthy${NC}"
    fi
    
    # PostgREST
    if curl -s http://localhost:3000 &> /dev/null; then
        echo -e "  PostgREST:    ${GREEN}✓ Healthy${NC}"
    else
        echo -e "  PostgREST:    ${RED}✗ Unhealthy${NC}"
    fi
    
    # Kong API Gateway
    if curl -s http://localhost:8000 &> /dev/null; then
        echo -e "  Kong Gateway: ${GREEN}✓ Healthy${NC}"
    else
        echo -e "  Kong Gateway: ${RED}✗ Unhealthy${NC}"
    fi
    
    # Supabase Studio
    if curl -s http://localhost:3001 &> /dev/null; then
        echo -e "  Studio:       ${GREEN}✓ Healthy${NC}"
    else
        echo -e "  Studio:       ${RED}✗ Unhealthy${NC}"
    fi
}

show_logs() {
    local service=${1:-}
    
    cd "$SUPABASE_CONFIG_DIR"
    
    if [[ -n "$service" ]]; then
        log_info "Showing logs for service: $service"
        docker compose logs -f "$service"
    else
        log_info "Showing logs for all services"
        docker compose logs -f
    fi
}

run_tests() {
    log_step "Running Supabase connection and performance tests..."
    
    # Source the environment variables
    source "$SUPABASE_CONFIG_DIR/local.env"
    
    # Test basic connectivity
    log_info "Testing basic connectivity..."
    
    if curl -s -H "apikey: $SUPABASE_ANON_KEY" "$SUPABASE_REST_URL/" | jq . &> /dev/null; then
        log_success "REST API connectivity test passed"
    else
        log_error "REST API connectivity test failed"
        return 1
    fi
    
    # Test database table access
    log_info "Testing database table access..."
    
    local response=$(curl -s -H "apikey: $SUPABASE_ANON_KEY" \
                         -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
                         "$SUPABASE_REST_URL/storage_perf_test?select=*&limit=1")
    
    if echo "$response" | jq . &> /dev/null; then
        log_success "Database table access test passed"
        log_info "Sample data: $(echo "$response" | jq -r 'length') records found"
    else
        log_error "Database table access test failed"
        echo "Response: $response"
        return 1
    fi
    
    # Run Rust performance tests
    log_info "Running Rust performance tests..."
    cd "$PROJECT_ROOT"
    
    if cargo test --features storage-supabase test_supabase_storage_performance -- --nocapture; then
        log_success "Rust performance tests passed"
    else
        log_warning "Rust performance tests had issues (check output above)"
    fi
    
    echo ""
    log_success "All tests completed!"
}

clean_everything() {
    local force=${1:-false}
    
    if [[ "$force" != "true" ]]; then
        log_warning "This will remove all Supabase containers and data"
        read -p "Are you sure? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Operation cancelled"
            return 0
        fi
    fi
    
    log_step "Cleaning up Supabase environment..."
    
    cd "$SUPABASE_CONFIG_DIR"
    
    # Stop and remove containers
    docker compose down -v --remove-orphans
    
    # Remove images
    docker compose down --rmi all &> /dev/null || true
    
    # Remove volumes
    docker volume rm supabase_db_data supabase_storage_data &> /dev/null || true
    
    log_success "Cleanup completed"
}

open_studio() {
    log_info "Opening Supabase Studio..."
    
    if curl -s http://localhost:3001 &> /dev/null; then
        log_success "Supabase Studio is available at: http://localhost:3001"
        
        # Try to open in browser (works on many systems)
        if command -v xdg-open &> /dev/null; then
            xdg-open http://localhost:3001 &> /dev/null || true
        elif command -v open &> /dev/null; then
            open http://localhost:3001 &> /dev/null || true
        fi
    else
        log_error "Supabase Studio is not running. Start services first with: $0 start"
    fi
}

connect_db_shell() {
    log_info "Connecting to PostgreSQL database..."
    
    cd "$SUPABASE_CONFIG_DIR"
    docker compose exec db psql -U postgres -d postgres
}

reset_test_data() {
    log_step "Resetting test data..."
    
    cd "$SUPABASE_CONFIG_DIR"
    
    # Run cleanup function
    if docker compose exec -T db psql -U postgres -d postgres -c "SELECT cleanup_perf_test_data();" &> /dev/null; then
        log_success "Test data cleaned up"
    else
        log_warning "Failed to clean up test data (table may not exist yet)"
    fi
    
    # Show current stats
    log_info "Current database statistics:"
    docker compose exec -T db psql -U postgres -d postgres -c "SELECT * FROM get_perf_test_stats();" || true
}

show_service_info() {
    echo -e "${BLUE}🚀 Local Supabase Instance Information${NC}"
    echo "========================================================"
    echo -e "API Gateway:      ${GREEN}http://localhost:8000${NC}"
    echo -e "REST API:         ${GREEN}http://localhost:8000/rest/v1${NC}"
    echo -e "Auth API:         ${GREEN}http://localhost:8000/auth/v1${NC}"
    echo -e "Storage API:      ${GREEN}http://localhost:8000/storage/v1${NC}"
    echo -e "Supabase Studio:  ${GREEN}http://localhost:3001${NC}"
    echo -e "PostgreSQL:       ${GREEN}localhost:5432${NC}"
    echo ""
    echo -e "${YELLOW}API Keys:${NC}"
    echo "  Anon Key:      eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
    echo "  Service Role:  eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU"
    echo ""
    echo -e "${BLUE}Quick Commands:${NC}"
    echo "  Test connection:   $0 test"
    echo "  Open Studio:       $0 studio"
    echo "  View logs:         $0 logs"
    echo "  Database shell:    $0 db-shell"
    echo "========================================================"
}

check_health() {
    log_step "Performing comprehensive health check..."
    
    local healthy=true
    
    # Check if services are running
    cd "$SUPABASE_CONFIG_DIR"
    if ! docker compose ps --services --filter "status=running" | grep -q .; then
        log_error "No services are running"
        healthy=false
    fi
    
    # Check individual service health
    local services=("db:5432" "rest:3000" "auth:9999" "storage:5000" "studio:3000" "meta:8080" "kong:8000")
    
    for service_port in "${services[@]}"; do
        local service=${service_port%:*}
        local port=${service_port#*:}
        
        if docker compose exec -T "$service" true &> /dev/null; then
            echo -e "  ${service}: ${GREEN}✓ Container running${NC}"
        else
            echo -e "  ${service}: ${RED}✗ Container not running${NC}"
            healthy=false
        fi
    done
    
    # Check external connectivity
    local endpoints=("http://localhost:8000" "http://localhost:3001" "http://localhost:3000")
    
    for endpoint in "${endpoints[@]}"; do
        if curl -s "$endpoint" &> /dev/null; then
            echo -e "  ${endpoint}: ${GREEN}✓ Accessible${NC}"
        else
            echo -e "  ${endpoint}: ${RED}✗ Not accessible${NC}"
            healthy=false
        fi
    done
    
    if [[ "$healthy" == "true" ]]; then
        log_success "All health checks passed!"
    else
        log_error "Some health checks failed"
        return 1
    fi
}

main() {
    local command=${1:-}
    local force=false
    local verbose=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --force)
                force=true
                shift
                ;;
            --verbose)
                verbose=true
                set -x
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
            *)
                if [[ -z "$command" ]]; then
                    command=$1
                fi
                shift
                ;;
        esac
    done
    
    print_banner
    
    case $command in
        setup)
            check_dependencies
            setup_supabase
            ;;
        start)
            start_services
            ;;
        stop)
            stop_services
            ;;
        restart)
            restart_services
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs "$2"
            ;;
        test)
            run_tests
            ;;
        clean)
            clean_everything "$force"
            ;;
        studio)
            open_studio
            ;;
        db-shell)
            connect_db_shell
            ;;
        reset-data)
            reset_test_data
            ;;
        health)
            check_health
            ;;
        "")
            log_error "No command provided"
            show_help
            exit 1
            ;;
        *)
            log_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
