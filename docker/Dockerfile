# Multi-stage Docker build for SPRX Blockchain Node
FROM rust:1.80-slim AS builder

WORKDIR /usr/src/sprax-chain

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

# Build release binaries
RUN cargo build --release -p sprax-cli

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

WORKDIR /root

# Copy compiled CLI binary
COPY --from=builder /usr/src/sprax-chain/target/release/sprax /usr/local/bin/sprax

# Copy entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 26656 26657 9090

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
