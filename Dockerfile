FROM rust:1.89.0-slim-trixie AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev libpq-dev build-essential ca-certificates binutils \
    && rm -rf /var/lib/apt/lists/*

COPY . .
ENV RUSTFLAGS="-C strip=symbols"
RUN cargo build --release

FROM debian:trixie-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 10001 appuser

COPY --from=builder /app/target/release/ruxlog /app/ruxlog

USER appuser
EXPOSE 8888
CMD ["/app/ruxlog"]
