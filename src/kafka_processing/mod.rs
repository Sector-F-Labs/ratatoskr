use crate::outgoing::{OutgoingMessage, OutgoingMessageType};
use crate::utils::{create_markup, create_reply_keyboard};
use futures_util::StreamExt;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::Message as KafkaMessageRd; // Ensure this is kept if used, or removed if not. It seems used by consumer.stream()
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::path::Path;
use std::sync::Arc;
use chrono::Utc;
use crate::kafka_processing::status_message::TelegramMessageSentStatus;
use teloxide::{
    payloads::{
        EditMessageReplyMarkupSetters, EditMessageTextSetters, SendAnimationSetters,
        SendAudioSetters, SendDocumentSetters, SendMessageSetters, SendPhotoSetters,
        SendStickerSetters, SendVideoNoteSetters, SendVideoSetters, SendVoiceSetters,
    },
    prelude::{Bot, ChatId, Requester},
    types::{InputFile, ParseMode},
};

pub mod outgoing;
pub mod status_message;

pub async fn start_kafka_consumer_loop(
    bot_consumer_clone: Bot,
    consumer: StreamConsumer,
    kafka_out_topic_clone: String,
    status_producer: Arc<FutureProducer>,
    status_topic: String,
) {
    tracing::info!(topic = %kafka_out_topic_clone, "Starting Kafka consumer stream for Telegram output...");
    let status_producer_clone = status_producer.clone(); // Clone for the loop
    let status_topic_clone = status_topic.clone(); // Clone for the loop
    let mut stream = consumer.stream();
    while let Some(result) = stream.next().await {
        match result {
            Ok(kafka_msg) => {
                tracing::debug!(topic = %kafka_msg.topic(), partition = %kafka_msg.partition(), offset = %kafka_msg.offset(), "Consumed message from Kafka");
                if let Some(payload) = kafka_msg.payload() {
                    match serde_json::from_slice::<OutgoingMessage>(payload) {
                        Ok(out_msg) => {
                            if let Err(e) = handle_outgoing_message(
                                &bot_consumer_clone,
                                out_msg,
                                status_producer_clone.clone(), // Pass cloned producer
                                status_topic_clone.clone(),   // Pass cloned topic
                            )
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
                } else {
                    tracing::warn!(topic = %kafka_msg.topic(), partition = %kafka_msg.partition(), offset = %kafka_msg.offset(), "Received Kafka message with empty payload");
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
    status_producer: Arc<FutureProducer>,
    status_topic: String,
) -> Result<Option<teloxide::types::Message>, Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = ChatId(message.target.chat_id);
    let correlation_id = message.correlation_id.clone(); // Clone correlation_id for use in status messages

    match message.message_type {
        OutgoingMessageType::TextMessage(data) => {
            tracing::info!(%chat_id, text_length = %data.text.len(), has_buttons = %data.buttons.is_some(), "Sending text message to Telegram");
            let mut msg_to_send = bot.send_message(chat_id, data.text);

            if let Some(parse_mode) = data.parse_mode {
                msg_to_send = match parse_mode.as_str() {
                    "HTML" => msg_to_send.parse_mode(ParseMode::Html),
                    "Markdown" => msg_to_send.parse_mode(ParseMode::MarkdownV2),
                    _ => msg_to_send,
                };
            }

            if let Some(markup) = create_markup(&data.buttons) {
                msg_to_send = msg_to_send.reply_markup(markup);
            }

            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                msg_to_send = msg_to_send.reply_markup(reply_keyboard);
            }

            let sent_message = msg_to_send.await?;

            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "TextMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for TextMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());

            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent message status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send message status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::ImageMessage(data) => {
            tracing::info!(%chat_id, image_path = %data.image_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending image message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "ImageMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for ImageMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent image status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send image status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::AudioMessage(data) => {
            tracing::info!(%chat_id, audio_path = %data.audio_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending audio message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "AudioMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for AudioMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent audio status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send audio status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::VoiceMessage(data) => {
            tracing::info!(%chat_id, voice_path = %data.voice_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending voice message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "VoiceMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for VoiceMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent voice status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send voice status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::VideoMessage(data) => {
            tracing::info!(%chat_id, video_path = %data.video_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending video message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "VideoMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for VideoMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent video status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send video status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::VideoNoteMessage(data) => {
            tracing::info!(%chat_id, video_note_path = %data.video_note_path, has_buttons = %data.buttons.is_some(), "Sending video note message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "VideoNoteMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for VideoNoteMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent video note status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send video note status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::StickerMessage(data) => {
            tracing::info!(%chat_id, sticker_path = %data.sticker_path, has_buttons = %data.buttons.is_some(), "Sending sticker message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "StickerMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for StickerMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent sticker status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send sticker status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::AnimationMessage(data) => {
            tracing::info!(%chat_id, animation_path = %data.animation_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending animation message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "AnimationMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for AnimationMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent animation status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send animation status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::DocumentMessage(data) => {
            tracing::info!(%chat_id, document_path = %data.document_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending document message to Telegram");

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

            let sent_message = msg_to_send.await?;
            let status_message_obj = TelegramMessageSentStatus::new(
                sent_message.chat.id.0,
                sent_message.id.0,
                "success".to_string(),
                "DocumentMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for DocumentMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&sent_message.chat.id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, "Successfully sent document status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %sent_message.chat.id.0, message_id = %sent_message.id.0, status_topic = %status_topic, error = %e, "Failed to send document status to Kafka"),
            }
            Ok(Some(sent_message))
        }

        OutgoingMessageType::EditMessage(data) => {
            tracing::info!(%chat_id, message_id = %data.message_id, has_new_text = %data.new_text.is_some(), has_new_buttons = %data.new_buttons.is_some(), "Editing message in Telegram");

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
                if let Some(markup) = create_markup(&data.new_buttons) {
                    bot.edit_message_reply_markup(
                        chat_id,
                        teloxide::types::MessageId(data.message_id),
                    )
                    .reply_markup(markup)
                    .await?;
                }
            }

            let status_message_obj = TelegramMessageSentStatus::new(
                chat_id.0, // Use the ChatId from the function arguments
                data.message_id, // Use message_id from EditMessageData
                "success".to_string(),
                "EditMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for EditMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&chat_id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %chat_id.0, message_id = %data.message_id, status_topic = %status_topic, "Successfully sent edit message status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %chat_id.0, message_id = %data.message_id, status_topic = %status_topic, error = %e, "Failed to send edit message status to Kafka"),
            }
            Ok(None)
        }

        OutgoingMessageType::DeleteMessage(data) => {
            tracing::info!(%chat_id, message_id = %data.message_id, "Deleting message in Telegram");
            bot.delete_message(chat_id, teloxide::types::MessageId(data.message_id))
                .await?;

            let status_message_obj = TelegramMessageSentStatus::new(
                chat_id.0, // Use the ChatId from the function arguments
                data.message_id, // Use message_id from DeleteMessageData
                "success".to_string(),
                "DeleteMessage".to_string(),
                correlation_id,
            );
            let status_payload = serde_json::to_string(&status_message_obj).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize TelegramMessageSentStatus for DeleteMessage");
                format!("{{\"error\":\"Failed to serialize status: {}\"}}", e)
            });
            let record = FutureRecord::to(&status_topic)
                .payload(&status_payload)
                .key(&chat_id.0.to_string());
            match status_producer.send(record, None).await {
                Ok(_) => tracing::info!(chat_id = %chat_id.0, message_id = %data.message_id, status_topic = %status_topic, "Successfully sent delete message status to Kafka"),
                Err((e, _)) => tracing::error!(chat_id = %chat_id.0, message_id = %data.message_id, status_topic = %status_topic, error = %e, "Failed to send delete message status to Kafka"),
            }
            Ok(None)
        }
    }
}
