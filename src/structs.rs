use serde::{Deserialize, Serialize};
use teloxide::types::Message as TelegramMessage;
use chrono::{DateTime, Utc};

// Wrapper types to distinguish between different string parameters in dependency injection
#[derive(Debug, Clone)]
pub struct KafkaInTopic(pub String);

#[derive(Debug, Clone)]
pub struct ImageStorageDir(pub String);

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
    pub downloaded_images: Vec<ImageInfo>,
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

// Unified outgoing message type for the OUT topic
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutgoingMessage {
    pub message_type: OutgoingMessageType,
    pub timestamp: DateTime<Utc>,
    pub target: MessageTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum OutgoingMessageType {
    TextMessage(TextMessageData),
    ImageMessage(ImageMessageData),
    DocumentMessage(DocumentMessageData),
    EditMessage(EditMessageData),
    DeleteMessage(DeleteMessageData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessageData {
    pub text: String,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub parse_mode: Option<String>, // "HTML", "Markdown", etc.
    pub disable_web_page_preview: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageMessageData {
    pub image_path: String,
    pub caption: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentMessageData {
    pub document_path: String,
    pub filename: Option<String>,
    pub caption: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditMessageData {
    pub message_id: i32,
    pub new_text: Option<String>,
    pub new_buttons: Option<Vec<Vec<ButtonInfo>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteMessageData {
    pub message_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageTarget {
    pub platform: String, // "telegram"
    pub chat_id: i64,
    pub thread_id: Option<i32>, // For forum groups
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ButtonInfo {
    pub text: String,
    pub callback_data: String,
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

// Legacy compatibility types (deprecated but kept for backwards compatibility)
#[deprecated(note = "Use IncomingMessage with IncomingMessageType::CallbackQuery instead")]
#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingCallbackMessage {
    pub chat_id: i64,
    pub user_id: u64,
    pub message_id: i32,
    pub callback_data: String,
    pub callback_query_id: String,
}

#[deprecated(note = "Use OutgoingMessage with OutgoingMessageType::TextMessage instead")]
#[derive(Serialize, Deserialize, Debug)]
pub struct OutgoingKafkaMessage {
    pub chat_id: i64,
    pub text: String,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
}

// Helper implementations
impl IncomingMessage {
    pub fn new_telegram_message(
        message: TelegramMessage,
        downloaded_images: Vec<ImageInfo>,
        bot_id: Option<u64>,
        bot_username: Option<String>,
    ) -> Self {
        Self {
            message_type: IncomingMessageType::TelegramMessage(TelegramMessageData {
                message,
                downloaded_images,
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

impl OutgoingMessage {
    pub fn new_text_message(
        chat_id: i64,
        text: String,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::TextMessage(TextMessageData {
                text,
                buttons,
                parse_mode: None,
                disable_web_page_preview: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    pub fn new_image_message(
        chat_id: i64,
        image_path: String,
        caption: Option<String>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::ImageMessage(ImageMessageData {
                image_path,
                caption,
                buttons,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    pub fn new_edit_message(
        chat_id: i64,
        message_id: i32,
        new_text: Option<String>,
        new_buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::EditMessage(EditMessageData {
                message_id,
                new_text,
                new_buttons,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    pub fn new_delete_message(chat_id: i64, message_id: i32) -> Self {
        Self {
            message_type: OutgoingMessageType::DeleteMessage(DeleteMessageData { message_id }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }
}