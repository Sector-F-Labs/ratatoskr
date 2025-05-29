use crate::structs::{ButtonInfo, IncomingCallbackMessage, ImageInfo};
use teloxide::types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, PhotoSize};
use teloxide::prelude::Requester;
use teloxide::Bot;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use chrono::Utc;

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

pub fn prepare_incoming_callback_message(query: &CallbackQuery) -> IncomingCallbackMessage {
    let chat_id = query.message.as_ref().map_or(0, |m| m.chat().id.0);
    let user_id = query.from.id.0;
    let message_id = query.message.as_ref().map_or(0, |m| m.id().0);
    let callback_data = query.data.clone().unwrap_or_default();
    let callback_query_id = query.id.clone();

    IncomingCallbackMessage {
        chat_id,
        user_id,
        message_id,
        callback_data,
        callback_query_id,
    }
}

/// Downloads an image from Telegram and saves it to the specified directory
pub async fn download_image(
    bot: &Bot,
    photo: &PhotoSize,
    storage_dir: &str,
    chat_id: i64,
    message_id: i32,
) -> Result<ImageInfo, Box<dyn Error + Send + Sync>> {
    // Create storage directory if it doesn't exist
    fs::create_dir_all(storage_dir).await?;

    // Get file info from Telegram
    let file = bot.get_file(&photo.file.id).await?;
    let file_path = &file.path;
    
    // Generate local filename
    let extension = Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg");
    let filename = format!("{}_{}_{}_{}.{}", 
        chat_id, 
        message_id, 
        photo.file.unique_id, 
        Utc::now().timestamp(), 
        extension
    );
    let local_path = Path::new(storage_dir).join(&filename);

    // Download the file
    let download_url = format!("https://api.telegram.org/file/bot{}/{}", bot.token(), file_path);
    let response = reqwest::get(&download_url).await?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download image: HTTP {}", response.status()).into());
    }

    let content = response.bytes().await?;
    
    // Save to local filesystem
    let mut file_handle = fs::File::create(&local_path).await?;
    file_handle.write_all(&content).await?;
    file_handle.flush().await?;

    tracing::info!(
        file_id = %photo.file.id,
        local_path = %local_path.display(),
        file_size = %content.len(),
        "Image downloaded successfully"
    );

    Ok(ImageInfo {
        file_id: photo.file.id.clone(),
        file_unique_id: photo.file.unique_id.clone(),
        width: photo.width,
        height: photo.height,
        file_size: photo.file.size,
        local_path: local_path.to_string_lossy().to_string(),
    })
}

/// Selects the best quality photo from a vector of PhotoSize
pub fn select_best_photo(photos: &[PhotoSize]) -> Option<&PhotoSize> {
    photos.iter()
        .max_by(|a, b| {
            // Compare by dimensions since file.size is not optional in teloxide
            (a.width * a.height).cmp(&(b.width * b.height))
        })
}
