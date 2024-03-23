FROM rust as builder
WORKDIR /app
COPY . .
# Build only (the release of) marecchia-tracker packcage
RUN cargo build --release --package marecchia-tracker

FROM debian:stable-slim
COPY --from=builder /app/target/release/marecchia-tracker .
ENTRYPOINT ["./marecchia-tracker"]
