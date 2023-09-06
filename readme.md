# Kafka Cli implemented in Rust

## TODOs: 
- Key assertion
- JSON and Protobuf Schemas
- Improve Clap usage

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
cargo run --bin kafka-cli --release consume --topic test --consumer-group-id local-test --partition 0 --schema-id 100007
cargo run --bin kafka-cli --release produce --topic test --message-file resources/msg.msg --key-file resources/key.msg 
--schema-id 100005


# All arguments. As of now, cannot be combined with folder or files
cargo run --bin kafka-cli --release integration-test --consumer-topic test --producer-topic test --producer-value-file resources/msg.json --producer-key-file resources/key.json  --assertion-value-file resources/msg.json --consumer-group-id test
cargo run --bin kafka-cli --release integration-test --consumer-topic test --producer-topic test --producer-value-file resources/msg.json --producer-key-file resources/key.json  --assertion-value-file resources/msg.json --consumer-group-id test

cargo run --bin kafka-cli --release test-folder resources -a msg.json

# Use config files, specific parts of config files CANNOT be overwriten with arguments

# Load using a folder with consumer.yaml, producer.yaml and assertion_value.json
cargo run --bin kafka-cli --release test-folder resources

# Explicit config of manadatory config files
cargo run --bin kafka-cli --release test-files -c resources/consumer.yaml -p resources/producer.yaml -a resources/msg.json

```

## Features

None
