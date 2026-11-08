FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin sunberry
RUN strip target/release/sunberry

FROM debian:12-slim AS runtime
COPY --from=builder /app/target/release/sunberry /bin/sunberry
ENTRYPOINT ["/bin/sunberry"]