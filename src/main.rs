use clap::Parser;
use dotenv::dotenv;
use std::env;
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
use broker::{MessageBroker, pipe::PipeBroker};
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
        .dependencies(dptree::deps![broker, auth_service])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
