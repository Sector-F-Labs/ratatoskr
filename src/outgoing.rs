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
pub struct AudioMessageData {
    pub audio_path: String,
    pub caption: Option<String>,
    pub duration: Option<u32>,
    pub performer: Option<String>,
    pub title: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoiceMessageData {
    pub voice_path: String,
    pub caption: Option<String>,
    pub duration: Option<u32>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoNoteMessageData {
    pub video_note_path: String,
    pub duration: Option<u32>,
    pub length: Option<u32>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StickerMessageData {
    pub sticker_path: String,
    pub emoji: Option<String>,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationMessageData {
    pub animation_path: String,
    pub caption: Option<String>,
    pub duration: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
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

