use apache_avro::types::Value;
use apache_avro::{Reader, Schema};
use kafka_config::SchemaRegistryConfig;
use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use opentelemetry::{global, Context, Key, KeyValue, StringValue};
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::{get_schema_by_id, SrSettings};
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use tracing::{error, info};

pub struct KafkaProducer<'a> {
    client: ClientConfig,
    topic: String,
    message_file: PathBuf,
    key_file: PathBuf,
    schema_registry: Option<SchemaRegistry<'a>>,
    producer: FutureProducer,
}

pub struct SchemaRegistry<'a> {
    // url: url::Url,
    subject_name_strategy: SubjectNameStrategy,
    schema_id: u32,
    avro_encoder: AvroEncoder<'a>,
    sr_settings: SrSettings,
    schema: Schema,
}

impl KafkaProducer<'_> {
    pub fn new(
        config: &kafka_config::KafkaConfig,
        topic: &String,
        key_file: &PathBuf,
        message_file: &PathBuf,
        schema_id: &Option<u32>,
    ) -> Self {
        let client: ClientConfig = config.clone().into();

        Self {
            topic: topic.clone(),
            producer: client.create().expect("Producer creation error"),
            client,
            message_file: message_file.clone(),
            key_file: key_file.clone(),
            schema_registry: schema_id.clone().map(|id| {
                let SchemaRegistryConfig {
                    username,
                    password,
                    endpoint,
                } = config
                    .schema_registry
                    .as_ref()
                    .expect("schema registry config must be provided when using schema registry");
                let sr_settings: SrSettings = SrSettings::new_builder(endpoint.to_string())
                    .set_basic_authorization(username, Some(password))
                    .build()
                    .expect("Failed to build schema registry settings");
                let avro_encoder = AvroEncoder::new(sr_settings.clone());
                let schema = fs::read_to_string("resources/msg.avro")
                    .expect("Should have been able to read the file");
                let schema: Schema = Schema::parse_str(&schema).unwrap();
                let subject_name_strategy =
                    SubjectNameStrategy::TopicNameStrategy(topic.clone(), false);
                // upload schema!
                // let subject_name_strategy = SubjectNameStrategy::TopicNameStrategyWithSchema(
                //     topic.clone(),
                //     true,
                //     get_supplied_schema(&schema),
                // );
                SchemaRegistry {
                    // url: endpoint.clone(),
                    schema,
                    sr_settings,
                    schema_id: id,
                    subject_name_strategy,
                    avro_encoder,
                }
            }),
        }
    }
    pub async fn produce(&self) -> anyhow::Result<(i32, i64)> {
        let mut span = global::tracer("producer").start("produce_to_kafka");
        span.set_attribute(KeyValue {
            key: Key::new("topic"),
            value: opentelemetry::Value::String(StringValue::from(self.topic.clone())),
        });
        // Values might be sensitive, so we don't want to log them
        // span.set_attribute(KeyValue {
        //     key: Key::new("payload"),
        //     value: opentelemetry::Value::String(StringValue::from(
        //         serde_json::to_string(&payload).expect("Failed to serialize payload"),
        //     )),
        // });

        let context = Context::current_with_span(span);
        let mut headers = OwnedHeaders::new().insert(Header {
            key: "key",
            value: Some("value"),
        });
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&context, &mut schema_registry::HeaderInjector(&mut headers))
        });

        let message =
            fs::read_to_string(&self.message_file).expect("Should have been able to read the file");

        let payload = if let Some(sr) = &self.schema_registry {
            info!("Using avro encoder");

            let schema1 = get_schema_by_id(sr.schema_id.clone(), &sr.sr_settings)
                .await
                .expect("schema not found");

            let schema1 = Schema::parse_str(&schema1.schema).unwrap();
            pretty_assertions::assert_eq!(sr.schema, schema1);
            let file = fs::File::open(self.message_file.clone()).unwrap();
            let extension = self
                .message_file
                .extension()
                .expect("message_file must have extension");

            let map = if extension == "avro" {
                let reader = BufReader::new(file);
                // let mut reader = Reader::with_schema(&schema1, reader).unwrap();
                let mut reader: Reader<'_, BufReader<fs::File>> = Reader::new(reader).unwrap();
                let value = reader.next().unwrap().unwrap();
                value
            } else if extension == "json" {
                // let reader = Cursor::new(file);
                let text = fs::read_to_string(self.message_file.clone()).unwrap();
                let value: serde_json::Value =
                    serde_json::from_str::<serde_json::Value>(&text).unwrap();
                println!("json value: {:?}", value);
                let value: Value = value.into();
                println!("avro value: {:?}", value);
                if !value.validate(&sr.schema) {
                    panic!(
                        "Invalid value! Json message {:?} is not a valid schema: {:?}",
                        value, &sr.schema
                    );
                }
                value
            } else {
                panic!("Unsupported file extension {:?}", extension)
            };

            let value = match map {
                Value::Map(ref record) => {
                    let map: Vec<(&str, Value)> = record
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.clone()))
                        .collect();
                    map
                }
                Value::Record(ref record) => {
                    let map: Vec<(&str, Value)> = record
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.clone()))
                        .collect();
                    map
                }
                _ => panic!("Unsupported value type {:?}", map),
            };
            println!("value: {:?}", value);

            let payload = match sr
                .avro_encoder
                .encode(value, sr.subject_name_strategy.clone())
                .await
            {
                Ok(v) => v,
                Err(e) => panic!("Error encoding avro payload: {}", e),
            };
            payload
        } else {
            message.as_bytes().to_vec()
        };

        let key =
            fs::read_to_string(&self.key_file).expect("Should have been able to read the file");

        let delivery_status = self
            .producer
            .send(
                FutureRecord::to(&self.topic)
                    .headers(headers)
                    .key(&key)
                    .payload(&payload),
                Timeout::After(std::time::Duration::from_secs(5)),
            )
            .await;

        match delivery_status {
            Ok(status) => {
                info!("message delivered");
                Ok(status)
            }
            Err(e) => {
                error!("{}", e.0.to_string());
                panic!("{}", e.0.to_string());
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::async_impl::easy_avro::{EasyAvroDecoder, EasyAvroEncoder};
//     use crate::async_impl::schema_registry::SrSettings;
//     use crate::avro_common::get_supplied_schema;
//     use crate::schema_registry_common::SubjectNameStrategy;
//     use apache_avro::types::Value;
//     use apache_avro::{from_value, Schema};
//     use mockito::{mock, server_address};
//     use test_utils::Heartbeat;

//     #[tokio::test]
//     async fn test_decoder_default() {
//         let _m = mock("GET", "/schemas/ids/1?deleted=true")
//             .with_status(200)
//             .with_header("content-type", "application/vnd.schemaregistry.v1+json")
//             .with_body(r#"{"schema":"{\"type\":\"record\",\"name\":\"Heartbeat\",\"namespace\":\"nl.openweb.data\",\"fields\":[{\"name\":\"beat\",\"type\":\"long\"}]}"}"#)
//             .create();

//         let sr_settings = SrSettings::new(format!("http://{}", server_address()));
//         let decoder = EasyAvroDecoder::new(sr_settings);
//         let heartbeat = decoder
//             .decode(Some(&[0, 0, 0, 0, 1, 6]))
//             .await
//             .unwrap()
//             .value;

//         assert_eq!(
//             heartbeat,
//             Value::Record(vec![("beat".to_string(), Value::Long(3))])
//         );

//         let item = match from_value::<Heartbeat>(&heartbeat) {
//             Ok(h) => h,
//             Err(_) => unreachable!(),
//         };
//         assert_eq!(item.beat, 3i64);
//     }

//     #[tokio::test]
//     async fn test_decode_with_schema_default() {
//         let _m = mock("GET", "/schemas/ids/1?deleted=true")
//             .with_status(200)
//             .with_header("content-type", "application/vnd.schemaregistry.v1+json")
//             .with_body(r#"{"schema":"{\"type\":\"record\",\"name\":\"Heartbeat\",\"namespace\":\"nl.openweb.data\",\"fields\":[{\"name\":\"beat\",\"type\":\"long\"}]}"}"#)
//             .create();

//         let sr_settings = SrSettings::new(format!("http://{}", server_address()));
//         let decoder = EasyAvroDecoder::new(sr_settings);
//         let heartbeat = decoder
//             .decode_with_schema(Some(&[0, 0, 0, 0, 1, 6]))
//             .await
//             .unwrap()
//             .unwrap()
//             .value;

//         assert_eq!(
//             heartbeat,
//             Value::Record(vec![("beat".to_string(), Value::Long(3))])
//         );

//         let item = match from_value::<Heartbeat>(&heartbeat) {
//             Ok(h) => h,
//             Err(_) => unreachable!(),
//         };
//         assert_eq!(item.beat, 3i64);
//     }

//     #[tokio::test]
//     async fn test_encode_value() {
//         let _m = mock("GET", "/subjects/heartbeat-value/versions/latest")
//             .with_status(200)
//             .with_header("content-type", "application/vnd.schemaregistry.v1+json")
//             .with_body(r#"{"subject":"heartbeat-value","version":1,"id":3,"schema":"{\"type\":\"record\",\"name\":\"Heartbeat\",\"namespace\":\"nl.openweb.data\",\"fields\":[{\"name\":\"beat\",\"type\":\"long\"}]}"}"#)
//             .create();

//         let sr_settings = SrSettings::new(format!("http://{}", server_address()));
//         let encoder = EasyAvroEncoder::new(sr_settings);

//         let value_strategy =
//             SubjectNameStrategy::TopicNameStrategy(String::from("heartbeat"), false);
//         let bytes = encoder
//             .encode(vec![("beat", Value::Long(3))], value_strategy)
//             .await
//             .unwrap();

//         assert_eq!(bytes, vec![0, 0, 0, 0, 3, 6])
//     }

//     #[tokio::test]
//     async fn test_primitive_schema() {
//         let sr_settings = SrSettings::new(format!("http://{}", server_address()));
//         let encoder = EasyAvroEncoder::new(sr_settings);

//         let _n = mock("POST", "/subjects/heartbeat-key/versions")
//             .with_status(200)
//             .with_header("content-type", "application/vnd.schemaregistry.v1+json")
//             .with_body(r#"{"id":4}"#)
//             .create();

//         let primitive_schema_strategy = SubjectNameStrategy::TopicNameStrategyWithSchema(
//             String::from("heartbeat"),
//             true,
//             get_supplied_schema(&Schema::String),
//         );
//         let bytes = encoder
//             .encode_struct("key-value", &primitive_schema_strategy)
//             .await;

//         assert_eq!(
//             bytes,
//             Ok(vec![
//                 0, 0, 0, 0, 4, 18, 107, 101, 121, 45, 118, 97, 108, 117, 101
//             ])
//         );
//     }
// }
