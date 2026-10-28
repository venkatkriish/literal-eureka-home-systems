# syntax=docker/dockerfile:1

FROM rust:1.79-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./Cargo.toml
COPY backend/Cargo.toml ./backend/Cargo.toml
COPY backend/src ./backend/src
COPY backend/tests ./backend/tests
COPY README.md ./README.md

RUN cargo build --release -p backend

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

ENV BIND_ADDR=0.0.0.0:3000
EXPOSE 3000
USER appuser

ENTRYPOINT ["/usr/local/bin/backend"]
