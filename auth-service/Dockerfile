# Use an official Rust image as the builder
FROM rust:latest AS builder

# Install required packages
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev

# Set build arguments for environment variables
ARG DATABASE_URL
ARG JWT_SECRET
ARG POSTGRES_PASSWORD
ARG REDIS_HOST_NAME
ARG REDIS_PASSWORD
ARG REDIS_PORT

# Set the DATABASE_URL environment variable for the build stage
ENV DATABASE_URL=${DATABASE_URL}
ENV SQLX_OFFLINE=true

# Create a new empty shell project
RUN USER=root cargo new --bin auth-service
WORKDIR /auth-service

# Copy manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Copy the source code
COPY ./src ./src

# Build the dependencies
RUN cargo build --release

# Create a new stage for the final image
FROM debian:buster-slim

# Install OpenSSL runtime libraries
RUN apt-get update && \
    apt-get install -y libssl1.1 && \
    rm -rf /var/lib/apt/lists/*

# Copy the build artifacts from the builder stage
COPY --from=builder /auth-service/target/release/auth-service /usr/local/bin/auth-service

# Set environment variables for runtime
ENV JWT_SECRET=${JWT_SECRET}
ENV DATABASE_URL=${DATABASE_URL}
ENV POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
ENV REDIS_HOST_NAME=${REDIS_HOST_NAME}
ENV REDIS_PASSWORD=${REDIS_PASSWORD}
ENV REDIS_PORT=${REDIS_PORT}

# Specify the startup command
CMD ["auth-service"]
