use self::outgoing::{OutgoingMessage, OutgoingMessageType};
use crate::utils::{create_markup, create_reply_keyboard};
use futures_util::StreamExt;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::Message as KafkaMessageRd;
use std::path::Path;
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

pub mod outgoing;

pub async fn start_kafka_consumer_loop(
    bot_consumer_clone: Bot,
    consumer: StreamConsumer,
    kafka_out_topic_clone: String,
) {
    tracing::info!(topic = %kafka_out_topic_clone, "Starting Kafka consumer stream for Telegram output...");
    let mut stream = consumer.stream();
    while let Some(result) = stream.next().await {
        match result {
            Ok(kafka_msg) => {
                tracing::debug!(topic = %kafka_msg.topic(), partition = %kafka_msg.partition(), offset = %kafka_msg.offset(), "Consumed message from Kafka");
                if let Some(payload) = kafka_msg.payload() {
                    match serde_json::from_slice::<OutgoingMessage>(payload) {
                        Ok(mut out_msg) => {
                            // Ensure trace_id is set (for backward compatibility)
                            if out_msg.trace_id.is_nil() {
                                out_msg.trace_id = uuid::Uuid::new_v4();
                                tracing::debug!(
                                    "Generated new trace ID for message without one: {}",
                                    out_msg.trace_id
                                );
                            }

                            let span = tracing::info_span!(
                                "handle_outgoing_message",
                                trace_id = %out_msg.trace_id,
                                chat_id = %out_msg.target.chat_id,
                                message_type = ?std::mem::discriminant(&out_msg.message_type)
                            );

                            if let Err(e) = handle_outgoing_message(&bot_consumer_clone, out_msg)
                                .instrument(span)
                                .await
                            {
                                tracing::error!(topic = %kafka_msg.topic(), error = ?e, "Error handling OutgoingMessage");
                            }
                        }
                        Err(e) => {
                            tracing::error!(topic = %kafka_msg.topic(), error = %e, "Error deserializing message from Kafka payload");
                            tracing::debug!(raw_payload = ?String::from_utf8_lossy(payload), "Problematic Kafka payload");
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!(topic = %kafka_out_topic_clone, error = %e, "Error consuming message from Kafka");
            }
        }
    }
    tracing::warn!(topic = %kafka_out_topic_clone, "Kafka consumer stream ended.");
}

async fn handle_outgoing_message(
    bot: &Bot,
    message: OutgoingMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = ChatId(message.target.chat_id);

    match message.message_type {
        OutgoingMessageType::TextMessage(data) => {
            tracing::info!(text_length = %data.text.len(), has_buttons = %data.buttons.is_some(), "Sending text message to Telegram");
            let mut msg_to_send = bot.send_message(chat_id, data.text);

            if let Some(parse_mode) = data.parse_mode {
                msg_to_send = match parse_mode.as_str() {
                    "HTML" => msg_to_send.parse_mode(ParseMode::Html),
                    "Markdown" => msg_to_send.parse_mode(ParseMode::MarkdownV2),
                    _ => msg_to_send,
                };
            }

            // Auto-organize buttons if they exist
            let organized_buttons = data.buttons.as_ref().map(|buttons| {
                if buttons.len() == 1 && buttons[0].len() > 1 {
                    // If we have a single row with multiple buttons, auto-organize them
                    tracing::info!(original_buttons = %buttons[0].len(), "Auto-organizing buttons based on text length");
                    self::outgoing::ButtonInfo::create_inline_keyboard(buttons[0].clone())
                } else {
                    // Keep existing organization
                    buttons.clone()
                }
            });

            if let Some(markup) = create_markup(&organized_buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::ImageMessage(data) => {
            tracing::info!(image_path = %data.image_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending image message to Telegram");

            if !Path::new(&data.image_path).exists() {
                return Err(format!("Image file not found: {}", data.image_path).into());
            }

            let input_file = InputFile::file(&data.image_path);
            let mut msg_to_send = bot.send_photo(chat_id, input_file);

            if let Some(caption) = data.caption {
                msg_to_send = msg_to_send.caption(caption);
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            msg_to_send.await?;
        }

        OutgoingMessageType::AudioMessage(data) => {
            tracing::info!(audio_path = %data.audio_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending audio message to Telegram");

            if !Path::new(&data.audio_path).exists() {
                return Err(format!("Audio file not found: {}", data.audio_path).into());
            }

            let input_file = InputFile::file(&data.audio_path);
            let mut msg_to_send = bot.send_audio(chat_id, input_file);

            if let Some(caption) = data.caption {
                msg_to_send = msg_to_send.caption(caption);
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
                msg_to_send = msg_to_send.caption(caption);
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
                msg_to_send = msg_to_send.caption(caption);
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
                msg_to_send = msg_to_send.caption(caption);
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
                msg_to_send = msg_to_send.caption(caption);
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
                let mut msg_to_edit = bot.edit_message_text(
                    chat_id,
                    teloxide::types::MessageId(data.message_id),
                    new_text,
                );

                if let Some(markup) = create_markup(&data.new_buttons) {
                    msg_to_edit = msg_to_edit.reply_markup(markup);
                }

                msg_to_edit.await?;
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
