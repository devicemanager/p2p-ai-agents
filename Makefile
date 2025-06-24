# Makefile for P2P AI Agents project

.PHONY: help check test build fmt clippy coverage clean install-tools all

# Default target
all: check test

help:
	@echo "Available targets:"
	@echo "  check      - Check code compilation"
	@echo "  test       - Run all tests"
	@echo "  build      - Build all targets"
	@echo "  fmt        - Format code"
	@echo "  fmt-check  - Check code formatting"
	@echo "  clippy     - Run clippy linter"
	@echo "  clippy-strict - Run clippy linter (treat warnings as errors)"
	@echo "  coverage   - Generate code coverage report"
	@echo "  clean      - Clean build artifacts"
	@echo "  install-tools - Install development tools"
	@echo "  all        - Run check and test"

# Install required development tools
install-tools:
	@echo "Installing development tools..."
	cargo install cargo-llvm-cov
	rustup component add rustfmt clippy llvm-tools-preview

# Check compilation
check:
	@echo "Checking compilation..."
	cargo check --all-targets --all-features

# Run tests
test:
	@echo "Running tests..."
	cargo test --all-features --workspace

# Build all targets
build:
	@echo "Building all targets..."
	cargo build --all-targets --all-features

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Check code formatting
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

# Run clippy
clippy:
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features

# Run clippy with strict warnings (treat warnings as errors)
clippy-strict:
	@echo "Running clippy (strict mode)..."
	cargo clippy --all-targets --all-features -- -D warnings

# Generate coverage report
coverage:
	@echo "Generating coverage report..."
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "Coverage report generated: lcov.info"

# Generate HTML coverage report
coverage-html:
	@echo "Generating HTML coverage report..."
	cargo llvm-cov --all-features --workspace --html
	@echo "HTML coverage report generated in target/llvm-cov/html/"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# CI-like check that runs all validations
ci-check: fmt-check clippy-strict check test

# Quick development check
dev-check: fmt check test
