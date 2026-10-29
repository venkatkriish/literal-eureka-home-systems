FROM rust:1.82.0-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --locked --release && mkdir -p /out && bin="$(find target/release -maxdepth 1 -type f -perm -111 ! -name '*.d' | head -n 1)" && test -n "$bin" && cp "$bin" /out/app

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 --create-home --shell /usr/sbin/nologin appuser
WORKDIR /app
COPY --from=builder /out/app /usr/local/bin/app
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/app"]
