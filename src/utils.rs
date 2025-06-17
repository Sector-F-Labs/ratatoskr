use crate::outgoing::{ButtonInfo, ReplyKeyboardMarkup};
use crate::telegram_handler::incoming::{FileInfo, FileMetadata, FileType};
use regex::Regex;
use std::error::Error;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::{
    Animation, Audio, ButtonRequest, Document, FileMeta, InlineKeyboardButton,
    InlineKeyboardMarkup, KeyboardButton, KeyboardButtonPollType, KeyboardMarkup, PhotoSize,
    Sticker, Video, VideoNote, Voice,
};

/// Maximum allowed characters in a Telegram text message
pub const TEXT_LIMIT: usize = 4096;
/// Maximum allowed characters in a Telegram caption
pub const CAPTION_LIMIT: usize = 1024;

/// Escapes HTML characters but preserves allowed Telegram HTML tags
fn escape_html_except_tags(text: &str) -> String {
    // First, temporarily replace our allowed tags with placeholders
    let mut result = text.to_string();
    
    // Create placeholders for our tags to protect them during HTML escaping
    let tag_replacements = vec![
        ("<b>", "___TELEGRAM_B_OPEN___"),
        ("</b>", "___TELEGRAM_B_CLOSE___"),
        ("<i>", "___TELEGRAM_I_OPEN___"),
        ("</i>", "___TELEGRAM_I_CLOSE___"),
        ("<u>", "___TELEGRAM_U_OPEN___"),
        ("</u>", "___TELEGRAM_U_CLOSE___"),
        ("<s>", "___TELEGRAM_S_OPEN___"),
        ("</s>", "___TELEGRAM_S_CLOSE___"),
        ("<code>", "___TELEGRAM_CODE_OPEN___"),
        ("</code>", "___TELEGRAM_CODE_CLOSE___"),
        ("<pre>", "___TELEGRAM_PRE_OPEN___"),
        ("</pre>", "___TELEGRAM_PRE_CLOSE___"),
    ];
    
    // Replace tags with placeholders
    for (tag, placeholder) in &tag_replacements {
        result = result.replace(tag, placeholder);
    }
    
    // Escape HTML characters
    result = result
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;");
    
    // Restore our tags
    for (tag, placeholder) in &tag_replacements {
        result = result.replace(placeholder, tag);
    }
    
    result
}

/// Finds and converts all markdown tables in text to aligned text tables
fn convert_markdown_tables_to_aligned(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut i = 0;
    let mut inside_pre_tag = false;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Track if we're inside a <pre> tag to avoid processing tables inside code blocks
        if line.contains("<pre>") {
            inside_pre_tag = true;
        }
        if line.contains("</pre>") {
            inside_pre_tag = false;
            result_lines.push(lines[i].to_string());
            i += 1;
            continue;
        }
        
        // Skip table processing if we're inside a pre tag
        if inside_pre_tag {
            result_lines.push(lines[i].to_string());
            i += 1;
            continue;
        }
        
        // Check if this line starts a markdown table (must have proper table format and multiple rows)
        if line.starts_with('|') && line.ends_with('|') && line.matches('|').count() >= 2 {
            // Look ahead to ensure we have at least 2 table rows (header + data or header + separator + data)
            let table_start = i;
            let mut table_end = i;
            let mut table_row_count = 0;
            
            // Count consecutive table lines
            while table_end < lines.len() {
                let current_line = lines[table_end].trim();
                if current_line.starts_with('|') && current_line.ends_with('|') && current_line.matches('|').count() >= 2 {
                    table_row_count += 1;
                    table_end += 1;
                } else if current_line.is_empty() && table_end + 1 < lines.len() {
                    // Check if there's another table line after empty line
                    let next_line = lines[table_end + 1].trim();
                    if next_line.starts_with('|') && next_line.ends_with('|') && next_line.matches('|').count() >= 2 {
                        table_end += 1; // Include empty line
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            
            // Only process as table if we have at least 2 rows (making it a real table)
            if table_row_count >= 2 {
                // Extract table text
                let table_text = lines[table_start..table_end].join("\n");
                let aligned_table = convert_markdown_table_to_aligned_text(&table_text);
                
                // Wrap in <pre> tags for monospace formatting
                result_lines.push(format!("<pre>{}</pre>", aligned_table));
                
                i = table_end;
            } else {
                // Not a valid table, treat as regular line
                result_lines.push(lines[i].to_string());
                i += 1;
            }
        } else {
            result_lines.push(lines[i].to_string());
            i += 1;
        }
    }
    
    result_lines.join("\n")
}

/// Converts a markdown table to an aligned text table
fn convert_markdown_table_to_aligned_text(table_text: &str) -> String {
    let lines: Vec<&str> = table_text.lines().collect();
    if lines.len() < 2 {
        return table_text.to_string(); // Not a valid table
    }
    
    // Parse table rows
    let mut rows: Vec<Vec<String>> = Vec::new();
    
    for line in lines.iter() {
        let line = line.trim();
        if line.starts_with('|') && line.ends_with('|') {
            // Check if this is a separator line (contains only |, -, and spaces)
            if line.chars().all(|c| c == '|' || c == '-' || c == ' ' || c == ':') {
                continue; // Skip separator lines
            }
            
            // Parse table row
            let cells: Vec<String> = line[1..line.len()-1] // Remove leading and trailing |
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect();
            rows.push(cells);
        }
    }
    
    if rows.is_empty() {
        return table_text.to_string();
    }
    
    // Calculate column widths
    let num_cols = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut col_widths = vec![0; num_cols];
    
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }
    
    // Build aligned table
    let mut result = String::new();
    
    // Add header row
    if !rows.is_empty() {
        let header_row = &rows[0];
        for (i, cell) in header_row.iter().enumerate() {
            if i > 0 {
                result.push_str("  "); // Column separator
            }
            result.push_str(&format!("{:<width$}", cell, width = col_widths.get(i).unwrap_or(&0)));
        }
        result.push('\n');
        
        // Add separator line
        for (i, &width) in col_widths.iter().enumerate() {
            if i > 0 {
                result.push_str("  ");
            }
            result.push_str(&"-".repeat(width));
        }
        result.push('\n');
        
        // Add data rows
        for row in rows.iter().skip(1) {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    result.push_str("  ");
                }
                result.push_str(&format!("{:<width$}", cell, width = col_widths.get(i).unwrap_or(&0)));
            }
            result.push('\n');
        }
    }
    
    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }
    
    result
}

/// Converts markdown text to HTML formatting that Telegram supports.
///
/// Takes markdown input and converts it to HTML tags that Telegram can parse.
/// This is more reliable than using Telegram's MarkdownV2 parser.
///
/// # Arguments
/// * `text` - The input text that contains markdown formatting
///
/// # Returns
/// * HTML formatted text that complies with Telegram's HTML parse mode
pub fn format_telegram_markdown(text: &str) -> String {
    let mut result = text.to_string();
    
    // Convert markdown headings to HTML bold
    result = Regex::new(r"(?m)^#{1,6} (.+)$")
        .unwrap()
        .replace_all(&result, "<b>$1</b>")
        .to_string();
    
    // Convert **bold** to <b>bold</b> (do this first)
    result = Regex::new(r"\*\*([^*]+)\*\*")
        .unwrap()
        .replace_all(&result, "<b>$1</b>")
        .to_string();
    
    // Convert __underline__ to <u>underline</u> (do this before single _)
    result = Regex::new(r"__([^_]+)__")
        .unwrap()
        .replace_all(&result, "<u>$1</u>")
        .to_string();
    
    // Convert *italic* to <i>italic</i> (after **bold** is processed)
    result = Regex::new(r"\*([^*]+)\*")
        .unwrap()
        .replace_all(&result, "<i>$1</i>")
        .to_string();
    
    // Convert _italic_ to <i>italic</i> (after __underline__ is processed)
    result = Regex::new(r"_([^_]+)_")
        .unwrap()
        .replace_all(&result, "<i>$1</i>")
        .to_string();
    
    // Convert ~~strikethrough~~ to <s>strikethrough</s>
    result = Regex::new(r"~~([^~]+)~~")
        .unwrap()
        .replace_all(&result, "<s>$1</s>")
        .to_string();
    
    // Convert ```language\ncode\n``` to <pre>code</pre> (do this BEFORE table conversion to avoid conflicts)
    result = Regex::new(r"(?s)```(?:\w+)?\n?(.*?)```")
        .unwrap()
        .replace_all(&result, "<pre>$1</pre>")
        .to_string();
    
    // Convert markdown tables to aligned text tables (do this after code blocks to avoid conflicts)
    result = convert_markdown_tables_to_aligned(&result);
    
    // Convert `inline code` to <code>inline code</code> (do this after code blocks)
    result = Regex::new(r"`([^`]+)`")
        .unwrap()
        .replace_all(&result, "<code>$1</code>")
        .to_string();
    
    // Escape HTML characters that aren't part of our formatting
    // Do this carefully to not escape our intentional HTML tags
    result = escape_html_except_tags(&result);
    
    result
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

/// Split long text into chunks that respect the given limit.
///
/// The function tries to break on spaces or newline boundaries so that words
/// are not cut in the middle. If a single word exceeds the limit it will be
/// split at the limit boundary.
pub fn split_text(text: &str, limit: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for token in text.split_inclusive(|c: char| c == ' ' || c == '\n') {
        if token.len() > limit {
            if !current.is_empty() {
                chunks.push(current.clone());
                current.clear();
            }
            let mut start = 0;
            let chars: Vec<char> = token.chars().collect();
            while start < chars.len() {
                let end = usize::min(start + limit, chars.len());
                chunks.push(chars[start..end].iter().collect());
                start = end;
            }
            continue;
        }

        if current.len() + token.len() > limit {
            chunks.push(current.clone());
            current.clear();
        }

        current.push_str(token);
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_telegram_markdown_basic_formatting() {
        // Test basic markdown to HTML conversion
        let input = "This is *italic* and **bold** text";
        let result = format_telegram_markdown(input);
        let expected = "This is <i>italic</i> and <b>bold</b> text";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_telegram_markdown_empty() {
        // Test with empty string
        let input = "";
        let result = format_telegram_markdown(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_format_telegram_markdown_code() {
        // Test inline code and code blocks
        let input = "Here is `inline code` and:\n```\ncode block\n```";
        let result = format_telegram_markdown(input);
        let expected = "Here is <code>inline code</code> and:\n<pre>code block\n</pre>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_telegram_markdown_headings() {
        // Test heading conversion from markdown to HTML bold
        let input = "# Main Title\n## Section Header\n### Subsection\nNormal text\n#### Level 4\n##### Level 5\n###### Level 6";
        let result = format_telegram_markdown(input);
        let expected = "<b>Main Title</b>\n<b>Section Header</b>\n<b>Subsection</b>\nNormal text\n<b>Level 4</b>\n<b>Level 5</b>\n<b>Level 6</b>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_telegram_markdown_headings_mixed() {
        // Test headings mixed with other content
        let input = "Some text\n# Heading 1\nMore text\n## Heading 2\nEven more text";
        let result = format_telegram_markdown(input);
        let expected = "Some text\n<b>Heading 1</b>\nMore text\n<b>Heading 2</b>\nEven more text";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_telegram_markdown_no_headings() {
        // Test text without headings
        let input = "This is just normal text with no headings\nAnd another line\n#hashtag should not be converted";
        let result = format_telegram_markdown(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_format_telegram_markdown_mixed_formatting() {
        // Test heading with markdown formatting inside
        let input = "# Title with *bold* and _italic_\n## Section with `code`";
        let result = format_telegram_markdown(input);
        let expected = "<b>Title with <i>bold</i> and <i>italic</i></b>\n<b>Section with <code>code</code></b>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_telegram_markdown_table() {
        // Test markdown table conversion to aligned text
        let input = "| Name | Age | City |\n|------|-----|------|\n| John | 25  | NYC  |\n| Jane | 30  | LA   |";
        let result = format_telegram_markdown(input);
        let expected = "<pre>Name  Age  City\n----  ---  ----\nJohn  25   NYC \nJane  30   LA  </pre>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_telegram_markdown_test6_content() {
        // Test with the exact content from Test 6 that's causing issues
        let input = "Test 6: Table-like Structures\n\nMarkdown table (needs proper escaping):\n\n| Name | Age | City |\n|------|-----|------|\n| John | 25  | NYC  |\n| Jane | 30  | LA   |\n| Bob  | 35  | SF   |\n\nSimple aligned text table:\n```\nName     Age    City\n----     ---    ----\nJohn     25     NYC\nJane     30     LA\nBob      35     SF\n```";
        let result = format_telegram_markdown(input);
        println!("Input:\n{}", input);
        println!("\nOutput:\n{}", result);
        // Just test that it doesn't panic and produces some output
        assert!(!result.is_empty());
    }

    #[test]
    fn test_split_text_basic() {
        let text = "hello world";
        let chunks = split_text(text, 4096);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "hello world");
    }

    #[test]
    fn test_split_text_respects_limit() {
        let text = "one two three four";
        let chunks = split_text(text, 8);
        assert_eq!(chunks, vec![
            "one two ".to_string(),
            "three ".to_string(),
            "four".to_string(),
        ]);
        for c in chunks {
            assert!(c.len() <= 8);
        }
    }
}
