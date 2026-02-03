use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ratatoskr", about = "Telegram bot bridge")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Path to users.toml config file
    #[arg(long, global = true, default_value = "/etc/ratatoskr/users.toml")]
    pub users_file: PathBuf,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the bot
    Serve,
    /// Manage users
    Users {
        #[command(subcommand)]
        action: UsersAction,
    },
    /// Send a message to Telegram via Kafka (reads text from stdin)
    Send {
        /// Target chat ID
        #[arg(long)]
        chat_id: i64,
        /// Parse mode (HTML, Markdown)
        #[arg(long)]
        parse_mode: Option<String>,
        /// Thread ID for forum groups
        #[arg(long)]
        thread_id: Option<i32>,
    },
}

#[derive(Subcommand)]
pub enum UsersAction {
    /// Add a user
    Add {
        /// System username
        #[arg(long)]
        system_user: String,
        /// Telegram username to allow (can be repeated)
        #[arg(long)]
        username: Vec<String>,
        /// Promote on first auth (capture telegram_user_id automatically)
        #[arg(long)]
        promote: bool,
    },
    /// Remove a user
    Remove {
        /// System username to remove
        #[arg(long)]
        system_user: String,
    },
    /// List all users
    List,
}
