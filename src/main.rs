use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::types::Update;
use teloxide::{dptree, prelude::*};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod structs;
use structs::{ImageStorageDir, KafkaInTopic};

mod telegram_handler;
use telegram_handler::{
    callback_query_handler, edited_message_handler, message_handler, message_reaction_handler,
};

mod utils;

mod broker;
use broker::{kafka::KafkaBroker, mqtt::MqttBroker, MessageBroker};
mod kafka_processing;
use kafka_processing::*;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Ratatoskr bot...");

    let telegram_token =
        env::var("TELEGRAM_BOT_TOKEN").expect("FATAL: TELEGRAM_BOT_TOKEN not set in environment");

    let broker_type = env::var("BROKER_TYPE").unwrap_or_else(|_| "kafka".to_string());
    tracing::info!(%broker_type, "Selected message broker type");

    let kafka_broker = env::var("KAFKA_BROKER").unwrap_or_else(|_| {
        tracing::info!("KAFKA_BROKER not set, defaulting to localhost:9092");
        "localhost:9092".to_string()
    });

    let kafka_in_topic_val = env::var("KAFKA_IN_TOPIC").unwrap_or_else(|_| {
        tracing::info!("KAFKA_IN_TOPIC not set, defaulting to com.sectorflabs.ratatoskr.in");
        "com.sectorflabs.ratatoskr.in".to_string()
    });
    let kafka_out_topic = env::var("KAFKA_OUT_TOPIC").unwrap_or_else(|_| {
        tracing::info!("KAFKA_OUT_TOPIC not set, defaulting to com.sectorflabs.ratatoskr.out");
        "com.sectorflabs.ratatoskr.out".to_string()
    });

    let image_storage_dir = env::var("IMAGE_STORAGE_DIR").unwrap_or_else(|_| {
        tracing::info!("IMAGE_STORAGE_DIR not set, defaulting to ./images");
        "./files/in".to_string()
    });
    let image_storage_dir = ImageStorageDir(image_storage_dir.clone());
    tracing::info!(image_storage_dir = %image_storage_dir.0, "Using image storage directory");

    let bot = Bot::new(telegram_token.clone());

    let (broker, in_topic_val, out_topic_val): (Arc<dyn MessageBroker>, String, String) =
        if broker_type.to_lowercase() == "mqtt" {
            let mqtt_broker_addr = env::var("MQTT_BROKER").unwrap_or_else(|_| "localhost:1883".to_string());
            let mqtt_in_topic = env::var("MQTT_IN_TOPIC").unwrap_or_else(|_| "com.sectorflabs.ratatoskr.in".to_string());
            let mqtt_out_topic = env::var("MQTT_OUT_TOPIC").unwrap_or_else(|_| "com.sectorflabs.ratatoskr.out".to_string());
            let broker = Arc::new(MqttBroker::new(&mqtt_broker_addr).await.expect("Failed to create MQTT broker")) as Arc<dyn MessageBroker>;
            (broker, mqtt_in_topic, mqtt_out_topic)
        } else {
            let broker = Arc::new(KafkaBroker::new(&kafka_broker).expect("Failed to create Kafka broker")) as Arc<dyn MessageBroker>;
            (broker, kafka_in_topic_val.clone(), kafka_out_topic.clone())
        };

    let in_topic = KafkaInTopic(in_topic_val.clone());

    let bot_consumer_clone = bot.clone();
    let out_topic_clone = out_topic_val.clone();
    let broker_clone = broker.clone();
    tokio::spawn(start_broker_consumer_loop(
        bot_consumer_clone,
        broker_clone,
        out_topic_clone,
    ));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_edited_message().endpoint(edited_message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler))
        .branch(Update::filter_message_reaction_updated().endpoint(message_reaction_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![broker, in_topic, image_storage_dir])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
