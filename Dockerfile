# syntax=docker/dockerfile:1

# ---------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------
FROM rust:1.93-bookworm AS builder

WORKDIR /build

# No OpenSSL needed: reqwest uses rustls-tls; image/rusttype are pure Rust
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest first for dependency-layer caching
COPY Cargo.toml Cargo.lock ./

# Scaffold a dummy crate so dependencies are pre-compiled and cached
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
    && cargo build --release

# Copy the real source and assets (fonts embedded via include_bytes!)
COPY . .

# Rebuild with real sources; dependency cache is reused
RUN cargo build --release

# ---------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/rust_api /app/rust_api

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000

CMD ["/app/rust_api"]
