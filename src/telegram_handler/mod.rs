use crate::broker::MessageBroker;
use crate::utils::{
    file_info_from_animation, file_info_from_audio, file_info_from_document, file_info_from_photo,
    file_info_from_sticker, file_info_from_video, file_info_from_video_note, file_info_from_voice,
    get_file_info, select_best_photo,
};
use anyhow::Result;
use incoming::{FileInfo, IncomingMessage};
use std::sync::Arc;
use teloxide::prelude::{Bot, CallbackQuery, Message, Requester};
use teloxide::types::MessageReactionUpdated;
use tracing::Instrument;
use uuid::Uuid;

pub mod incoming;

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    producer: Arc<dyn MessageBroker>,
) -> Result<()> {
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("message_handler", trace_id = %trace_id, message_id = %msg.id.0, chat_id = %msg.chat.id.0);

    async move {
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
    let mut incoming_msg = IncomingMessage::new_telegram_message(
        msg.clone(),
        file_infos.clone(),
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );
    // Override the auto-generated trace_id with our span's trace_id
    incoming_msg.trace_id = trace_id;

    let json = serde_json::to_string(&incoming_msg)
        .map_err(|e| {
            tracing::error!(message_id = %msg.id.0, chat_id = %msg.chat.id.0, error = %e, "Failed to serialize IncomingMessage to JSON");
            e
        })?;

    tracing::info!(key = "message", has_files = %(!file_infos.is_empty()), file_count = %file_infos.len(), "Sending Telegram message to broker");
    producer.publish(json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "message", error = %e, "Failed to send message to broker");
        e
    })?;
    Ok(())
    }.instrument(span).await
}

pub async fn message_reaction_handler(
    _bot: Bot,
    reaction: MessageReactionUpdated,
    producer: Arc<dyn MessageBroker>,
) -> Result<()> {
    let chat_id = reaction.chat.id.0;
    let message_id = reaction.message_id.0;
    let user_id = reaction.actor.user().map(|u| u.id.0);
    let date = reaction.date;
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("message_reaction_handler", trace_id = %trace_id, chat_id = %chat_id, message_id = %message_id);

    async move {

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

    let mut incoming_msg = IncomingMessage::new_message_reaction(
        chat_id,
        message_id,
        user_id,
        date,
        old_reaction,
        new_reaction,
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );
    // Override the auto-generated trace_id with our span's trace_id
    incoming_msg.trace_id = trace_id;

    let json = serde_json::to_string(&incoming_msg)
        .map_err(|e| {
            tracing::error!(chat_id = %chat_id, message_id = %message_id, error = %e, "Failed to serialize message reaction to JSON");
            e
        })?;

    tracing::info!(key = "message_reaction", user_id = ?user_id, "Sending message reaction to broker");

    producer.publish(json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "message_reaction", error = %e, "Failed to send message reaction to broker");
        e
    })?;

    Ok(())
    }.instrument(span).await
}

pub async fn callback_query_handler(
    bot: Bot,
    query: CallbackQuery,
    producer: Arc<dyn MessageBroker>,
) -> Result<()> {
    let user_id = query.from.id.0;
    let query_id = query.id.clone();
    let data = query.data.as_deref().unwrap_or_default();
    let message_id = query.message.as_ref().map(|m| m.id().0);
    let chat_id = query.message.as_ref().map_or(0, |m| m.chat().id.0);
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("callback_query_handler", trace_id = %trace_id, user_id = %user_id, chat_id = %chat_id, callback_query_id = %query_id);

    async move {

    tracing::debug!(callback_query_id = %query_id, %user_id, message_id = ?message_id, callback_data = %data, "Received callback query");

    if let Err(e) = bot.answer_callback_query(query.id.clone()).await {
        tracing::warn!(callback_query_id = %query_id, user_id = %user_id, error = %e, "Failed to answer callback query");
    }

    let mut incoming_msg = IncomingMessage::new_callback_query(
        chat_id,
        user_id,
        message_id.unwrap_or(0),
        data.to_string(),
        query_id.clone(),
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );
    // Override the auto-generated trace_id with our span's trace_id
    incoming_msg.trace_id = trace_id;

    let json = serde_json::to_string(&incoming_msg)
        .map_err(|e| {
            tracing::error!(callback_query_id = %query_id, user_id = %user_id, error = %e, "Failed to serialize IncomingMessage to JSON");
            e
        })?;

    tracing::info!(key = "callback_query", "Sending callback data to broker");
    producer.publish(json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "callback_query", error = %e, "Failed to send callback data to broker");
        e
    })?;

    Ok(())
    }.instrument(span).await
}

pub async fn edited_message_handler(
    bot: Bot,
    msg: Message,
    producer: Arc<dyn MessageBroker>,
) -> Result<()> {
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("edited_message_handler", trace_id = %trace_id, message_id = %msg.id.0, chat_id = %msg.chat.id.0);

    async move {
        // Handle file info gathering for all supported file types (same as message_handler)
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
                "Getting file info from edited Telegram message"
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
                        "Failed to get photo file info from edited message"
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
            "Getting file info from edited Telegram message"
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
                    "Failed to get audio file info from edited message"
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
            "Getting file info from edited Telegram message"
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
                    "Failed to get voice file info from edited message"
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
            "Getting file info from edited Telegram message"
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
                    "Failed to get video file info from edited message"
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
            "Getting file info from edited Telegram message"
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
                    "Failed to get video note file info from edited message"
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
            "Getting file info from edited Telegram message"
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
                    "Failed to get document file info from edited message"
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
            "Getting file info from edited Telegram message"
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
                    "Failed to get sticker file info from edited message"
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
            "Getting file info from edited Telegram message"
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
                    "Failed to get animation file info from edited message"
                );
            }
        }
    }

    // Create unified incoming message for edited message
    let mut incoming_msg = IncomingMessage::new_edited_message(
        msg.clone(),
        file_infos.clone(),
        msg.edit_date().map(|dt| dt.timestamp() as i32),
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );
    // Override the auto-generated trace_id with our span's trace_id
    incoming_msg.trace_id = trace_id;

    let json = serde_json::to_string(&incoming_msg)
        .map_err(|e| {
            tracing::error!(message_id = %msg.id.0, chat_id = %msg.chat.id.0, error = %e, "Failed to serialize edited IncomingMessage to JSON");
            e
        })?;

    tracing::info!(key = "edited_message", has_files = %(!file_infos.is_empty()), file_count = %file_infos.len(), edit_date = ?msg.edit_date(), "Sending edited Telegram message to broker");
    producer.publish(json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "edited_message", error = %e, "Failed to send edited message to broker");
        e
    })?;
    Ok(())
    }.instrument(span).await
}
