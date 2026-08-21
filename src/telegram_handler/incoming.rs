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
    /// Confirms an OutgoingMessage was actually delivered, carrying the real
    /// Telegram message_id Telegram assigned it - consumers publish
    /// OutgoingMessages without ever learning that id otherwise, since
    /// sending happens entirely on this side. Delivered on the IN topic
    /// (not a separate one) because it flows in the same direction as
    /// everything else there (ratatoskr -> consumer), and the trace_id here
    /// is the *outgoing* message's own trace_id (not a fresh one), so a
    /// consumer can correlate this back to whichever message it sent.
    MessageSent(MessageSentData),
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
pub struct MessageSentData {
    pub chat_id: i64,
    pub message_id: i32,
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

    #[allow(clippy::too_many_arguments)]
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

    /// Unlike the other constructors, `trace_id` is taken as a parameter
    /// rather than freshly generated - it must be the *outgoing* message's
    /// own trace_id, so whoever published that OutgoingMessage can correlate
    /// this delivery confirmation back to it.
    pub fn new_message_sent(trace_id: Uuid, chat_id: i64, message_id: i32) -> Self {
        Self {
            trace_id,
            message_type: IncomingMessageType::MessageSent(MessageSentData { chat_id, message_id }),
            timestamp: Utc::now(),
            source: MessageSource {
                platform: "telegram".to_string(),
                bot_id: None,
                bot_username: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_callback_query_wires_fields_and_generates_trace_id() {
        let msg = IncomingMessage::new_callback_query(
            42,
            7,
            99,
            "payload".to_string(),
            "query-id".to_string(),
            Some(1),
            Some("mybot".to_string()),
        );

        assert!(!msg.trace_id.is_nil());
        assert_eq!(msg.source.platform, "telegram");
        assert_eq!(msg.source.bot_id, Some(1));
        assert_eq!(msg.source.bot_username.as_deref(), Some("mybot"));
        match msg.message_type {
            IncomingMessageType::CallbackQuery(data) => {
                assert_eq!(data.chat_id, 42);
                assert_eq!(data.user_id, 7);
                assert_eq!(data.message_id, 99);
                assert_eq!(data.callback_data, "payload");
                assert_eq!(data.callback_query_id, "query-id");
            }
            other => panic!("expected CallbackQuery, got {other:?}"),
        }
    }

    #[test]
    fn new_message_reaction_wires_fields() {
        let now = Utc::now();
        let msg = IncomingMessage::new_message_reaction(
            1,
            2,
            Some(3),
            now,
            vec!["👍".to_string()],
            vec!["🔥".to_string()],
            None,
            None,
        );

        match msg.message_type {
            IncomingMessageType::MessageReaction(data) => {
                assert_eq!(data.chat_id, 1);
                assert_eq!(data.message_id, 2);
                assert_eq!(data.user_id, Some(3));
                assert_eq!(data.date, now);
                assert_eq!(data.old_reaction, vec!["👍".to_string()]);
                assert_eq!(data.new_reaction, vec!["🔥".to_string()]);
            }
            other => panic!("expected MessageReaction, got {other:?}"),
        }
    }

    /// `new_message_sent` is the one constructor that takes `trace_id` as a
    /// parameter rather than generating a fresh one - it must correlate back
    /// to the outgoing message's own trace_id.
    #[test]
    fn new_message_sent_preserves_given_trace_id() {
        let trace_id = Uuid::new_v4();
        let msg = IncomingMessage::new_message_sent(trace_id, 555, 123);

        assert_eq!(msg.trace_id, trace_id);
        match msg.message_type {
            IncomingMessageType::MessageSent(data) => {
                assert_eq!(data.chat_id, 555);
                assert_eq!(data.message_id, 123);
            }
            other => panic!("expected MessageSent, got {other:?}"),
        }
    }

    /// Downstream Kafka consumers depend on the `{"type": ..., "data": ...}`
    /// tagging shape - this asserts it survives a serialize/deserialize round
    /// trip for every variant.
    #[test]
    fn message_type_serde_round_trip_preserves_tag_shape() {
        let cases: Vec<(&str, IncomingMessage)> = vec![
            (
                "CallbackQuery",
                IncomingMessage::new_callback_query(
                    1,
                    2,
                    3,
                    "data".to_string(),
                    "qid".to_string(),
                    None,
                    None,
                ),
            ),
            (
                "MessageReaction",
                IncomingMessage::new_message_reaction(
                    1,
                    2,
                    None,
                    Utc::now(),
                    vec![],
                    vec![],
                    None,
                    None,
                ),
            ),
            (
                "MessageSent",
                IncomingMessage::new_message_sent(Uuid::new_v4(), 1, 2),
            ),
        ];

        for (tag, original) in cases {
            let value = serde_json::to_value(&original).unwrap();
            assert_eq!(
                value["message_type"]["type"], tag,
                "unexpected tag for {tag}"
            );
            assert!(
                value["message_type"]["data"].is_object(),
                "missing data object for {tag}"
            );

            let round_tripped: IncomingMessage = serde_json::from_value(value).unwrap();
            assert_eq!(round_tripped.trace_id, original.trace_id);
        }
    }

    #[test]
    fn file_metadata_variants_serialize_with_expected_fields() {
        let photo = FileMetadata::Photo {
            width: 100,
            height: 200,
        };
        assert_eq!(
            serde_json::to_value(&photo).unwrap(),
            json!({ "Photo": { "width": 100, "height": 200 } })
        );

        let doc = FileMetadata::Document {
            file_name: Some("report.pdf".to_string()),
            mime_type: None,
        };
        assert_eq!(
            serde_json::to_value(&doc).unwrap(),
            json!({ "Document": { "file_name": "report.pdf", "mime_type": null } })
        );
    }
}
