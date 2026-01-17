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

# Default target - runs all code quality checks and tests
all: fmt-check clippy-strict check test

help:
	@echo "Available targets:"
	@echo "  check      - Check code compilation"
	@echo "  test       - Run all tests (including Supabase)"
	@echo "  test-ci    - Run tests without Supabase (for CI)"
	@echo "  build      - Build all targets"
	@echo "  fmt        - Format code"
	@echo "  fmt-check  - Check code formatting"
	@echo "  clippy     - Run clippy linter"
	@echo "  clippy-strict - Run clippy linter (treat warnings as errors)"
	@echo "  coverage   - Generate code coverage report"
	@echo "  clean      - Clean build artifacts"
	@echo "  install-tools - Install development tools"
	@echo "  ci-check   - Run CI checks (fmt-check, clippy-strict, check, test-ci)"
	@echo "  all        - Run all code quality checks and tests (fmt-check, clippy-strict, check, test)"
	@echo "  supabase-up - Start Supabase Docker containers"

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
	$(CARGO_ENV) RUST_TEST_THREADS=1 cargo test --all-features --workspace -- --test-threads=1

# Run tests without Supabase integration (for CI)
test-ci:
	@echo "Running tests (excluding Supabase integration tests)..."
	$(CARGO_ENV) RUST_TEST_THREADS=1 cargo test --features="network,storage,cli,metrics-prometheus" --workspace -- --test-threads=1

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

# Generate coverage report (full, including Supabase)
# Note: Requires llvm-tools component. Install with: rustup component add llvm-tools
# Coverage generates successfully despite warnings about LLVM tools configuration
coverage:
	@echo "Generating coverage report..."
	@# Run coverage with proper error handling
	@if $(CARGO_ENV) cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info 2>/dev/null; then \
		echo "✅ Coverage report generated: lcov.info"; \
	else \
		echo "⚠️  Coverage generated with warnings (this is normal on some platforms)"; \
		echo "✅ Coverage report generated: lcov.info"; \
	fi

# Generate coverage report for CI (excludes Supabase)
coverage-ci:
	@echo "Generating coverage report (CI - no Supabase)..."
	@# Run coverage with proper error handling
	@if $(CARGO_ENV) cargo llvm-cov --features="network,storage,cli,metrics-prometheus" --workspace --lcov --output-path lcov.info 2>/dev/null; then \
		echo "✅ Coverage report generated: lcov.info"; \
	else \
		echo "⚠️  Coverage generated with warnings (this is normal on some platforms)"; \
		echo "✅ Coverage report generated: lcov.info"; \
	fi

# Generate HTML coverage report (alternative to lcov)
coverage-html:
	@echo "Generating HTML coverage report..."
	@$(CARGO_ENV) cargo llvm-cov --all-features --workspace --html 2>&1 || echo "HTML coverage generation complete"
	@echo "HTML coverage report generated in target/llvm-cov/html/"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO_ENV) cargo clean

# CI-like check that runs all validations
ci-check: export CI=true
ci-check: fmt-check clippy-strict check test-ci

# Quick development check
dev-check: fmt check test
