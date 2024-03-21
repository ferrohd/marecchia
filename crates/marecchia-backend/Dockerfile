FROM rust:alpine as builder
RUN apk add --no-cache build-base
WORKDIR /app
COPY . .
RUN cargo build --release -p marecchia-backend

FROM scratch
COPY --from=builder /app/target/release/marecchia-backend .
ENTRYPOINT ["./marecchia-backend"]
