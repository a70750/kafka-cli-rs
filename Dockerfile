ARG BASE_IMAGE=debian:bookworm-slim

FROM rust:bookworm AS builder
WORKDIR /app
RUN apt-get update && \
    apt-get upgrade -y && \
    apt install -y libssl-dev

COPY . .

RUN cargo build --release --bin kafka-cli

FROM $BASE_IMAGE AS runtime

# https://github.com/fede1024/rust-rdkafka/issues/594 we can switch to ssl-vendored once it is supported by rust-rdkafka with confluent cloud
# pkg-config is needed to detect openssl
RUN apt update && \
    apt upgrade -y && \
    apt install -y ca-certificates openssl pkg-config libssl-dev

WORKDIR /app
COPY --from=builder /app/target/release/kafka-cli /usr/local/bin
USER 1000
CMD ["-V"]
ENTRYPOINT ["/usr/local/bin/kafka-cli"]
