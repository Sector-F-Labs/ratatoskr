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

// Helper implementations
impl OutgoingMessage {
    #[allow(dead_code)]
    pub fn new_text_message(
        chat_id: i64,
        text: String,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::TextMessage(TextMessageData {
                text,
                buttons,
                reply_keyboard: None,
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

    #[allow(dead_code)]
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
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn new_audio_message(
        chat_id: i64,
        audio_path: String,
        caption: Option<String>,
        duration: Option<u32>,
        performer: Option<String>,
        title: Option<String>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::AudioMessage(AudioMessageData {
                audio_path,
                caption,
                duration,
                performer,
                title,
                buttons,
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn new_voice_message(
        chat_id: i64,
        voice_path: String,
        caption: Option<String>,
        duration: Option<u32>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::VoiceMessage(VoiceMessageData {
                voice_path,
                caption,
                duration,
                buttons,
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn new_video_message(
        chat_id: i64,
        video_path: String,
        caption: Option<String>,
        duration: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
        supports_streaming: Option<bool>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::VideoMessage(VideoMessageData {
                video_path,
                caption,
                duration,
                width,
                height,
                supports_streaming,
                buttons,
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn new_video_note_message(
        chat_id: i64,
        video_note_path: String,
        duration: Option<u32>,
        length: Option<u32>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::VideoNoteMessage(VideoNoteMessageData {
                video_note_path,
                duration,
                length,
                buttons,
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn new_sticker_message(
        chat_id: i64,
        sticker_path: String,
        emoji: Option<String>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::StickerMessage(StickerMessageData {
                sticker_path,
                emoji,
                buttons,
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn new_animation_message(
        chat_id: i64,
        animation_path: String,
        caption: Option<String>,
        duration: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::AnimationMessage(AnimationMessageData {
                animation_path,
                caption,
                duration,
                width,
                height,
                buttons,
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn new_document_message(
        chat_id: i64,
        document_path: String,
        filename: Option<String>,
        caption: Option<String>,
        buttons: Option<Vec<Vec<ButtonInfo>>>,
    ) -> Self {
        Self {
            message_type: OutgoingMessageType::DocumentMessage(DocumentMessageData {
                document_path,
                filename,
                caption,
                buttons,
                reply_keyboard: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id,
                thread_id: None,
            },
        }
    }
}

// Helper implementations for reply keyboards
impl ReplyKeyboardButton {
    #[allow(dead_code)]
    pub fn new_text(text: String) -> Self {
        Self {
            text,
            request_contact: None,
            request_location: None,
            request_poll: None,
            web_app: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_location(text: String) -> Self {
        Self {
            text,
            request_contact: None,
            request_location: Some(true),
            request_poll: None,
            web_app: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_contact(text: String) -> Self {
        Self {
            text,
            request_contact: Some(true),
            request_location: None,
            request_poll: None,
            web_app: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_poll(text: String, poll_type: Option<String>) -> Self {
        Self {
            text,
            request_contact: None,
            request_location: None,
            request_poll: Some(RequestPoll { poll_type }),
            web_app: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_web_app(text: String, url: String) -> Self {
        Self {
            text,
            request_contact: None,
            request_location: None,
            request_poll: None,
            web_app: Some(WebApp { url }),
        }
    }
}

impl ReplyKeyboardMarkup {
    #[allow(dead_code)]
    pub fn new(keyboard: Vec<Vec<ReplyKeyboardButton>>) -> Self {
        Self {
            keyboard,
            is_persistent: None,
            resize_keyboard: Some(true),
            one_time_keyboard: None,
            input_field_placeholder: None,
            selective: None,
        }
    }

    #[allow(dead_code)]
    pub fn one_time(mut self) -> Self {
        self.one_time_keyboard = Some(true);
        self
    }

    #[allow(dead_code)]
    pub fn persistent(mut self) -> Self {
        self.is_persistent = Some(true);
        self
    }

    #[allow(dead_code)]
    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.input_field_placeholder = Some(placeholder);
        self
    }

    #[allow(dead_code)]
    pub fn selective(mut self) -> Self {
        self.selective = Some(true);
        self
    }
}

