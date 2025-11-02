# Build stage
FROM rust:1.82-slim AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/big-bangers-bot /app/big-bangers-bot

# Copy config file (this should be mounted or provided at runtime)
# Uncomment the line below if you want to include a default config.json
# COPY config.json /app/config.json

# Run the binary
CMD ["./big-bangers-bot"]
