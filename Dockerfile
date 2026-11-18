# Build Stage
FROM rust:1.86-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --bin payswitch-node

# Runtime Stage
FROM alpine:3.20

RUN apk add --no-cache ca-certificates tzdata && \
    addgroup -S payswitch && adduser -S -G payswitch payswitch

WORKDIR /home/payswitch
COPY --from=builder /app/target/release/payswitch-node /usr/local/bin/payswitch-node

RUN chown -R payswitch:payswitch /home/payswitch
USER payswitch

ENV PORT=8080
ENV PAYSWITCH_WAL_PATH=/home/payswitch/data/payswitch.wal
VOLUME ["/home/payswitch/data"]

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/payswitch-node"]
