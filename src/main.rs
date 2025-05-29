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

mod incoming;
mod outgoing;

mod telegram_handlers;
use telegram_handlers::*;

mod utils;

mod kafka_processing;
use kafka_processing::*;

#[tokio::main]
async fn main() {
    dotenv().ok();

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

    let kafka_in_topic_val = env::var("KAFKA_IN_TOPIC").unwrap_or_else(|_| {
        tracing::info!("KAFKA_IN_TOPIC not set, defaulting to com.sectorflabs.ratatoskr.in");
        "com.sectorflabs.ratatoskr.in".to_string()
    });
    let kafka_in_topic = KafkaInTopic(kafka_in_topic_val.clone());
    tracing::info!(kafka_in_topic = %kafka_in_topic.0, "Using Kafka IN topic");

    let kafka_out_topic = env::var("KAFKA_OUT_TOPIC").unwrap_or_else(|_| {
        tracing::info!("KAFKA_OUT_TOPIC not set, defaulting to com.sectorflabs.ratatoskr.out");
        "com.sectorflabs.ratatoskr.out".to_string()
    });
    tracing::info!(kafka_out_topic = %kafka_out_topic, "Using Kafka OUT topic");

    let image_storage_dir = env::var("IMAGE_STORAGE_DIR").unwrap_or_else(|_| {
        tracing::info!("IMAGE_STORAGE_DIR not set, defaulting to ./images");
        "./files/in".to_string()
    });
    let image_storage_dir = ImageStorageDir(image_storage_dir.clone());
    tracing::info!(image_storage_dir = %image_storage_dir.0, "Using image storage directory");

    let bot = Bot::new(telegram_token.clone());

    let producer: Arc<FutureProducer> = Arc::new(
        ClientConfig::new()
            .set("bootstrap.servers", &kafka_broker)
            .create()
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "Kafka producer creation error");
                panic!("Kafka producer creation error: {}", e);
            }),
    );
    tracing::info!("Kafka producer created successfully.");

    let consumer: StreamConsumer = ClientConfig::new()
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

    consumer.subscribe(&[&kafka_out_topic]).unwrap_or_else(|e| {
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
        consumer,
        kafka_out_topic_clone,
    ));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![producer, kafka_in_topic, image_storage_dir])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
