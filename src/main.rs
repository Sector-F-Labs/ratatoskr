use clap::Parser;
use dotenv::dotenv;
use std::env;
use std::io::Read as _;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::types::Update;
use teloxide::{dptree, prelude::*};
use tokio::sync::RwLock;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
use cli::{Cli, Command, UsersAction};

mod telegram_handler;
use telegram_handler::{
    callback_query_handler, edited_message_handler, message_handler, message_reaction_handler,
};

mod utils;

mod broker;
use broker::{MessageBroker, kafka::KafkaBroker};
mod kafka_processing;
use kafka_processing::*;

mod auth;
use auth::AuthService;
mod config;
use config::UsersConfig;
mod users;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve => run_serve(&cli).await,
        Command::Users { ref action } => run_users(&cli, action),
        Command::Send {
            chat_id,
            ref parse_mode,
            thread_id,
        } => run_send(chat_id, parse_mode.as_deref(), thread_id),
    }
}

fn run_users(cli: &Cli, action: &UsersAction) {
    match action {
        UsersAction::Add {
            system_user,
            username,
            promote,
        } => {
            if let Err(e) = users::add_user(&cli.users_file, system_user, username, *promote) {
                eprintln!("Error: {e:#}");
                std::process::exit(1);
            }
        }
        UsersAction::Remove { system_user } => {
            if let Err(e) = users::remove_user(&cli.users_file, system_user) {
                eprintln!("Error: {e:#}");
                std::process::exit(1);
            }
        }
        UsersAction::List => {
            if let Err(e) = users::list_users(&cli.users_file) {
                eprintln!("Error: {e:#}");
                std::process::exit(1);
            }
        }
    }
}

fn run_send(chat_id: i64, parse_mode: Option<&str>, thread_id: Option<i32>) {
    use kafka_processing::outgoing::{
        MessageTarget, OutgoingMessage, OutgoingMessageType, TextMessageData,
    };
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
    use std::time::Duration;

    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .expect("Failed to read from stdin");

    let text = text.trim_end().to_string();
    if text.is_empty() {
        eprintln!("Error: empty input");
        std::process::exit(1);
    }

    let brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    let topic_prefix = env::var("KAFKA_TOPIC_PREFIX").unwrap_or_else(|_| "ratatoskr".to_string());
    let topic = format!("{}.out", topic_prefix);

    let msg = OutgoingMessage {
        trace_id: uuid::Uuid::new_v4(),
        message_type: OutgoingMessageType::TextMessage(TextMessageData {
            text,
            buttons: None,
            reply_keyboard: None,
            parse_mode: parse_mode.map(String::from),
            disable_web_page_preview: None,
        }),
        timestamp: chrono::Utc::now(),
        target: MessageTarget {
            platform: "telegram".to_string(),
            chat_id,
            thread_id,
        },
    };

    let json = serde_json::to_string(&msg).expect("Failed to serialize message");

    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to create Kafka producer: {e}");
            std::process::exit(1);
        });

    let key = chat_id.to_string();
    producer
        .send(
            BaseRecord::to(&topic)
                .key(&key)
                .payload(json.as_bytes()),
        )
        .unwrap_or_else(|(e, _)| {
            eprintln!("Error: failed to send to Kafka: {e}");
            std::process::exit(1);
        });

    // Flush to ensure message is sent
    producer.flush(Duration::from_secs(5)).unwrap_or_else(|e| {
        eprintln!("Error: failed to flush Kafka producer: {e}");
        std::process::exit(1);
    });
}

async fn run_serve(cli: &Cli) {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Ratatoskr bot...");

    // Load auth config
    let users_config = UsersConfig::load(&cli.users_file).unwrap_or_else(|e| {
        tracing::warn!(error = %e, path = %cli.users_file.display(), "Failed to load users config, starting without auth");
        UsersConfig::default()
    });
    let has_users = !users_config.users.is_empty();
    let auth_service = Arc::new(RwLock::new(AuthService::new(
        users_config,
        cli.users_file.clone(),
    )));
    if has_users {
        tracing::info!(path = %cli.users_file.display(), "Auth enabled — loaded users config");
    } else {
        tracing::info!("No users configured — auth gate disabled (all messages pass through)");
    }

    let telegram_token =
        env::var("TELEGRAM_BOT_TOKEN").expect("FATAL: TELEGRAM_BOT_TOKEN not set in environment");
    let kafka_brokers =
        env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_topic_prefix = env::var("KAFKA_TOPIC_PREFIX").ok();

    let bot = Bot::new(telegram_token.clone());

    tracing::info!(
        brokers = %kafka_brokers,
        topic_prefix = ?kafka_topic_prefix,
        "Using Kafka broker"
    );
    let kafka_broker = KafkaBroker::new(
        &kafka_brokers,
        kafka_topic_prefix.as_deref(),
        None,
    )
    .expect("Failed to create Kafka broker");

    // Ensure topics exist
    kafka_broker
        .ensure_topics()
        .await
        .expect("Failed to create Kafka topics");

    let broker = Arc::new(kafka_broker) as Arc<dyn MessageBroker>;

    // Start consumer loop for outgoing messages
    let bot_consumer_clone = bot.clone();
    let broker_clone = broker.clone();
    tokio::spawn(start_broker_consumer_loop(bot_consumer_clone, broker_clone));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_edited_message().endpoint(edited_message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler))
        .branch(Update::filter_message_reaction_updated().endpoint(message_reaction_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![broker, auth_service])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
