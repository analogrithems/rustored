# syntax=docker/dockerfile:1

# ---- Build Stage ----
FROM rust:1.85.1-slim as builder
WORKDIR /app
# Install build dependencies for OpenSSL and pkg-config
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release --locked

# ---- Distroless Stage ----
FROM gcr.io/distroless/cc-debian12 AS distroless
COPY --from=builder /app/target/release/rustored /rustored
USER nonroot
ENTRYPOINT ["/rustored"]
