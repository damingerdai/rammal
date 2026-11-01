FROM rust:1.99.1-slim-trixie AS builder

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

RUN cargo build --release


FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /app/target/release/rammal /app/rammal

EXPOSE 8000

ENTRYPOINT ["/app/rammal"]
