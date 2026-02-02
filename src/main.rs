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
use broker::{MessageBroker, pipe::PipeBroker, user_pipes::UserPipeRouter};
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
            ref pipe_dir,
        } => run_send(chat_id, parse_mode.as_deref(), thread_id, pipe_dir.as_deref()),
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

fn run_send(chat_id: i64, parse_mode: Option<&str>, thread_id: Option<i32>, pipe_dir: Option<&str>) {
    use kafka_processing::outgoing::{
        MessageTarget, OutgoingMessage, OutgoingMessageType, TextMessageData,
    };

    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .expect("Failed to read from stdin");

    let text = text.trim_end().to_string();
    if text.is_empty() {
        eprintln!("Error: empty input");
        std::process::exit(1);
    }

    let pipe_dir = match pipe_dir {
        Some(d) => d.to_string(),
        None => {
            let uid = nix::unistd::getuid();
            format!("/run/user/{}/ratatoskr/", uid)
        }
    };

    let out_pipe = std::path::Path::new(&pipe_dir).join("out.pipe");

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
    let mut line = json;
    line.push('\n');

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(&out_pipe)
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to open {}: {e}", out_pipe.display());
            std::process::exit(1);
        });

    use std::io::Write;
    file.write_all(line.as_bytes()).unwrap_or_else(|e| {
        eprintln!("Error: failed to write to {}: {e}", out_pipe.display());
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

    let bot = Bot::new(telegram_token.clone());

    let inbound_pipe =
        env::var("PIPE_OUTBOUND_PATH").unwrap_or_else(|_| "./ratatoskr_out.pipe".to_string());
    tracing::info!(path = %inbound_pipe, "Using pipe broker (stdout -> handler, pipe -> Telegram)");
    let broker = Arc::new(PipeBroker::new(inbound_pipe)) as Arc<dyn MessageBroker>;

    // Set up per-user pipe routing if users are configured, otherwise fall back to global broker
    let user_router: Option<Arc<UserPipeRouter>> = if has_users {
        let (router, response_rx) = UserPipeRouter::new(auth_service.clone());
        let router = Arc::new(router);
        router.start_readers().await;
        let bot_pipe_clone = bot.clone();
        tokio::spawn(start_user_pipe_consumer_loop(bot_pipe_clone, response_rx));
        tracing::info!("Per-user pipe routing enabled");
        Some(router)
    } else {
        let bot_consumer_clone = bot.clone();
        let broker_clone = broker.clone();
        tokio::spawn(start_broker_consumer_loop(bot_consumer_clone, broker_clone));
        None
    };

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_edited_message().endpoint(edited_message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler))
        .branch(Update::filter_message_reaction_updated().endpoint(message_reaction_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![broker, auth_service, user_router])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
