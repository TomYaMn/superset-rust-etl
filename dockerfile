# Stage 1: Builder (Uses latest stable Rust toolchain on Bookworm)
FROM rust:bookworm AS builder

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source files and build
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /usr/src/app/target/release/superset-rust-etl /usr/local/bin/superset-rust-etl

# Copy default fallback config files into image
COPY config.toml /app/config.toml
COPY sp500_tech.json /app/sp500_tech.json

ENV DATABASE_URL="postgres://user:pass@host/db"

CMD ["superset-rust-etl"]