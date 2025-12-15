use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::types::Update;
use teloxide::{dptree, prelude::*};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod telegram_handler;
use telegram_handler::{
    callback_query_handler, edited_message_handler, message_handler, message_reaction_handler,
};

mod utils;

mod broker;
use broker::{MessageBroker, pipe::PipeBroker};
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

    let bot = Bot::new(telegram_token.clone());

    let inbound_pipe =
        env::var("PIPE_OUTBOUND_PATH").unwrap_or_else(|_| "./ratatoskr_out.pipe".to_string());
    tracing::info!(path = %inbound_pipe, "Using pipe broker (stdout -> handler, pipe -> Telegram)");
    let broker = Arc::new(PipeBroker::new(inbound_pipe)) as Arc<dyn MessageBroker>;

    let bot_consumer_clone = bot.clone();
    let broker_clone = broker.clone();
    tokio::spawn(start_broker_consumer_loop(bot_consumer_clone, broker_clone));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_edited_message().endpoint(edited_message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler))
        .branch(Update::filter_message_reaction_updated().endpoint(message_reaction_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![broker])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
