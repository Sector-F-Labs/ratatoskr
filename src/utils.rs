use crate::incoming::{FileInfo, FileMetadata, FileType};
use crate::outgoing::{ButtonInfo, ReplyKeyboardMarkup};
use chrono::Utc;
use std::error::Error;
use std::path::Path;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::{
    Animation, Audio, Document, FileMeta, InlineKeyboardButton, InlineKeyboardMarkup, PhotoSize,
    Sticker, Video, VideoNote, Voice, KeyboardButton, KeyboardMarkup,
    KeyboardButtonPollType, ButtonRequest,
};
use tokio::fs;
use tokio::io::AsyncWriteExt;

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
        let keyboard_buttons: Vec<Vec<KeyboardButton>> = keyboard.keyboard.iter().map(|row| {
            row.iter().map(|button| {
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
            }).collect()
        }).collect();

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

/// Downloads a file from Telegram and saves it to the specified directory
pub async fn download_file(
    bot: &Bot,
    file: &FileMeta,
    file_type: FileType,
    metadata: FileMetadata,
    storage_dir: &str,
    chat_id: i64,
    message_id: i32,
) -> Result<FileInfo, Box<dyn Error + Send + Sync>> {
    // Create storage directory if it doesn't exist
    fs::create_dir_all(storage_dir).await?;

    // Get file info from Telegram
    let telegram_file = bot.get_file(&file.id).await?;
    let file_path = &telegram_file.path;

    // Generate local filename with appropriate extension
    let extension = match file_type {
        FileType::Photo => "jpg",
        FileType::Audio => "mp3",
        FileType::Voice => "ogg",
        FileType::Video => "mp4",
        FileType::VideoNote => "mp4",
        FileType::Document => {
            if let FileMetadata::Document { file_name, .. } = &metadata {
                if let Some(name) = file_name {
                    Path::new(name)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("bin")
                } else {
                    "bin"
                }
            } else {
                "bin"
            }
        }
        FileType::Sticker => "webp",
        FileType::Animation => "gif",
    };

    let filename = format!(
        "{}_{}_{}_{}_{}.{}",
        file_type_to_string(&file_type),
        chat_id,
        message_id,
        file.unique_id,
        Utc::now().timestamp(),
        extension
    );
    let local_path = Path::new(storage_dir).join(&filename);

    // Download the file
    let download_url = format!(
        "https://api.telegram.org/file/bot{}/{}",
        bot.token(),
        file_path
    );
    let response = reqwest::get(&download_url).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: HTTP {}", response.status()).into());
    }

    let content = response.bytes().await?;

    // Save to local filesystem
    let mut file_handle = fs::File::create(&local_path).await?;
    file_handle.write_all(&content).await?;
    file_handle.flush().await?;

    // Get absolute path for applications to use
    let absolute_path = local_path
        .canonicalize()
        .map_err(|e| format!("Failed to get absolute path: {}", e))?;

    tracing::info!(
        file_id = %file.id,
        file_type = %file_type_to_string(&file_type),
        local_path = %absolute_path.display(),
        file_size = %content.len(),
        "File downloaded successfully"
    );

    Ok(FileInfo {
        file_id: file.id.clone(),
        file_unique_id: file.unique_id.clone(),
        file_type,
        file_size: file.size,
        local_path: absolute_path.to_string_lossy().to_string(),
        metadata,
    })
}

/// Selects the best quality photo from a vector of PhotoSize
pub fn select_best_photo(photos: &[PhotoSize]) -> Option<&PhotoSize> {
    photos.iter().max_by(|a, b| {
        // Compare by dimensions since file.size is not optional in teloxide
        (a.width * a.height).cmp(&(b.width * b.height))
    })
}

/// Helper function to convert FileType to string for logging and filenames
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

/// Creates FileInfo from PhotoSize
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

/// Creates FileInfo from Audio
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

/// Creates FileInfo from Voice
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

/// Creates FileInfo from Video
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

/// Creates FileInfo from VideoNote
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

/// Creates FileInfo from Document
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

/// Creates FileInfo from Sticker
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

/// Creates FileInfo from Animation
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
