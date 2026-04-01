# syntax=docker/dockerfile:1.7

FROM rust:1.88-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release --bin bybit \
    && cp /app/target/release/bybit /tmp/bybit

FROM debian:bookworm-slim

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 bybit \
    && mkdir -p /data/bybit \
    && chown -R bybit:bybit /data/bybit /home/bybit

COPY --from=build /tmp/bybit /usr/local/bin/bybit

ENV BYBIT_CONFIG_DIR=/data/bybit
VOLUME ["/data/bybit"]
EXPOSE 8811

USER bybit
WORKDIR /home/bybit

ENTRYPOINT ["/usr/local/bin/bybit", "mcp"]
