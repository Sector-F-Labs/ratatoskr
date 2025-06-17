use self::outgoing::{OutgoingMessage, OutgoingMessageType};
use crate::utils::{
    create_markup, create_reply_keyboard, format_telegram_markdown, split_text,
    CAPTION_LIMIT, TEXT_LIMIT,
};
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
            tracing::info!(
                text_length = %data.text.len(),
                has_buttons = %data.buttons.is_some(),
                "Sending text message to Telegram"
            );

            // Auto-organize buttons if they exist
            let organized_buttons = data.buttons.as_ref().map(|buttons| {
                if buttons.len() == 1 && buttons[0].len() > 1 {
                    tracing::info!(
                        original_buttons = %buttons[0].len(),
                        "Auto-organizing buttons based on text length"
                    );
                    self::outgoing::ButtonInfo::create_inline_keyboard(buttons[0].clone())
                } else {
                    buttons.clone()
                }
            });

            let formatted_text = if data.parse_mode.is_some() {
                Some(format_telegram_markdown(&data.text))
            } else {
                None
            };

            let formatted_chunks = if let Some(ref ft) = formatted_text {
                split_text(ft, TEXT_LIMIT)
            } else {
                split_text(&data.text, TEXT_LIMIT)
            };
            let plain_chunks = split_text(&data.text, TEXT_LIMIT);

            for (idx, chunk) in formatted_chunks.iter().enumerate() {
                let plain_chunk = plain_chunks
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| chunk.clone());

                let mut msg_to_send = bot.send_message(chat_id, chunk.clone());

                if let Some(parse_mode) = &data.parse_mode {
                    msg_to_send = match parse_mode.as_str() {
                        "HTML" => msg_to_send.parse_mode(ParseMode::Html),
                        "Markdown" => msg_to_send.parse_mode(ParseMode::Html),
                        _ => msg_to_send,
                    };
                }

                if idx == 0 {
                    if let Some(markup) = create_markup(&organized_buttons) {
                        msg_to_send = msg_to_send.reply_markup(markup);
                    }
                    if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                        msg_to_send = msg_to_send.reply_markup(reply_keyboard);
                    }
                    if let Some(disable) = data.disable_web_page_preview {
                        msg_to_send = msg_to_send.disable_web_page_preview(disable);
                    }
                }

                try_send_with_fallback(
                    msg_to_send.await,
                    || async {
                        let mut plain_msg = bot.send_message(chat_id, plain_chunk.clone());
                        if idx == 0 {
                            if let Some(markup) = create_markup(&organized_buttons) {
                                plain_msg = plain_msg.reply_markup(markup);
                            }
                            if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                                plain_msg = plain_msg.reply_markup(reply_keyboard);
                            }
                            if let Some(disable) = data.disable_web_page_preview {
                                plain_msg = plain_msg.disable_web_page_preview(disable);
                            }
                        }
                        plain_msg.await
                    },
                    "text message",
                )
                .await?;
            }
        }

        OutgoingMessageType::ImageMessage(data) => {
            tracing::info!(image_path = %data.image_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending image message to Telegram");

            if !Path::new(&data.image_path).exists() {
                return Err(format!("Image file not found: {}", data.image_path).into());
            }

            let input_file = InputFile::file(&data.image_path);

            if let Some(caption) = &data.caption {
                let formatted_caption = format_telegram_markdown(caption);
                let formatted_chunks = split_text(&formatted_caption, CAPTION_LIMIT);
                let plain_chunks = split_text(caption, CAPTION_LIMIT);

                let first_formatted = formatted_chunks.first().cloned().unwrap_or_default();
                let first_plain = plain_chunks.first().cloned().unwrap_or_default();

                let mut msg_to_send = bot
                    .send_photo(chat_id, input_file.clone())
                    .caption(first_formatted)
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
                            bot.send_photo(chat_id, input_file.clone()).caption(first_plain.clone());
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

                for i in 1..formatted_chunks.len() {
                    let follow_formatted = formatted_chunks[i].clone();
                    let follow_plain = plain_chunks.get(i).cloned().unwrap_or_else(|| follow_formatted.clone());
                    try_send_with_fallback(
                        bot.send_message(chat_id, follow_formatted)
                            .parse_mode(ParseMode::Html)
                            .await,
                        || async { bot.send_message(chat_id, follow_plain).await },
                        "caption continuation",
                    )
                    .await?;
                }
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
            let mut msg_to_send = bot.send_audio(chat_id, input_file.clone());

            let caption_chunks = data
                .caption
                .as_ref()
                .map(|c| split_text(&format_telegram_markdown(c), CAPTION_LIMIT));
            let caption_plain_chunks =
                data.caption.as_ref().map(|c| split_text(c, CAPTION_LIMIT));

            if let Some(chunks) = caption_chunks.as_ref() {
                if let Some(first) = chunks.first() {
                    msg_to_send = msg_to_send.caption(first.clone());
                }
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

            try_send_with_fallback(
                msg_to_send.await,
                || async {
                    let mut plain = bot.send_audio(chat_id, input_file.clone());
                    if let Some(first_plain) = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.first())
                    {
                        plain = plain.caption(first_plain.clone());
                    }
                    if let Some(markup) = create_markup(&data.buttons) {
                        plain = plain.reply_markup(markup);
                    }
                    if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                        plain = plain.reply_markup(reply_keyboard);
                    }
                    plain.await
                },
                "audio message",
            )
            .await?;

            if let Some(chunks) = caption_chunks {
                for i in 1..chunks.len() {
                    let follow_formatted = chunks[i].clone();
                    let follow_plain = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .cloned()
                        .unwrap_or_else(|| follow_formatted.clone());
                    try_send_with_fallback(
                        bot.send_message(chat_id, follow_formatted).parse_mode(ParseMode::Html).await,
                        || async { bot.send_message(chat_id, follow_plain).await },
                        "caption continuation",
                    )
                    .await?;
                }
            }
        }

        OutgoingMessageType::VoiceMessage(data) => {
            tracing::info!(voice_path = %data.voice_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending voice message to Telegram");

            if !Path::new(&data.voice_path).exists() {
                return Err(format!("Voice file not found: {}", data.voice_path).into());
            }

            let input_file = InputFile::file(&data.voice_path);
            let mut msg_to_send = bot.send_voice(chat_id, input_file.clone());

            let caption_chunks = data
                .caption
                .as_ref()
                .map(|c| split_text(&format_telegram_markdown(c), CAPTION_LIMIT));
            let caption_plain_chunks =
                data.caption.as_ref().map(|c| split_text(c, CAPTION_LIMIT));

            if let Some(chunks) = caption_chunks.as_ref() {
                if let Some(first) = chunks.first() {
                    msg_to_send = msg_to_send.caption(first.clone());
                }
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

            try_send_with_fallback(
                msg_to_send.await,
                || async {
                    let mut plain = bot.send_voice(chat_id, input_file.clone());
                    if let Some(first_plain) = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.first())
                    {
                        plain = plain.caption(first_plain.clone());
                    }
                    if let Some(markup) = create_markup(&data.buttons) {
                        plain = plain.reply_markup(markup);
                    }
                    if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                        plain = plain.reply_markup(reply_keyboard);
                    }
                    plain.await
                },
                "voice message",
            )
            .await?;

            if let Some(chunks) = caption_chunks {
                for i in 1..chunks.len() {
                    let follow_formatted = chunks[i].clone();
                    let follow_plain = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .cloned()
                        .unwrap_or_else(|| follow_formatted.clone());
                    try_send_with_fallback(
                        bot.send_message(chat_id, follow_formatted).parse_mode(ParseMode::Html).await,
                        || async { bot.send_message(chat_id, follow_plain).await },
                        "caption continuation",
                    )
                    .await?;
                }
            }
        }

        OutgoingMessageType::VideoMessage(data) => {
            tracing::info!(video_path = %data.video_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending video message to Telegram");

            if !Path::new(&data.video_path).exists() {
                return Err(format!("Video file not found: {}", data.video_path).into());
            }

            let input_file = InputFile::file(&data.video_path);
            let mut msg_to_send = bot.send_video(chat_id, input_file.clone());

            let caption_chunks = data
                .caption
                .as_ref()
                .map(|c| split_text(&format_telegram_markdown(c), CAPTION_LIMIT));
            let caption_plain_chunks =
                data.caption.as_ref().map(|c| split_text(c, CAPTION_LIMIT));

            if let Some(chunks) = caption_chunks.as_ref() {
                if let Some(first) = chunks.first() {
                    msg_to_send = msg_to_send.caption(first.clone());
                }
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

            try_send_with_fallback(
                msg_to_send.await,
                || async {
                    let mut plain = bot.send_video(chat_id, input_file.clone());
                    if let Some(first_plain) = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.first())
                    {
                        plain = plain.caption(first_plain.clone());
                    }
                    if let Some(duration) = data.duration {
                        plain = plain.duration(duration);
                    }
                    if let Some(width) = data.width {
                        plain = plain.width(width);
                    }
                    if let Some(height) = data.height {
                        plain = plain.height(height);
                    }
                    if let Some(supports_streaming) = data.supports_streaming {
                        plain = plain.supports_streaming(supports_streaming);
                    }
                    if let Some(markup) = create_markup(&data.buttons) {
                        plain = plain.reply_markup(markup);
                    }
                    if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                        plain = plain.reply_markup(reply_keyboard);
                    }
                    plain.await
                },
                "video message",
            )
            .await?;

            if let Some(chunks) = caption_chunks {
                for i in 1..chunks.len() {
                    let follow_formatted = chunks[i].clone();
                    let follow_plain = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .cloned()
                        .unwrap_or_else(|| follow_formatted.clone());
                    try_send_with_fallback(
                        bot.send_message(chat_id, follow_formatted).parse_mode(ParseMode::Html).await,
                        || async { bot.send_message(chat_id, follow_plain).await },
                        "caption continuation",
                    )
                    .await?;
                }
            }
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

            try_send_with_fallback(
                msg_to_send.await,
                || async {
                    let mut plain = bot.send_animation(chat_id, input_file.clone());
                    if let Some(first_plain) = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.first())
                    {
                        plain = plain.caption(first_plain.clone());
                    }
                    if let Some(duration) = data.duration {
                        plain = plain.duration(duration);
                    }
                    if let Some(width) = data.width {
                        plain = plain.width(width);
                    }
                    if let Some(height) = data.height {
                        plain = plain.height(height);
                    }
                    if let Some(markup) = create_markup(&data.buttons) {
                        plain = plain.reply_markup(markup);
                    }
                    if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                        plain = plain.reply_markup(reply_keyboard);
                    }
                    plain.await
                },
                "animation message",
            )
            .await?;

            if let Some(chunks) = caption_chunks {
                for i in 1..chunks.len() {
                    let follow_formatted = chunks[i].clone();
                    let follow_plain = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .cloned()
                        .unwrap_or_else(|| follow_formatted.clone());
                    try_send_with_fallback(
                        bot.send_message(chat_id, follow_formatted).parse_mode(ParseMode::Html).await,
                        || async { bot.send_message(chat_id, follow_plain).await },
                        "caption continuation",
                    )
                    .await?;
                }
            }
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

            try_send_with_fallback(
                msg_to_send.await,
                || async {
                    let mut plain = bot.send_document(chat_id, input_file.clone());
                    if let Some(first_plain) = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.first())
                    {
                        plain = plain.caption(first_plain.clone());
                    }
                    if let Some(markup) = create_markup(&data.buttons) {
                        plain = plain.reply_markup(markup);
                    }
                    if let Some(reply_keyboard) = create_reply_keyboard(&data.reply_keyboard) {
                        plain = plain.reply_markup(reply_keyboard);
                    }
                    plain.await
                },
                "document message",
            )
            .await?;

            if let Some(chunks) = caption_chunks {
                for i in 1..chunks.len() {
                    let follow_formatted = chunks[i].clone();
                    let follow_plain = caption_plain_chunks
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .cloned()
                        .unwrap_or_else(|| follow_formatted.clone());
                    try_send_with_fallback(
                        bot.send_message(chat_id, follow_formatted).parse_mode(ParseMode::Html).await,
                        || async { bot.send_message(chat_id, follow_plain).await },
                        "caption continuation",
                    )
                    .await?;
                }
            }
        }

        OutgoingMessageType::AnimationMessage(data) => {
            tracing::info!(animation_path = %data.animation_path, has_caption = %data.caption.is_some(), has_buttons = %data.buttons.is_some(), "Sending animation message to Telegram");

            if !Path::new(&data.animation_path).exists() {
                return Err(format!("Animation file not found: {}", data.animation_path).into());
            }

            let input_file = InputFile::file(&data.animation_path);
            let mut msg_to_send = bot.send_animation(chat_id, input_file.clone());

            let caption_chunks = data
                .caption
                .as_ref()
                .map(|c| split_text(&format_telegram_markdown(c), CAPTION_LIMIT));
            let caption_plain_chunks =
                data.caption.as_ref().map(|c| split_text(c, CAPTION_LIMIT));

            if let Some(chunks) = caption_chunks.as_ref() {
                if let Some(first) = chunks.first() {
                    msg_to_send = msg_to_send.caption(first.clone());
                }
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

            let mut msg_to_send = bot.send_document(chat_id, input_file.clone());

            let caption_chunks = data
                .caption
                .as_ref()
                .map(|c| split_text(&format_telegram_markdown(c), CAPTION_LIMIT));
            let caption_plain_chunks =
                data.caption.as_ref().map(|c| split_text(c, CAPTION_LIMIT));

            if let Some(chunks) = caption_chunks.as_ref() {
                if let Some(first) = chunks.first() {
                    msg_to_send = msg_to_send.caption(first.clone());
                }
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
