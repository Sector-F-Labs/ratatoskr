use crate::auth::AuthService;
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
use tokio::sync::RwLock;
use tracing::Instrument;
use uuid::Uuid;

pub mod incoming;

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    producer: Arc<dyn MessageBroker>,
    auth: Arc<RwLock<AuthService>>,
) -> Result<()> {
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("message_handler", trace_id = %trace_id, message_id = %msg.id.0, chat_id = %msg.chat.id.0);

    async move {
        // Auth gate
        if let Some(from) = msg.from.as_ref() {
            let tg_id = from.id.0;
            let tg_username = from.username.as_deref();
            let auth_read = auth.read().await;
            match auth_read.check(tg_id, tg_username) {
                None => {
                    if !auth_read.is_empty() {
                        tracing::warn!(telegram_user_id = tg_id, username = ?tg_username, "Unauthorized message — dropping");
                        return Ok(());
                    }
                }
                Some(idx) => {
                    let needs_promote = auth_read.get_user(idx).is_some_and(|u| u.telegram_user_id.is_none());
                    drop(auth_read);
                    if needs_promote {
                        let mut auth_write = auth.write().await;
                        let _ = auth_write.promote(idx, tg_id);
                    }
                }
            }
        }

        // Handle file info gathering for all supported file types
        let mut file_infos: Vec<FileInfo> = Vec::new();

    // Handle photos
    if let Some(photos) = msg.photo()
        && let Some(best_photo) = select_best_photo(photos) {
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

    // Use telegram_user_id as the key for Kafka partitioning
    let kafka_key = msg.from.as_ref().map(|f| f.id.0.to_string());
    tracing::info!(key = "message", kafka_key = ?kafka_key, has_files = %(!file_infos.is_empty()), file_count = %file_infos.len(), "Sending Telegram message to Kafka");

    producer.publish(kafka_key.as_deref(), json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "message", error = %e, "Failed to send message to Kafka");
        e
    })?;

    Ok(())
    }.instrument(span).await
}

pub async fn message_reaction_handler(
    _bot: Bot,
    reaction: MessageReactionUpdated,
    producer: Arc<dyn MessageBroker>,
    auth: Arc<RwLock<AuthService>>,
) -> Result<()> {
    let chat_id = reaction.chat.id.0;
    let message_id = reaction.message_id.0;
    let user_id = reaction.actor.user().map(|u| u.id.0);
    let date = reaction.date;
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("message_reaction_handler", trace_id = %trace_id, chat_id = %chat_id, message_id = %message_id);

    async move {
    // Auth gate
    if let Some(tg_id) = user_id {
        let tg_username = reaction.actor.user().and_then(|u| u.username.as_deref().map(String::from));
        let auth_read = auth.read().await;
        match auth_read.check(tg_id, tg_username.as_deref()) {
            None => {
                if !auth_read.is_empty() {
                    tracing::warn!(telegram_user_id = tg_id, "Unauthorized reaction — dropping");
                    return Ok(());
                }
            }
            Some(_idx) => {}
        }
    }

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

    // Use telegram_user_id as the key for Kafka partitioning
    let kafka_key = user_id.map(|id| id.to_string());
    tracing::info!(key = "message_reaction", kafka_key = ?kafka_key, user_id = ?user_id, "Sending message reaction to Kafka");

    producer.publish(kafka_key.as_deref(), json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "message_reaction", error = %e, "Failed to send message reaction to Kafka");
        e
    })?;

    Ok(())
    }.instrument(span).await
}

pub async fn callback_query_handler(
    bot: Bot,
    query: CallbackQuery,
    producer: Arc<dyn MessageBroker>,
    auth: Arc<RwLock<AuthService>>,
) -> Result<()> {
    let user_id = query.from.id.0;
    let query_id = query.id.clone();
    let data = query.data.as_deref().unwrap_or_default();
    let message_id = query.message.as_ref().map(|m| m.id().0);
    let chat_id = query.message.as_ref().map_or(0, |m| m.chat().id.0);
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("callback_query_handler", trace_id = %trace_id, user_id = %user_id, chat_id = %chat_id, callback_query_id = %query_id);

    async move {
    // Auth gate
    {
        let tg_username = query.from.username.as_deref();
        let auth_read = auth.read().await;
        match auth_read.check(user_id, tg_username) {
            None => {
                if !auth_read.is_empty() {
                    tracing::warn!(telegram_user_id = user_id, "Unauthorized callback query — dropping");
                    return Ok(());
                }
            }
            Some(_idx) => {}
        }
    }

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

    // Use telegram_user_id as the key for Kafka partitioning
    let kafka_key = user_id.to_string();
    tracing::info!(key = "callback_query", kafka_key = %kafka_key, "Sending callback data to Kafka");

    producer.publish(Some(&kafka_key), json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "callback_query", error = %e, "Failed to send callback data to Kafka");
        e
    })?;

    Ok(())
    }.instrument(span).await
}

pub async fn edited_message_handler(
    bot: Bot,
    msg: Message,
    producer: Arc<dyn MessageBroker>,
    auth: Arc<RwLock<AuthService>>,
) -> Result<()> {
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("edited_message_handler", trace_id = %trace_id, message_id = %msg.id.0, chat_id = %msg.chat.id.0);

    async move {
        // Auth gate
        if let Some(from) = msg.from.as_ref() {
            let tg_id = from.id.0;
            let tg_username = from.username.as_deref();
            let auth_read = auth.read().await;
            match auth_read.check(tg_id, tg_username) {
                None => {
                    if !auth_read.is_empty() {
                        tracing::warn!(telegram_user_id = tg_id, "Unauthorized edited message — dropping");
                        return Ok(());
                    }
                }
                Some(_idx) => {}
            }
        }

        // Handle file info gathering for all supported file types (same as message_handler)
        let mut file_infos: Vec<FileInfo> = Vec::new();

    // Handle photos
    if let Some(photos) = msg.photo()
        && let Some(best_photo) = select_best_photo(photos) {
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

    // Use telegram_user_id as the key for Kafka partitioning
    let kafka_key = msg.from.as_ref().map(|f| f.id.0.to_string());
    tracing::info!(key = "edited_message", kafka_key = ?kafka_key, has_files = %(!file_infos.is_empty()), file_count = %file_infos.len(), edit_date = ?msg.edit_date(), "Sending edited Telegram message to Kafka");

    producer.publish(kafka_key.as_deref(), json.as_bytes()).await.map_err(|e| {
        tracing::error!(key = "edited_message", error = %e, "Failed to send edited message to Kafka");
        e
    })?;

    Ok(())
    }.instrument(span).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::test_support::MockMessageBroker;
    use crate::config::{UserEntry, UsersConfig};
    use std::path::PathBuf;
    use teloxide::types::{CallbackQuery, MessageReactionUpdated};
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_entry(system_user: &str, tg_id: Option<u64>) -> UserEntry {
        UserEntry {
            system_user: system_user.to_string(),
            enabled: true,
            telegram_user_id: tg_id,
            promote_on_first_auth: false,
            allowed_usernames: vec![],
            first_seen_at: None,
            last_seen_at: None,
        }
    }

    fn auth_allowing(tg_id: u64) -> Arc<RwLock<AuthService>> {
        let config = UsersConfig {
            users: vec![make_entry("alice", Some(tg_id))],
        };
        Arc::new(RwLock::new(AuthService::new(config, PathBuf::from("/tmp/test.toml"))))
    }

    fn auth_denying_everyone() -> Arc<RwLock<AuthService>> {
        let config = UsersConfig {
            users: vec![make_entry("alice", Some(1))],
        };
        Arc::new(RwLock::new(AuthService::new(config, PathBuf::from("/tmp/test.toml"))))
    }

    fn auth_open() -> Arc<RwLock<AuthService>> {
        let config = UsersConfig { users: vec![] };
        Arc::new(RwLock::new(AuthService::new(config, PathBuf::from("/tmp/test.toml"))))
    }

    fn mock_broker() -> Arc<MockMessageBroker> {
        Arc::new(MockMessageBroker::new())
    }

    fn test_bot(api_url: &str) -> Bot {
        Bot::new("test_token").set_api_url(reqwest::Url::parse(api_url).unwrap())
    }

    /// A plain-text message from user 555, with no file attachments - so
    /// `message_handler`/`edited_message_handler` never call `get_file_info`
    /// and this fixture never needs a live/mocked `Bot`.
    fn text_message_json(from_id: u64) -> serde_json::Value {
        serde_json::json!({
            "message_id": 100,
            "from": {
                "id": from_id,
                "is_bot": false,
                "first_name": "Alice",
                "username": "alice"
            },
            "chat": {
                "id": from_id,
                "first_name": "Alice",
                "username": "alice",
                "type": "private"
            },
            "date": 1_700_000_000,
            "text": "hello world"
        })
    }

    fn text_message(from_id: u64) -> Message {
        serde_json::from_value(text_message_json(from_id)).unwrap()
    }

    // --- message_reaction_handler: `_bot` is unused, so no network involved ---

    fn reaction_updated() -> MessageReactionUpdated {
        let json = serde_json::json!({
            "chat": { "id": -1002184233434i64, "title": "Test", "type": "supergroup" },
            "message_id": 35,
            "user": {
                "id": 1459074222u64,
                "is_bot": false,
                "first_name": "shadowchain",
                "username": "shdwchn10"
            },
            "date": 1_721_306_082,
            "old_reaction": [],
            "new_reaction": [{ "type": "emoji", "emoji": "🌭" }]
        });
        serde_json::from_value(json).unwrap()
    }

    #[tokio::test]
    async fn message_reaction_handler_publishes_when_authorized() {
        let broker = mock_broker();
        let auth = auth_allowing(1459074222);
        let bot = Bot::new("unused");

        let result = message_reaction_handler(bot, reaction_updated(), broker.clone(), auth).await;
        assert!(result.is_ok());

        let published = broker.published();
        assert_eq!(published.len(), 1);

        let incoming: incoming::IncomingMessage = serde_json::from_slice(&published[0].1).unwrap();
        match incoming.message_type {
            incoming::IncomingMessageType::MessageReaction(data) => {
                assert_eq!(data.chat_id, -1002184233434);
                assert_eq!(data.message_id, 35);
                assert_eq!(data.new_reaction, vec!["🌭".to_string()]);
            }
            other => panic!("expected MessageReaction, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn message_reaction_handler_drops_unauthorized_reaction() {
        let broker = mock_broker();
        let auth = auth_denying_everyone();
        let bot = Bot::new("unused");

        let result = message_reaction_handler(bot, reaction_updated(), broker.clone(), auth).await;
        assert!(result.is_ok());

        assert!(broker.published().is_empty());
    }

    // --- message_handler / edited_message_handler: text-only fixture, no network ---

    #[tokio::test]
    async fn message_handler_publishes_authorized_text_message() {
        let broker = mock_broker();
        let auth = auth_allowing(777);
        let bot = Bot::new("unused");

        let result = message_handler(bot, text_message(777), broker.clone(), auth).await;
        assert!(result.is_ok());

        let published = broker.published();
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0.as_deref(), Some("777"));

        let incoming: incoming::IncomingMessage = serde_json::from_slice(&published[0].1).unwrap();
        match incoming.message_type {
            incoming::IncomingMessageType::TelegramMessage(data) => {
                assert_eq!(data.message.text(), Some("hello world"));
                assert!(data.file_attachments.is_empty());
            }
            other => panic!("expected TelegramMessage, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn message_handler_drops_unauthorized_text_message() {
        let broker = mock_broker();
        let auth = auth_denying_everyone();
        let bot = Bot::new("unused");

        let result = message_handler(bot, text_message(999), broker.clone(), auth).await;
        assert!(result.is_ok());

        assert!(broker.published().is_empty());
    }

    #[tokio::test]
    async fn message_handler_publishes_when_no_users_configured() {
        let broker = mock_broker();
        let auth = auth_open();
        let bot = Bot::new("unused");

        let result = message_handler(bot, text_message(123), broker.clone(), auth).await;
        assert!(result.is_ok());

        assert_eq!(broker.published().len(), 1);
    }

    #[tokio::test]
    async fn edited_message_handler_publishes_authorized_edit() {
        let broker = mock_broker();
        let auth = auth_allowing(777);
        let bot = Bot::new("unused");

        let result = edited_message_handler(bot, text_message(777), broker.clone(), auth).await;
        assert!(result.is_ok());

        assert_eq!(broker.published().len(), 1);
    }

    // --- callback_query_handler: unconditionally calls bot.answer_callback_query ---

    fn callback_query(user_id: u64) -> CallbackQuery {
        let json = serde_json::json!({
            "id": "query-id",
            "from": { "id": user_id, "is_bot": false, "first_name": "Alice", "username": "alice" },
            "message": text_message_json(user_id),
            "chat_instance": "instance",
            "data": "button:clicked"
        });
        serde_json::from_value(json).unwrap()
    }

    #[tokio::test]
    async fn callback_query_handler_answers_and_publishes_when_authorized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"(?i)^/bot[^/]+/answerCallbackQuery$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true, "result": true
            })))
            .mount(&server)
            .await;

        let broker = mock_broker();
        let auth = auth_allowing(777);
        let bot = test_bot(&server.uri());

        let result = callback_query_handler(bot, callback_query(777), broker.clone(), auth).await;
        assert!(result.is_ok());

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let published = broker.published();
        assert_eq!(published.len(), 1);

        let incoming: incoming::IncomingMessage = serde_json::from_slice(&published[0].1).unwrap();
        match incoming.message_type {
            incoming::IncomingMessageType::CallbackQuery(data) => {
                assert_eq!(data.callback_data, "button:clicked");
                assert_eq!(data.chat_id, 777);
            }
            other => panic!("expected CallbackQuery, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn callback_query_handler_denies_before_calling_bot() {
        // The auth gate runs before `bot.answer_callback_query`, so a denied
        // caller never needs a reachable Bot API - this points at an
        // unreachable address to prove that.
        let broker = mock_broker();
        let auth = auth_denying_everyone();
        let bot = test_bot("http://127.0.0.1:1");

        let result = callback_query_handler(bot, callback_query(999), broker.clone(), auth).await;
        assert!(result.is_ok());

        assert!(broker.published().is_empty());
    }
}
