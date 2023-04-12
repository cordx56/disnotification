FROM rust:1 as builder
WORKDIR /app
RUN cargo install cargo-build-deps
COPY Cargo.toml Cargo.lock ./
RUN cargo build-deps --release

COPY src ./src
RUN cargo install --root . --path .


FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/bin/disnot .
ENTRYPOINT ["./disnot"]
