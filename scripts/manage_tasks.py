#!/usr/bin/env python3
"""
Task Management System for P2P AI Agents
Converts implementation checklist items to Markdown task files and manages their status.
"""

import json
import re
import sys
import argparse
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple
import subprocess

class TaskManager:
    def __init__(self, root_dir: str):
        self.root_dir = Path(root_dir)
        self.tasks_dir = self.root_dir / "tasks"
        self.todo_dir = self.tasks_dir / "todo"
        self.in_progress_dir = self.tasks_dir / "in-progress"
        self.completed_dir = self.tasks_dir / "completed"
        
        # Ensure directories exist
        for dir_path in [self.tasks_dir, self.todo_dir, self.in_progress_dir, self.completed_dir]:
            dir_path.mkdir(exist_ok=True)
    
    def sanitize_filename(self, text: str) -> str:
        """Convert text to a safe filename"""
        # Remove emojis and special characters
        text = re.sub(r'[^\w\s-]', '', text)
        # Replace spaces with hyphens and convert to lowercase
        text = re.sub(r'\s+', '-', text.strip()).lower()
        # Remove multiple consecutive hyphens
        text = re.sub(r'-+', '-', text)
        return text[:100]  # Limit length
    
    def generate_task_id(self, component: str, title: str) -> str:
        """Generate a unique task ID"""
        comp_short = self.sanitize_filename(component)[:20]
        title_short = self.sanitize_filename(title)[:30]
        return f"{comp_short}-{title_short}"
    
    def create_task_from_issue(self, issue_data: Dict) -> Path:
        """Create a Markdown task file from issue data"""
        task_id = self.generate_task_id(issue_data.get('component', 'unknown'), issue_data['title'])
        filename = f"{task_id}.md"
        filepath = self.todo_dir / filename
        
        # Extract clean title (remove emoji and brackets)
        title = re.sub(r'^[^\w]*\[.*?\]\s*', '', issue_data['title'])
        
        # Parse component and section
        component = issue_data.get('component', 'Unknown')
        section = issue_data.get('section', 'General')
        priority = issue_data.get('priority', 'medium')
        source_file = issue_data.get('source_file', '')
        source_line = issue_data.get('source_line', '')
        
        # Generate task content
        task_content = f"""# {title}

## Task Information

**Task ID**: `{task_id}`  
**Component**: {component}  
**Section**: {section}  
**Priority**: {priority}  
**Status**: TODO  
**Created**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  
**Source**: `{source_file}` (line {source_line})  

## Description

{self.extract_description_from_body(issue_data.get('body', ''))}

## Acceptance Criteria

- [ ] Implementation completed according to specifications
- [ ] Code follows project coding standards and best practices
- [ ] Appropriate tests written and passing (unit, integration, performance as applicable)
- [ ] Documentation updated to reflect changes
- [ ] Code reviewed and approved by team
- [ ] Security considerations addressed (if applicable)

## Implementation Notes

<!-- Add specific implementation notes, design decisions, or technical requirements here -->

## Testing Strategy

<!-- Describe the testing approach for this task -->

## Progress Log

<!-- Add progress updates here -->
- {datetime.now().strftime('%Y-%m-%d')}: Task created from implementation checklist

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code merged to main branch
- [ ] CI/CD pipeline passing
- [ ] Stakeholder approval received

---

*Generated from implementation checklist: `{source_file}` (line {source_line})*
"""

        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(task_content)
        
        return filepath
    
    def extract_description_from_body(self, body: str) -> str:
        """Extract the main description from the issue body"""
        lines = body.split('\n')
        in_description = False
        description_lines = []
        
        for line in lines:
            if line.startswith('### Description'):
                in_description = True
                continue
            elif in_description and line.startswith('###'):
                break
            elif in_description and line.strip():
                description_lines.append(line)
        
        return '\n'.join(description_lines).strip() if description_lines else "No description available"
    
    def generate_tasks_from_json(self, json_file: Path) -> List[Path]:
        """Generate task files from JSON issue data"""
        if not json_file.exists():
            print(f"❌ JSON file not found: {json_file}")
            return []
        
        with open(json_file, 'r', encoding='utf-8') as f:
            issues = json.load(f)
        
        created_tasks = []
        print(f"📋 Generating {len(issues)} tasks from {json_file.name}...")
        
        for issue in issues:
            try:
                task_path = self.create_task_from_issue(issue)
                created_tasks.append(task_path)
                print(f"✅ Created: {task_path.name}")
            except Exception as e:
                print(f"❌ Failed to create task for '{issue.get('title', 'Unknown')}': {e}")
        
        return created_tasks
    
    def auto_commit_and_push(self, task_name: str, action: str = "completed") -> bool:
        """Automatically commit and push changes when a task is completed"""
        try:
            # Check if we're in a git repository
            result = subprocess.run(['git', 'status'], 
                                  cwd=self.root_dir, 
                                  capture_output=True, 
                                  text=True)
            if result.returncode != 0:
                print("⚠️ Not in a git repository, skipping auto-commit")
                return False
            
            # Check if there are changes to commit
            result = subprocess.run(['git', 'diff', '--cached', '--name-only'], 
                                  cwd=self.root_dir, 
                                  capture_output=True, 
                                  text=True)
            
            # Stage all changes
            subprocess.run(['git', 'add', '.'], 
                          cwd=self.root_dir, 
                          check=True)
            
            # Check again for staged changes
            result = subprocess.run(['git', 'diff', '--cached', '--name-only'], 
                                  cwd=self.root_dir, 
                                  capture_output=True, 
                                  text=True)
            
            if not result.stdout.strip():
                print("ℹ️ No changes to commit")
                return True
            
            # Create commit message
            commit_msg = f"Task {action}: {task_name}\n\n"
            commit_msg += f"✅ {action.capitalize()} task: {task_name}\n"
            commit_msg += f"📅 {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n"
            commit_msg += f"🔧 Task management: Auto-commit on task {action}"
            
            # Commit changes
            result = subprocess.run(['git', 'commit', '-m', commit_msg], 
                                  cwd=self.root_dir, 
                                  capture_output=True, 
                                  text=True)
            
            if result.returncode != 0:
                print(f"❌ Failed to commit: {result.stderr}")
                return False
            
            print(f"✅ Committed changes for task: {task_name}")
            
            # Push to remote (if configured)
            result = subprocess.run(['git', 'push'], 
                                  cwd=self.root_dir, 
                                  capture_output=True, 
                                  text=True)
            
            if result.returncode == 0:
                print(f"🚀 Pushed changes to remote repository")
                return True
            else:
                print(f"⚠️ Failed to push to remote: {result.stderr}")
                print("💡 You may need to push manually later")
                return True  # Still return True since commit succeeded
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Git operation failed: {e}")
            return False
        except Exception as e:
            print(f"❌ Unexpected error during git operations: {e}")
            return False

    def move_task(self, task_file: Path, new_status: str, auto_commit: bool = True) -> bool:
        """Move a task to a different status directory"""
        status_dirs = {
            'todo': self.todo_dir,
            'in-progress': self.in_progress_dir,
            'completed': self.completed_dir
        }
        
        if new_status not in status_dirs:
            print(f"❌ Invalid status: {new_status}")
            return False
        
        if not task_file.exists():
            print(f"❌ Task file not found: {task_file}")
            return False
        
        new_path = status_dirs[new_status] / task_file.name
        
        # Update the task file content
        self.update_task_status(task_file, new_status)
        
        # Move the file
        task_file.rename(new_path)
        print(f"✅ Moved {task_file.name} to {new_status}")
        
        if auto_commit:
            self.auto_commit_and_push(task_file.name, new_status)
        
        return True
    
    def update_task_status(self, task_file: Path, new_status: str):
        """Update the status in the task file content"""
        with open(task_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Update status line
        content = re.sub(
            r'\*\*Status\*\*: \w+', 
            f'**Status**: {new_status.upper().replace("-", " ")}', 
            content
        )
        
        # Add progress log entry
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        progress_entry = f"- {timestamp}: Status changed to {new_status}"
        
        if "## Progress Log" in content:
            content = content.replace(
                "<!-- Add progress updates here -->",
                f"<!-- Add progress updates here -->\n{progress_entry}"
            )
        
        with open(task_file, 'w', encoding='utf-8') as f:
            f.write(content)
    
    def generate_index(self) -> None:
        """Generate the main task index file"""
        index_content = f"""# Task Management Index

*Last updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*

## Overview

This directory contains all implementation tasks for the P2P AI Agents project, organized by status.

## Task Statistics

"""
        
        # Count tasks in each directory
        todo_count = len(list(self.todo_dir.glob("*.md")))
        in_progress_count = len(list(self.in_progress_dir.glob("*.md")))
        completed_count = len(list(self.completed_dir.glob("*.md")))
        total_count = todo_count + in_progress_count + completed_count
        
        if total_count > 0:
            progress_percent = round((completed_count / total_count) * 100, 1)
        else:
            progress_percent = 0
        
        index_content += f"""- **Total Tasks**: {total_count}
- **Completed**: {completed_count} ({progress_percent}%)
- **In Progress**: {in_progress_count}
- **TODO**: {todo_count}

## Task Directories

### 📋 [TODO Tasks](./todo/) ({todo_count} tasks)
Tasks that are planned but not yet started.

### 🚧 [In Progress Tasks](./in-progress/) ({in_progress_count} tasks)
Tasks that are currently being worked on.

### ✅ [Completed Tasks](./completed/) ({completed_count} tasks)
Tasks that have been finished and verified.

## Task Management

### Moving Tasks

Use the task management script to move tasks between statuses:

```bash
# Move a task to in-progress
python scripts/manage_tasks.py move task-filename.md in-progress

# Move a task to completed
python scripts/manage_tasks.py move task-filename.md completed

# Move a task back to todo
python scripts/manage_tasks.py move task-filename.md todo
```

### Generating Tasks

```bash
# Generate tasks from implementation checklist
python scripts/manage_tasks.py generate

# Update the index file
python scripts/manage_tasks.py index
```

## Task File Format

Each task follows a standardized Markdown format with:
- Task metadata (ID, component, priority, status)
- Description and acceptance criteria
- Implementation notes and testing strategy
- Progress log with timestamps
- Definition of done checklist

## Quick Navigation

"""
        
        # Add quick navigation for each directory
        for status, directory in [("TODO", self.todo_dir), ("In Progress", self.in_progress_dir), ("Completed", self.completed_dir)]:
            index_content += f"\n### {status} Tasks\n\n"
            
            tasks = sorted(directory.glob("*.md"))
            if tasks:
                for task_file in tasks:
                    # Extract title from first line of file
                    try:
                        with open(task_file, 'r', encoding='utf-8') as f:
                            first_line = f.readline().strip()
                            title = first_line.replace('# ', '') if first_line.startswith('# ') else task_file.stem
                        
                        relative_path = task_file.relative_to(self.tasks_dir)
                        index_content += f"- [{title}](./{relative_path})\n"
                    except Exception:
                        index_content += f"- [{task_file.stem}](./{task_file.relative_to(self.tasks_dir)})\n"
            else:
                index_content += "No tasks in this category.\n"
        
        index_content += f"""
---

*This index is automatically generated. To update it, run:*
```bash
python scripts/manage_tasks.py index
```
"""
        
        # Write the index file
        index_file = self.tasks_dir / "README.md"
        with open(index_file, 'w', encoding='utf-8') as f:
            f.write(index_content)
        
        print(f"✅ Updated task index: {index_file}")

def main():
    parser = argparse.ArgumentParser(description="Manage implementation tasks")
    parser.add_argument('action', choices=['generate', 'move', 'index', 'stats'], 
                       help='Action to perform')
    parser.add_argument('target', nargs='?', help='Target file or status for move action')
    parser.add_argument('status', nargs='?', help='New status for move action')
    parser.add_argument('--json-file', default='implementation_issues.json', 
                       help='JSON file to generate tasks from')
    parser.add_argument('--no-auto-commit', action='store_true', 
                       help='Disable automatic git commit and push when completing tasks')
    
    args = parser.parse_args()
    
    root_dir = "/workspaces/p2p-ai-agents"
    task_manager = TaskManager(root_dir)
    
    if args.action == 'generate':
        json_file = Path(root_dir) / args.json_file
        created_tasks = task_manager.generate_tasks_from_json(json_file)
        print(f"\n📊 Created {len(created_tasks)} task files")
        task_manager.generate_index()
        
    elif args.action == 'move':
        if not args.target or not args.status:
            print("❌ Move action requires both target file and new status")
            sys.exit(1)
        
        # Find the task file
        task_file = None
        for directory in [task_manager.todo_dir, task_manager.in_progress_dir, task_manager.completed_dir]:
            potential_file = directory / args.target
            if potential_file.exists():
                task_file = potential_file
                break
        
        if not task_file:
            print(f"❌ Task file not found: {args.target}")
            sys.exit(1)
        
        # Auto-commit is enabled by default, disabled only if --no-auto-commit is specified
        auto_commit = not args.no_auto_commit
        task_manager.move_task(task_file, args.status, auto_commit)
        task_manager.generate_index()
        
    elif args.action == 'index':
        task_manager.generate_index()
        
    elif args.action == 'stats':
        todo_count = len(list(task_manager.todo_dir.glob("*.md")))
        in_progress_count = len(list(task_manager.in_progress_dir.glob("*.md")))
        completed_count = len(list(task_manager.completed_dir.glob("*.md")))
        total_count = todo_count + in_progress_count + completed_count
        
        print(f"📊 Task Statistics:")
        print(f"   Total: {total_count}")
        print(f"   TODO: {todo_count}")
        print(f"   In Progress: {in_progress_count}")
        print(f"   Completed: {completed_count}")
        if total_count > 0:
            print(f"   Progress: {round((completed_count / total_count) * 100, 1)}%")

if __name__ == "__main__":
    main()
