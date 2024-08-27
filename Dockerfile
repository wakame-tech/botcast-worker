FROM rust:1.80-alpine as builder
WORKDIR /app

RUN apk update && apk add --no-cache musl-dev openssl-dev openssl-libs-static
RUN rustup target add x86_64-unknown-linux-musl

COPY rust-toolchain rust-toolchain
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
RUN mkdir src/
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release --target=x86_64-unknown-linux-musl

COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/worker .

CMD ["/app/worker"]
