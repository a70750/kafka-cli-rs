use std::path::PathBuf;

pub struct ProducerConfig {
    pub topic: String,
    pub key_file: PathBuf,
    pub value_file: PathBuf,
    pub key_schema_file: Option<PathBuf>,
    pub value_schema_file: Option<PathBuf>,
    pub key_schema_id: Option<u32>,
    pub value_schema_id: Option<u32>,
}
