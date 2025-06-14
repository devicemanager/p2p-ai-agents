#!/usr/bin/env python3
"""
Simple GitHub Issue Generator from Checklists
Creates GitHub issues from implementation checklists
"""

import re
import json
from pathlib import Path
from datetime import datetime

def parse_simple_checklist(file_path):
    """Parse checklist and extract incomplete items"""
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        return []
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    issues = []
    current_section = "General"
    
    lines = content.split('\n')
    for line in lines:
        # Track current section
        if line.startswith('## '):
            current_section = line[3:].strip()
        elif line.startswith('### '):
            current_section = line[4:].strip()
            
        # Find unchecked items
        if re.match(r'^\s*- \[ \]', line):
            item_text = re.sub(r'^\s*- \[ \]\s*', '', line).strip()
            if item_text:
                issues.append({
                    "section": current_section,
                    "task": item_text,
                    "component": extract_component(file_path)
                })
    
    return issues

def extract_component(file_path):
    """Extract component name from file path"""
    path_str = str(file_path).lower()
    if "network" in path_str:
        return "Network"
    elif "agent" in path_str:
        return "Agent"
    elif "task" in path_str:
        return "Task"
    elif "storage" in path_str:
        return "Storage"
    return "Core"

def create_github_issue_data(item):
    """Create GitHub issue data structure"""
    component = item["component"]
    section = item["section"]
    task = item["task"]
    
    # Determine issue type and labels
    labels = ["implementation", f"component-{component.lower()}"]
    
    if "test" in task.lower():
        labels.append("testing")
        emoji = "🧪"
    elif "documentation" in task.lower():
        labels.append("documentation")
        emoji = "📝"
    else:
        emoji = "🔧"
    
    if "unit test" in task.lower():
        labels.append("unit-testing")
    elif "integration test" in task.lower():
        labels.append("integration-testing")
    elif "performance" in task.lower():
        labels.append("performance")
    elif "security" in task.lower():
        labels.append("security")
        
    title = f"{emoji} [{component}] {task}"
    
    body = f"""## Implementation Task

**Component**: {component}
**Section**: {section}
**Task**: {task}

### Description
This task is part of the {component} component implementation, specifically for {section.lower()}.

### Acceptance Criteria
- [ ] Implementation completed according to specifications
- [ ] Code follows project coding standards
- [ ] Appropriate tests written and passing
- [ ] Documentation updated if needed
- [ ] Code reviewed and approved

### Implementation Notes
<!-- Add implementation details, design decisions, or notes here -->

### Testing Strategy
<!-- Describe how this will be tested -->

---
*Auto-generated from implementation checklist*
"""

    return {
        "title": title,
        "body": body,
        "labels": labels,
        "milestone": f"{component} Implementation"
    }

def main():
    print("🔍 Scanning for implementation checklists...")
    
    # List of checklist files to process
    checklist_files = [
        "docs/implementation/CHECKLIST.md",
        "docs/implementation/network/CHECKLIST.md"
    ]
    
    all_issues = []
    
    for file_path in checklist_files:
        if Path(file_path).exists():
            print(f"📋 Processing {file_path}...")
            items = parse_simple_checklist(file_path)
            
            for item in items:
                issue_data = create_github_issue_data(item)
                all_issues.append(issue_data)
                
            print(f"   Found {len(items)} incomplete tasks")
        else:
            print(f"⚠️  Skipping {file_path} (not found)")
    
    print(f"\n📊 Summary: {len(all_issues)} GitHub issues to create")
    
    # Create output directory
    output_dir = Path("github_issues")
    output_dir.mkdir(exist_ok=True)
    
    # Save issues to JSON
    output_file = output_dir / "issues.json"
    with open(output_file, 'w') as f:
        json.dump({
            "generated_at": datetime.now().isoformat(),
            "total_issues": len(all_issues),
            "issues": all_issues
        }, f, indent=2)
    
    # Generate GitHub CLI script
    script_file = output_dir / "create_issues.sh"
    with open(script_file, 'w') as f:
        f.write("#!/bin/bash\n")
        f.write("# Create GitHub issues from implementation checklists\n\n")
        f.write("set -e\n\n")
        f.write("echo '🚀 Creating GitHub issues...'\n\n")
        
        for i, issue in enumerate(all_issues, 1):
            labels_str = ",".join(issue["labels"])
            title = issue["title"].replace('"', '\\"')
            
            f.write(f"# Issue {i}\n")
            f.write(f"cat > issue_{i}.md << 'EOF'\n")
            f.write(issue["body"])
            f.write("\nEOF\n\n")
            
            f.write(f'gh issue create \\\n')
            f.write(f'  --title "{title}" \\\n')
            f.write(f'  --body-file issue_{i}.md \\\n')
            f.write(f'  --label "{labels_str}"\n\n')
            f.write(f"rm issue_{i}.md\n")
            f.write(f"echo '✅ Created: {title}'\n\n")
        
        f.write("echo '🎉 All issues created!'\n")
    
    # Make script executable
    script_file.chmod(0o755)
    
    # Generate summary
    summary_file = output_dir / "summary.md"
    with open(summary_file, 'w') as f:
        f.write("# GitHub Issues Summary\n\n")
        f.write(f"Generated {len(all_issues)} GitHub issues from implementation checklists.\n\n")
        
        # Group by component
        by_component = {}
        for issue in all_issues:
            for label in issue["labels"]:
                if label.startswith("component-"):
                    component = label.replace("component-", "").title()
                    by_component.setdefault(component, []).append(issue)
                    break
        
        f.write("## Issues by Component\n")
        for component, issues in by_component.items():
            f.write(f"- **{component}**: {len(issues)} issues\n")
        
        f.write(f"\n## Files Generated\n")
        f.write(f"- `issues.json` - Issue data in JSON format\n")
        f.write(f"- `create_issues.sh` - Script to create issues via GitHub CLI\n")
        f.write(f"- `summary.md` - This summary\n")
        
        f.write(f"\n## Usage\n")
        f.write(f"```bash\n")
        f.write(f"cd github_issues\n")
        f.write(f"./create_issues.sh\n")
        f.write(f"```\n")
    
    print(f"\n✅ Generated files in {output_dir}/:")
    print(f"   📄 issues.json")
    print(f"   📜 create_issues.sh")
    print(f"   📋 summary.md")
    print(f"\n🚀 To create issues: cd {output_dir} && ./create_issues.sh")

if __name__ == "__main__":
    main()
