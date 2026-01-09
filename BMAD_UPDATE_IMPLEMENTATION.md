# BMAD-METHOD Update Implementation Summary

## Issue Overview

**Problem Statement**: "This project used to run the copilot with the https://github.com/bmad-code-org/BMAD-METHOD. How can I use an updated version in opencode?"

## Solution Implemented

This implementation provides comprehensive documentation and tooling to update the BMAD-METHOD framework and enable OpenCode support for this project.

## What Was Changed

### 1. Comprehensive Documentation

#### Main Update Guide: `docs/development/bmad-method-update-guide.md`
- **Purpose**: Complete guide for updating BMAD-METHOD
- **Content**:
  - Current installation details (v6.0.0-alpha.22)
  - What OpenCode is and how it differs from GitHub Copilot
  - Three update methods (NPX, fast update, global NPM)
  - Step-by-step instructions for switching to OpenCode
  - OpenCode-specific features and configuration
  - Post-update verification steps
  - Version history and migration notes
  - Comprehensive troubleshooting section
  - Links to external resources

#### Quick Reference: `docs/development/bmad-quick-reference.md`
- **Purpose**: Fast lookup for common commands
- **Content**:
  - Quick update commands
  - Comparison table: GitHub Copilot vs OpenCode
  - IDE switching instructions
  - Current project setup summary
  - Common issues and solutions
  - One-command references

### 2. Interactive Update Script

#### Script: `scripts/update-bmad.sh`
- **Purpose**: User-friendly interactive update tool
- **Features**:
  - Shows current BMAD configuration
  - Provides 4 options:
    1. Update to latest ALPHA (recommended for OpenCode)
    2. Update to latest STABLE
    3. Fast update (preserves customizations)
    4. Cancel
  - Displays new configuration after update
  - Provides next steps and documentation links
  - Checks for `_bmad/` directory existence
- **Usage**: `./scripts/update-bmad.sh`

### 3. Documentation Index Updates

- **`docs/INDEX.md`**: Added links to both new BMAD guides
- **`docs/development/README.md`**: Added BMAD guides to related documentation
- **`scripts/README.md`**: Documented the new update script with usage examples
- **`README.md`**: Added BMAD-METHOD section in development workflow

### 4. Cross-References

All documentation is properly cross-referenced:
- Main README → BMAD guides
- Documentation index → BMAD guides
- Development README → BMAD guides
- Scripts README → Update script
- Guides reference each other appropriately

## Key Information Provided

### Current BMAD Installation
- **Version**: 6.0.0-alpha.22
- **Location**: `_bmad/` directory
- **Current IDE**: GitHub Copilot
- **Modules**: core, bmm
- **Installation Date**: 2026-01-05

### OpenCode vs GitHub Copilot

| Aspect | GitHub Copilot | OpenCode |
|--------|---------------|----------|
| **Configuration** | `.github/copilot-instructions.md` | `.opencode.jsonc` or `.opencode/` |
| **Agent Activation** | Always active | Must switch modes |
| **Agent Invocation** | Direct in chat | Greet after switching |
| **Cost** | Paid subscription | Free & open source |
| **Privacy** | Cloud-based | Can be fully local |
| **Model Support** | GitHub's models | 75+ LLM providers + local |

### Update Commands

#### Primary Method (Recommended):
```bash
npx bmad-method@alpha install
```

#### Alternative Methods:
```bash
# Fast update
npx bmad-method update

# Interactive script
./scripts/update-bmad.sh

# Global update
npm update -g bmad-method
```

### OpenCode Support Information

From research and GitHub issues:
- OpenCode support was added in BMAD v6
- Issue #818 tracked OpenCode integration
- OpenCode uses different agent activation model
- Agents are "switchable modes" vs always-active
- Supports subagents for specialized workflows
- Configuration via `.opencode.jsonc` or `.opencode/` directory

## Files Created

1. `docs/development/bmad-method-update-guide.md` (6.3 KB)
2. `docs/development/bmad-quick-reference.md` (2.6 KB)
3. `scripts/update-bmad.sh` (3.0 KB, executable)

## Files Modified

1. `docs/INDEX.md` - Added BMAD guide links
2. `docs/development/README.md` - Added BMAD guides to related docs
3. `scripts/README.md` - Documented update script
4. `README.md` - Added BMAD-METHOD section in development workflow

## Validation Performed

✅ Script execution tested (cancel option works correctly)
✅ All files created successfully
✅ Git commits completed
✅ Documentation cross-references verified
✅ File permissions set correctly (script is executable)
✅ Markdown formatting validated

## How Users Can Update

### Simple Method:
```bash
./scripts/update-bmad.sh
```
Follow the interactive prompts to select update method and IDE.

### Advanced Method:
```bash
# For OpenCode support
npx bmad-method@alpha install

# When prompted, select "OpenCode" as your IDE
# The installer will configure everything automatically
```

### Post-Update:
1. Verify version: `cat _bmad/_config/manifest.yaml`
2. Check IDE setting in manifest
3. Install OpenCode separately if not already installed
4. Look for `.opencode*` configuration files

## Important Notes

### BMAD is NOT Installed Globally
- The project has BMAD installed locally in `_bmad/`
- Updates should use `npx` (automatically uses latest version)
- No global `bmad` command is available

### OpenCode Must Be Installed Separately
- BMAD only configures the integration
- OpenCode itself must be installed: `npm install -g @opencode/cli`
- Or follow OpenCode's official installation guide

### Configuration Differences
- **GitHub Copilot**: Uses `.github/copilot-instructions.md`
- **OpenCode**: Uses `.opencode.jsonc` or `.opencode/` directory
- Both can coexist, but only one is active at a time

### Version Channels
- **Alpha**: Latest features, including OpenCode support (recommended)
- **Stable**: Production-ready features only
- Project currently on alpha channel (v6.0.0-alpha.22)

## Additional Resources

### Documentation Created:
- [BMAD Update Guide](docs/development/bmad-method-update-guide.md)
- [BMAD Quick Reference](docs/development/bmad-quick-reference.md)

### External Resources Referenced:
- [BMAD-METHOD GitHub](https://github.com/bmad-code-org/BMAD-METHOD)
- [BMAD Installation Docs](https://deepwiki.com/bmadcode/BMAD-METHOD/2-installation-and-setup)
- [OpenCode Official Site](https://opencode.ai/)
- [V6 OpenCode Support Issue](https://github.com/bmad-code-org/BMAD-METHOD/issues/818)

## Success Criteria Met

✅ **Question Answered**: How to update BMAD-METHOD for OpenCode
✅ **Documentation Provided**: Comprehensive guides with examples
✅ **Tooling Created**: Interactive update script
✅ **Current State Documented**: Version, location, configuration
✅ **Comparison Provided**: OpenCode vs GitHub Copilot differences
✅ **Troubleshooting Included**: Common issues and solutions
✅ **References Added**: External resources and links
✅ **Integration Complete**: All docs properly linked and indexed

## Next Steps for Users

1. **Review the documentation**:
   - Read: `docs/development/bmad-method-update-guide.md`
   - Quick ref: `docs/development/bmad-quick-reference.md`

2. **Run the update**:
   ```bash
   ./scripts/update-bmad.sh
   ```

3. **Select OpenCode** when prompted for IDE

4. **Install OpenCode** if not already installed

5. **Verify** the update in `_bmad/_config/manifest.yaml`

6. **Start using** OpenCode with BMAD agents

---

**Implementation completed successfully!** 🎉

All documentation, tooling, and cross-references are in place to help users update BMAD-METHOD and enable OpenCode support.
