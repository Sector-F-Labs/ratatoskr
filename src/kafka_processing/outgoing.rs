use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    AudioMessage(AudioMessageData),
    VoiceMessage(VoiceMessageData),
    VideoMessage(VideoMessageData),
    VideoNoteMessage(VideoNoteMessageData),
    DocumentMessage(DocumentMessageData),
    StickerMessage(StickerMessageData),
    AnimationMessage(AnimationMessageData),
    EditMessage(EditMessageData),
    DeleteMessage(DeleteMessageData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessageData {
    pub text: String,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
    pub parse_mode: Option<String>, // "HTML", "Markdown", etc.
    pub disable_web_page_preview: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageMessageData {
    pub image_path: String,
    pub caption: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioMessageData {
    pub audio_path: String,
    pub caption: Option<String>,
    pub duration: Option<u32>,
    pub performer: Option<String>,
    pub title: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoiceMessageData {
    pub voice_path: String,
    pub caption: Option<String>,
    pub duration: Option<u32>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoMessageData {
    pub video_path: String,
    pub caption: Option<String>,
    pub duration: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub supports_streaming: Option<bool>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoNoteMessageData {
    pub video_note_path: String,
    pub duration: Option<u32>,
    pub length: Option<u32>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StickerMessageData {
    pub sticker_path: String,
    pub emoji: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationMessageData {
    pub animation_path: String,
    pub caption: Option<String>,
    pub duration: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentMessageData {
    pub document_path: String,
    pub filename: Option<String>,
    pub caption: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
    pub reply_keyboard: Option<ReplyKeyboardMarkup>,
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
pub struct ReplyKeyboardButton {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_contact: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_location: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_poll: Option<RequestPoll>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_app: Option<WebApp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestPoll {
    #[serde(rename = "type")]
    pub poll_type: Option<String>, // "quiz" or "regular"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebApp {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<ReplyKeyboardButton>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_persistent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resize_keyboard: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_time_keyboard: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_field_placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selective: Option<bool>,
}
