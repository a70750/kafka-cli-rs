use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use std::fs;
use std::path::PathBuf;

pub struct KafkaProducer {
    client: ClientConfig,
    topic: String,
    message_file: PathBuf,
    key_file: PathBuf,
    schema_registry_url: &Option<url::Url>,
    producer: FutureProducer
}

impl KafkaProducer {
    pub fn new(
        config: &kafka_config::KafkaConfig,
        topic: &String,
        key_file: &PathBuf,
        message_file: &PathBuf,
        schema_registry_url: &Option<url::Url>,
        producer: FutureProducer
    ) -> Self {
        Self {
            topic: topic.clone(),
            client: config.clone().into(),
            message_file: message_file.clone(),
            key_file: key_file.clone(),
            producer: self.client.create().expect("Producer creation error"),
            schema_registry_url
        }
    }
    pub async fn produce(&self) -> anyhow::Result<(i32, i64)> {
        let value_strategy = SubjectNameStrategy::TopicNameStrategyWithSchema(
            self.topic.clone(),
            true,
            schema_registry::get_supplied_schema(&T::get_schema()),
        );

        let message =
        fs::read_to_string(&self.message_file).expect("Should have been able to read the file");

        // let payload = match self
        //     .avro_encoder
        //     .clone()
        //     .encode_struct(payload, &value_strategy)
        //     .await
        // {
        //     Ok(v) => v,
        //     Err(e) => panic!("Error getting payload: {}", e),
        // };

      
        let key =
            fs::read_to_string(&self.key_file).expect("Should have been able to read the file");

        let delivery_status = producer
            .send(
                FutureRecord::to(&self.topic).key(&key).payload(&message),
                Timeout::After(std::time::Duration::from_secs(1)),
            )
            .await;

        Ok(delivery_status.expect("Should have been able to send the message"))
    }
}
