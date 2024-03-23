FROM rust as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p marecchia-tracker

FROM debian:stable-slim
COPY --from=builder /app/target/release/marecchia-tracker .
ENTRYPOINT ["./marecchia-backend"]
