FROM debian:bullseye-slim AS base

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

FROM rust:slim-bullseye AS chef

WORKDIR /app
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --package discord_bot

FROM base AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/discord_bot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/discord_bot"]
