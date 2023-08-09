use clap::Parser;
use env_logger::{Builder, Target};

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
            ref schema_registry_url,
            ref schema_id
        }) => {
            let consumer = kafka_consumer::KafkaConsumer::new(
                &config,
                topic,
                key_file,
                consumer_group_id,
                partition,
            );
            match consumer.consume().await {
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
            ref schema_registry_url,
            ref schema_id,
        }) => {
            let producer = kafka_producer::KafkaProducer::new(
                &config,
                topic,
                key_file,
                message_file.as_ref().unwrap(),
                schema_registry_url
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
            ref schema_registry_url,
            ref schema_id
        }) => {
            let producer = kafka_producer::KafkaProducer::new(
                &config,
                topic,
                key_file,
                message_file.as_ref().unwrap(),
                schema_registry_url
            );
            let consumer = kafka_consumer::KafkaConsumer::new(
                &config,
                topic,
                &Some(key_file.clone()),
                consumer_group_id,
                partition,
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
            match consumer.consume().await {
                Ok(_) => {
                    log::info!("Integration test passed");
                }
                Err(e) => {
                    log::error!("Integration test failed: {:?}", e);
                    panic!("Integration test failed: {:?}", e);
                }
            }
        }
    }
}
