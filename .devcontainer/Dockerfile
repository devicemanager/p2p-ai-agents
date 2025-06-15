FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Install additional packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        git \
        curl \
        wget \
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Install cargo extensions
RUN cargo install cargo-watch cargo-edit

# Set working directory
WORKDIR /workspaces/p2p-ai-agents
