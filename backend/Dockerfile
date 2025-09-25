FROM rust:1.85 AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/d-bo-backend /app/d-bo-backend
CMD ["./d-bo-backend"]