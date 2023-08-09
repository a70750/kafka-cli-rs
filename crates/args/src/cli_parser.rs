use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "Kafka Cli")]
#[command(author)]
#[command(version)]
#[command(propagate_version = true)]
#[command(about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub action: Action,
    #[clap(value_enum, default_value_t=LogOutput::StdOut)]
    #[arg(short, long)]
    pub log_output: LogOutput,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogOutput {
    StdOut,
    StdErr,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    Consume(ConsumerConfig),
    Produce(ProducerConfig),
    IntegrationTest(TestConfig),
}

#[derive(Args, Debug)]
pub struct TestConfig {
    #[arg(short, long)]
    pub topic: String,
    #[arg(short, long)]
    pub key_file: PathBuf,
    #[arg(short, long)]
    pub message_file: Option<PathBuf>,
    #[arg(short, long)]
    pub consumer_group_id: String,
    #[arg(short, long)]
    pub partition: Option<u32>,
    #[arg(short, long)]
    pub schema_registry_url: Option<url::Url>,
    #[arg(short, long)]
    pub schema_id: Option<String>,
}
#[derive(Args, Debug)]
pub struct ConsumerConfig {
    #[arg(short, long)]
    pub topic: String,
    #[arg(short, long)]
    pub key_file: Option<PathBuf>,
    #[arg(short, long)]
    pub consumer_group_id: String,
    #[arg(short, long)]
    pub partition: Option<u32>,
    #[arg(short, long)]
    pub schema_registry_url: Option<url::Url>,
    #[arg(short, long)]
    pub schema_id: Option<String>,
}
#[derive(Args, Debug)]
pub struct ProducerConfig {
    #[arg(short, long)]
    pub topic: String,
    #[arg(short, long)]
    pub key_file: PathBuf,
    #[arg(short, long)]
    pub message_file: Option<PathBuf>,
    #[arg(short, long)]
    pub schema_registry_url: Option<url::Url>,
    #[arg(short, long)]
    pub schema_id: Option<String>,

}
