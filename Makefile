# Makefile for P2P AI Agents project

# Ensure cargo is available by sourcing the cargo environment
SHELL := /bin/bash
.SHELLFLAGS := -c
CARGO_ENV := source "$$HOME/.cargo/env" &&

.PHONY: help check test build fmt clippy coverage clean install-tools all

# Start Supabase containers
supabase-up:
	@if [ ! -f lab/docker/.env ]; then \
		cp lab/docker/.env.example lab/docker/.env; \
		echo "[INFO] lab/docker/.env was missing. Created from .env.example. Please update with your own secrets if needed."; \
	fi
	@echo "Starting Supabase containers with local environment variables..."
	cd lab/docker && docker compose --env-file .env up -d

# Default target
all: supabase-up fmt-check clippy-strict check test

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
	$(CARGO_ENV) cargo install cargo-llvm-cov
	$(CARGO_ENV) rustup component add rustfmt clippy llvm-tools-preview

# Check compilation
check:
	@echo "Checking compilation..."
	$(CARGO_ENV) cargo check --all-targets --all-features

# Run tests
test:
	@echo "Running tests..."
	$(CARGO_ENV) cargo test --all-features --workspace

# Build all targets
build:
	@echo "Building all targets..."
	$(CARGO_ENV) cargo build --all-targets --all-features

# Format code
fmt:
	@echo "Formatting code..."
	$(CARGO_ENV) cargo fmt --all

# Check code formatting
fmt-check:
	@echo "Checking code formatting..."
	$(CARGO_ENV) cargo fmt --all -- --check

# Run clippy
clippy:
	@echo "Running clippy..."
	$(CARGO_ENV) cargo clippy --all-targets --all-features

# Run clippy with strict warnings (treat warnings as errors)
clippy-strict:
	@echo "Running clippy (strict mode)..."
	$(CARGO_ENV) cargo clippy --all-targets --all-features -- -D warnings

# Generate coverage report
coverage:
	@echo "Generating coverage report..."
	$(CARGO_ENV) cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "Coverage report generated: lcov.info"

# Generate HTML coverage report
coverage-html:
	@echo "Generating HTML coverage report..."
	$(CARGO_ENV) cargo llvm-cov --all-features --workspace --html
	@echo "HTML coverage report generated in target/llvm-cov/html/"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO_ENV) cargo clean

# CI-like check that runs all validations
ci-check: fmt-check clippy-strict check test

# Quick development check
dev-check: fmt check test
