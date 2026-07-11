FROM rust:1.87-alpine AS builder
WORKDIR /app

RUN apk add --no-cache build-base curl pkgconfig musl-dev

COPY Cargo.toml Cargo.lock ./
COPY migrations/Cargo.toml migrations/Cargo.toml
COPY src src
COPY migrations/src migrations/src

RUN cargo build --release

FROM alpine:3.23
WORKDIR /app

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/money_manager_backend /usr/local/bin/money_manager_backend

EXPOSE 8000
CMD ["money_manager_backend"]
