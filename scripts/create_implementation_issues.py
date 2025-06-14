#!/usr/bin/env python3
"""
Create GitHub Issues from Implementation Checklists
"""

import re
import json
import sys
import subprocess
import time
from pathlib import Path

def parse_checklist(file_path, component_name):
    """Parse checklist file and extract unchecked items"""
    if not Path(file_path).exists():
        print(f"⚠️  File not found: {file_path}")
        return []
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    issues = []
    current_section = "General"
    current_subsection = None
    
    lines = content.split('\n')
    for line_num, line in enumerate(lines, 1):
        # Track sections
        if line.startswith('## ') and not line.startswith('### '):
            current_section = line[3:].strip()
            current_subsection = None
        elif line.startswith('### '):
            current_subsection = line[4:].strip()
            
        # Find unchecked items
        if re.match(r'^\s*- \[ \]', line):
            item_text = re.sub(r'^\s*- \[ \]\s*', '', line).strip()
            if item_text:
                # Determine issue type and priority
                labels = ["implementation", f"component-{component_name.lower().replace(' ', '-')}"]
                priority = "medium"
                
                if "unit test" in item_text.lower():
                    labels.extend(["testing", "unit-testing"])
                    emoji = "🧪"
                elif "integration test" in item_text.lower():
                    labels.extend(["testing", "integration-testing"])
                    emoji = "🔗"
                elif "performance test" in item_text.lower():
                    labels.extend(["testing", "performance"])
                    emoji = "⚡"
                elif "security test" in item_text.lower():
                    labels.extend(["testing", "security"])
                    emoji = "🔒"
                elif "documentation" in item_text.lower():
                    labels.append("documentation")
                    emoji = "📝"
                else:
                    emoji = "🔧"
                    
                if any(word in item_text.lower() for word in ["critical", "essential", "required"]):
                    priority = "high"
                    labels.append("priority-high")
                elif any(word in item_text.lower() for word in ["optional", "nice to have"]):
                    priority = "low"
                    labels.append("priority-low")
                else:
                    labels.append("priority-medium")
                    
                section_context = current_subsection if current_subsection else current_section
                
                title = f"{emoji} [{component_name}] {item_text}"
                
                body = f"""## Implementation Task

**Component**: {component_name}
**Section**: {section_context}
**Priority**: {priority}
**Source**: `{file_path}` (line {line_num})

### Description
{item_text}

This task is part of the {component_name} component implementation, specifically for {section_context.lower()}.

### Acceptance Criteria
- [ ] Implementation completed according to specifications
- [ ] Code follows project coding standards and best practices
- [ ] Appropriate tests written and passing (unit, integration, performance as applicable)
- [ ] Documentation updated to reflect changes
- [ ] Code reviewed and approved by team
- [ ] Security considerations addressed (if applicable)

### Implementation Guidelines
<!-- Add specific implementation notes, design decisions, or technical requirements here -->

### Testing Strategy
<!-- Describe the testing approach for this task -->

### Definition of Done
- [ ] All acceptance criteria met
- [ ] Code merged to main branch
- [ ] CI/CD pipeline passing
- [ ] Stakeholder approval received

---
*This issue was auto-generated from implementation checklist: `{file_path}`*
*Line: {line_num} | Section: {section_context}*
"""

                issues.append({
                    "title": title,
                    "body": body,
                    "labels": labels,
                    "component": component_name,
                    "section": section_context,
                    "priority": priority,
                    "source_file": file_path,
                    "source_line": line_num
                })
    
    return issues

def create_github_issues(issues, dry_run=False):
    """Create GitHub issues using GitHub CLI"""
    if dry_run:
        print("🔍 DRY RUN MODE - No issues will be created")
        return
        
    print(f"🚀 Creating {len(issues)} GitHub issues...")
    
    created_count = 0
    failed_count = 0
    
    for i, issue in enumerate(issues, 1):
        try:
            print(f"Creating issue {i}/{len(issues)}: {issue['title'][:60]}...")
            
            # Write issue body to temp file
            with open(f"issue_body_{i}.md", "w") as f:
                f.write(issue["body"])
            
            # Create issue using GitHub CLI
            cmd = [
                "gh", "issue", "create",
                "--title", issue["title"],
                "--body-file", f"issue_body_{i}.md",
                "--label", ",".join(issue["labels"])
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0:
                created_count += 1
                issue_url = result.stdout.strip()
                print(f"✅ Created: {issue_url}")
            else:
                failed_count += 1
                print(f"❌ Failed: {result.stderr}")
            
            # Clean up temp file
            subprocess.run(["rm", f"issue_body_{i}.md"], check=False)
            
            # Rate limiting - be nice to GitHub API
            if i % 10 == 0:
                print("⏸️  Pausing for rate limiting...")
                time.sleep(2)
                
        except Exception as e:
            failed_count += 1
            print(f"❌ Exception creating issue {i}: {e}")
            subprocess.run([f"rm", "issue_body_{i}.md"], shell=True, check=False)
    
    print(f"\n📊 Results:")
    print(f"✅ Created: {created_count} issues")
    print(f"❌ Failed: {failed_count} issues")
    print(f"📋 Total: {len(issues)} issues processed")
    
    if created_count > 0:
        print(f"\n🎉 Successfully created {created_count} GitHub issues!")
        print(f"View them: gh issue list --label implementation")

def main():
    dry_run = "--dry-run" in sys.argv
    component_filter = None
    
    # Parse command line arguments
    for i, arg in enumerate(sys.argv):
        if arg == "--component" and i + 1 < len(sys.argv):
            component_filter = sys.argv[i + 1]
    
    print("📋 Creating GitHub Issues from Implementation Checklists")
    print("=" * 60)
    
    # Parse checklist files
    all_issues = []
    
    checklist_files = [
        ("docs/implementation/CHECKLIST.md", "Core Implementation"),
        ("docs/implementation/network/CHECKLIST.md", "Network")
    ]
    
    for file_path, component in checklist_files:
        if component_filter and component_filter.lower() not in component.lower():
            print(f"⏭️  Skipping {component} (filtered out)")
            continue
            
        if Path(file_path).exists():
            print(f"📋 Processing {file_path} ({component})...")
            issues = parse_checklist(file_path, component)
            all_issues.extend(issues)
            print(f"   Found {len(issues)} tasks to implement")
        else:
            print(f"⚠️  File not found: {file_path}")
    
    print(f"\n📊 Total: {len(all_issues)} GitHub issues ready to create")
    
    # Group by component for summary
    by_component = {}
    for issue in all_issues:
        comp = issue["component"]
        if comp not in by_component:
            by_component[comp] = {"total": 0, "by_type": {}}
        by_component[comp]["total"] += 1
        
        # Count by type
        issue_type = "implementation"
        if "testing" in issue["labels"]:
            issue_type = "testing"
        elif "documentation" in issue["labels"]:
            issue_type = "documentation"
            
        by_component[comp]["by_type"][issue_type] = by_component[comp]["by_type"].get(issue_type, 0) + 1
    
    print("\n📈 Breakdown by component:")
    for comp, data in by_component.items():
        print(f"   {comp}: {data['total']} issues")
        for issue_type, count in data["by_type"].items():
            print(f"     - {issue_type}: {count}")
    
    # Save issues data
    with open("implementation_issues.json", "w") as f:
        json.dump(all_issues, f, indent=2)
    print(f"\n💾 Saved issue data to implementation_issues.json")
    
    if dry_run:
        print("\n🔍 DRY RUN MODE - No issues were created")
        print("Review the issues data and run without --dry-run to create them.")
    else:
        # Confirm before creating
        print(f"\n⚠️  About to create {len(all_issues)} GitHub issues!")
        response = input("Continue? (y/N): ")
        if response.lower() == 'y':
            create_github_issues(all_issues, dry_run=False)
        else:
            print("❌ Cancelled by user")

if __name__ == "__main__":
    main()
