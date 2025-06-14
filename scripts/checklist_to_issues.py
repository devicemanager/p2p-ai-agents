#!/usr/bin/env python3
"""
GitHub Issue Generator from Implementation Checklists
Converts implementation checklists and plans into trackable GitHub issues.
"""

import os
import re
import json
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime

class ChecklistToIssuesConverter:
    def __init__(self, repo_path: str):
        self.repo_path = Path(repo_path)
        self.docs_path = self.repo_path / "docs"
        self.implementation_path = self.docs_path / "implementation"
        
    def parse_checklist_file(self, file_path: Path) -> Dict[str, Any]:
        """Parse a checklist markdown file and extract trackable items"""
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            
        # Extract metadata
        file_info = {
            "file_path": str(file_path.relative_to(self.repo_path)),
            "component": self._extract_component_name(file_path),
            "sections": [],
            "total_items": 0,
            "completed_items": 0
        }
        
        current_section = None
        current_subsection = None
        
        lines = content.split('\n')
        for line in lines:
            # Section headers (## or ###)
            if line.startswith('## ') and not line.startswith('### '):
                current_section = {
                    "title": line[3:].strip(),
                    "level": 2,
                    "subsections": [],
                    "items": []
                }
                file_info["sections"].append(current_section)
                current_subsection = None
                
            elif line.startswith('### '):
                if current_section:
                    current_subsection = {
                        "title": line[4:].strip(),
                        "level": 3,
                        "items": []
                    }
                    current_section["subsections"].append(current_subsection)
                    
            # Checklist items
            elif re.match(r'^\s*- \[[x ]\]', line):
                item = self._parse_checklist_item(line)
                file_info["total_items"] += 1
                
                if item["completed"]:
                    file_info["completed_items"] += 1
                    
                # Add to appropriate section
                if current_subsection:
                    current_subsection["items"].append(item)
                elif current_section:
                    current_section["items"].append(item)
                    
        return file_info
    
    def _extract_component_name(self, file_path: Path) -> str:
        """Extract component name from file path"""
        parts = file_path.parts
        if "network" in parts:
            return "Network"
        elif "agent" in str(file_path).lower():
            return "Agent System"
        elif "task" in str(file_path).lower():
            return "Task Processing"
        elif "storage" in str(file_path).lower():
            return "Storage"
        else:
            return "Core"
            
    def _parse_checklist_item(self, line: str) -> Dict[str, Any]:
        """Parse a single checklist item"""
        # Extract completion status
        completed = '[x]' in line
        
        # Extract text and any sub-items
        text = re.sub(r'^\s*- \[[x ]\]\s*', '', line).strip()
        
        # Extract any additional metadata
        priority = "medium"  # default
        if any(word in text.lower() for word in ["critical", "urgent", "blocker"]):
            priority = "high"
        elif any(word in text.lower() for word in ["nice to have", "optional", "future"]):
            priority = "low"
            
        # Extract testing type if present
        test_type = None
        if "unit test" in text.lower():
            test_type = "unit-testing"
        elif "integration test" in text.lower():
            test_type = "integration-testing"
        elif "performance test" in text.lower():
            test_type = "performance-testing"
        elif "security test" in text.lower():
            test_type = "security-testing"
            
        return {
            "text": text,
            "completed": completed,
            "priority": priority,
            "test_type": test_type,
            "line": line.strip()
        }
    
    def generate_github_issues(self, checklist_data: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate GitHub issues from parsed checklist data"""
        issues = []
        component = checklist_data["component"]
        
        for section in checklist_data["sections"]:
            # Create epic issue for major sections
            if section["level"] == 2 and (section["subsections"] or len(section["items"]) > 3):
                epic_issue = self._create_epic_issue(section, component, checklist_data["file_path"])
                issues.append(epic_issue)
                
                # Create individual issues for subsections/items
                for subsection in section["subsections"]:
                    for item in subsection["items"]:
                        if not item["completed"]:
                            issue = self._create_task_issue(item, subsection["title"], component, 
                                                          epic_issue["title"], checklist_data["file_path"])
                            issues.append(issue)
                            
                # Create issues for direct section items
                for item in section["items"]:
                    if not item["completed"]:
                        issue = self._create_task_issue(item, section["title"], component,
                                                      epic_issue["title"], checklist_data["file_path"])
                        issues.append(issue)
            else:
                # Create individual issues for smaller sections
                for item in section["items"]:
                    if not item["completed"]:
                        issue = self._create_task_issue(item, section["title"], component,
                                                      None, checklist_data["file_path"])
                        issues.append(issue)
                        
        return issues
    
    def _create_epic_issue(self, section: Dict[str, Any], component: str, file_path: str) -> Dict[str, Any]:
        """Create an epic issue for a major section"""
        total_items = len(section["items"])
        for subsection in section["subsections"]:
            total_items += len(subsection["items"])
            
        completed_items = sum(1 for item in section["items"] if item["completed"])
        for subsection in section["subsections"]:
            completed_items += sum(1 for item in subsection["items"] if item["completed"])
            
        progress = f"{completed_items}/{total_items}"
        
        title = f"🏗️ [{component}] {section['title']}"
        
        body = f"""## Epic: {section['title']}

**Component**: {component}
**Progress**: {progress} items completed
**Source**: `{file_path}`

### Overview
This epic tracks the implementation of {section['title'].lower()} for the {component} component.

### Acceptance Criteria
"""
        
        # Add acceptance criteria from subsections and items
        for subsection in section["subsections"]:
            body += f"\n#### {subsection['title']}\n"
            for item in subsection["items"]:
                status = "✅" if item["completed"] else "⏳"
                body += f"- {status} {item['text']}\n"
                
        for item in section["items"]:
            status = "✅" if item["completed"] else "⏳"
            body += f"- {status} {item['text']}\n"
            
        body += f"""
### Related Documentation
- [Implementation Checklist]({file_path})
- [Implementation Plan](docs/implementation/IMPLEMENTATION_PLAN.md)

### Definition of Done
- [ ] All sub-tasks completed
- [ ] Code reviewed and merged
- [ ] Tests passing (unit, integration, performance)
- [ ] Documentation updated
- [ ] Security review completed (if applicable)

---
*This issue was auto-generated from implementation checklists*
"""
        
        labels = [
            "epic",
            f"component:{component.lower().replace(' ', '-')}",
            "implementation",
            "documentation"
        ]
        
        if any("security" in item["text"].lower() for item in section["items"]):
            labels.append("security")
        if any("performance" in item["text"].lower() for item in section["items"]):
            labels.append("performance")
            
        return {
            "title": title,
            "body": body,
            "labels": labels,
            "milestone": f"{component} Implementation",
            "assignees": [],
            "type": "epic"
        }
    
    def _create_task_issue(self, item: Dict[str, Any], section_title: str, component: str,
                          epic_title: Optional[str], file_path: str) -> Dict[str, Any]:
        """Create a task issue for a checklist item"""
        
        # Determine issue type and emoji
        if item["test_type"]:
            emoji = "🧪"
            issue_type = "testing"
        elif "documentation" in item["text"].lower():
            emoji = "📝"
            issue_type = "documentation"
        elif any(word in item["text"].lower() for word in ["implement", "create", "build"]):
            emoji = "🔧"
            issue_type = "implementation"
        else:
            emoji = "✨"
            issue_type = "enhancement"
            
        title = f"{emoji} [{component}] {item['text']}"
        
        body = f"""## Task: {item['text']}

**Component**: {component}
**Section**: {section_title}
**Priority**: {item['priority']}
**Source**: `{file_path}`
"""
        
        if epic_title:
            body += f"**Epic**: {epic_title}\n"
            
        if item["test_type"]:
            body += f"**Test Type**: {item['test_type']}\n"
            
        body += f"""
### Description
{self._generate_task_description(item, section_title, component)}

### Acceptance Criteria
- [ ] Implementation completed
- [ ] Code follows project standards
- [ ] Tests written and passing
- [ ] Documentation updated
- [ ] Code reviewed
"""
        
        if item["test_type"]:
            body += f"- [ ] {item['test_type'].replace('-', ' ').title()} coverage adequate\n"
            
        body += """
### Implementation Notes
<!-- Add implementation details, design decisions, or notes here -->

### Testing Strategy
<!-- Describe how this will be tested -->

### Documentation Updates
<!-- List documentation that needs to be updated -->

---
*This issue was auto-generated from implementation checklists*
"""
        
        # Generate labels
        labels = [
            issue_type,
            f"component:{component.lower().replace(' ', '-')}",
            f"priority:{item['priority']}"
        ]
        
        if item["test_type"]:
            labels.append(item["test_type"])
            
        if any(word in item["text"].lower() for word in ["security", "auth", "encryption"]):
            labels.append("security")
        if any(word in item["text"].lower() for word in ["performance", "benchmark", "optimization"]):
            labels.append("performance")
        if "api" in item["text"].lower():
            labels.append("api")
            
        return {
            "title": title,
            "body": body,
            "labels": labels,
            "milestone": f"{component} Implementation",
            "assignees": [],
            "type": "task",
            "epic": epic_title
        }
    
    def _generate_task_description(self, item: Dict[str, Any], section_title: str, component: str) -> str:
        """Generate a description for a task based on its content"""
        text = item["text"].lower()
        
        if "unit test" in text:
            return f"Implement comprehensive unit tests for {component} {section_title.lower()}. Ensure all public APIs, edge cases, and error conditions are covered."
        elif "integration test" in text:
            return f"Create integration tests to verify {component} {section_title.lower()} works correctly with other components and handles end-to-end workflows."
        elif "performance test" in text:
            return f"Develop performance tests for {component} {section_title.lower()} to ensure response time, resource usage, and scalability requirements are met."
        elif "security test" in text:
            return f"Implement security tests for {component} {section_title.lower()} including authentication, authorization, input validation, and secure communication."
        elif "documentation" in text:
            return f"Create or update documentation for {component} {section_title.lower()} ensuring it's comprehensive, accurate, and follows project standards."
        else:
            return f"Implement {item['text']} as part of the {component} {section_title.lower()} component according to the project specifications."
    
    def generate_milestone_plan(self, all_issues: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Generate milestone plan based on dependencies and priorities"""
        milestones = {}
        
        # Group by component
        by_component = {}
        for issue in all_issues:
            component = None
            for label in issue["labels"]:
                if label.startswith("component:"):
                    component = label.split(":")[1]
                    break
            if component:
                if component not in by_component:
                    by_component[component] = []
                by_component[component].append(issue)
        
        # Create milestone plan
        for component, issues in by_component.items():
            epics = [i for i in issues if i["type"] == "epic"]
            tasks = [i for i in issues if i["type"] == "task"]
            
            milestones[f"{component.replace('-', ' ').title()} Implementation"] = {
                "title": f"{component.replace('-', ' ').title()} Implementation",
                "description": f"Complete implementation and testing of {component.replace('-', ' ')} component",
                "due_date": None,  # Will be set based on project timeline
                "epics": len(epics),
                "tasks": len(tasks),
                "total_issues": len(issues)
            }
            
        return milestones
    
    def export_issues_json(self, issues: List[Dict[str, Any]], output_path: Path):
        """Export issues to JSON format for GitHub CLI or API"""
        export_data = {
            "generated_at": datetime.now().isoformat(),
            "total_issues": len(issues),
            "epics": len([i for i in issues if i["type"] == "epic"]),
            "tasks": len([i for i in issues if i["type"] == "task"]),
            "issues": issues
        }
        
        with open(output_path, 'w') as f:
            json.dump(export_data, f, indent=2)
            
    def generate_github_cli_script(self, issues: List[Dict[str, Any]], output_path: Path):
        """Generate GitHub CLI script to create all issues"""
        script_lines = [
            "#!/bin/bash",
            "# Auto-generated script to create GitHub issues from implementation checklists",
            "set -e",
            "",
            "echo '🚀 Creating GitHub issues from implementation checklists...'",
            ""
        ]
        
        for i, issue in enumerate(issues, 1):
            labels_str = ",".join(issue["labels"])
            
            # Escape quotes and special characters
            title = issue["title"].replace('"', '\\"')
            body_file = f"issue_{i}_body.md"
            
            script_lines.extend([
                f"# Issue {i}: {title}",
                f"cat > {body_file} << 'EOF'",
                issue["body"],
                "EOF",
                "",
                f'gh issue create \\',
                f'  --title "{title}" \\',
                f'  --body-file {body_file} \\',
                f'  --label "{labels_str}" \\',
                f'  --milestone "{issue["milestone"]}"',
                "",
                f"rm {body_file}",
                f"echo '✅ Created issue {i}: {title}'",
                ""
            ])
            
        script_lines.extend([
            "echo '🎉 All issues created successfully!'",
            "echo 'View issues: gh issue list'",
            "echo 'View milestones: gh api repos/:owner/:repo/milestones'"
        ])
        
        with open(output_path, 'w') as f:
            f.write('\n'.join(script_lines))
            
        # Make script executable
        os.chmod(output_path, 0o755)

def main():
    parser = argparse.ArgumentParser(description="Convert implementation checklists to GitHub issues")
    parser.add_argument("--repo-path", default=".", help="Path to repository root")
    parser.add_argument("--output-dir", default="./github_issues", help="Output directory for generated files")
    parser.add_argument("--dry-run", action="store_true", help="Generate files without creating issues")
    
    args = parser.parse_args()
    
    converter = ChecklistToIssuesConverter(args.repo_path)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(exist_ok=True)
    
    print("🔍 Scanning for implementation checklists...")
    
    # Find all checklist files
    checklist_files = [
        converter.implementation_path / "CHECKLIST.md",
        converter.implementation_path / "network" / "CHECKLIST.md"
    ]
    
    # Add any other checklist files found
    for file_path in converter.implementation_path.rglob("*CHECKLIST*.md"):
        if file_path not in checklist_files:
            checklist_files.append(file_path)
    
    all_issues = []
    
    for file_path in checklist_files:
        if file_path.exists():
            print(f"📋 Processing {file_path.relative_to(converter.repo_path)}...")
            checklist_data = converter.parse_checklist_file(file_path)
            issues = converter.generate_github_issues(checklist_data)
            all_issues.extend(issues)
            print(f"   Generated {len(issues)} issues ({checklist_data['completed_items']}/{checklist_data['total_items']} items completed)")
    
    print(f"\n📊 Summary:")
    print(f"   Total issues: {len(all_issues)}")
    print(f"   Epics: {len([i for i in all_issues if i['type'] == 'epic'])}")
    print(f"   Tasks: {len([i for i in all_issues if i['type'] == 'task'])}")
    
    # Generate milestone plan
    milestones = converter.generate_milestone_plan(all_issues)
    
    # Export files
    converter.export_issues_json(all_issues, output_dir / "issues.json")
    converter.generate_github_cli_script(all_issues, output_dir / "create_issues.sh")
    
    # Generate summary report
    with open(output_dir / "summary.md", 'w') as f:
        f.write(f"""# GitHub Issues Generation Summary

Generated {len(all_issues)} GitHub issues from implementation checklists.

## Issues Breakdown
- **Epics**: {len([i for i in all_issues if i['type'] == 'epic'])}
- **Tasks**: {len([i for i in all_issues if i['type'] == 'task'])}

## Milestones
""")
        for milestone_name, milestone_data in milestones.items():
            f.write(f"- **{milestone_name}**: {milestone_data['total_issues']} issues ({milestone_data['epics']} epics, {milestone_data['tasks']} tasks)\n")
            
        f.write(f"""
## Next Steps
1. Review generated issues in `issues.json`
2. Run `./create_issues.sh` to create issues in GitHub
3. Set up milestones in GitHub project settings
4. Assign team members to appropriate issues
5. Start tracking progress!

## Files Generated
- `issues.json` - All issues in JSON format
- `create_issues.sh` - Script to create issues via GitHub CLI
- `summary.md` - This summary file

Generated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
""")
    
    print(f"\n✅ Generated files in {output_dir}:")
    print(f"   📄 issues.json - {len(all_issues)} issues in JSON format")
    print(f"   📜 create_issues.sh - GitHub CLI script")
    print(f"   📋 summary.md - Generation summary")
    
    if args.dry_run:
        print(f"\n🔍 Dry run mode - no issues created")
        print(f"   Review the generated files and run create_issues.sh when ready")
    else:
        print(f"\n🚀 To create issues in GitHub:")
        print(f"   cd {output_dir}")
        print(f"   ./create_issues.sh")

if __name__ == "__main__":
    main()
