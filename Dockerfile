FROM rust:1.97.1-slim-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release


FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /app/target/release/rammal /app/rammal

EXPOSE 8000

ENTRYPOINT ["/app/rammal"]
