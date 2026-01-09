# BMAD-METHOD Quick Reference

## Quick Update Commands

### Update to Latest Alpha (Recommended for OpenCode support)
```bash
npx bmad-method@alpha install
```

### Fast Update (Preserves Customizations)
```bash
npx bmad-method update
```

### Check Current Version
```bash
cat _bmad/_config/manifest.yaml | grep version
```

## OpenCode vs GitHub Copilot

| Feature | GitHub Copilot | OpenCode |
|---------|---------------|----------|
| **Configuration** | `.github/copilot-instructions.md` | `.opencode.jsonc` or `.opencode/` |
| **Agent Activation** | Always active in chat | Must switch to agent mode |
| **Agent Invocation** | Direct in chat | Greet agent after switching |
| **File Format** | Markdown instructions | JSON config or Markdown |
| **Cost** | Paid subscription | Free & open source |
| **Privacy** | Cloud-based | Can be fully local |

## Switching IDEs

1. Run the installer:
   ```bash
   npx bmad-method@alpha install
   ```

2. When prompted, select your preferred IDE:
   - GitHub Copilot (current)
   - OpenCode (new)
   - Claude Code
   - Codex
   - Other options

3. Verify the change:
   ```bash
   cat _bmad/_config/manifest.yaml | grep -A 5 ides
   ```

## Current Project Setup

- **BMAD Version**: 6.0.0-alpha.22
- **Location**: `_bmad/` directory
- **Current IDE**: GitHub Copilot
- **Modules**: core, bmm

## OpenCode Installation

OpenCode must be installed separately:

### Via NPM
```bash
npm install -g @opencode/cli
```

### Via Official Installer
Visit [https://opencode.ai/](https://opencode.ai/) for installation instructions.

## Common Issues

### "Command not found: bmad"
BMAD is not installed globally. Use `npx`:
```bash
npx bmad-method@alpha install
```

### "Update not detected"
Ensure you're in the project root (where `_bmad/` exists):
```bash
cd /path/to/p2p-ai-agents
npx bmad-method@alpha install
```

### "OpenCode agents not working"
1. Verify OpenCode is installed: `opencode --version`
2. Check configuration exists: `ls -la .opencode*`
3. Activate agent in OpenCode before use
4. Greet the agent to initiate conversation

## Next Steps

For detailed information, see:
- [Full Update Guide](./bmad-method-update-guide.md)
- [BMAD-METHOD Documentation](https://deepwiki.com/bmadcode/BMAD-METHOD/)
- [OpenCode Documentation](https://opencode.ai/docs/)

## Support

- **BMAD Issues**: [GitHub Issues](https://github.com/bmad-code-org/BMAD-METHOD/issues)
- **OpenCode Issues**: [OpenCode GitHub](https://github.com/opencode/opencode)
- **Project Issues**: [p2p-ai-agents Issues](https://github.com/devicemanager/p2p-ai-agents/issues)
