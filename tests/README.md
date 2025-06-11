# Tests

This directory contains the test suite for P2P AI Agents.

## Test Structure

```
tests/
├── unit/           # Unit tests for individual components
├── integration/    # Integration tests for component interactions
├── e2e/           # End-to-end tests for full workflows
├── performance/   # Performance and benchmark tests
└── fixtures/      # Test data and fixtures
```

## Running Tests

```bash
# Run all tests
pytest

# Run specific test categories
pytest tests/unit/
pytest tests/integration/
pytest tests/e2e/

# Run with coverage
pytest --cov=p2p_ai_agents --cov-report=html

# Run performance tests
pytest tests/performance/ --benchmark-only
```

## Test Guidelines

- Write tests for all new features and bug fixes
- Maintain minimum 80% code coverage
- Use descriptive test names and docstrings
- Mock external dependencies in unit tests
- Use fixtures for common test data
- Follow the Arrange-Act-Assert pattern

## Test Categories

### Unit Tests
- Test individual functions and classes
- Mock all external dependencies
- Fast execution (< 1 second per test)

### Integration Tests
- Test component interactions
- May use real network connections
- Test database operations

### End-to-End Tests
- Test complete user workflows
- Multi-agent scenarios
- Full system integration

### Performance Tests
- Benchmark critical operations
- Memory usage tests
- Scalability validation
