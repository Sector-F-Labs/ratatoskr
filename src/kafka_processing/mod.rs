use self::outgoing::{ButtonInfo, OutgoingMessage, OutgoingMessageType};
use crate::broker::MessageBroker;
use crate::telegram_handler::incoming::IncomingMessage;
use crate::utils::{create_markup, create_reply_keyboard, format_telegram_markdown};
use futures_util::StreamExt;
use std::path::Path;
use std::sync::Arc;
use teloxide::{
    payloads::{
        EditMessageReplyMarkupSetters, EditMessageTextSetters, SendAnimationSetters,
        SendAudioSetters, SendDocumentSetters, SendMessageSetters, SendPhotoSetters,
        SendStickerSetters, SendVideoNoteSetters, SendVideoSetters, SendVoiceSetters,
    },
    prelude::{Bot, ChatId, Requester},
    types::{InputFile, ParseMode},
};
use tracing::Instrument;
use uuid::Uuid;

/// Publishes a `MessageSent` confirmation on the IN topic so whoever
/// produced this `OutgoingMessage` can learn the real Telegram `message_id`
/// it was actually sent as - not otherwise knowable, since sending happens
/// entirely on this side. Not fatal if this fails: the message itself was
/// already delivered successfully, only the confirmation is lost.
async fn report_message_sent(broker: &Arc<dyn MessageBroker>, trace_id: Uuid, chat_id: i64, message_id: i32) {
    let incoming = IncomingMessage::new_message_sent(trace_id, chat_id, message_id);
    match serde_json::to_string(&incoming) {
        Ok(json) => {
            if let Err(e) = broker.publish(None, json.as_bytes()).await {
                tracing::warn!(%trace_id, error = %e, "failed to publish MessageSent confirmation");
            }
        }
        Err(e) => tracing::warn!(%trace_id, error = %e, "failed to serialize MessageSent confirmation"),
    }
}

pub mod outgoing;

/// Simple helper to try sending with markdown, falling back to plain text if it fails
async fn try_send_with_fallback<T, F, Fut>(
    markdown_result: Result<T, teloxide::RequestError>,
    fallback_fn: F,
    message_type: &str,
) -> Result<T, teloxide::RequestError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, teloxide::RequestError>>,
{
    match markdown_result {
        Ok(result) => Ok(result),
        Err(_) => {
            tracing::warn!(
                "Failed to send {} with formatting, retrying with plain text",
                message_type
            );
            fallback_fn().await
        }
    }
}

/// If a single row holds more than one button, re-flow it across rows sized
/// to fit Telegram's inline keyboard width; otherwise keep the caller's
/// row layout as-is.
fn organize_buttons_for_send(
    buttons: &Option<Vec<Vec<ButtonInfo>>>,
) -> Option<Vec<Vec<ButtonInfo>>> {
    buttons.as_ref().map(|buttons| {
        if buttons.len() == 1 && buttons[0].len() > 1 {
            tracing::info!(original_buttons = %buttons[0].len(), "Auto-organizing buttons based on text length");
            self::outgoing::ButtonInfo::create_inline_keyboard(buttons[0].clone())
        } else {
            buttons.clone()
        }
    })
}

async fn handle_outgoing_message(
    bot: &Bot,
    broker: &Arc<dyn MessageBroker>,
    message: OutgoingMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = ChatId(message.target.chat_id);
    let trace_id = message.trace_id;

    match message.message_type {
        OutgoingMessageType::TextMessage(data) => {
            tracing::info!(text_length = %data.text.len(), has_buttons = %data.buttons.is_some(), "Sending text message to Telegram");

            // Auto-organize buttons if they exist
            let organized_buttons = organize_buttons_for_send(&data.buttons);

            // Try with markdown first, fallback to plain text if parsing fails
            if data.parse_mode.is_some() {
                let formatted_text = format_telegram_markdown(&data.text);
                tracing::debug!(
                    original_length = %data.text.len(),
                    formatted_length = %formatted_text.len(),
                    "Formatted text for sending"
                );
                tracing::trace!(original_text = %data.text, formatted_text = %formatted_text, "Text formatting details");
                let mut msg_to_send = bot.send_message(chat_id, formatted_text);

                if let Some(parse_mode) = &data.parse_mode {
                    msg_to_send = match parse_mode.as_str() {
                        "HTML" => msg_to_send.parse_mode(ParseMode::Html),
                        "Markdown" => msg_to_send.parse_mode(ParseMode::Html), // Convert markdown to HTML
                        _ => msg_to_send,
                    };
                }

                if let Some(markup) = create_markup(&organized_buttons) {
                    msg_to_send = msg_to_send.reply_markup(markup);
                }

                if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                    msg_to_send = msg_to_send.reply_markup(reply_keyboard);
                }

                let sent = try_send_with_fallback(
                    msg_to_send.await,
                    || async {
                        let mut plain_msg = bot.send_message(chat_id, &data.text);
                        if let Some(markup) = create_markup(&organized_buttons) {
                            plain_msg = plain_msg.reply_markup(markup);
                        }
                        if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                            plain_msg = plain_msg.reply_markup(reply_keyboard);
                        }
                        plain_msg.await
                    },
                    "text message",
                )
                .await?;
                report_message_sent(broker, trace_id, chat_id.0, sent.id.0).await;
            } else {
                // No parse mode, send as plain text
                let mut msg_to_send = bot.send_message(chat_id, &data.text);
                if let Some(markup) = create_markup(&organized_buttons) {
                    msg_to_send = msg_to_send.reply_markup(markup);
                }
                if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                    msg_to_send = msg_to_send.reply_markup(reply_keyboard);
                }
                let sent = msg_to_send.await?;
                report_message_sent(broker, trace_id, chat_id.0, sent.id.0).await;
            }
        }

        OutgoingMessageType::ImageMessage(data) => {
            tracing::info!(image_path = %data.image_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending image message to Telegram");

            if !Path::new(&data.image_path).exists() {
                return Err(format!("Image file not found: {}", data.image_path).into());
            }

            let input_file = InputFile::file(&data.image_path);

            // Try with markdown caption first, fallback to plain text if parsing fails
            if let Some(caption) = &data.caption {
                let formatted_caption = format_telegram_markdown(caption);
                let mut msg_to_send = bot
                    .send_photo(chat_id, input_file.clone())
                    .caption(formatted_caption)
                    .parse_mode(ParseMode::Html);

                if let Some(markup) = create_markup(&data.buttons) {
                    msg_to_send = msg_to_send.reply_markup(markup);
                }

                if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                    msg_to_send = msg_to_send.reply_markup(reply_keyboard);
                }

                try_send_with_fallback(
                    msg_to_send.await,
                    || async {
                        let mut plain_msg =
                            bot.send_photo(chat_id, input_file.clone()).caption(caption);
                        if let Some(markup) = create_markup(&data.buttons) {
                            plain_msg = plain_msg.reply_markup(markup);
                        }
                        if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                            plain_msg = plain_msg.reply_markup(reply_keyboard);
                        }
                        plain_msg.await
                    },
                    "image message",
                )
                .await?;
            } else {
                // No caption, send without formatting
                let mut msg_to_send = bot.send_photo(chat_id, input_file);

                if let Some(markup) = create_markup(&data.buttons) {
                    msg_to_send = msg_to_send.reply_markup(markup);
                }

                if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                    msg_to_send = msg_to_send.reply_markup(reply_keyboard);
                }

                msg_to_send.await?;
            }
        }

        OutgoingMessageType::AudioMessage(data) => {
            tracing::info!(audio_path = %data.audio_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending audio message to Telegram");

            if !Path::new(&data.audio_path).exists() {
                return Err(format!("Audio file not found: {}", data.audio_path).into());
            }

            let input_file = InputFile::file(&data.audio_path);
            let mut msg_to_send = bot.send_audio(chat_id, input_file);

            if let Some(caption) = data.caption {
                let formatted_caption = format_telegram_markdown(&caption);
                msg_to_send = msg_to_send.caption(formatted_caption);
            }

            if let Some(duration) = data.duration {
                msg_to_send = msg_to_send.duration(duration);
            }

            if let Some(performer) = data.performer {
                msg_to_send = msg_to_send.performer(performer);
            }

            if let Some(title) = data.title {
                msg_to_send = msg_to_send.title(title);
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::VoiceMessage(data) => {
            tracing::info!(voice_path = %data.voice_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending voice message to Telegram");

            if !Path::new(&data.voice_path).exists() {
                return Err(format!("Voice file not found: {}", data.voice_path).into());
            }

            let input_file = InputFile::file(&data.voice_path);
            let mut msg_to_send = bot.send_voice(chat_id, input_file);

            if let Some(caption) = data.caption {
                let formatted_caption = format_telegram_markdown(&caption);
                msg_to_send = msg_to_send.caption(formatted_caption);
            }

            if let Some(duration) = data.duration {
                msg_to_send = msg_to_send.duration(duration);
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::VideoMessage(data) => {
            tracing::info!(video_path = %data.video_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending video message to Telegram");

            if !Path::new(&data.video_path).exists() {
                return Err(format!("Video file not found: {}", data.video_path).into());
            }

            let input_file = InputFile::file(&data.video_path);
            let mut msg_to_send = bot.send_video(chat_id, input_file);

            if let Some(caption) = data.caption {
                let formatted_caption = format_telegram_markdown(&caption);
                msg_to_send = msg_to_send.caption(formatted_caption);
            }

            if let Some(duration) = data.duration {
                msg_to_send = msg_to_send.duration(duration);
            }

            if let Some(width) = data.width {
                msg_to_send = msg_to_send.width(width);
            }

            if let Some(height) = data.height {
                msg_to_send = msg_to_send.height(height);
            }

            if let Some(supports_streaming) = data.supports_streaming {
                msg_to_send = msg_to_send.supports_streaming(supports_streaming);
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::VideoNoteMessage(data) => {
            tracing::info!(video_note_path = %data.video_note_path, has_buttons = %data.buttons.is_some(), "Sending video note message to Telegram");

            if !Path::new(&data.video_note_path).exists() {
                return Err(format!("Video note file not found: {}", data.video_note_path).into());
            }

            let input_file = InputFile::file(&data.video_note_path);
            let mut msg_to_send = bot.send_video_note(chat_id, input_file);

            if let Some(duration) = data.duration {
                msg_to_send = msg_to_send.duration(duration);
            }

            if let Some(length) = data.length {
                msg_to_send = msg_to_send.length(length);
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::StickerMessage(data) => {
            tracing::info!(sticker_path = %data.sticker_path, has_buttons = %data.buttons.is_some(), "Sending sticker message to Telegram");

            if !Path::new(&data.sticker_path).exists() {
                return Err(format!("Sticker file not found: {}", data.sticker_path).into());
            }

            let input_file = InputFile::file(&data.sticker_path);
            let mut msg_to_send = bot.send_sticker(chat_id, input_file);

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::AnimationMessage(data) => {
            tracing::info!(animation_path = %data.animation_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending animation message to Telegram");

            if !Path::new(&data.animation_path).exists() {
                return Err(format!("Animation file not found: {}", data.animation_path).into());
            }

            let input_file = InputFile::file(&data.animation_path);
            let mut msg_to_send = bot.send_animation(chat_id, input_file);

            if let Some(caption) = data.caption {
                let formatted_caption = format_telegram_markdown(&caption);
                msg_to_send = msg_to_send.caption(formatted_caption);
            }

            if let Some(duration) = data.duration {
                msg_to_send = msg_to_send.duration(duration);
            }

            if let Some(width) = data.width {
                msg_to_send = msg_to_send.width(width);
            }

            if let Some(height) = data.height {
                msg_to_send = msg_to_send.height(height);
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::DocumentMessage(data) => {
            tracing::info!(document_path = %data.document_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending document message to Telegram");

            if !Path::new(&data.document_path).exists() {
                return Err(format!("Document file not found: {}", data.document_path).into());
            }

            let input_file = if let Some(filename) = &data.filename {
                InputFile::file(&data.document_path).file_name(filename.clone())
            } else {
                InputFile::file(&data.document_path)
            };

            let mut msg_to_send = bot.send_document(chat_id, input_file);

            if let Some(caption) = data.caption {
                let formatted_caption = format_telegram_markdown(&caption);
                msg_to_send = msg_to_send.caption(formatted_caption);
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::EditMessage(data) => {
            tracing::info!(message_id = %data.message_id, has_new_text = %data.new_text.is_some(), has_new_buttons = %data.new_buttons.is_some(), "Editing message in Telegram");

            if let Some(new_text) = data.new_text {
                let formatted_text = format_telegram_markdown(&new_text);
                let mut msg_to_edit = bot
                    .edit_message_text(
                        chat_id,
                        teloxide::types::MessageId(data.message_id),
                        formatted_text,
                    )
                    .parse_mode(ParseMode::MarkdownV2);

                if let Some(markup) = create_markup(&data.new_buttons) {
                    msg_to_edit = msg_to_edit.reply_markup(markup);
                }

                try_send_with_fallback(
                    msg_to_edit.await,
                    || async {
                        let mut plain_edit = bot.edit_message_text(
                            chat_id,
                            teloxide::types::MessageId(data.message_id),
                            &new_text,
                        );
                        if let Some(markup) = create_markup(&data.new_buttons) {
                            plain_edit = plain_edit.reply_markup(markup);
                        }
                        plain_edit.await
                    },
                    "edit message",
                )
                .await?;
            } else if data.new_buttons.is_some() {
                // Edit only buttons if no new text is provided
                if let Some(markup) = create_markup(&data.new_buttons) {
                    bot.edit_message_reply_markup(
                        chat_id,
                        teloxide::types::MessageId(data.message_id),
                    )
                    .reply_markup(markup)
                    .await?;
                }
            }
        }

        OutgoingMessageType::DeleteMessage(data) => {
            tracing::info!(message_id = %data.message_id, "Deleting message in Telegram");
            bot.delete_message(chat_id, teloxide::types::MessageId(data.message_id))
                .await?;
        }

        OutgoingMessageType::TypingMessage(_data) => {
            tracing::info!("Sending typing action to Telegram");
            bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
                .await?;
        }
    }

    Ok(())
}

pub async fn start_broker_consumer_loop(bot_consumer_clone: Bot, broker: Arc<dyn MessageBroker>) {
    tracing::info!("Starting broker consumer stream for Telegram output...");
    let mut stream = match broker.subscribe().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to subscribe to broker stream");
            return;
        }
    };
    while let Some(payload) = stream.next().await {
        match serde_json::from_slice::<OutgoingMessage>(&payload) {
            Ok(out_msg) => {
                if out_msg.trace_id.is_nil() {
                    tracing::warn!(
                        "Generated new trace ID for message without one: {}",
                        out_msg.trace_id
                    );
                    // early return
                    return;
                }

                let span = tracing::info_span!(
                    "handle_outgoing_message",
                    trace_id = %out_msg.trace_id,
                    chat_id = %out_msg.target.chat_id,
                    message_type = ?std::mem::discriminant(&out_msg.message_type)
                );

                if let Err(e) = handle_outgoing_message(&bot_consumer_clone, &broker, out_msg)
                    .instrument(span)
                    .await
                {
                    tracing::error!(error = ?e, "Error handling OutgoingMessage");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Error deserializing message from broker payload");
                tracing::debug!(raw_payload = ?String::from_utf8_lossy(&payload), "Problematic broker payload");
            }
        }
    }
    tracing::warn!("Broker consumer stream ended.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::test_support::MockMessageBroker;
    use crate::kafka_processing::outgoing::{
        AnimationMessageData, AudioMessageData, DeleteMessageData, DocumentMessageData,
        EditMessageData, ImageMessageData, MessageTarget, StickerMessageData, TextMessageData,
        TypingMessageData, VideoMessageData, VideoNoteMessageData, VoiceMessageData,
    };
    use std::cell::Cell;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_message(message_type: OutgoingMessageType) -> OutgoingMessage {
        OutgoingMessage {
            trace_id: Uuid::new_v4(),
            message_type,
            timestamp: chrono::Utc::now(),
            target: MessageTarget {
                platform: "telegram".to_string(),
                chat_id: 42,
                thread_id: None,
            },
        }
    }

    fn mock_broker() -> Arc<dyn MessageBroker> {
        Arc::new(MockMessageBroker::new())
    }

    fn test_bot(api_url: &str) -> Bot {
        Bot::new("test_token").set_api_url(reqwest::Url::parse(api_url).unwrap())
    }

    // --- organize_buttons_for_send ---

    #[test]
    fn organize_buttons_none_stays_none() {
        assert!(organize_buttons_for_send(&None).is_none());
    }

    #[test]
    fn organize_buttons_preserves_multi_row_layout() {
        let buttons = vec![
            vec![ButtonInfo {
                text: "A".into(),
                callback_data: "a".into(),
            }],
            vec![ButtonInfo {
                text: "B".into(),
                callback_data: "b".into(),
            }],
        ];
        let result = organize_buttons_for_send(&Some(buttons)).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn organize_buttons_reflows_single_overlong_row() {
        // Same fixture as outgoing::tests::test_inline_button_organization_exact_limit,
        // which documents that 24 + 7 = 31 chars splits into two rows.
        let buttons = vec![vec![
            ButtonInfo {
                text: "Exactly26Characters Here".into(),
                callback_data: "a".into(),
            },
            ButtonInfo {
                text: "OneMore".into(),
                callback_data: "b".into(),
            },
        ]];
        let result = organize_buttons_for_send(&Some(buttons)).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn organize_buttons_single_row_single_button_unchanged() {
        let buttons = vec![vec![ButtonInfo {
            text: "Only".into(),
            callback_data: "only".into(),
        }]];
        let result = organize_buttons_for_send(&Some(buttons)).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
    }

    // --- try_send_with_fallback ---

    #[tokio::test]
    async fn try_send_with_fallback_returns_ok_without_calling_fallback() {
        let fallback_called = Cell::new(false);
        let result = try_send_with_fallback(
            Ok::<_, teloxide::RequestError>(7),
            || async {
                fallback_called.set(true);
                Ok::<_, teloxide::RequestError>(0)
            },
            "test",
        )
        .await;
        assert_eq!(result.unwrap(), 7);
        assert!(!fallback_called.get());
    }

    #[tokio::test]
    async fn try_send_with_fallback_invokes_fallback_on_error() {
        let result = try_send_with_fallback(
            Err::<i32, _>(teloxide::RequestError::Api(teloxide::ApiError::BotBlocked)),
            || async { Ok::<_, teloxide::RequestError>(99) },
            "test",
        )
        .await;
        assert_eq!(result.unwrap(), 99);
    }

    // --- file-not-found early returns (no network involved) ---

    #[tokio::test]
    async fn missing_files_error_before_any_bot_call() {
        // Port 1 is never a listening HTTP server, so any real request here would
        // hang/fail loudly - these assertions only hold if the path-exists check
        // short-circuits before `Bot` is ever touched.
        let bot = test_bot("http://127.0.0.1:1");
        let broker = mock_broker();

        let messages = vec![
            OutgoingMessageType::ImageMessage(ImageMessageData {
                image_path: "/nonexistent/x.png".into(),
                caption: None,
                buttons: None,
                reply_keyboard: None,
            }),
            OutgoingMessageType::AudioMessage(AudioMessageData {
                audio_path: "/nonexistent/x.mp3".into(),
                caption: None,
                duration: None,
                performer: None,
                title: None,
                buttons: None,
                reply_keyboard: None,
            }),
            OutgoingMessageType::VoiceMessage(VoiceMessageData {
                voice_path: "/nonexistent/x.ogg".into(),
                caption: None,
                duration: None,
                buttons: None,
                reply_keyboard: None,
            }),
            OutgoingMessageType::VideoMessage(VideoMessageData {
                video_path: "/nonexistent/x.mp4".into(),
                caption: None,
                duration: None,
                width: None,
                height: None,
                supports_streaming: None,
                buttons: None,
                reply_keyboard: None,
            }),
            OutgoingMessageType::VideoNoteMessage(VideoNoteMessageData {
                video_note_path: "/nonexistent/x.mp4".into(),
                duration: None,
                length: None,
                buttons: None,
                reply_keyboard: None,
            }),
            OutgoingMessageType::StickerMessage(StickerMessageData {
                sticker_path: "/nonexistent/x.webp".into(),
                emoji: None,
                buttons: None,
                reply_keyboard: None,
            }),
            OutgoingMessageType::AnimationMessage(AnimationMessageData {
                animation_path: "/nonexistent/x.gif".into(),
                caption: None,
                duration: None,
                width: None,
                height: None,
                buttons: None,
                reply_keyboard: None,
            }),
            OutgoingMessageType::DocumentMessage(DocumentMessageData {
                document_path: "/nonexistent/x.pdf".into(),
                filename: None,
                caption: None,
                buttons: None,
                reply_keyboard: None,
            }),
        ];

        for message_type in messages {
            let result = handle_outgoing_message(&bot, &broker, sample_message(message_type)).await;
            assert!(result.is_err());
        }
    }

    // --- HTTP-mocked: exercises the actual teloxide Bot/Requester surface ---

    #[tokio::test]
    async fn text_message_with_parse_mode_retries_plain_on_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"(?i)^/bot[^/]+/sendMessage$"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "ok": false, "error_code": 400, "description": "can't parse entities"
            })))
            .mount(&server)
            .await;

        let bot = test_bot(&server.uri());
        let broker = mock_broker();
        let msg = sample_message(OutgoingMessageType::TextMessage(TextMessageData {
            text: "*bold*".to_string(),
            buttons: None,
            reply_keyboard: None,
            parse_mode: Some("HTML".to_string()),
            disable_web_page_preview: None,
        }));

        let _ = handle_outgoing_message(&bot, &broker, msg).await;

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2, "expected a formatted attempt plus a plain-text fallback");

        let first: serde_json::Value = requests[0].body_json().unwrap();
        assert_eq!(first["parse_mode"], "HTML");

        let second: serde_json::Value = requests[1].body_json().unwrap();
        assert!(second.get("parse_mode").is_none());
        assert_eq!(second["text"], "*bold*");
    }

    #[tokio::test]
    async fn text_message_without_parse_mode_sends_plain_once() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"(?i)^/bot[^/]+/sendMessage$"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "ok": false, "error_code": 400, "description": "boom"
            })))
            .mount(&server)
            .await;

        let bot = test_bot(&server.uri());
        let broker = mock_broker();
        let msg = sample_message(OutgoingMessageType::TextMessage(TextMessageData {
            text: "hello".to_string(),
            buttons: None,
            reply_keyboard: None,
            parse_mode: None,
            disable_web_page_preview: None,
        }));

        let _ = handle_outgoing_message(&bot, &broker, msg).await;

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1, "no parse_mode means no fallback retry");
        let body: serde_json::Value = requests[0].body_json().unwrap();
        assert!(body.get("parse_mode").is_none());
        assert_eq!(body["text"], "hello");
    }

    #[tokio::test]
    async fn edit_message_text_retries_plain_on_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"(?i)^/bot[^/]+/editMessageText$"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "ok": false, "error_code": 400, "description": "boom"
            })))
            .mount(&server)
            .await;

        let bot = test_bot(&server.uri());
        let broker = mock_broker();
        let msg = sample_message(OutgoingMessageType::EditMessage(EditMessageData {
            message_id: 10,
            new_text: Some("*bold*".to_string()),
            new_buttons: None,
        }));

        let _ = handle_outgoing_message(&bot, &broker, msg).await;

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        let first: serde_json::Value = requests[0].body_json().unwrap();
        assert_eq!(first["parse_mode"], "MarkdownV2");
        let second: serde_json::Value = requests[1].body_json().unwrap();
        assert!(second.get("parse_mode").is_none());
    }

    #[tokio::test]
    async fn edit_message_buttons_only_hits_reply_markup_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"(?i)^/bot[^/]+/editMessageReplyMarkup$"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "ok": false, "error_code": 400, "description": "boom"
            })))
            .mount(&server)
            .await;

        let bot = test_bot(&server.uri());
        let broker = mock_broker();
        let msg = sample_message(OutgoingMessageType::EditMessage(EditMessageData {
            message_id: 10,
            new_text: None,
            new_buttons: Some(vec![vec![ButtonInfo {
                text: "Go".into(),
                callback_data: "go".into(),
            }]]),
        }));

        let _ = handle_outgoing_message(&bot, &broker, msg).await;

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
    }

    #[tokio::test]
    async fn delete_message_sends_expected_request_and_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"(?i)^/bot[^/]+/deleteMessage$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true, "result": true
            })))
            .mount(&server)
            .await;

        let bot = test_bot(&server.uri());
        let broker = mock_broker();
        let msg = sample_message(OutgoingMessageType::DeleteMessage(DeleteMessageData {
            message_id: 5,
        }));

        let result = handle_outgoing_message(&bot, &broker, msg).await;
        assert!(result.is_ok());

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = requests[0].body_json().unwrap();
        assert_eq!(body["message_id"], 5);
    }

    #[tokio::test]
    async fn typing_message_sends_typing_chat_action() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"(?i)^/bot[^/]+/sendChatAction$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true, "result": true
            })))
            .mount(&server)
            .await;

        let bot = test_bot(&server.uri());
        let broker = mock_broker();
        let msg = sample_message(OutgoingMessageType::TypingMessage(TypingMessageData {
            action: Some("typing".to_string()),
        }));

        let result = handle_outgoing_message(&bot, &broker, msg).await;
        assert!(result.is_ok());

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = requests[0].body_json().unwrap();
        assert_eq!(body["action"], "typing");
    }
}
