use crate::structs::{ImageStorageDir, KafkaInTopic};
use crate::utils::{
    download_file, file_info_from_animation, file_info_from_audio, file_info_from_document,
    file_info_from_photo, file_info_from_sticker, file_info_from_video, file_info_from_video_note,
    file_info_from_voice, select_best_photo,
};
use crate::messaging::MessageSystem; // Added
use crate::telegram_handler::incoming::IncomingMessage as UnifiedIncomingMessage; // Renamed for clarity if needed, or use existing
use incoming::{FileInfo, IncomingMessage}; // Assuming IncomingMessage is the one we need for publishing
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::{Bot, CallbackQuery, Message, Requester};
use teloxide::types::MessageReactionUpdated;

pub mod incoming;

pub async fn message_handler<MS>(
    bot: Bot,
    msg: Message,
    message_system: Arc<MS>,
    kafka_in_topic: KafkaInTopic, // This might become part of MS config or removed if MS knows its publish topic
    image_storage_dir: ImageStorageDir,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    MS: MessageSystem<PublishMessage = IncomingMessage> + 'static, // Specify PublishMessage type
{
    // Handle file downloads for all supported file types
    let mut downloaded_files: Vec<FileInfo> = Vec::new();

    // Handle photos
    if let Some(photos) = msg.photo() {
        if let Some(best_photo) = select_best_photo(photos) {
            let (file, file_type, metadata) = file_info_from_photo(best_photo);
            tracing::info!(
                message_id = %msg.id.0,
                chat_id = %msg.chat.id.0,
                file_id = %file.id,
                file_type = "photo",
                "Downloading file from Telegram message"
            );

            match download_file(
                &bot,
                &file,
                file_type,
                metadata,
                &image_storage_dir.0,
                msg.chat.id.0,
                msg.id.0,
            )
            .await
            {
                Ok(file_info) => {
                    downloaded_files.push(file_info);
                }
                Err(e) => {
                    tracing::error!(
                        message_id = %msg.id.0,
                        chat_id = %msg.chat.id.0,
                        file_id = %file.id,
                        error = %e,
                        "Failed to download photo"
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
            "Downloading file from Telegram message"
        );

        match download_file(
            &bot,
            &file,
            file_type,
            metadata,
            &image_storage_dir.0,
            msg.chat.id.0,
            msg.id.0,
        )
        .await
        {
            Ok(file_info) => {
                downloaded_files.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to download audio"
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
            "Downloading file from Telegram message"
        );

        match download_file(
            &bot,
            &file,
            file_type,
            metadata,
            &image_storage_dir.0,
            msg.chat.id.0,
            msg.id.0,
        )
        .await
        {
            Ok(file_info) => {
                downloaded_files.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to download voice"
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
            "Downloading file from Telegram message"
        );

        match download_file(
            &bot,
            &file,
            file_type,
            metadata,
            &image_storage_dir.0,
            msg.chat.id.0,
            msg.id.0,
        )
        .await
        {
            Ok(file_info) => {
                downloaded_files.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to download video"
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
            "Downloading file from Telegram message"
        );

        match download_file(
            &bot,
            &file,
            file_type,
            metadata,
            &image_storage_dir.0,
            msg.chat.id.0,
            msg.id.0,
        )
        .await
        {
            Ok(file_info) => {
                downloaded_files.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to download video note"
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
            "Downloading file from Telegram message"
        );

        match download_file(
            &bot,
            &file,
            file_type,
            metadata,
            &image_storage_dir.0,
            msg.chat.id.0,
            msg.id.0,
        )
        .await
        {
            Ok(file_info) => {
                downloaded_files.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to download document"
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
            "Downloading file from Telegram message"
        );

        match download_file(
            &bot,
            &file,
            file_type,
            metadata,
            &image_storage_dir.0,
            msg.chat.id.0,
            msg.id.0,
        )
        .await
        {
            Ok(file_info) => {
                downloaded_files.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to download sticker"
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
            "Downloading file from Telegram message"
        );

        match download_file(
            &bot,
            &file,
            file_type,
            metadata,
            &image_storage_dir.0,
            msg.chat.id.0,
            msg.id.0,
        )
        .await
        {
            Ok(file_info) => {
                downloaded_files.push(file_info);
            }
            Err(e) => {
                tracing::error!(
                    message_id = %msg.id.0,
                    chat_id = %msg.chat.id.0,
                    file_id = %file.id,
                    error = %e,
                    "Failed to download animation"
                );
            }
        }
    }

    // Create unified incoming message
    let incoming_msg = IncomingMessage::new_telegram_message(
        msg.clone(),
        downloaded_files.clone(),
        None, // bot_id - could be retrieved from bot.get_me() if needed
        None, // bot_username - could be retrieved from bot.get_me() if needed
    );

    // The MessageSystem's publish method handles serialization.
    // The topic is also handled by the MessageSystem implementation (e.g., KafkaAdapter's produce_topic config)
    tracing::info!(
        message_id = %msg.id.0,
        chat_id = %msg.chat.id.0,
        has_files = %(!downloaded_files.is_empty()),
        file_count = %downloaded_files.len(),
        "Publishing Telegram message via MessageSystem"
    );

    if let Err(e) = message_system.publish(&incoming_msg).await {
        tracing::error!(
            message_id = %msg.id.0,
            chat_id = %msg.chat.id.0,
            error = ?e, // Use ?e for rich error display if available
            "Failed to publish message via MessageSystem"
        );
        // Convert MS::Error to Box<dyn Error + Send + Sync>
        // This conversion might need to be more sophisticated depending on MS::Error.
        // For now, a simple to_string() and then boxing it.
        // A better way would be to ensure MS::Error itself is Box<dyn Error + Send + Sync>
        // or provide a conversion.
        return Err(format!("Publishing error: {}", e).into());
    }

    Ok(())
}

pub async fn message_reaction_handler<MS>(
    _bot: Bot,
    reaction: MessageReactionUpdated,
    message_system: Arc<MS>,
    kafka_in_topic: KafkaInTopic, // This might become part of MS config or removed
    _image_storage_dir: ImageStorageDir,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    MS: MessageSystem<PublishMessage = IncomingMessage> + 'static,
{
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

    tracing::info!(
        chat_id = %chat_id,
        message_id = %message_id,
        user_id = ?user_id,
        "Publishing message reaction via MessageSystem"
    );

    if let Err(e) = message_system.publish(&incoming_msg).await {
        tracing::error!(
            chat_id = %chat_id,
            message_id = %message_id,
            error = ?e,
            "Failed to publish message reaction via MessageSystem"
        );
        return Err(format!("Publishing error: {}", e).into());
    }

    Ok(())
}

pub async fn callback_query_handler<MS>(
    bot: Bot,
    query: CallbackQuery,
    message_system: Arc<MS>,
    kafka_in_topic: KafkaInTopic, // This might become part of MS config or removed
    _image_storage_dir: ImageStorageDir,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    MS: MessageSystem<PublishMessage = IncomingMessage> + 'static,
{
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

    tracing::info!(callback_query_id = %query_id, user_id = %user_id, "Publishing callback data via MessageSystem");

    if let Err(e) = message_system.publish(&incoming_msg).await {
        tracing::error!(
            callback_query_id = %query_id,
            user_id = %user_id,
            error = ?e,
            "Failed to publish callback data via MessageSystem"
        );
        return Err(format!("Publishing error: {}", e).into());
    }

    Ok(())
}
