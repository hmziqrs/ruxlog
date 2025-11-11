FROM rust:1.89.0-slim-trixie AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev libpq-dev build-essential ca-certificates binutils \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/Cargo.toml
COPY migration/Cargo.lock migration/Cargo.lock
RUN cargo fetch --locked

COPY src ./src
COPY scripts ./scripts
COPY migration ./migration
COPY diesel.toml ./
ENV RUSTFLAGS="-C strip=symbols"
RUN cargo build --release --locked

FROM debian:trixie-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        libpq5 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 10001 appuser

COPY --from=builder /app/target/release/ruxlog /app/ruxlog

USER appuser
EXPOSE 8888
CMD ["/app/ruxlog"]
