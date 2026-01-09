# BMAD-METHOD Update Guide

## Overview

This guide explains how to update the BMAD-METHOD (Breakthrough Method for Agile AI-Driven Development) framework used in this project, including how to enable OpenCode support.

## Current Installation

This project currently has:
- **Version**: 6.0.0-alpha.22
- **Installation Location**: `_bmad/` directory
- **Current IDE**: GitHub Copilot
- **Modules**: core, bmm

## What is OpenCode?

OpenCode is an open source AI coding agent that:
- Supports multiple AI language models (Claude, GPT, Gemini, and more)
- Works across terminals, IDEs, and desktop applications
- Provides privacy-first development (doesn't store user code or context)
- Supports parallel multi-agent sessions
- Has over 50,000 GitHub stars

## Updating BMAD-METHOD

### Method 1: NPX Update (Recommended)

The easiest way to update BMAD-METHOD is using NPX, which always fetches the latest version:

#### For Stable Version:
```bash
npx bmad-method install
```

#### For Latest Alpha/Dev Version:
```bash
npx bmad-method@alpha install
```

The installer will:
1. Detect your current installation in `_bmad/`
2. Prompt you to update
3. Ask which IDE you want to use (GitHub Copilot, OpenCode, etc.)
4. Update all agents, modules, and configuration files
5. Configure your selected IDE integration

### Method 2: Fast In-Place Update

For quick updates without reconfiguring:
```bash
npx bmad-method update
```

This intelligently merges changes while preserving your customizations.

### Method 3: Global NPM Update

If you have BMAD installed globally:
```bash
npm update -g bmad-method
```

Or for a specific alpha version:
```bash
npm install -g bmad-method@latest
```

## Switching to OpenCode

### During Installation/Update

When you run the installer, you'll be prompted to select your IDE. Choose **OpenCode** from the list.

### OpenCode Configuration

OpenCode integration works differently than GitHub Copilot:

**GitHub Copilot** (current setup):
- Uses `.github/copilot-instructions.md`
- Agent prompts are always active
- Agents are invoked directly in chat

**OpenCode** (after update):
- Can use `.opencode.jsonc` file OR `.opencode/` directory
- Agents are switchable modes
- You must explicitly activate an agent before using it
- Supports subagents for specialized workflows

### OpenCode-Specific Features

OpenCode supports:
- **Subagents**: Specialized modes for different workflow steps
- **Markdown-based agent definitions**: Store agent configurations in `.opencode/*.md` files
- **Multiple model support**: Switch between 75+ LLM providers
- **Local model support**: Use locally-hosted models for complete privacy

## Post-Update Verification

After updating, verify the installation:

### Check Version
```bash
cat _bmad/_config/manifest.yaml
```

Look for the `installation.version` field.

### Verify IDE Configuration
```bash
cat _bmad/_config/manifest.yaml
```

Look for the `ides` field - it should list your selected IDE(s).

### Check Module Structure

After update, you should see:
```
_bmad/
├── _config/           # Configuration and manifests
├── core/              # Core BMAD resources
├── bmm/               # BMM module (if installed)
└── [other modules]    # Any additional modules
```

### For OpenCode Users

If you selected OpenCode, check for:
```bash
ls -la .opencode*
```

You should see either:
- `.opencode.jsonc` - Configuration file
- `.opencode/` - Directory with agent definitions

## Version History Changes

### v6.x Changes
- **Modularization**: Changed from `.bmad-core/` to `.bmad/` with modular structure
- **Multiple IDE Support**: Added OpenCode, Claude Code, Codex
- **AgentVibes**: Enhanced agent communication features
- **Improved Update Process**: Smarter merging of customizations

### Migration from v4 to v6

If you're migrating from an older v4 installation:
1. The installer will handle directory structure changes
2. Follow prompts for manual intervention if needed
3. Review migration notes in the installer output

## Troubleshooting

### Update Doesn't Detect Current Installation

If the update command doesn't detect your `_bmad/` installation:
```bash
# Remove and reinstall
rm -rf _bmad/
npx bmad-method@alpha install
```

⚠️ **Warning**: This will remove any custom agents or modifications. Back up first!

### Permission Errors

If you get permission errors:
```bash
# Use sudo for global installs only
sudo npm update -g bmad-method
```

For local project updates, fix permissions:
```bash
sudo chown -R $USER:$USER _bmad/
```

### IDE Not Switching

After update, if the IDE isn't switching:
1. Check `_bmad/_config/manifest.yaml` - it should list your chosen IDE
2. Look for IDE-specific configuration files
3. Restart your IDE/editor
4. For OpenCode: Ensure you have OpenCode installed separately

### OpenCode Not Found

OpenCode is a separate tool that must be installed:
```bash
# Install OpenCode separately
npm install -g @opencode/cli
# or follow OpenCode installation docs
```

BMAD-METHOD only configures the integration - it doesn't install OpenCode itself.

## Keeping Up-to-Date

### Check for Updates

Visit the [BMAD-METHOD releases page](https://github.com/bmad-code-org/BMAD-METHOD/releases) to see:
- Latest version numbers
- New features and bug fixes
- Breaking changes
- Migration guides

### Recommended Update Schedule

- **Alpha users**: Update monthly to get latest features
- **Stable users**: Update quarterly or when new major features are released
- **Always** update if security fixes are announced

## Additional Resources

- [BMAD-METHOD GitHub](https://github.com/bmad-code-org/BMAD-METHOD)
- [BMAD Installation Docs](https://deepwiki.com/bmadcode/BMAD-METHOD/2-installation-and-setup)
- [OpenCode Official Site](https://opencode.ai/)
- [OpenCode Documentation](https://opencode.ai/docs/)
- [V6 OpenCode Support Issue](https://github.com/bmad-code-org/BMAD-METHOD/issues/818)

## Summary

To update BMAD-METHOD with OpenCode support:

1. **Run the installer**:
   ```bash
   npx bmad-method@alpha install
   ```

2. **Select OpenCode** when prompted for IDE choice

3. **Verify** the update:
   ```bash
   cat _bmad/_config/manifest.yaml
   ```

4. **Install OpenCode separately** if not already installed

5. **Configure your agents** in `.opencode/` directory or `.opencode.jsonc`

The process is straightforward and the installer handles most of the complexity automatically.
