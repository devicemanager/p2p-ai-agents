#!/bin/bash
#
# Task Management Wrapper Script
# Provides convenient commands for managing implementation tasks.
#

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TASK_SCRIPT="$SCRIPT_DIR/manage_tasks.py"

show_help() {
    echo -e "${BLUE}📋 Task Management System${NC}"
    echo ""
    echo "Usage: $0 <command> [arguments] [options]"
    echo ""
    echo "Commands:"
    echo -e "  ${GREEN}generate${NC}                     Generate tasks from implementation checklist"
    echo -e "  ${GREEN}list${NC} [status]               List tasks (all, todo, in-progress, completed)"
    echo -e "  ${GREEN}move${NC} <task-file> <status>    Move task to new status (todo, in-progress, completed)"
    echo -e "  ${GREEN}start${NC} <task-file>            Move task to in-progress"
    echo -e "  ${GREEN}complete${NC} <task-file>         Move task to completed"
    echo -e "  ${GREEN}todo${NC} <task-file>             Move task back to todo"
    echo -e "  ${GREEN}stats${NC}                        Show task statistics"
    echo -e "  ${GREEN}index${NC}                        Update the task index"
    echo -e "  ${GREEN}search${NC} <term>               Search for tasks containing term"
    echo -e "  ${RED}yolo${NC} [options]              🚀 YOLO Mode: Automated task execution"
    echo ""
    echo "Options:"
    echo -e "  ${YELLOW}--no-auto-commit${NC}             Disable automatic git commit and push (for move operations)"
    echo ""
    echo "YOLO Mode Options:"
    echo -e "  ${YELLOW}--max-tasks N${NC}               Maximum tasks to process (default: 1)"
    echo -e "  ${YELLOW}--component NAME${NC}             Filter by component (network, storage, agent, docs)"
    echo -e "  ${YELLOW}--timeout SECONDS${NC}            Execution timeout (default: 300)"
    echo -e "  ${YELLOW}--max-files N${NC}               Max files per task (default: 10)"
    echo -e "  ${YELLOW}--dry-run${NC}                   Show actions without executing"
    echo -e "  ${YELLOW}--strict-validation${NC}         Treat warnings as errors"
    echo -e "  ${YELLOW}--skip-validation${NC}           Skip validation checks (emergency override)"
    echo ""
    echo "Examples:"
    echo "  $0 generate                       # Generate all tasks from checklist"
    echo "  $0 list todo                      # List all TODO tasks"
    echo "  $0 start task-name.md             # Start working on a task"
    echo "  $0 complete task-name.md          # Mark task as completed (with auto-commit)"
    echo "  $0 complete task-name.md --no-auto-commit  # Mark completed without committing"
    echo "  $0 search 'network manager'       # Find tasks related to network manager"
    echo ""
}

list_tasks() {
    local status="$1"
    local tasks_dir="$PROJECT_ROOT/tasks"
    
    case "$status" in
        "todo"|"")
            echo -e "${BLUE}📋 TODO Tasks:${NC}"
            find "$tasks_dir/todo" -name "*.md" -exec basename {} \; 2>/dev/null | sort
            ;;
        "in-progress"|"progress")
            echo -e "${YELLOW}🚧 In Progress Tasks:${NC}"
            find "$tasks_dir/in-progress" -name "*.md" -exec basename {} \; 2>/dev/null | sort
            ;;
        "completed"|"done")
            echo -e "${GREEN}✅ Completed Tasks:${NC}"
            find "$tasks_dir/completed" -name "*.md" -exec basename {} \; 2>/dev/null | sort
            ;;
        "all"|*)
            echo -e "${BLUE}📋 All Tasks:${NC}"
            echo ""
            list_tasks "todo"
            echo ""
            list_tasks "in-progress" 
            echo ""
            list_tasks "completed"
            ;;
    esac
}

search_tasks() {
    local term="$1"
    local tasks_dir="$PROJECT_ROOT/tasks"
    
    if [ -z "$term" ]; then
        echo -e "${RED}❌ Search term required${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}🔍 Searching for: '$term'${NC}"
    echo ""
    
    # Search in filenames and content
    find "$tasks_dir" -name "*.md" -type f | while read -r file; do
        # Check filename
        if basename "$file" | grep -qi "$term"; then
            status=$(basename "$(dirname "$file")")
            echo -e "${GREEN}📄 $(basename "$file")${NC} (${status})"
        # Check content
        elif grep -qi "$term" "$file" 2>/dev/null; then
            status=$(basename "$(dirname "$file")")
            echo -e "${GREEN}📄 $(basename "$file")${NC} (${status})"
            # Show matching lines
            grep -ni "$term" "$file" 2>/dev/null | head -2 | sed 's/^/     /'
        fi
    done
}

# Parse options
AUTO_COMMIT_FLAG=""
ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-auto-commit)
            AUTO_COMMIT_FLAG="--no-auto-commit"
            shift
            ;;
        *)
            ARGS+=("$1")
            shift
            ;;
    esac
done

# Restore positional parameters
set -- "${ARGS[@]}"

# Main command handling
case "$1" in
    "generate")
        echo -e "${BLUE}📋 Generating tasks from implementation checklist...${NC}"
        python3 "$TASK_SCRIPT" generate
        ;;
    "list"|"ls")
        list_tasks "$2"
        ;;
    "move")
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo -e "${RED}❌ Usage: $0 move <task-file> <status>${NC}"
            exit 1
        fi
        echo -e "${BLUE}📋 Moving task $2 to $3...${NC}"
        python3 "$TASK_SCRIPT" move "$2" "$3" $AUTO_COMMIT_FLAG
        ;;
    "start")
        if [ -z "$2" ]; then
            echo -e "${RED}❌ Usage: $0 start <task-file>${NC}"
            exit 1
        fi
        echo -e "${YELLOW}🚧 Starting work on $2...${NC}"
        python3 "$TASK_SCRIPT" move "$2" "in-progress" $AUTO_COMMIT_FLAG
        ;;
    "complete"|"done")
        if [ -z "$2" ]; then
            echo -e "${RED}❌ Usage: $0 complete <task-file>${NC}"
            exit 1
        fi
        echo -e "${GREEN}✅ Completing task $2...${NC}"
        python3 "$TASK_SCRIPT" move "$2" "completed" $AUTO_COMMIT_FLAG
        ;;
    "todo"|"reopen")
        if [ -z "$2" ]; then
            echo -e "${RED}❌ Usage: $0 todo <task-file>${NC}"
            exit 1
        fi
        echo -e "${BLUE}📋 Moving task $2 back to TODO...${NC}"
        python3 "$TASK_SCRIPT" move "$2" "todo" $AUTO_COMMIT_FLAG
        ;;
    "stats"|"status")
        echo -e "${BLUE}📊 Task Statistics:${NC}"
        python3 "$TASK_SCRIPT" stats
        ;;
    "index"|"update")
        echo -e "${BLUE}📋 Updating task index...${NC}"
        python3 "$TASK_SCRIPT" index
        ;;
    "search"|"find")
        search_tasks "$2"
        ;;
    "yolo")
        echo -e "${RED}🚀 YOLO Mode - Automated Task Execution${NC}"
        echo -e "${YELLOW}⚠️  This mode will automatically process tasks with safety checks${NC}"
        echo -e "${BLUE}ℹ️  Press Ctrl+C to stop at any time${NC}"
        echo ""
        
        # Pass all remaining arguments to the Python script
        shift  # Remove 'yolo' from arguments
        python3 "$TASK_SCRIPT" yolo "$@"
        ;;
    "help"|"-h"|"--help"|"")
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac
