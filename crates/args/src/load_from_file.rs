use std::path::PathBuf;

use config::{Config, File};

use crate::{ConsumerConfig, ProducerConfig};

impl ConsumerConfig {
    pub fn from_file(file_or_folder: Option<PathBuf>) -> Self {
        let mut builder = Config::builder();
        // builder = builder.add_source(Config::try_from(&ConsumerConfig::default())?);
        if let Some(file_or_folder) = file_or_folder {
            if file_or_folder.is_file() {
                builder = builder.add_source(File::from(file_or_folder));
            } else {
                builder = builder
                    .add_source(File::from(file_or_folder.join("consumer.yaml")).required(false));
                builder = builder
                    .add_source(File::from(file_or_folder.join("consumer.json")).required(false));
            }
        } else {
            builder = builder
                .add_source(File::new("consumer.yaml", config::FileFormat::Yaml).required(false));
            builder = builder
                .add_source(File::new("consumer.json", config::FileFormat::Json5).required(false));
        }
        // builder = builder.add_source(config::Environment::with_prefix("KAFKA").separator("_"));
        let config: ConsumerConfig = builder
            .build()
            .expect("builder construction")
            .try_deserialize()
            .expect("deserialization of Consumer config failed");
        config
    }
}

impl ProducerConfig {
    pub fn from_file(file_or_folder: Option<PathBuf>) -> Self {
        let mut builder = Config::builder();
        // builder = builder.add_source(Config::try_from(&ConsumerConfig::default())?);
        if let Some(file_or_folder) = file_or_folder {
            if file_or_folder.is_file() {
                builder = builder.add_source(File::from(file_or_folder));
            } else {
                builder = builder
                    .add_source(File::from(file_or_folder.join("producer.yaml")).required(false));
                builder = builder
                    .add_source(File::from(file_or_folder.join("producer.json")).required(false));
            }
        } else {
            builder = builder
                .add_source(File::new("producer.yaml", config::FileFormat::Yaml).required(false));
            builder = builder
                .add_source(File::new("producer.json", config::FileFormat::Json5).required(false));
        }
        // builder = builder.add_source(config::Environment::with_prefix("KAFKA").separator("_"));
        let config: ProducerConfig = builder
            .build()
            .expect("builder construction")
            .try_deserialize()
            .expect("deserialization of Producer config failed");
        config
    }
}
