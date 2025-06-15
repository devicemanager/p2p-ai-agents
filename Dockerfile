# Use Rust official image
FROM rust:1.70-slim

# Install system dependencies needed for Rust compilation
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

# Build the Rust project and run tests
RUN cargo build --release && cargo test

# Create a non-root user for security
RUN useradd -r -s /bin/false appuser
USER appuser

CMD ["./target/release/p2p-ai-agents"]
