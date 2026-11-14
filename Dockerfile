FROM rust:1.100.0-slim-trixie AS builder

WORKDIR /app

# Git metadata injected at build time (e.g. via docker build --build-arg).
# Defaults to "unknown" when the caller doesn't provide them; build.rs
# falls back accordingly so the crate still compiles.
ARG VERGEN_GIT_DESCRIBE=unknown
ARG VERGEN_GIT_SHA=unknown
ARG VERGEN_GIT_COMMIT_DATE=unknown
ENV VERGEN_GIT_DESCRIBE=$VERGEN_GIT_DESCRIBE \
    VERGEN_GIT_SHA=$VERGEN_GIT_SHA \
    VERGEN_GIT_COMMIT_DATE=$VERGEN_GIT_COMMIT_DATE

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src

RUN cargo build --release --locked


FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rammal /app/rammal

COPY config/default.toml /app/config/default.toml

USER 10001:10001
EXPOSE 8000

ENTRYPOINT ["/app/rammal"]
