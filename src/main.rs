use clap::Parser;
use dotenv::dotenv;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::FutureProducer;
use std::env;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::types::Update;
use teloxide::{dptree, prelude::*};
use tracing_subscriber::{EnvFilter, fmt};

mod structs;
use structs::{ImageStorageDir, KafkaInTopic};

mod telegram_handler;
use telegram_handler::{callback_query_handler, message_handler, message_reaction_handler};

mod utils;

mod kafka_processing;
use kafka_processing::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(long, action = clap::ArgAction::SetTrue, help = "Disable Kafka streaming")]
    no_streaming: bool,

    #[clap(long, help = "Override the directory for saving downloaded files")]
    files_dir: Option<String>,
}

// Helper function for determining image_storage_dir_string
fn determine_image_storage_dir_string(
    cli_files_dir: Option<String>,
    env_image_storage_dir: Option<String>, // Simulates env::var result
    default_path: &str
) -> String {
    if let Some(files_dir_override) = cli_files_dir {
        files_dir_override
    } else if let Some(env_dir) = env_image_storage_dir {
        env_dir
    } else {
        default_path.to_string()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli_args = CliArgs::parse();

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set global default tracing subscriber: {}", e);
    }

    tracing::info!("Starting Ratatoskr bot...");

    let telegram_token =
        env::var("TELEGRAM_BOT_TOKEN").expect("FATAL: TELEGRAM_BOT_TOKEN not set in environment");

    let kafka_broker = env::var("KAFKA_BROKER").unwrap_or_else(|_| {
        tracing::info!("KAFKA_BROKER not set, defaulting to localhost:9092");
        "localhost:9092".to_string()
    });
    tracing::info!(kafka_broker = %kafka_broker, "Using Kafka broker");

    // kafka_in_topic is now Option<KafkaInTopic> and initialized conditionally later
    // let kafka_in_topic_val = env::var("KAFKA_IN_TOPIC").unwrap_or_else(|_| {
    //     tracing::info!("KAFKA_IN_TOPIC not set, defaulting to com.sectorflabs.ratatoskr.in");
    //     "com.sectorflabs.ratatoskr.in".to_string()
    // });
    // let kafka_in_topic = KafkaInTopic(kafka_in_topic_val.clone());
    // tracing::info!(kafka_in_topic = %kafka_in_topic.0, "Using Kafka IN topic");

    let kafka_out_topic = env::var("KAFKA_OUT_TOPIC").unwrap_or_else(|_| {
        tracing::info!("KAFKA_OUT_TOPIC not set, defaulting to com.sectorflabs.ratatoskr.out");
        "com.sectorflabs.ratatoskr.out".to_string()
    });
    tracing::info!(kafka_out_topic = %kafka_out_topic, "Using Kafka OUT topic");

    let files_dir_from_cli = cli_args.files_dir.clone();
    let image_storage_env_var = env::var("IMAGE_STORAGE_DIR").ok();

    let image_storage_dir_string = determine_image_storage_dir_string(
        files_dir_from_cli, // Already an Option<String>
        image_storage_env_var.clone(), // Clone because we check it again for tracing
        "./files/in" // The default path
    );

    if cli_args.files_dir.is_some() {
        tracing::info!(files_dir = %image_storage_dir_string, "Using --files_dir override for image storage directory");
    } else if image_storage_env_var.is_some() {
         tracing::info!(env_dir = %image_storage_dir_string, "Using IMAGE_STORAGE_DIR for image storage directory");
    } else {
        tracing::info!(default_dir = %image_storage_dir_string, "Using default path for image storage directory");
    }
    let image_storage_dir = ImageStorageDir(image_storage_dir_string); // .clone() removed
    tracing::info!(path = %image_storage_dir.0, "Final image storage directory set");

    let bot = Bot::new(telegram_token.clone());

    let producer: Option<Arc<FutureProducer>>;
    let kafka_in_topic: Option<KafkaInTopic>;

    if !cli_args.no_streaming {
        let topic_name = env::var("KAFKA_IN_TOPIC").unwrap_or_else(|_| {
            tracing::info!("KAFKA_IN_TOPIC not set, defaulting to com.sectorflabs.ratatoskr.in");
            "com.sectorflabs.ratatoskr.in".to_string()
        });
        kafka_in_topic = Some(KafkaInTopic(topic_name));
        tracing::info!(kafka_in_topic = %kafka_in_topic.as_ref().unwrap().0, "Using Kafka IN topic");

        let actual_producer = Arc::new(
            ClientConfig::new()
                .set("bootstrap.servers", &kafka_broker)
                .create()
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Kafka producer creation error");
                    panic!("Kafka producer creation error: {}", e);
                }),
        );
        producer = Some(actual_producer);
        tracing::info!("Kafka producer created successfully.");

        // kafka_in_topic is initialized above, within this block.

        let actual_consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", "ratatoskr-bot-consumer")
            .set("bootstrap.servers", &kafka_broker)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "Kafka consumer creation error");
                panic!("Kafka consumer creation error: {}", e);
            });
        tracing::info!("Kafka consumer created successfully.");

        actual_consumer.subscribe(&[&kafka_out_topic]).unwrap_or_else(|e| {
            tracing::error!(topic = %kafka_out_topic, error = %e, "Failed to subscribe to Kafka topic");
            panic!(
                "Failed to subscribe to Kafka topic {}: {}",
                kafka_out_topic, e
            );
        });
        tracing::info!(topic = %kafka_out_topic, "Subscribed to Kafka topic successfully.");

        let bot_consumer_clone = bot.clone();
        let kafka_out_topic_clone = kafka_out_topic.clone();
        tokio::spawn(start_kafka_consumer_loop(
            bot_consumer_clone,
            actual_consumer,
            kafka_out_topic_clone,
        ));
    } else {
        producer = None;
        kafka_in_topic = None;
        tracing::info!("Kafka streaming is disabled via --no-streaming flag. Kafka producer, consumer, and message loop will not be initialized.");
    }

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler))
        .branch(Update::filter_message_reaction_updated().endpoint(message_reaction_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![producer, kafka_in_topic, image_storage_dir])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[cfg(test)]
mod tests {
    use super::*; // To import CliArgs and determine_image_storage_dir_string

    #[test]
    fn test_cli_args_no_streaming() {
        let args = CliArgs::try_parse_from(&["ratatoskr", "--no-streaming"]).unwrap();
        assert!(args.no_streaming);
        assert!(args.files_dir.is_none());
    }

    #[test]
    fn test_cli_args_files_dir() {
        let args = CliArgs::try_parse_from(&["ratatoskr", "--files_dir", "/custom/path"]).unwrap();
        assert!(!args.no_streaming);
        assert_eq!(args.files_dir, Some("/custom/path".to_string()));
    }

    #[test]
    fn test_cli_args_both() {
        let args = CliArgs::try_parse_from(&["ratatoskr", "--no-streaming", "--files_dir", "/another/path"]).unwrap();
        assert!(args.no_streaming);
        assert_eq!(args.files_dir, Some("/another/path".to_string()));
    }

    #[test]
    fn test_cli_args_none() {
        let args = CliArgs::try_parse_from(&["ratatoskr"]).unwrap();
        assert!(!args.no_streaming);
        assert!(args.files_dir.is_none());
    }

    #[test]
    fn test_determine_image_storage_dir_cli_override() {
        let cli_arg = Some("/cli/path".to_string());
        let env_arg = Some("/env/path".to_string());
        let default = "./files/in";
        assert_eq!(determine_image_storage_dir_string(cli_arg, env_arg, default), "/cli/path");
    }

    #[test]
    fn test_determine_image_storage_dir_env_override() {
        let cli_arg = None;
        let env_arg = Some("/env/path".to_string());
        let default = "./files/in";
        assert_eq!(determine_image_storage_dir_string(cli_arg, env_arg, default), "/env/path");
    }

    #[test]
    fn test_determine_image_storage_dir_default() {
        let cli_arg = None;
        let env_arg = None;
        let default = "./files/in";
        assert_eq!(determine_image_storage_dir_string(cli_arg, env_arg, default), "./files/in");
    }
}
