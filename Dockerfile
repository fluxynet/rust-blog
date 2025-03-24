FROM rust:1.85-bullseye AS builder
WORKDIR /app

COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

COPY src /app/src
COPY .sqlx /app/.sqlx

RUN SQLX_OFFLINE=true cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/blog /app/blog
COPY config.toml /app/config.toml

EXPOSE 8000

ENTRYPOINT ["/app/blog"]

CMD ["--help"]