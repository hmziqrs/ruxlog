# Builder stage
FROM rust:1.82-slim-bullseye as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .


RUN cargo install diesel_cli --no-default-features --features postgres

RUN cargo build --release

RUN diesel migration run --database-url postgres://rroot:root@host.docker.internal:5000/ruxlog

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/ruxlog /app/ruxlog

# Expose the port
EXPOSE 8888

# Command to run the binary
CMD ["/app/ruxlog"]
