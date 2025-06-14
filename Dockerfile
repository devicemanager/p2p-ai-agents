# Use Rust official image (stub)
FROM rust:1.70-slim

WORKDIR /app

COPY . .

# Build the Rust project (using cargo) and run tests (using cargo test) (stub)
RUN cargo build --release && cargo test --test local_test -- --nocapture

CMD ["./target/release/p2p-ai-agents"]
