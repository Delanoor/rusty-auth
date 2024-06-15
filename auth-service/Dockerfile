# Use an official Rust image as the builder
FROM rust:latest AS builder

# Install required packages
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev

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

# Specify the startup command
CMD ["auth-service"]