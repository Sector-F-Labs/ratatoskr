use crate::structs::{OutgoingMessage, OutgoingMessageType, OutgoingKafkaMessage};
use crate::utils::create_markup;
use futures_util::StreamExt;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::Message as KafkaMessageRd;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::{Bot, ChatId, Requester},
    types::{InputFile, ParseMode},
};
use std::path::Path;

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
                    // Try new unified message format first
                    match serde_json::from_slice::<OutgoingMessage>(payload) {
                        Ok(out_msg) => {
                            if let Err(e) = handle_outgoing_message(&bot_consumer_clone, out_msg).await {
                                tracing::error!(topic = %kafka_msg.topic(), error = ?e, "Error handling OutgoingMessage");
                            }
                        }
                        Err(_) => {
                            // Fall back to legacy format for backwards compatibility
                            match serde_json::from_slice::<OutgoingKafkaMessage>(payload) {
                                Ok(legacy_msg) => {
                                    let chat_id = ChatId(legacy_msg.chat_id);
                                    tracing::info!(%chat_id, text_length = %legacy_msg.text.len(), has_buttons = %legacy_msg.buttons.is_some(), "Sending legacy message to Telegram");
                                    let mut msg_to_send =
                                        bot_consumer_clone.send_message(chat_id, legacy_msg.text.clone());
                                    if let Some(markup) = create_markup(&legacy_msg.buttons) {
                                        msg_to_send = msg_to_send.reply_markup(markup);
                                    }
                                    if let Err(e) = msg_to_send.await {
                                        tracing::error!(%chat_id, error = ?e, "Error sending legacy message to Telegram");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(topic = %kafka_msg.topic(), error = %e, "Error deserializing message from Kafka payload");
                                    tracing::debug!(raw_payload = ?String::from_utf8_lossy(payload), "Problematic Kafka payload");
                                }
                            }
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = ChatId(message.target.chat_id);
    
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
            
            msg_to_send.await?;
        }
        
        OutgoingMessageType::ImageMessage(data) => {
            tracing::info!(%chat_id, image_path = %data.image_path, has_caption = %data.caption.is_some(), "Sending image message to Telegram");
            
            if !Path::new(&data.image_path).exists() {
                return Err(format!("Image file not found: {}", data.image_path).into());
            }
            
            let input_file = InputFile::file(&data.image_path);
            bot.send_photo(chat_id, input_file).await?;
        }
        
        OutgoingMessageType::DocumentMessage(data) => {
            tracing::info!(%chat_id, document_path = %data.document_path, has_caption = %data.caption.is_some(), "Sending document message to Telegram");
            
            if !Path::new(&data.document_path).exists() {
                return Err(format!("Document file not found: {}", data.document_path).into());
            }
            
            let input_file = InputFile::file(&data.document_path);
            bot.send_document(chat_id, input_file).await?;
        }
        
        OutgoingMessageType::EditMessage(data) => {
            tracing::info!(%chat_id, message_id = %data.message_id, has_new_text = %data.new_text.is_some(), "Editing message in Telegram");
            
            if let Some(new_text) = data.new_text {
                bot.edit_message_text(chat_id, teloxide::types::MessageId(data.message_id), new_text).await?;
            }
        }
        
        OutgoingMessageType::DeleteMessage(data) => {
            tracing::info!(%chat_id, message_id = %data.message_id, "Deleting message in Telegram");
            bot.delete_message(chat_id, teloxide::types::MessageId(data.message_id)).await?;
        }
    }
    
    Ok(())
}
