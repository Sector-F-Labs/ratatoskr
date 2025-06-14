use crate::structs::{ImageStorageDir, KafkaInTopic};
use crate::utils::{
    file_info_from_animation, file_info_from_audio, file_info_from_document, file_info_from_photo,
    file_info_from_sticker, file_info_from_video, file_info_from_video_note, file_info_from_voice,
    get_file_info, select_best_photo,
};
use incoming::{FileInfo, IncomingMessage};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::{Bot, CallbackQuery, Message, Requester};
use teloxide::types::MessageReactionUpdated;

pub mod incoming;

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    producer: Arc<FutureProducer>,
    kafka_in_topic: KafkaInTopic,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Handle file info gathering for all supported file types
    let mut file_infos: Vec<FileInfo> = Vec::new();

    // Handle photos
    if let Some(photos) = msg.photo() {
        if let Some(best_photo) = select_best_photo(photos) {
            let (file, file_type, metadata) = file_info_from_photo(best_photo);
            tracing::info!(
                message_id = %msg.id.0,
                chat_id = %msg.chat.id.0,
                file_id = %file.id,
                file_type = "photo",
                "Getting file info from Telegram message"
            );

            match get_file_info(&bot, &file, file_type, metadata).await {
                Ok(file_info) => {
                    file_infos.push(file_info);
                }
                Err(e) => {
                    tracing::error!(
                        message_id = %msg.id.0,
                        chat_id = %msg.chat.id.0,
                        file_id = %file.id,
                        error = %e,
                        "Failed to get photo file info"
                    );
                }
            }
        }
    }

    // Handle audio
    if let Some(audio) = &msg.audio() {
        let (file, file_type, metadata) = file_info_from_audio(audio);
        tracing::info!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            file_id = %file.id,
            file_type = "audio",
            "Getting file info from Telegram message"
        );

        match get_file_info(&bot, &file, file_type, metadata).await {
            Ok(file_info) => {
                file_infos.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to get audio file info"
                );
            }
        }
    }

    // Handle voice
    if let Some(voice) = &msg.voice() {
        let (file, file_type, metadata) = file_info_from_voice(voice);
        tracing::info!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            file_id = %file.id,
            file_type = "voice",
            "Getting file info from Telegram message"
        );

        match get_file_info(&bot, &file, file_type, metadata).await {
            Ok(file_info) => {
                file_infos.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to get voice file info"
                );
            }
        }
    }

    // Handle video
    if let Some(video) = &msg.video() {
        let (file, file_type, metadata) = file_info_from_video(video);
        tracing::info!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            file_id = %file.id,
            file_type = "video",
            "Getting file info from Telegram message"
        );

        match get_file_info(&bot, &file, file_type, metadata).await {
            Ok(file_info) => {
                file_infos.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to get video file info"
                );
            }
        }
    }

    // Handle video note
    if let Some(video_note) = &msg.video_note() {
        let (file, file_type, metadata) = file_info_from_video_note(video_note);
        tracing::info!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            file_id = %file.id,
            file_type = "video_note",
            "Getting file info from Telegram message"
        );

        match get_file_info(&bot, &file, file_type, metadata).await {
            Ok(file_info) => {
                file_infos.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to get video note file info"
                );
            }
        }
    }

    // Handle document
    if let Some(document) = &msg.document() {
        let (file, file_type, metadata) = file_info_from_document(document);
        tracing::info!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            file_id = %file.id,
            file_type = "document",
            "Getting file info from Telegram message"
        );

        match get_file_info(&bot, &file, file_type, metadata).await {
            Ok(file_info) => {
                file_infos.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to get document file info"
                );
            }
        }
    }

    // Handle sticker
    if let Some(sticker) = &msg.sticker() {
        let (file, file_type, metadata) = file_info_from_sticker(sticker);
        tracing::info!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            file_id = %file.id,
            file_type = "sticker",
            "Getting file info from Telegram message"
        );

        match get_file_info(&bot, &file, file_type, metadata).await {
            Ok(file_info) => {
                file_infos.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to get sticker file info"
                );
            }
        }
    }

    // Handle animation
    if let Some(animation) = &msg.animation() {
        let (file, file_type, metadata) = file_info_from_animation(animation);
        tracing::info!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            file_id = %file.id,
            file_type = "animation",
            "Getting file info from Telegram message"
        );

        match get_file_info(&bot, &file, file_type, metadata).await {
            Ok(file_info) => {
                file_infos.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to get animation file info"
                );
            }
        }
    }

    // Create unified incoming message
    let incoming_msg = IncomingMessage::new_telegram_message(
        msg.clone(),
        file_infos.clone(),
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );

    let json = match serde_json::to_string(&incoming_msg) {
        Ok(json_string) => json_string,
        Err(e) => {
            tracing::error!(message_id = %msg.id.0, chat_id = %msg.chat.id.0, error = %e, "Failed to serialize IncomingMessage to JSON");
            return Err(Box::new(e));
        }
    };

    tracing::info!(
        topic = %kafka_in_topic.0,
        key = "message",
        message_id = %msg.id.0,
        chat_id = %msg.chat.id.0,
        has_files = %(!file_infos.is_empty()),
        file_count = %file_infos.len(),
        "Sending Telegram message to Kafka"
    );
    let record = FutureRecord::to(kafka_in_topic.0.as_str())
        .payload(&json)
        .key("message");

    if let Err((e, _)) = producer.send(record, None).await {
        tracing::error!(topic = %kafka_in_topic.0, key = "message", message_id = %msg.id.0, chat_id = %msg.chat.id.0, error = %e, "Failed to send message to Kafka");
        return Err(Box::new(e));
    }
    Ok(())
}

pub async fn message_reaction_handler(
    _bot: Bot,
    reaction: MessageReactionUpdated,
    producer: Arc<FutureProducer>,
    kafka_in_topic: KafkaInTopic,
    _image_storage_dir: ImageStorageDir,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = reaction.chat.id.0;
    let message_id = reaction.message_id.0;
    let user_id = reaction.actor.user().map(|u| u.id.0);
    let date = reaction.date;

    // Convert reaction types to strings
    let old_reaction: Vec<String> = reaction
        .old_reaction
        .iter()
        .map(|r| match r {
            teloxide::types::ReactionType::Emoji { emoji } => emoji.clone(),
            teloxide::types::ReactionType::CustomEmoji { custom_emoji_id } => {
                format!("custom:{}", custom_emoji_id)
            }
            teloxide::types::ReactionType::Paid => "paid".to_string(),
        })
        .collect();

    let new_reaction: Vec<String> = reaction
        .new_reaction
        .iter()
        .map(|r| match r {
            teloxide::types::ReactionType::Emoji { emoji } => emoji.clone(),
            teloxide::types::ReactionType::CustomEmoji { custom_emoji_id } => {
                format!("custom:{}", custom_emoji_id)
            }
            teloxide::types::ReactionType::Paid => "paid".to_string(),
        })
        .collect();

    tracing::info!(
        chat_id = %chat_id,
        message_id = %message_id,
        user_id = ?user_id,
        old_reactions = ?old_reaction,
        new_reactions = ?new_reaction,
        "Processing message reaction"
    );

    let incoming_msg = IncomingMessage::new_message_reaction(
        chat_id,
        message_id,
        user_id,
        date,
        old_reaction,
        new_reaction,
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );

    let json = match serde_json::to_string(&incoming_msg) {
        Ok(json_string) => json_string,
        Err(e) => {
            tracing::error!(chat_id = %chat_id, message_id = %message_id, error = %e, "Failed to serialize message reaction to JSON");
            return Err(Box::new(e));
        }
    };

    tracing::info!(
        topic = %kafka_in_topic.0,
        key = "message_reaction",
        chat_id = %chat_id,
        message_id = %message_id,
        user_id = ?user_id,
        "Sending message reaction to Kafka"
    );

    let record = FutureRecord::to(kafka_in_topic.0.as_str())
        .payload(&json)
        .key("message_reaction");

    if let Err((e, _)) = producer.send(record, None).await {
        tracing::error!(topic = %kafka_in_topic.0, key = "message_reaction", chat_id = %chat_id, message_id = %message_id, error = %e, "Failed to send message reaction to Kafka");
        return Err(Box::new(e));
    }

    Ok(())
}

pub async fn callback_query_handler(
    bot: Bot,
    query: CallbackQuery,
    producer: Arc<FutureProducer>,
    kafka_in_topic: KafkaInTopic,
    _image_storage_dir: ImageStorageDir,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user_id = query.from.id.0;
    let query_id = query.id.clone();
    let data = query.data.as_deref().unwrap_or_default();
    let message_id = query.message.as_ref().map(|m| m.id().0);
    let chat_id = query.message.as_ref().map_or(0, |m| m.chat().id.0);

    tracing::debug!(callback_query_id = %query_id, %user_id, message_id = ?message_id, callback_data = %data, "Received callback query");

    if let Err(e) = bot.answer_callback_query(query.id.clone()).await {
        tracing::warn!(callback_query_id = %query_id, user_id = %user_id, error = %e, "Failed to answer callback query");
    }

    let incoming_msg = IncomingMessage::new_callback_query(
        chat_id,
        user_id,
        message_id.unwrap_or(0),
        data.to_string(),
        query_id.clone(),
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );

    let json = match serde_json::to_string(&incoming_msg) {
        Ok(json_string) => json_string,
        Err(e) => {
            tracing::error!(callback_query_id = %query_id, user_id = %user_id, error = %e, "Failed to serialize IncomingMessage to JSON");
            return Err(Box::new(e));
        }
    };

    tracing::info!(topic = %kafka_in_topic.0, key = "callback_query", callback_query_id = %query_id, user_id = %user_id, "Sending callback data to Kafka");
    let record = FutureRecord::to(kafka_in_topic.0.as_str())
        .payload(&json)
        .key("callback_query");

    if let Err((e, _)) = producer.send(record, None).await {
        tracing::error!(topic = %kafka_in_topic.0, key = "callback_query", callback_query_id = %query_id, user_id = %user_id, error = %e, "Failed to send callback data to Kafka");
        return Err(Box::new(e));
    }

    Ok(())
}
