use dotenv::dotenv;
// Kafka-specific rdkafka imports are not needed when MQTT is active
// use rdkafka::config::ClientConfig;
// use rdkafka::producer::FutureProducer;
use std::env;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::types::Update;
use teloxide::{dptree, prelude::*};
use tracing_subscriber::{EnvFilter, fmt};

// Module declarations
mod messaging;
mod kafka_adapter; // Active
mod mqtt_adapter;  // Keep for potential switch

mod structs;
use structs::{ImageStorageDir, KafkaInTopic};

mod telegram_handler;
use telegram_handler::{callback_query_handler, message_handler, message_reaction_handler};

mod utils;

mod kafka_processing;
use kafka_processing::handle_outgoing_message; // Used by both consumer callbacks
use kafka_processing::outgoing::OutgoingMessage;
use crate::telegram_handler::incoming::IncomingMessage;

// Imports for MessageSystem and adapters
use crate::messaging::MessageSystem;
use crate::kafka_adapter::{KafkaAdapter, KafkaAdapterConfig, KafkaAdapterError}; // Active
use crate::mqtt_adapter::{MqttAdapter, MqttAdapterConfig, MqttAdapterError};   // Keep for commented section

// Type alias for the MessageSystem trait object
// For Kafka:
type DynMessageSystem = Arc<dyn MessageSystem<ConsumeMessage = OutgoingMessage, PublishMessage = IncomingMessage, Error = KafkaAdapterError> + Send + Sync>;
// For MQTT (illustrative):
// type DynMessageSystem = Arc<dyn MessageSystem<ConsumeMessage = OutgoingMessage, PublishMessage = IncomingMessage, Error = MqttAdapterError> + Send + Sync>;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    // Kafka specific env vars
    let kafka_brokers = env::var("KAFKA_BROKER").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_in_topic_val = env::var("KAFKA_IN_TOPIC").unwrap_or_else(|_| "com.sectorflabs.ratatoskr.in".to_string());
    let kafka_out_topic_val = env::var("KAFKA_OUT_TOPIC").unwrap_or_else(|_| "com.sectorflabs.ratatoskr.out".to_string());

    let kafka_in_topic_dependency = KafkaInTopic(kafka_in_topic_val.clone());

    let image_storage_dir = env::var("IMAGE_STORAGE_DIR").unwrap_or_else(|_| "./files/in".to_string());
    let image_storage_dir_dependency = ImageStorageDir(image_storage_dir.clone());

    let bot = Bot::new(telegram_token.clone());
    let bot_for_consumer_task = bot.clone();

    // TODO: Implement a mechanism to select the messaging system at runtime (e.g., via config).

    // --- Kafka Adapter Setup (Default) ---
    tracing::info!("Active Setup: Using Kafka Adapter");
    let kafka_adapter_config = KafkaAdapterConfig {
        brokers: kafka_brokers.clone(),
        group_id: "ratatoskr-bot-consumer".to_string(),
        consume_topic: kafka_out_topic_val.clone(),
        produce_topic: kafka_in_topic_val.clone(),
    };

    let mut consume_adapter = KafkaAdapter::new(kafka_adapter_config.clone())
        .expect("Failed to create KafkaAdapter for consumption");

    let mut publish_adapter = KafkaAdapter::new(kafka_adapter_config)
        .expect("Failed to create KafkaAdapter for publishing");

    tracing::info!("Connecting consumer KafkaAdapter...");
    consume_adapter.connect().await.expect("Failed to connect consumer KafkaAdapter");
    tracing::info!("Consumer KafkaAdapter connected successfully.");

    // For Kafka, publish_adapter doesn't strictly need connect() if producer is created in new().
    let shared_publish_system: DynMessageSystem = Arc::new(publish_adapter); // DynMessageSystem uses KafkaAdapterError here

    tokio::spawn(async move {
        tracing::info!("Starting Kafka consumer loop...");
        let consume_result = consume_adapter.consume(|message: OutgoingMessage| {
            let bot_clone = bot_for_consumer_task.clone();
            async move {
                tracing::debug!(message_type = ?message.message_type, "Received message from Kafka");
                if let Err(e) = handle_outgoing_message(&bot_clone, message).await {
                    tracing::error!(error = ?e, "Error handling OutgoingMessage from Kafka");
                }
            }
        }).await;

        if let Err(e) = consume_result {
            tracing::error!(error = ?e, "Kafka consumption loop failed: {:?}", e);
        } else {
            tracing::info!("Kafka consumption loop finished.");
        }
    });
    // --- End of Kafka Adapter Setup ---


    // --- Illustrative MQTT Adapter Setup (Commented out) ---
    /*
    tracing::info!("Commented Setup: Using MQTT Adapter");

    let mqtt_broker_addr = env::var("MQTT_BROKER_ADDR").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_broker_port = env::var("MQTT_BROKER_PORT")
        .map(|p| p.parse::<u16>().unwrap_or(1883))
        .unwrap_or(1883);
    let mqtt_consume_topic = env::var("MQTT_TG_OUTGOING_TOPIC").unwrap_or_else(|_| "ratatoskr/telegram/outgoing".to_string());
    let mqtt_publish_topic = env::var("MQTT_TG_INCOMING_TOPIC").unwrap_or_else(|_| "ratatoskr/telegram/incoming".to_string());

    let mqtt_adapter_config = MqttAdapterConfig {
        broker_address: mqtt_broker_addr,
        broker_port: mqtt_broker_port,
        client_id_prefix: "ratatoskr_mqtt_".to_string(),
        consume_topic: mqtt_consume_topic.clone(),
        publish_topic: mqtt_publish_topic.clone(),
        consume_qos: rumqttc::QoS::AtLeastOnce,
        publish_qos: rumqttc::QoS::AtLeastOnce,
        keep_alive_interval: std::time::Duration::from_secs(30),
    };

    let mut mqtt_consume_adapter = MqttAdapter::new(mqtt_adapter_config.clone())
        .expect("Failed to create MQTT consume adapter");

    let mut mqtt_publish_adapter = MqttAdapter::new(mqtt_adapter_config)
        .expect("Failed to create MQTT publish adapter");

    tracing::info!("Connecting MQTT consume adapter...");
    mqtt_consume_adapter.connect().await.expect("Failed to connect MQTT consume adapter");
    tracing::info!("MQTT consume adapter connected.");

    tracing::info!("Connecting MQTT publish adapter (initializes client and subscribes)...");
    mqtt_publish_adapter.connect().await.expect("Failed to connect MQTT publish adapter");
    tracing::info!("MQTT publish adapter client initialized and subscribed.");

    // let shared_publish_system: DynMessageSystem = Arc::new(mqtt_publish_adapter); // DynMessageSystem should use MqttAdapterError for this block

    tokio::spawn(async move {
        tracing::info!("Starting MQTT consumer loop (topic: {})...", mqtt_consume_adapter.config.consume_topic);
        let consume_result = mqtt_consume_adapter.consume(|message: OutgoingMessage| {
            let bot_clone = bot_for_consumer_task.clone();
            async move {
                tracing::debug!(message_type = ?message.message_type, "Received message from MQTT");
                if let Err(e) = handle_outgoing_message(&bot_clone, message).await {
                    tracing::error!(error = ?e, "Error handling OutgoingMessage from MQTT");
                }
            }
        }).await;

        if let Err(e) = consume_result {
            tracing::error!(error = ?e, "MQTT consumption loop failed: {:?}", e);
        } else {
            tracing::info!("MQTT consumption loop finished (this might indicate an issue if it wasn't intentional).");
        }
    });
    */
    // --- End of Illustrative MQTT Adapter Setup ---


    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler))
        .branch(Update::filter_message_reaction_updated().endpoint(message_reaction_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![shared_publish_system, kafka_in_topic_dependency, image_storage_dir_dependency])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
