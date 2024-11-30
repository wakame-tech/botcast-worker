FROM rust:1.80-alpine AS builder
WORKDIR /app

ARG DATABASE_URL

RUN apk update && apk add --no-cache musl-dev openssl-dev openssl-libs-static
COPY rust-toolchain .
RUN rustup target add x86_64-unknown-linux-musl

COPY crates crates
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /app

RUN apk add --update --no-cache ffmpeg

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/worker .

# CMD ["sleep", "infinity"]
CMD ["/app/worker"]
