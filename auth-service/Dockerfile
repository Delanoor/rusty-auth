FROM rust:1 AS chef 
USER root

# Add cargo-chef to cache dependencies
RUN cargo install cargo-chef

# Install protobuf compiler
# RUN apk add --no-cache protobuf

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

# We do not need the Rust toolchain to run the binary!
# Start with a minimal image and copy over the binary and assets folder.
FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/auth-service /usr/local/bin
COPY --from=builder /app/assets /app/assets

# Set environment variables for runtime
ENV REDIS_HOST_NAME=redis
ENV JWT_SECRET=${JWT_SECRET}
ENV DATABASE_URL=${DATABASE_URL}
ENV POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
ENV REDIS_HOST_NAME=${REDIS_HOST_NAME}
ENV REDIS_PASSWORD=${REDIS_PASSWORD}
ENV REDIS_PORT=${REDIS_PORT}

EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/auth-service"]