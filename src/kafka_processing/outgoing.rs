use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unified outgoing message type for the OUT topic
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutgoingMessage {
    #[serde(default = "generate_trace_id")]
    pub trace_id: Uuid,
    pub message_type: OutgoingMessageType,
    pub timestamp: DateTime<Utc>,
    pub target: MessageTarget,
}

/// Generate a new trace ID for messages that don't have one
fn generate_trace_id() -> Uuid {
    Uuid::new_v4()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[allow(clippy::enum_variant_names)]
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
    TypingMessage(TypingMessageData),
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
pub struct TypingMessageData {
    pub action: Option<String>, // e.g., "typing"
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

// Constants for button layout
const INLINE_BUTTON_TEXT_LENGTH: usize = 26;

#[allow(dead_code)]
const REPLY_KEYBOARD_BUTTON_TEXT_LENGTH: usize = 20; // Typically shorter for reply keyboards

impl ButtonInfo {
    /// Create buttons organized into rows based on text length
    pub fn create_inline_keyboard(buttons: Vec<ButtonInfo>) -> Vec<Vec<ButtonInfo>> {
        let mut keyboard = Vec::new();
        let mut current_row = Vec::new();
        let mut current_line_length = 0;

        for button in buttons {
            if current_line_length + button.text.len() > INLINE_BUTTON_TEXT_LENGTH
                && !current_row.is_empty()
            {
                keyboard.push(current_row);
                current_row = Vec::new();
                current_line_length = 0;
            }
            current_line_length += button.text.len();
            current_row.push(button);
        }

        // Don't forget the last row
        if !current_row.is_empty() {
            keyboard.push(current_row);
        }

        keyboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_button_organization_short_buttons() {
        let buttons = vec![
            ButtonInfo {
                text: "A".to_string(),
                callback_data: "a".to_string(),
            },
            ButtonInfo {
                text: "B".to_string(),
                callback_data: "b".to_string(),
            },
            ButtonInfo {
                text: "C".to_string(),
                callback_data: "c".to_string(),
            },
            ButtonInfo {
                text: "D".to_string(),
                callback_data: "d".to_string(),
            },
        ];

        let organized = ButtonInfo::create_inline_keyboard(buttons);

        // All short buttons should fit on one row (4 chars total)
        assert_eq!(organized.len(), 1);
        assert_eq!(organized[0].len(), 4);
        assert_eq!(organized[0][0].text, "A");
        assert_eq!(organized[0][1].text, "B");
        assert_eq!(organized[0][2].text, "C");
        assert_eq!(organized[0][3].text, "D");
    }

    #[test]
    fn test_inline_button_organization_mixed_lengths() {
        let buttons = vec![
            ButtonInfo {
                text: "Short".to_string(), // 5 chars
                callback_data: "short".to_string(),
            },
            ButtonInfo {
                text: "Medium Length".to_string(), // 13 chars
                callback_data: "medium".to_string(),
            },
            ButtonInfo {
                text: "Very Long Button Text".to_string(), // 21 chars
                callback_data: "long".to_string(),
            },
            ButtonInfo {
                text: "X".to_string(), // 1 char
                callback_data: "x".to_string(),
            },
        ];

        let organized = ButtonInfo::create_inline_keyboard(buttons);

        // Should create multiple rows based on 26 char limit
        // Row 1: "Short" + "Medium Length" = 18 chars - fits
        // But "Very Long Button Text" = 21 chars, so 18 + 21 = 39 > 26, so it goes to Row 2
        // Row 2: "Very Long Button Text" = 21 chars - fits
        // "X" = 1 char, so 21 + 1 = 22 <= 26, so it stays on Row 2
        assert_eq!(organized.len(), 2);

        assert_eq!(organized[0].len(), 2);
        assert_eq!(organized[0][0].text, "Short");
        assert_eq!(organized[0][1].text, "Medium Length");

        assert_eq!(organized[1].len(), 2);
        assert_eq!(organized[1][0].text, "Very Long Button Text");
        assert_eq!(organized[1][1].text, "X");
    }

    #[test]
    fn test_inline_button_organization_single_long_button() {
        let buttons = vec![ButtonInfo {
            text: "This is an extremely long button text that exceeds the limit".to_string(), // 61 chars
            callback_data: "very_long".to_string(),
        }];

        let organized = ButtonInfo::create_inline_keyboard(buttons);

        // Single long button should be on its own row
        assert_eq!(organized.len(), 1);
        assert_eq!(organized[0].len(), 1);
        assert_eq!(
            organized[0][0].text,
            "This is an extremely long button text that exceeds the limit"
        );
    }

    pub fn create_reply_keyboard(
        buttons: Vec<ReplyKeyboardButton>,
    ) -> Vec<Vec<ReplyKeyboardButton>> {
        let mut keyboard = Vec::new();
        let mut current_row = Vec::new();
        let mut current_line_length = 0;

        for button in buttons {
            if current_line_length + button.text.len() > REPLY_KEYBOARD_BUTTON_TEXT_LENGTH {
                if !current_row.is_empty() {
                    keyboard.push(current_row);
                    current_row = Vec::new();
                    current_line_length = 0;
                }
            }
            current_line_length += button.text.len();
            current_row.push(button);
        }

        // Don't forget the last row
        if !current_row.is_empty() {
            keyboard.push(current_row);
        }

        keyboard
    }

    #[test]
    fn test_inline_button_organization_many_small_buttons() {
        let buttons = vec![
            ButtonInfo {
                text: "1".to_string(),
                callback_data: "1".to_string(),
            },
            ButtonInfo {
                text: "2".to_string(),
                callback_data: "2".to_string(),
            },
            ButtonInfo {
                text: "3".to_string(),
                callback_data: "3".to_string(),
            },
            ButtonInfo {
                text: "4".to_string(),
                callback_data: "4".to_string(),
            },
            ButtonInfo {
                text: "5".to_string(),
                callback_data: "5".to_string(),
            },
            ButtonInfo {
                text: "6".to_string(),
                callback_data: "6".to_string(),
            },
            ButtonInfo {
                text: "7".to_string(),
                callback_data: "7".to_string(),
            },
            ButtonInfo {
                text: "8".to_string(),
                callback_data: "8".to_string(),
            },
            ButtonInfo {
                text: "9".to_string(),
                callback_data: "9".to_string(),
            },
            ButtonInfo {
                text: "10".to_string(),
                callback_data: "10".to_string(),
            },
            ButtonInfo {
                text: "11".to_string(),
                callback_data: "11".to_string(),
            },
            ButtonInfo {
                text: "12".to_string(),
                callback_data: "12".to_string(),
            },
        ];

        let organized = ButtonInfo::create_inline_keyboard(buttons);

        // Should efficiently pack small buttons
        // Each single digit is 1 char, double digits are 2 chars
        // Should be able to fit many per row within 26 char limit
        assert!(organized.len() >= 1);

        // Verify all buttons are present
        let total_buttons: usize = organized.iter().map(|row| row.len()).sum();
        assert_eq!(total_buttons, 12);

        // First row should have many single-digit buttons
        assert!(organized[0].len() > 1);
    }

    #[test]
    fn test_inline_button_organization_exact_limit() {
        let buttons = vec![
            ButtonInfo {
                text: "Exactly26Characters Here".to_string(), // 24 chars
                callback_data: "exact26".to_string(),
            },
            ButtonInfo {
                text: "OneMore".to_string(), // 7 chars
                callback_data: "onemore".to_string(),
            },
        ];

        let organized = ButtonInfo::create_inline_keyboard(buttons);

        // 24 + 7 = 31 > 26, so should split into two rows
        assert_eq!(organized.len(), 2);
        assert_eq!(organized[0].len(), 1);
        assert_eq!(organized[0][0].text, "Exactly26Characters Here");
        assert_eq!(organized[1].len(), 1);
        assert_eq!(organized[1][0].text, "OneMore");
    }

    #[test]
    fn test_reply_keyboard_organization() {
        let buttons = vec![
            ReplyKeyboardButton {
                text: "Short".to_string(), // 5 chars
                request_contact: None,
                request_location: None,
                request_poll: None,
                web_app: None,
            },
            ReplyKeyboardButton {
                text: "Medium Length Button".to_string(), // 20 chars
                request_contact: None,
                request_location: None,
                request_poll: None,
                web_app: None,
            },
            ReplyKeyboardButton {
                text: "X".to_string(), // 1 char
                request_contact: None,
                request_location: None,
                request_poll: None,
                web_app: None,
            },
        ];

        let organized = create_reply_keyboard(buttons);

        // Should organize based on 20 char limit for reply keyboards
        // Row 1: "Short" = 5 chars
        // Row 2: "Medium Length Button" = 20 chars (exactly at limit)
        // Row 3: "X" = 1 char
        assert_eq!(organized.len(), 3);
        assert_eq!(organized[0].len(), 1);
        assert_eq!(organized[0][0].text, "Short");
        assert_eq!(organized[1].len(), 1);
        assert_eq!(organized[1][0].text, "Medium Length Button");
        assert_eq!(organized[2].len(), 1);
        assert_eq!(organized[2][0].text, "X");
    }

    #[test]
    fn test_empty_button_lists() {
        let empty_inline: Vec<ButtonInfo> = vec![];
        let organized_inline = ButtonInfo::create_inline_keyboard(empty_inline);
        assert_eq!(organized_inline.len(), 0);

        let empty_reply: Vec<ReplyKeyboardButton> = vec![];
        let organized_reply = create_reply_keyboard(empty_reply);
        assert_eq!(organized_reply.len(), 0);
    }
}
