# Kafka Cli implemented in Rust

## How to use

### Arguments


### Kafka Config

```
cat <<EOF >.kafka.config.yaml
sasl:
  username: $KAFKA_KEY
  password: $KAFKA_SECRET
endpoint: $KAFKA_BOOTSTRAP_URL

EOF

Or directly from $KAFKA_SASL_USERNAME, $KAFKA_SASL_PASSWORD, $KAFKA_ENDPOINT

```

### Docker


### Cargo install


### Build from source

```bash
cargo run --bin kafka-cli --release consume --topic test --consumer-group-id local-test --partition 0 --schema-id 100005
cargo run --bin kafka-cli --release produce --topic test --message-file resources/msg.msg --key-file resources/key.msg --schema-id 100005
```

## Features

None
