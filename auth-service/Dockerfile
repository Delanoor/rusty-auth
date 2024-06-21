# First stage: Rust build environment
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
USER root

# Add cargo-chef to cache dependencies
# RUN apk update add --no-cache musl-dev libssl-dev & cargo install cargo-chef
WORKDIR /app
 
FROM chef AS planner
COPY . .
# Capture info needed to build dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin auth-service


# Final stage: minimal runtime environment
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt install -y \
    openssl \
    ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/auth-service /usr/local/bin
COPY --from=builder /app/assets /app/assets

COPY config.production.yaml /app/config.production.yaml
COPY config.yaml /app/config.yaml


# Set environment variables for runtime
ENV APP_ENV=production
ENV REDIS_HOST_NAME=redis
ENV JWT_SECRET=${JWT_SECRET}
ENV DATABASE_URL=${DATABASE_URL}
ENV POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
ENV REDIS_HOST_NAME=${REDIS_HOST_NAME}
ENV REDIS_PASSWORD=${REDIS_PASSWORD}
ENV REDIS_PORT=${REDIS_PORT}

ENTRYPOINT ["/usr/local/bin/auth-service"]
