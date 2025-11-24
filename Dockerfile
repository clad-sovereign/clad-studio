# Multi-stage Docker build for Clad Studio node
# Stage 1: Build the node binary

FROM rust:1.83-bookworm AS builder

# Install dependencies
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    git \
    clang \
    curl \
    libssl-dev \
    llvm \
    libudev-dev \
    make \
    protobuf-compiler \
    pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Install wasm target for Rust
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /clad-studio

# Copy the entire workspace
COPY . .

# Build the node binary in release mode with locked dependencies
RUN cargo build --release --locked -p clad-node

# Verify the binary was built
RUN ls -lh /clad-studio/target/release/clad-node

# Stage 2: Create the runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    curl && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 -U -s /bin/sh -d /clad-node clad && \
    mkdir -p /clad-node/.local/share && \
    chown -R clad:clad /clad-node

# Copy the binary from builder stage
COPY --from=builder /clad-studio/target/release/clad-node /usr/local/bin/

# Set user
USER clad

# Expose ports
# 9944: WebSocket RPC
# 9933: HTTP RPC
# 30333: P2P networking
# 9615: Prometheus metrics
EXPOSE 9944 9933 30333 9615

# Set working directory
WORKDIR /clad-node

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:9933/health || exit 1

# Default command (can be overridden in docker-compose)
ENTRYPOINT ["/usr/local/bin/clad-node"]
CMD ["--dev", "--rpc-external", "--rpc-cors", "all"]
