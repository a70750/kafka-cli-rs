use std::{io::BufReader, time::Duration};

use clap::Parser;
use env_logger::{Builder, Target};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let args = args::Cli::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let mut builder = Builder::from_default_env();
    match args.log_output {
        args::LogOutput::StdOut => {
            builder.target(Target::Stdout);
        }
        args::LogOutput::StdErr => {
            builder.target(Target::Stderr);
        }
    }
    builder.init();

    let config = kafka_config::KafkaConfig::from_env().unwrap();

    match args.action {
        args::Action::Consume(args::ConsumerConfig {
            ref topic,
            ref key_file,
            ref consumer_group_id,
            ref partition,
            ref schema_id,
        }) => {
            let consumer = kafka_consumer::KafkaConsumer::new(
                &config,
                topic,
                key_file,
                consumer_group_id,
                partition,
                schema_id,
            );
            match consumer.consume(None).await {
                Ok(_) => {
                    log::info!("Integration test passed");
                }
                Err(e) => {
                    log::error!("Integration test failed: {:?}", e);
                    panic!("Integration test failed: {:?}", e);
                }
            }
        }
        args::Action::Produce(args::ProducerConfig {
            ref topic,
            ref key_file,
            ref message_file,
            ref schema_id,
        }) => {
            let producer = kafka_producer::KafkaProducer::new(
                &config,
                topic,
                key_file,
                message_file.as_ref().unwrap(),
                schema_id,
            );
            match producer.produce().await {
                Ok(_) => {
                    log::info!("Message produced");
                }
                Err(e) => {
                    log::error!("Message not produced: {:?}", e);
                    panic!("Message not produced: {:?}", e);
                }
            }
        }
        args::Action::IntegrationTest(args::TestConfig {
            ref topic,
            ref key_file,
            ref message_file,
            ref consumer_group_id,
            ref partition,
            ref schema_id,
        }) => {
            let producer = kafka_producer::KafkaProducer::new(
                &config,
                topic,
                key_file,
                message_file.as_ref().unwrap(),
                schema_id,
            );
            let consumer = kafka_consumer::KafkaConsumer::new(
                &config,
                topic,
                &Some(key_file.clone()),
                consumer_group_id,
                partition,
                schema_id,
            );
            match producer.produce().await {
                Ok((partition, offset)) => {
                    log::info!("Message produced at partiton {partition} and offset {offset}");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    match consumer.consume(Some((partition, offset))).await {
                        Ok(msg) => {
                            let expect = serde_json::from_slice::<Value>(
                                std::fs::read(message_file.as_ref().unwrap())
                                    .unwrap()
                                    .as_slice(),
                            )
                            .unwrap();
                            let msg =
                                if let Ok(test) = serde_json::from_slice::<Value>(msg.as_slice()) {
                                    log::info!("json result {test}");
                                    test
                                } else {
                                    let reader = BufReader::new(msg.as_slice());
                                    // let mut reader = Reader::with_schema(&schema1, reader).unwrap();
                                    let mut reader: apache_avro::Reader<'_, BufReader<_>> =
                                        apache_avro::Reader::new(reader).unwrap();
                                    let value = reader.next().unwrap().unwrap();
                                    let value: Value =
                                        value.try_into().expect("cannot convert avro to json");
                                    log::info!("msg {value}");
                                    value
                                };
                            log::info!("expect {expect}");
                            assert_eq!(msg, expect);
                            log::info!("Integration test passed");
                        }
                        Err(e) => {
                            log::error!("Integration test failed: {:?}", e);
                            panic!("Integration test failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Message not produced: {:?}", e);
                    panic!("Message not produced: {:?}", e);
                }
            }
        }
    }
}
