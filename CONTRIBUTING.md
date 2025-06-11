# Contributing to P2P Agent

Thank you for your interest in contributing to the future of decentralized AI! We welcome all kinds of contributions, from code and documentation to research and community building.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contribution Types](#contribution-types)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Community](#community)
- [📏 File Size Requirements](#-file-size-requirements)

---

## 📖 Related Documentation

- **[Documentation Index](docs/INDEX.md)** - Complete documentation overview
- **[README](README.md)** - Project overview and getting started
- **[Quick Reference](docs/QUICK_REFERENCE.md)** - Commands and configuration
- **[High Level Design](HIGH_LEVEL_DESIGN.md)** - Technical architecture
- **[Agent Protocol](AGENT_PROTOCOL.md)** - Communication protocol specification

---

## Code of Conduct

This project adheres to our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold these standards and create a welcoming environment for all contributors.

## Getting Started

### Prerequisites

- Python 3.8 or higher
- Git
- Docker (optional, for containerized development)
- Basic understanding of P2P networking concepts
- Familiarity with AI/ML concepts (helpful but not required)

### Quick Start for Contributors

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/p2p-agent.git
   cd p2p-agent
   ```
3. **Set up development environment**:
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   pip install -r requirements-dev.txt
   ```
4. **Install pre-commit hooks**:
   ```bash
   pre-commit install
   ```
5. **Run the tests** to verify your setup:
   ```bash
   pytest
   ```

## Development Setup

### Environment Configuration

Create a `.env` file for development settings:
```bash
cp .env.example .env
# Edit .env with your configuration
```

### Docker Development (Alternative)

```bash
docker-compose -f docker-compose.dev.yml up
```

This provides a complete development environment with all dependencies.

## Contribution Types

### 🛠 Code Contributions

**Core Agent Development**
- Agent communication protocols
- Task distribution algorithms
- P2P networking implementation
- Security and authentication systems

**AI/ML Components**
- Text processing and chunking
- Vector embeddings and similarity search
- Federated learning algorithms
- Model optimization for edge devices

**Infrastructure**
- Performance monitoring and metrics
- Deployment tools and configuration
- CI/CD pipeline improvements
- Testing framework enhancements

### 📚 Documentation

- API documentation and references
- User guides and tutorials
- Architecture documentation
- Code comments and docstrings

### 🧪 Testing and Quality Assurance

- Unit and integration tests
- Performance benchmarking
- Security auditing
- Cross-platform compatibility testing

### 🔬 Research Contributions

- Academic papers and research
- Performance analysis
- Security research
- Novel P2P algorithms

## Development Workflow

### Branch Naming Convention

- `feature/description-of-feature`
- `bugfix/description-of-fix`
- `docs/description-of-docs`
- `refactor/description-of-refactor`

### Commit Message Format

```
type(scope): brief description

Detailed explanation of the change (optional)

Fixes #123
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Examples**:
```
feat(networking): add peer discovery via mDNS
fix(security): resolve signature verification issue
docs(api): update task distribution protocol
```

### Pull Request Process

1. **Create a feature branch** from `main`
2. **Make your changes** following our coding standards
3. **Write/update tests** for your changes
4. **Update documentation** if needed
5. **Run the full test suite** and ensure it passes
6. **Submit a pull request** with:
   - Clear description of changes
   - Reference to related issues
   - Screenshots/demos for UI changes
   - Breaking change notes if applicable

### Review Process

- All PRs require at least one approval from a maintainer
- Automated tests must pass
- Code coverage should not decrease
- Documentation must be updated for API changes

## Code Standards

### Python Code Style

- **PEP 8** compliance (enforced by `black` and `flake8`)
- **Type hints** required for all public functions
- **Docstrings** required for all public classes and functions
- **Maximum line length**: 100 characters

### Code Quality Tools

We use several tools to maintain code quality:

```bash
# Format code
black .
isort .

# Lint code
flake8
pylint p2p_ai_agents/

# Type checking
mypy p2p_ai_agents/

# Security scanning
bandit -r p2p_ai_agents/
```

### Documentation Standards

- **Docstring format**: Google style
- **API documentation**: Auto-generated from docstrings
- **Markdown linting**: Using `markdownlint`
- **Link checking**: Automated verification of external links

## Testing Requirements

### Test Categories

**Unit Tests**
- Individual function and class testing
- Mocked external dependencies
- Fast execution (< 1 second per test)

**Integration Tests**
- Multi-component interaction testing
- Real P2P network communication
- Database and storage testing

**End-to-End Tests**
- Full workflow testing
- Multi-agent scenarios
- Performance validation

### Test Commands

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=p2p_ai_agents

# Run specific test category
pytest tests/unit/
pytest tests/integration/
pytest tests/e2e/

# Run performance tests
pytest tests/performance/ --benchmark-only
```

### Test Requirements

- **Minimum coverage**: 80% for new code
- **All tests must pass** on supported Python versions (3.8, 3.9, 3.10, 3.11)
- **Performance tests** must not regress by > 10%

## Documentation

### Documentation Structure

```
docs/
├── user-guide/          # End-user documentation
├── developer-guide/     # Developer documentation
├── api-reference/       # Auto-generated API docs
├── architecture/        # System design documents
└── tutorials/           # Step-by-step tutorials
```

### Building Documentation

```bash
cd docs/
make html  # Build HTML documentation
make livehtml  # Build with auto-reload for development
```

## Security Considerations

### Security Review Requirements

All security-related changes require:
- Review by security-focused maintainer
- Threat model analysis
- Security testing with appropriate tools

### Reporting Security Issues

**Do not report security vulnerabilities in public issues.**

Email security issues to: [security@p2p-agent.org](mailto:security@p2p-agent.org)

## Community

### Communication Channels

- **Discord**: [Join our community](https://discord.gg/p2p-agent)
- **GitHub Discussions**: For design discussions and Q&A
- **Twitter**: [@P2PAIAgents](https://twitter.com/P2PAIAgents)
- **Weekly Calls**: See community calendar

### Recognition

Contributors are recognized through:
- Contributor list in README
- Annual contributor awards
- Conference speaking opportunities
- Co-authorship on research papers

## Getting Help

### Mentorship Program

New contributors can request mentorship through:
- Comment on "good first issue" tickets
- Join our Discord #mentorship channel
- Attend weekly newcomer calls

### Resources

- [Architecture Overview](HIGH_LEVEL_DESIGN.md)
- [Protocol Specification](AGENT_PROTOCOL.md)
- [Developer Tutorials](docs/tutorials/)
- [FAQ](docs/FAQ.md)

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project (MIT License).

---

**Thank you for helping build the future of decentralized AI!** 🚀

Every contribution, no matter how small, makes a difference in creating a more open, sustainable, and accessible AI ecosystem.

---

## 📏 File Size Requirements

### 500-Line Limit Policy

**All files in P2P AI Agents must not exceed 500 lines.** This includes:

- **Source code files** (.py, .js, .ts, etc.)
- **Documentation files** (.md, .rst, .txt)
- **Configuration files** (.yaml, .toml, .json)
- **Scripts and tools** (.sh, .bat, etc.)

### Rationale

- **Small model compatibility**: AI assistants can process entire files efficiently
- **Resource efficiency**: Lower memory usage on development machines
- **Better maintainability**: Forced modularization improves code quality
- **Easier code review**: Manageable file sizes for human reviewers

### Implementation

When a file approaches 400 lines:

1. **Create an issue** to track the refactoring need
2. **Identify logical break points** for splitting
3. **Extract modules/sections** into separate files
4. **Update imports/references** accordingly
5. **Maintain comprehensive tests** for refactored code

For documentation files:
1. **Create a table of contents** in the main file
2. **Split content** into numbered sub-files
3. **Link between sections** for navigation
4. **Update cross-references** in other documents

### Automated Enforcement

```bash
# Pre-commit hook checks file sizes
pre-commit run --all-files

# Manual check for oversized files
find . -name "*.py" -o -name "*.md" | xargs wc -l | awk '$1 > 500'
```

See **[500-Line Limit Policy](docs/500_LINE_LIMIT_POLICY.md)** for complete guidelines.

*Note: This contributing guide is continuously updated. Check the [documentation](https://p2p-agent.readthedocs.io/) for the latest contribution guidelines.*
