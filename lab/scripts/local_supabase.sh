#!/bin/bash

# Local Supabase Setup and Management Script
# This script manages a local Supabase instance for performance testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LAB_DIR="$PROJECT_ROOT/lab"
DOCKER_DIR="$LAB_DIR/docker"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

log_header() {
    echo -e "${CYAN}"
    echo "========================================================"
    echo "$1"
    echo "========================================================"
    echo -e "${NC}"
}

# Configuration
SUPABASE_URL="http://localhost:3000"
SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
SUPABASE_SERVICE_ROLE_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU"

show_help() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start          Start local Supabase instance"
    echo "  stop           Stop local Supabase instance"
    echo "  restart        Restart local Supabase instance"
    echo "  status         Show status of all services"
    echo "  logs [service] Show logs (all services or specific service)"
    echo "  clean          Stop and remove all containers and volumes"
    echo "  setup          Initial setup and verification"
    echo "  test           Run connectivity tests"
    echo "  studio         Open Supabase Studio in browser"
    echo "  env            Show environment variables for testing"
    echo "  db             Connect to PostgreSQL database"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start                    # Start all services"
    echo "  $0 logs supabase-lab-db     # Show database logs"
    echo "  $0 test                     # Test connectivity"
    echo ""
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker first."
        exit 1
    fi
    
    log_success "All dependencies are available"
}

start_services() {
    log_header "🚀 Starting Local Supabase Instance"
    
    cd "$DOCKER_DIR"
    
    log_info "Starting services with Docker Compose..."
    if command -v docker-compose &> /dev/null; then
        docker-compose up -d
    else
        docker compose up -d
    fi
    
    log_info "Waiting for services to become healthy..."
    sleep 10
    
    # Wait for database to be ready
    local retries=30
    local count=0
    while [ $count -lt $retries ]; do
        if docker exec supabase-lab-db pg_isready -U postgres &> /dev/null; then
            break
        fi
        count=$((count + 1))
        log_info "Waiting for database... ($count/$retries)"
        sleep 2
    done
    
    if [ $count -eq $retries ]; then
        log_error "Database failed to start within timeout"
        return 1
    fi
    
    # Wait for PostgREST to be ready
    count=0
    while [ $count -lt $retries ]; do
        if curl -s http://localhost:3000/ &> /dev/null; then
            break
        fi
        count=$((count + 1))
        log_info "Waiting for PostgREST API... ($count/$retries)"
        sleep 2
    done
    
    if [ $count -eq $retries ]; then
        log_error "PostgREST API failed to start within timeout"
        return 1
    fi
    
    log_success "Local Supabase instance started successfully!"
    echo ""
    log_info "Services available at:"
    echo "  📊 Supabase Studio: http://localhost:3001"
    echo "  🔗 PostgREST API:   http://localhost:3000"
    echo "  🗄️  PostgreSQL:     localhost:54322"
    echo "  🔐 Auth Service:    http://localhost:9999"
    echo ""
}

stop_services() {
    log_header "🛑 Stopping Local Supabase Instance"
    
    cd "$DOCKER_DIR"
    
    if command -v docker-compose &> /dev/null; then
        docker-compose down
    else
        docker compose down
    fi
    
    log_success "Local Supabase instance stopped"
}

restart_services() {
    log_header "🔄 Restarting Local Supabase Instance"
    stop_services
    sleep 2
    start_services
}

show_status() {
    log_header "📊 Service Status"
    
    cd "$DOCKER_DIR"
    
    if command -v docker-compose &> /dev/null; then
        docker-compose ps
    else
        docker compose ps
    fi
    
    echo ""
    log_info "Service health checks:"
    
    # Check database
    if docker exec supabase-lab-db pg_isready -U postgres &> /dev/null; then
        echo -e "  ${GREEN}✅${NC} PostgreSQL Database: Running"
    else
        echo -e "  ${RED}❌${NC} PostgreSQL Database: Not running"
    fi
    
    # Check PostgREST
    if curl -s http://localhost:3000/ &> /dev/null; then
        echo -e "  ${GREEN}✅${NC} PostgREST API: Running"
    else
        echo -e "  ${RED}❌${NC} PostgREST API: Not running"
    fi
    
    # Check Studio
    if curl -s http://localhost:3001/ &> /dev/null; then
        echo -e "  ${GREEN}✅${NC} Supabase Studio: Running"
    else
        echo -e "  ${RED}❌${NC} Supabase Studio: Not running"
    fi
}

show_logs() {
    local service=${1:-}
    
    cd "$DOCKER_DIR"
    
    if [ -n "$service" ]; then
        log_info "Showing logs for $service..."
        if command -v docker-compose &> /dev/null; then
            docker-compose logs -f "$service"
        else
            docker compose logs -f "$service"
        fi
    else
        log_info "Showing logs for all services..."
        if command -v docker-compose &> /dev/null; then
            docker-compose logs -f
        else
            docker compose logs -f
        fi
    fi
}

clean_all() {
    log_header "🧹 Cleaning Local Supabase Instance"
    log_warning "This will remove all containers and data!"
    
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Operation cancelled"
        return 0
    fi
    
    cd "$DOCKER_DIR"
    
    log_info "Stopping and removing containers..."
    if command -v docker-compose &> /dev/null; then
        docker-compose down -v --remove-orphans
    else
        docker compose down -v --remove-orphans
    fi
    
    log_info "Removing volumes..."
    docker volume rm $(docker volume ls -q | grep supabase-lab) 2>/dev/null || true
    
    log_info "Cleaning up data directory..."
    rm -rf "$DOCKER_DIR/data"
    
    log_success "Local Supabase instance cleaned"
}

setup_instance() {
    log_header "🏗️  Setting up Local Supabase Instance"
    
    check_dependencies
    
    # Create data directory
    mkdir -p "$DOCKER_DIR/data"
    
    # Start services
    start_services
    
    # Run connectivity test
    run_tests
    
    log_success "Local Supabase setup completed!"
    echo ""
    log_info "You can now run performance tests with:"
    echo "  cd $PROJECT_ROOT"
    echo "  ./lab/scripts/supabase_perf_lab.sh --local --full"
}

run_tests() {
    log_header "🧪 Running Connectivity Tests"
    
    # Test database connection
    log_info "Testing PostgreSQL connection..."
    if docker exec supabase-lab-db psql -U postgres -d postgres -c "SELECT 1;" &> /dev/null; then
        log_success "PostgreSQL connection: OK"
    else
        log_error "PostgreSQL connection: FAILED"
        return 1
    fi
    
    # Test PostgREST API
    log_info "Testing PostgREST API..."
    local api_response=$(curl -s http://localhost:3000/)
    if [ $? -eq 0 ]; then
        log_success "PostgREST API: OK"
    else
        log_error "PostgREST API: FAILED"
        return 1
    fi
    
    # Test table creation
    log_info "Testing table access..."
    local table_test=$(curl -s -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
        "http://localhost:3000/storage_perf_test?limit=1")
    if [ $? -eq 0 ]; then
        log_success "Table access: OK"
    else
        log_error "Table access: FAILED"
        return 1
    fi
    
    log_success "All connectivity tests passed!"
}

open_studio() {
    log_info "Opening Supabase Studio..."
    
    # Check if Studio is running
    if ! curl -s http://localhost:3001/ &> /dev/null; then
        log_error "Supabase Studio is not running. Please start the services first."
        return 1
    fi
    
    # Try to open in browser (Linux)
    if command -v xdg-open &> /dev/null; then
        xdg-open http://localhost:3001
    # Try to open in browser (macOS)
    elif command -v open &> /dev/null; then
        open http://localhost:3001
    # Try to open in browser (Windows)
    elif command -v start &> /dev/null; then
        start http://localhost:3001
    else
        log_info "Please open http://localhost:3001 in your browser"
    fi
}

show_env() {
    log_header "🔑 Environment Variables for Testing"
    
    echo "Copy these environment variables to use with performance tests:"
    echo ""
    echo "export SUPABASE_URL=\"$SUPABASE_URL\""
    echo "export SUPABASE_ANON_KEY=\"$SUPABASE_ANON_KEY\""
    echo "export SUPABASE_SERVICE_ROLE_KEY=\"$SUPABASE_SERVICE_ROLE_KEY\""
    echo ""
    echo "Or run with local configuration:"
    echo "./lab/scripts/supabase_perf_lab.sh --local --full"
}

connect_db() {
    log_info "Connecting to PostgreSQL database..."
    docker exec -it supabase-lab-db psql -U postgres -d postgres
}

main() {
    case ${1:-help} in
        start)
            check_dependencies
            start_services
            ;;
        stop)
            stop_services
            ;;
        restart)
            check_dependencies
            restart_services
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs "$2"
            ;;
        clean)
            clean_all
            ;;
        setup)
            setup_instance
            ;;
        test)
            run_tests
            ;;
        studio)
            open_studio
            ;;
        env)
            show_env
            ;;
        db)
            connect_db
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

# Run main function with all arguments
main "$@"
