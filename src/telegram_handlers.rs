use rdkafka::producer::{FutureProducer, FutureRecord};
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::{Bot, CallbackQuery, Message, Requester};
use crate::utils::{download_image, select_best_photo};
use crate::structs::{ImageInfo, IncomingMessage, KafkaInTopic, ImageStorageDir};

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    producer: Arc<FutureProducer>,
    kafka_in_topic: KafkaInTopic,
    image_storage_dir: ImageStorageDir,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Handle image download if message contains photos
    let mut downloaded_images: Vec<ImageInfo> = Vec::new();
    if let Some(photos) = msg.photo() {
        if let Some(best_photo) = select_best_photo(photos) {
            tracing::info!(
                message_id = %msg.id.0, 
                chat_id = %msg.chat.id.0, 
                file_id = %best_photo.file.id,
                width = %best_photo.width,
                height = %best_photo.height,
                "Downloading image from Telegram message"
            );
            
            match download_image(&bot, best_photo, &image_storage_dir.0, msg.chat.id.0, msg.id.0).await {
                Ok(image_info) => {
                    tracing::info!(
                        message_id = %msg.id.0,
                        chat_id = %msg.chat.id.0,
                        local_path = %image_info.local_path,
                        "Image downloaded successfully"
                    );
                    downloaded_images.push(image_info);
                }
                Err(e) => {
                    tracing::error!(
                        message_id = %msg.id.0,
                        chat_id = %msg.chat.id.0,
                        file_id = %best_photo.file.id,
                        error = %e,
                        "Failed to download image"
                    );
                    // Continue processing even if image download fails
                }
            }
        }
    }

    // Create unified incoming message
    let incoming_msg = IncomingMessage::new_telegram_message(
        msg.clone(),
        downloaded_images.clone(),
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
        has_images = %(!downloaded_images.is_empty()),
        image_count = %downloaded_images.len(),
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
