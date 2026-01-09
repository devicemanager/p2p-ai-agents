# 🤖 GitHub Workflows for Documentation

This directory contains GitHub Actions workflows that automate documentation quality, maintenance, and health monitoring for the P2P AI Agents project.

## 📋 Available Workflows

### 1. 📚 Documentation Quality Check (`documentation-check.yml`)

**Triggers**: Pull requests and pushes affecting documentation files

**Purpose**: Validates documentation quality and consistency before changes are merged

**Features**:
- ✅ Validates internal links and cross-references
- ✅ Checks for TODO/FIXME items
- ✅ Runs markdown linting with project standards
- ✅ Generates documentation quality reports
- ✅ Prevents broken documentation from being merged

**Usage**:
```bash
# Automatically runs on:
# - Pull requests touching docs/
# - Pushes to main/master/develop with doc changes

# Manual trigger:
gh workflow run documentation-check.yml
```

### 2. 🔧 Documentation Auto-Maintenance (`documentation-maintenance.yml`)

**Triggers**: Weekly schedule (Sundays 2 AM UTC) + manual dispatch

**Purpose**: Automatically maintains documentation consistency and freshness

**Features**:
- 🕒 Updates timestamps across all documentation
- 🔗 Fixes common link patterns and references
- 📁 Ensures all directories have proper README files
- 📚 Updates quick reference and cross-references
- 🚀 Creates pull requests with maintenance changes

**Usage**:
```bash
# Manual trigger with options:
gh workflow run documentation-maintenance.yml \
  --field force_update=true

# Runs automatically every Sunday at 2 AM UTC
```

**Configuration**:
- Add team members to `reviewers` section
- Configure `assignees` for auto-maintenance PRs
- Adjust schedule in cron expression if needed

### 3. 📝 Generate Documentation Template (`documentation-template.yml`)

**Triggers**: Manual workflow dispatch only

**Purpose**: Creates new documentation from smart templates

**Features**:
- 📝 Generates implementation, architecture, or user-guide docs
- 🎯 Smart defaults based on document type
- ⚙️ Configurable status, difficulty, and output paths
- 🚀 Creates pull requests with generated templates
- 📋 Provides clear next-steps guidance

**Usage**:
```bash
# Generate implementation documentation:
gh workflow run documentation-template.yml \
  --field doc_type=implementation \
  --field component_name="NetworkProtocol" \
  --field status="In Development"

# Generate user guide:
gh workflow run documentation-template.yml \
  --field doc_type=user-guide \
  --field component_name="Advanced Configuration" \
  --field difficulty="Advanced"

# Generate architecture document:
gh workflow run documentation-template.yml \
  --field doc_type=architecture \
  --field component_name="Security Model"
```

**Input Options**:
- `doc_type`: implementation | architecture | user-guide
- `component_name`: Name of the component/guide
- `output_path`: Custom output path (optional)
- `status`: In Development | Completed | Planned | Deprecated
- `difficulty`: Beginner | Intermediate | Advanced

### 4. 📊 Documentation Health Dashboard (`documentation-health.yml`)

**Triggers**: Pushes to main, daily schedule (6 AM UTC) + manual dispatch

**Purpose**: Monitors documentation health and provides actionable insights

**Features**:
- 📊 Calculates comprehensive health score (0-100)
- 📈 Tracks metrics: file count, completion, link health, freshness
- 🚨 Creates issues when health score drops below 60%
- ✅ Auto-closes health issues when problems are resolved
- 💬 Comments on PRs with health status
- 📋 Generates detailed reports and recommendations

**Health Score Factors**:
- Version information presence
- TODO/placeholder completion
- Link validation
- Content freshness (< 6 months old)
- Overall documentation coverage

**Usage**:
```bash
# Manual health check:
gh workflow run documentation-health.yml

# View latest health report:
gh run download --name documentation-health-report
```

## 🔧 Setup Instructions

### 1. **Repository Configuration**

Ensure your repository has these permissions:
```yaml
permissions:
  contents: write        # For committing changes
  pull-requests: write   # For creating PRs
  issues: write         # For health monitoring
```

### 2. **Team Configuration**

Edit the workflows to add your team:

```yaml
# In documentation-maintenance.yml
reviewers: |
  your-username
  team-member-1
  team-member-2

assignees: |
  documentation-lead
```

### 3. **Branch Protection Rules**

Recommended branch protection for main/master:
- ✅ Require status checks to pass
- ✅ Require "Documentation Quality Check" workflow
- ✅ Require up-to-date branches
- ✅ Include administrators

### 4. **Notification Setup**

Configure GitHub notifications for:
- Documentation health issues
- Auto-maintenance PRs
- Template generation PRs

## 📈 Integration with Local Tools

The workflows integrate seamlessly with local documentation tools:

### Local Script Integration
```bash
# The workflows use the same scripts you can run locally:
./scripts/maintain_docs.sh          # Used by maintenance workflow
python scripts/generate_docs.py     # Used by template workflow  
python scripts/validate_docs.py     # Used by quality check workflow
```

### Pre-commit Hook Integration
```bash
# Install the pre-commit hook locally:
cp scripts/pre-commit-docs.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## 🎯 Workflow Triggers Summary

| Workflow | Auto Triggers | Manual Trigger | Frequency |
|----------|---------------|----------------|-----------|
| **Quality Check** | PR + Push to main | ✅ | On demand |
| **Maintenance** | Weekly schedule | ✅ | Weekly |
| **Template Generator** | None | ✅ | On demand |
| **Health Monitor** | Push to main + Daily | ✅ | Daily |

## 📊 Monitoring and Reports

### Artifacts Generated
Each workflow generates artifacts you can download:

```bash
# Download quality reports
gh run download --name documentation-quality-report

# Download maintenance reports  
gh run download --name maintenance-report

# Download health analysis
gh run download --name documentation-health-report

# Download template info
gh run download --name template-generation-info
```

### Health Score Interpretation
- **80-100**: 🟢 Excellent - Documentation is in great shape
- **60-79**: 🟡 Good - Minor improvements needed  
- **40-59**: 🟠 Needs Work - Significant issues to address
- **0-39**: 🔴 Critical - Immediate attention required

## 🚀 Best Practices

### For Contributors
1. **Before Creating PRs**: Let the quality check workflow validate your changes
2. **For New Docs**: Use the template generator for consistency
3. **Regular Updates**: Review and update your documentation monthly

### for Maintainers
1. **Monitor Health Score**: Keep it above 80%
2. **Review Auto-PRs**: Check auto-maintenance PRs weekly
3. **Template Updates**: Keep templates current with project needs
4. **Issue Triage**: Address health issues promptly

### For Teams
1. **Assign Reviewers**: Configure team members in workflow files
2. **Set Expectations**: Document your team's documentation standards
3. **Regular Reviews**: Use health reports for planning documentation work

## 🔧 Customization

### Adjusting Health Thresholds
Edit `documentation-health.yml`:
```python
# Change health score calculation
health_score = max(0, 100 - (issues_count / total_checks * 100))

# Adjust issue creation threshold
if: env.HEALTH_SCORE < 60  # Change this value
```

### Modifying Maintenance Schedule
Edit `documentation-maintenance.yml`:
```yaml
schedule:
  # Change from weekly to daily:
  - cron: '0 2 * * *'  # Daily at 2 AM UTC
  # Or monthly:
  - cron: '0 2 1 * *'  # First day of month at 2 AM UTC
```

### Adding Custom Checks
Extend `documentation-check.yml`:
```yaml
- name: 🎯 Custom Validation
  run: |
    # Add your custom validation logic here
    echo "Running custom documentation checks..."
```

## 🆘 Troubleshooting

### Common Issues

**Workflow Permissions Error**:
```bash
# Ensure repository settings allow GitHub Actions to:
# Settings > Actions > General > Workflow permissions
# Set to "Read and write permissions"
```

**Missing Dependencies**:
```bash
# Workflows use Python 3.11 and standard libraries
# If you add dependencies, update the workflow files
```

**Script Execution Errors**:
```bash
# Ensure scripts are executable:
chmod +x scripts/*.sh

# Check script paths in workflows match repository structure
```

### Getting Help

1. **Check Workflow Logs**: View detailed logs in GitHub Actions tab
2. **Review Artifacts**: Download generated reports for insights
3. **Test Locally**: Run the same scripts locally to debug issues
4. **Health Reports**: Use health monitoring to identify systemic issues

---

## 📚 Related Documentation

- [Documentation Scripts](../../scripts/README.md) - Local tool documentation
- [GLOSSARY.md](../../docs/GLOSSARY.md) - Project terminology
- [Development Guide](../../docs/development/README.md) - Setup and contribution guidelines

---
*This documentation is maintained by the automated documentation system 🤖*
