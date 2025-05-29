use serde::{Deserialize, Serialize};
use teloxide::types::Message as TelegramMessage;
use chrono::{DateTime, Utc};

// Unified incoming message type for the IN topic
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncomingMessage {
    pub message_type: IncomingMessageType,
    pub timestamp: DateTime<Utc>,
    pub source: MessageSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum IncomingMessageType {
    TelegramMessage(TelegramMessageData),
    CallbackQuery(CallbackQueryData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelegramMessageData {
    pub message: TelegramMessage,
    pub downloaded_files: Vec<FileInfo>,
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
pub struct MessageSource {
    pub platform: String, // "telegram"
    pub bot_id: Option<u64>,
    pub bot_username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_type: FileType,
    pub file_size: u32,
    pub local_path: String,
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
    Photo { width: u32, height: u32 },
    Audio { duration: u32, performer: Option<String>, title: Option<String> },
    Voice { duration: u32 },
    Video { width: u32, height: u32, duration: u32 },
    VideoNote { length: u32, duration: u32 },
    Document { file_name: Option<String>, mime_type: Option<String> },
    Sticker { width: u32, height: u32, emoji: Option<String> },
    Animation { width: u32, height: u32, duration: u32 },
}

// Legacy compatibility - deprecated
#[deprecated(note = "Use FileInfo instead")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageInfo {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u32,
    pub local_path: String,
}

#[deprecated(note = "Use IncomingMessage with IncomingMessageType::CallbackQuery instead")]
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
        downloaded_files: Vec<FileInfo>,
        bot_id: Option<u64>,
        bot_username: Option<String>,
    ) -> Self {
        Self {
            message_type: IncomingMessageType::TelegramMessage(TelegramMessageData {
                message,
                downloaded_files,
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
}