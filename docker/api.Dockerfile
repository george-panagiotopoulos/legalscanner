# Build stage
FROM rust:latest AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY legalscanner-api/Cargo.toml ./legalscanner-api/

# Create dummy source to cache dependencies
RUN mkdir -p legalscanner-api/src && \
    echo "fn main() {}" > legalscanner-api/src/main.rs && \
    cargo build --release && \
    rm -rf legalscanner-api/src target/release/legalscanner-api* target/release/deps/legalscanner_api*

# Copy actual source code
COPY legalscanner-api/src ./legalscanner-api/src
COPY legalscanner-api/migrations ./legalscanner-api/migrations

# Build for release
RUN cargo build --release --bin legalscanner-api

# Runtime stage
FROM debian:trixie-slim

# Install runtime dependencies including Docker CLI
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    git \
    curl \
    gnupg \
    lsb-release \
    && mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update \
    && apt-get install -y docker-ce-cli \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/legalscanner-api /usr/local/bin/legalscanner-api

# Create data and workspace directories
RUN mkdir -p /data /tmp/scans

# Expose port
EXPOSE 8080

# Run the binary
CMD ["legalscanner-api"]
