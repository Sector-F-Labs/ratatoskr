use crate::outgoing::{ButtonInfo, ReplyKeyboardMarkup};
use crate::telegram_handler::incoming::{FileInfo, FileMetadata, FileType};
use std::error::Error;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::{
    Animation, Audio, ButtonRequest, Document, FileMeta, InlineKeyboardButton,
    InlineKeyboardMarkup, KeyboardButton, KeyboardButtonPollType, KeyboardMarkup, PhotoSize,
    Sticker, Video, VideoNote, Voice,
};

/// Formats text to comply with Telegram's markdown formatting requirements.
///
/// Telegram uses a strict subset of markdown, and certain characters need to be
/// escaped or handled specially. This function ensures that the text will be
/// properly formatted for Telegram's MarkdownV2 parse mode.
///
/// # Arguments
/// * `text` - The input text that may contain markdown formatting
///
/// # Returns
/// * Formatted text that complies with Telegram's markdown requirements
///
/// # Note
/// Currently returns input text unchanged - implementation to be completed later.
pub fn format_telegram_markdown(text: &str) -> String {
    // TODO: Implement Telegram markdown formatting
    // This should handle:
    // - Escaping special characters: _*[]()~`>#+-=|{}.!
    // - Converting standard markdown to Telegram MarkdownV2 format
    // - Handling nested formatting correctly
    // - Preserving valid Telegram markdown while escaping invalid sequences

    // Convert hadings to bold, so # Heading becomes **Heading**
    let text = text.replace("# ", "**");

    text.to_string()
}

pub fn create_markup(buttons_opt: &Option<Vec<Vec<ButtonInfo>>>) -> Option<InlineKeyboardMarkup> {
    buttons_opt.as_ref().map(|buttons| {
        InlineKeyboardMarkup::new(buttons.iter().map(|row| {
            row.iter().map(|button_info| {
                InlineKeyboardButton::callback(
                    button_info.text.clone(),
                    button_info.callback_data.clone(),
                )
            })
        }))
    })
}

pub fn create_reply_keyboard(keyboard_opt: &Option<ReplyKeyboardMarkup>) -> Option<KeyboardMarkup> {
    keyboard_opt.as_ref().map(|keyboard| {
        let keyboard_buttons: Vec<Vec<KeyboardButton>> = keyboard
            .keyboard
            .iter()
            .map(|row| {
                row.iter()
                    .map(|button| {
                        let mut kb_button = KeyboardButton::new(button.text.clone());

                        if let Some(true) = button.request_contact {
                            kb_button = kb_button.request(ButtonRequest::Contact);
                        }

                        if let Some(true) = button.request_location {
                            kb_button = kb_button.request(ButtonRequest::Location);
                        }

                        if let Some(poll) = &button.request_poll {
                            let poll_type = match poll.poll_type.as_deref() {
                                Some("quiz") => KeyboardButtonPollType::Quiz,
                                Some("regular") => KeyboardButtonPollType::Regular,
                                _ => KeyboardButtonPollType::Regular,
                            };
                            kb_button = kb_button.request(ButtonRequest::Poll(poll_type));
                        }

                        if let Some(_web_app) = &button.web_app {
                            // Note: WebApp functionality requires additional setup
                            // For now, we'll skip web app buttons
                        }

                        kb_button
                    })
                    .collect()
            })
            .collect();

        let mut reply_keyboard = KeyboardMarkup::new(keyboard_buttons);

        if let Some(resize) = keyboard.resize_keyboard {
            if resize {
                reply_keyboard = reply_keyboard.resize_keyboard();
            }
        }

        if let Some(one_time) = keyboard.one_time_keyboard {
            if one_time {
                reply_keyboard = reply_keyboard.one_time_keyboard();
            }
        }

        if let Some(persistent) = keyboard.is_persistent {
            if persistent {
                reply_keyboard = reply_keyboard.persistent();
            }
        }

        if let Some(placeholder) = &keyboard.input_field_placeholder {
            reply_keyboard = reply_keyboard.input_field_placeholder(placeholder.clone());
        }

        if let Some(selective) = keyboard.selective {
            if selective {
                reply_keyboard = reply_keyboard.selective();
            }
        }

        reply_keyboard
    })
}

pub async fn get_file_info(
    bot: &Bot,
    file: &FileMeta,
    file_type: FileType,
    metadata: FileMetadata,
) -> Result<FileInfo, Box<dyn Error + Send + Sync>> {
    // Get file info from Telegram
    let telegram_file = bot.get_file(&file.id).await?;
    let file_path = &telegram_file.path;

    // Generate the Telegram file URL
    let telegram_file_url = format!(
        "https://api.telegram.org/file/bot{}/{}",
        bot.token(),
        file_path
    );

    tracing::info!(
        file_id = %file.id,
        file_type = %file_type_to_string(&file_type),
        telegram_path = %file_path,
        telegram_url = %telegram_file_url,
        "File info obtained from Telegram"
    );

    Ok(FileInfo {
        file_id: file.id.clone(),
        file_unique_id: file.unique_id.clone(),
        file_type,
        file_size: file.size,
        file_url: telegram_file_url, // Using telegram URL instead of local path
        metadata,
    })
}

pub fn select_best_photo(photos: &[PhotoSize]) -> Option<&PhotoSize> {
    photos.iter().max_by(|a, b| {
        // Compare by dimensions since file.size is not optional in teloxide
        (a.width * a.height).cmp(&(b.width * b.height))
    })
}

fn file_type_to_string(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Photo => "photo",
        FileType::Audio => "audio",
        FileType::Voice => "voice",
        FileType::Video => "video",
        FileType::VideoNote => "videonote",
        FileType::Document => "document",
        FileType::Sticker => "sticker",
        FileType::Animation => "animation",
    }
}

pub fn file_info_from_photo(photo: &PhotoSize) -> (FileMeta, FileType, FileMetadata) {
    (
        photo.file.clone(),
        FileType::Photo,
        FileMetadata::Photo {
            width: photo.width,
            height: photo.height,
        },
    )
}

pub fn file_info_from_audio(audio: &Audio) -> (FileMeta, FileType, FileMetadata) {
    (
        audio.file.clone(),
        FileType::Audio,
        FileMetadata::Audio {
            duration: serde_json::to_value(&audio.duration)
                .ok()
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(0),
            performer: audio.performer.clone(),
            title: audio.title.clone(),
        },
    )
}

pub fn file_info_from_voice(voice: &Voice) -> (FileMeta, FileType, FileMetadata) {
    (
        voice.file.clone(),
        FileType::Voice,
        FileMetadata::Voice {
            duration: serde_json::to_value(&voice.duration)
                .ok()
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(0),
        },
    )
}

pub fn file_info_from_video(video: &Video) -> (FileMeta, FileType, FileMetadata) {
    (
        video.file.clone(),
        FileType::Video,
        FileMetadata::Video {
            width: video.width,
            height: video.height,
            duration: serde_json::to_value(&video.duration)
                .ok()
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(0),
        },
    )
}

pub fn file_info_from_video_note(video_note: &VideoNote) -> (FileMeta, FileType, FileMetadata) {
    (
        video_note.file.clone(),
        FileType::VideoNote,
        FileMetadata::VideoNote {
            length: video_note.length,
            duration: serde_json::to_value(&video_note.duration)
                .ok()
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(0),
        },
    )
}

pub fn file_info_from_document(document: &Document) -> (FileMeta, FileType, FileMetadata) {
    (
        document.file.clone(),
        FileType::Document,
        FileMetadata::Document {
            file_name: document.file_name.clone(),
            mime_type: document.mime_type.as_ref().map(|m| m.to_string()),
        },
    )
}

pub fn file_info_from_sticker(sticker: &Sticker) -> (FileMeta, FileType, FileMetadata) {
    (
        sticker.file.clone(),
        FileType::Sticker,
        FileMetadata::Sticker {
            width: sticker.width as u32,
            height: sticker.height as u32,
            emoji: sticker.emoji.clone(),
        },
    )
}

pub fn file_info_from_animation(animation: &Animation) -> (FileMeta, FileType, FileMetadata) {
    (
        animation.file.clone(),
        FileType::Animation,
        FileMetadata::Animation {
            width: animation.width,
            height: animation.height,
            duration: serde_json::to_value(&animation.duration)
                .ok()
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(0),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_telegram_markdown_stub() {
        // Test the stub implementation - should return input unchanged
        let input = "This is a *test* message with _markdown_ formatting";
        let result = format_telegram_markdown(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_format_telegram_markdown_empty() {
        // Test with empty string
        let input = "";
        let result = format_telegram_markdown(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_format_telegram_markdown_special_chars() {
        // Test with Telegram special characters that need escaping
        let input = "Text with _*[]()~`>#+-=|{}.! special chars";
        let result = format_telegram_markdown(input);
        // Currently should return unchanged since it's a stub
        assert_eq!(result, input);
    }
}
