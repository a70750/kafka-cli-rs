use log::{info, warn};

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::*;
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;

use std::path::PathBuf;

pub struct KafkaConsumer {
    client: ClientConfig,
    topic: String,
    key_file: Option<PathBuf>,
    consumer_group_id: String,
    partition: Option<u32>,
}

impl KafkaConsumer {
    pub fn new(
        config: &kafka_config::KafkaConfig,
        topic: &String,
        key_file: &Option<PathBuf>,
        consumer_group_id: &String,
        partition: &Option<u32>,
    ) -> Self {
        Self {
            topic: topic.clone(),
            client: config.clone().into(),
            key_file: key_file.clone(),
            consumer_group_id: consumer_group_id.clone(),
            partition: partition.clone(),
        }
    }
    pub async fn consume(&self) -> anyhow::Result<String> {
        let context = CustomContext;
        let consumer: LoggingConsumer = self.client.clone()
            .set("group.id", self.consumer_group_id.clone())
            // .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false") // TODO do we want to commit?
            // .set("delivery.timeout.ms", "1000") // THIS IS THE DEFAULT IN KafkaConfig
            //.set("statistics.interval.ms", "30000")
            //.set("auto.offset.reset", "smallest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Consumer creation failed");

        match consumer.recv().await {
            Err(e) => panic!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        info!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                // TODO do we want to commit?
                // consumer.commit_message(&m, CommitMode::Async).unwrap();

                Ok(payload.to_string())
            }
        }
    }
}

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;
