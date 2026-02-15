FROM rust:1.83-bookworm AS builder

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifest and lock
COPY rust/Cargo.toml rust/Cargo.lock* ./rust/
COPY proto/ ./proto/

# Copy all crate manifests for dependency caching
COPY rust/crates/cortex-common/Cargo.toml ./rust/crates/cortex-common/Cargo.toml
COPY rust/crates/cortex-store/Cargo.toml ./rust/crates/cortex-store/Cargo.toml
COPY rust/crates/cortex-ml-client/Cargo.toml ./rust/crates/cortex-ml-client/Cargo.toml
COPY rust/crates/cortex-chunker/Cargo.toml ./rust/crates/cortex-chunker/Cargo.toml
COPY rust/crates/cortex-connectors/Cargo.toml ./rust/crates/cortex-connectors/Cargo.toml
COPY rust/crates/cortex-ingestion/Cargo.toml ./rust/crates/cortex-ingestion/Cargo.toml
COPY rust/crates/cortex-scheduler/Cargo.toml ./rust/crates/cortex-scheduler/Cargo.toml
COPY rust/crates/cortex-api/Cargo.toml ./rust/crates/cortex-api/Cargo.toml

# Create dummy source files for dependency caching
RUN mkdir -p rust/crates/cortex-common/src && echo "pub mod config; pub mod telemetry; pub mod types;" > rust/crates/cortex-common/src/lib.rs && \
    mkdir -p rust/crates/cortex-store/src && echo "" > rust/crates/cortex-store/src/lib.rs && \
    mkdir -p rust/crates/cortex-ml-client/src && echo "" > rust/crates/cortex-ml-client/src/lib.rs && \
    mkdir -p rust/crates/cortex-chunker/src && echo "" > rust/crates/cortex-chunker/src/lib.rs && \
    mkdir -p rust/crates/cortex-connectors/src && echo "" > rust/crates/cortex-connectors/src/lib.rs && \
    mkdir -p rust/crates/cortex-ingestion/src && echo "" > rust/crates/cortex-ingestion/src/lib.rs && \
    mkdir -p rust/crates/cortex-scheduler/src && echo "" > rust/crates/cortex-scheduler/src/lib.rs && \
    mkdir -p rust/crates/cortex-api/src && echo "fn main() {}" > rust/crates/cortex-api/src/main.rs

# Copy build.rs for ml-client
COPY rust/crates/cortex-ml-client/build.rs ./rust/crates/cortex-ml-client/build.rs

# Build dependencies (this layer is cached)
RUN cd rust && cargo build --release 2>/dev/null || true

# Copy actual source code
COPY rust/crates/ ./rust/crates/

# Build the actual binary
RUN cd rust && cargo build --release --bin cortex-api

# ── Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/rust/target/release/cortex-api /usr/local/bin/cortex-api

EXPOSE 8080

CMD ["cortex-api"]
