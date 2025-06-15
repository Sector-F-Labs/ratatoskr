use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use teloxide::types::Message as TelegramMessage;
use uuid::Uuid;

// Unified incoming message type for the IN topic
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncomingMessage {
    pub trace_id: Uuid,
    pub message_type: IncomingMessageType,
    pub timestamp: DateTime<Utc>,
    pub source: MessageSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum IncomingMessageType {
    TelegramMessage(TelegramMessageData),
    CallbackQuery(CallbackQueryData),
    MessageReaction(MessageReactionData),
    EditedMessage(EditedMessageData),
}

/// Data for incoming Telegram messages
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelegramMessageData {
    /// The original Telegram message
    pub message: TelegramMessage,
    /// File attachments with download URLs - files are not downloaded yet
    pub file_attachments: Vec<FileInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallbackQueryData {
    pub chat_id: i64,
    pub user_id: u64,
    pub message_id: i32,
    pub callback_data: String,
    pub callback_query_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageReactionData {
    pub chat_id: i64,
    pub message_id: i32,
    pub user_id: Option<u64>, // None if anonymous
    pub date: DateTime<Utc>,
    pub old_reaction: Vec<String>, // emoji strings
    pub new_reaction: Vec<String>, // emoji strings
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditedMessageData {
    /// The edited Telegram message (contains both original and new content)
    pub message: TelegramMessage,
    /// File attachments with download URLs - files are not downloaded yet
    pub file_attachments: Vec<FileInfo>,
    /// Edit date from Telegram (when the message was edited)
    pub edit_date: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageSource {
    pub platform: String, // "telegram"
    pub bot_id: Option<u64>,
    pub bot_username: Option<String>,
}

/// Information about a file attached to a Telegram message
/// Contains metadata and download URL for the file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    /// Telegram file identifier - can be used to download the file
    pub file_id: String,
    /// Unique file identifier which is supposed to be the same over time and for different bots
    pub file_unique_id: String,
    /// Type of the file (photo, video, document, etc.)
    pub file_type: FileType,
    /// File size in bytes
    pub file_size: u32,
    /// Direct URL to download the file from Telegram servers
    pub file_url: String,
    /// Additional file-specific metadata
    pub metadata: FileMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileType {
    Photo,
    Audio,
    Voice,
    Video,
    VideoNote,
    Document,
    Sticker,
    Animation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileMetadata {
    Photo {
        width: u32,
        height: u32,
    },
    Audio {
        duration: u32,
        performer: Option<String>,
        title: Option<String>,
    },
    Voice {
        duration: u32,
    },
    Video {
        width: u32,
        height: u32,
        duration: u32,
    },
    VideoNote {
        length: u32,
        duration: u32,
    },
    Document {
        file_name: Option<String>,
        mime_type: Option<String>,
    },
    Sticker {
        width: u32,
        height: u32,
        emoji: Option<String>,
    },
    Animation {
        width: u32,
        height: u32,
        duration: u32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageInfo {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u32,
    pub local_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingCallbackMessage {
    pub chat_id: i64,
    pub user_id: u64,
    pub message_id: i32,
    pub callback_data: String,
    pub callback_query_id: String,
}

// Helper implementations
impl IncomingMessage {
    pub fn new_telegram_message(
        message: TelegramMessage,
        file_attachments: Vec<FileInfo>,
        bot_id: Option<u64>,
        bot_username: Option<String>,
    ) -> Self {
        Self {
            trace_id: Uuid::new_v4(),
            message_type: IncomingMessageType::TelegramMessage(TelegramMessageData {
                message,
                file_attachments,
            }),
            timestamp: Utc::now(),
            source: MessageSource {
                platform: "telegram".to_string(),
                bot_id,
                bot_username,
            },
        }
    }

    pub fn new_callback_query(
        chat_id: i64,
        user_id: u64,
        message_id: i32,
        callback_data: String,
        callback_query_id: String,
        bot_id: Option<u64>,
        bot_username: Option<String>,
    ) -> Self {
        Self {
            trace_id: Uuid::new_v4(),
            message_type: IncomingMessageType::CallbackQuery(CallbackQueryData {
                chat_id,
                user_id,
                message_id,
                callback_data,
                callback_query_id,
            }),
            timestamp: Utc::now(),
            source: MessageSource {
                platform: "telegram".to_string(),
                bot_id,
                bot_username,
            },
        }
    }

    pub fn new_message_reaction(
        chat_id: i64,
        message_id: i32,
        user_id: Option<u64>,
        date: DateTime<Utc>,
        old_reaction: Vec<String>,
        new_reaction: Vec<String>,
        bot_id: Option<u64>,
        bot_username: Option<String>,
    ) -> Self {
        Self {
            trace_id: Uuid::new_v4(),
            message_type: IncomingMessageType::MessageReaction(MessageReactionData {
                chat_id,
                message_id,
                user_id,
                date,
                old_reaction,
                new_reaction,
            }),
            timestamp: Utc::now(),
            source: MessageSource {
                platform: "telegram".to_string(),
                bot_id,
                bot_username,
            },
        }
    }

    pub fn new_edited_message(
        message: TelegramMessage,
        file_attachments: Vec<FileInfo>,
        edit_date: Option<i32>,
        bot_id: Option<u64>,
        bot_username: Option<String>,
    ) -> Self {
        Self {
            trace_id: Uuid::new_v4(),
            message_type: IncomingMessageType::EditedMessage(EditedMessageData {
                message,
                file_attachments,
                edit_date,
            }),
            timestamp: Utc::now(),
            source: MessageSource {
                platform: "telegram".to_string(),
                bot_id,
                bot_username,
            },
        }
    }
}
