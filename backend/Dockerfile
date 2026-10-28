# syntax=docker/dockerfile:1

FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cargo.lock pins the dependency versions the project is developed and tested with;
# without it every image build resolves the newest ones (which may need a newer Rust).
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/Cargo.toml
COPY frontend/Cargo.toml ./frontend/Cargo.toml
# This image builds only the backend, but cargo loads every workspace member:
# give the frontend a placeholder target instead of copying its sources.
RUN mkdir -p frontend/src && echo 'fn main() {}' > frontend/src/main.rs
COPY backend/src ./backend/src
COPY backend/tests ./backend/tests
COPY README.md ./README.md

RUN cargo build --release --locked -p backend

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
