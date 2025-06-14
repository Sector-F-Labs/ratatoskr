/**
 * Ratatoskr Message Types
 *
 * TypeScript definitions for all message types used in the Ratatoskr Telegram <-> Kafka bridge.
 * These types can be used with quicktype to generate client types in other languages.
 *
 * Usage with quicktype:
 * npx quicktype --src ratatoskr-types.ts --lang [target-language] --out types.[ext]
 *
 * Supported languages: typescript, javascript, python, java, csharp, go, rust, kotlin, swift, dart, etc.
 *
 * @version 1.0.0
 */

// =============================================================================
// INCOMING MESSAGE TYPES (Telegram -> Kafka IN topic)
// =============================================================================

/**
 * Main wrapper for all incoming messages from Telegram
 */
export interface IncomingMessage {
  message_type: IncomingMessageType;
  timestamp: string; // ISO 8601 datetime string
  source: MessageSource;
}

/**
 * Union type for different kinds of incoming messages
 */
export type IncomingMessageType =
  | { type: "TelegramMessage"; data: TelegramMessageData }
  | { type: "CallbackQuery"; data: CallbackQueryData }
  | { type: "MessageReaction"; data: MessageReactionData };

/**
 * Data for incoming Telegram messages
 */
export interface TelegramMessageData {
  /** The original Telegram message object */
  message: TelegramMessage;
  /** File attachments with download URLs - files are not downloaded yet */
  file_attachments: FileInfo[];
}

/**
 * Data for callback query interactions (button presses)
 */
export interface CallbackQueryData {
  chat_id: number;
  user_id: number;
  message_id: number;
  callback_data: string;
  callback_query_id: string;
}

/**
 * Data for message reaction events
 */
export interface MessageReactionData {
  chat_id: number;
  message_id: number;
  user_id?: number; // undefined if anonymous
  date: string; // ISO 8601 datetime string
  old_reaction: string[]; // emoji strings
  new_reaction: string[]; // emoji strings
}

/**
 * Information about the message source platform
 */
export interface MessageSource {
  platform: string; // "telegram"
  bot_id?: number;
  bot_username?: string;
}

// =============================================================================
// TELEGRAM MESSAGE STRUCTURE
// =============================================================================

/**
 * Telegram message object (simplified version of teloxide Message)
 */
export interface TelegramMessage {
  message_id: number;
  date: number; // Unix timestamp
  chat: TelegramChat;
  from?: TelegramUser;
  text?: string;
  caption?: string;
  photo?: TelegramPhotoSize[];
  audio?: TelegramAudio;
  voice?: TelegramVoice;
  video?: TelegramVideo;
  video_note?: TelegramVideoNote;
  document?: TelegramDocument;
  sticker?: TelegramSticker;
  animation?: TelegramAnimation;
  reply_to_message?: TelegramMessage;
  forward_from?: TelegramUser;
  forward_from_chat?: TelegramChat;
  forward_date?: number;
  edit_date?: number;
  media_group_id?: string;
  author_signature?: string;
  reply_markup?: TelegramInlineKeyboardMarkup;
}

export interface TelegramUser {
  id: number;
  is_bot: boolean;
  first_name: string;
  last_name?: string;
  username?: string;
  language_code?: string;
}

export interface TelegramChat {
  id: number;
  type: "private" | "group" | "supergroup" | "channel";
  title?: string;
  username?: string;
  first_name?: string;
  last_name?: string;
  description?: string;
  invite_link?: string;
  pinned_message?: TelegramMessage;
}

export interface TelegramPhotoSize {
  file_id: string;
  file_unique_id: string;
  width: number;
  height: number;
  file_size?: number;
}

export interface TelegramAudio {
  file_id: string;
  file_unique_id: string;
  duration: number;
  performer?: string;
  title?: string;
  file_name?: string;
  mime_type?: string;
  file_size?: number;
}

export interface TelegramVoice {
  file_id: string;
  file_unique_id: string;
  duration: number;
  mime_type?: string;
  file_size?: number;
}

export interface TelegramVideo {
  file_id: string;
  file_unique_id: string;
  width: number;
  height: number;
  duration: number;
  thumb?: TelegramPhotoSize;
  file_name?: string;
  mime_type?: string;
  file_size?: number;
}

export interface TelegramVideoNote {
  file_id: string;
  file_unique_id: string;
  length: number;
  duration: number;
  thumb?: TelegramPhotoSize;
  file_size?: number;
}

export interface TelegramDocument {
  file_id: string;
  file_unique_id: string;
  thumb?: TelegramPhotoSize;
  file_name?: string;
  mime_type?: string;
  file_size?: number;
}

export interface TelegramSticker {
  file_id: string;
  file_unique_id: string;
  width: number;
  height: number;
  is_animated: boolean;
  is_video: boolean;
  thumb?: TelegramPhotoSize;
  emoji?: string;
  set_name?: string;
  premium_animation?: TelegramFile;
  mask_position?: TelegramMaskPosition;
  custom_emoji_id?: string;
  needs_repainting?: boolean;
  file_size?: number;
}

export interface TelegramAnimation {
  file_id: string;
  file_unique_id: string;
  width: number;
  height: number;
  duration: number;
  thumb?: TelegramPhotoSize;
  file_name?: string;
  mime_type?: string;
  file_size?: number;
}

export interface TelegramFile {
  file_id: string;
  file_unique_id: string;
  file_size?: number;
  file_path?: string;
}

export interface TelegramMaskPosition {
  point: "forehead" | "eyes" | "mouth" | "chin";
  x_shift: number;
  y_shift: number;
  scale: number;
}

export interface TelegramInlineKeyboardMarkup {
  inline_keyboard: TelegramInlineKeyboardButton[][];
}

export interface TelegramInlineKeyboardButton {
  text: string;
  url?: string;
  login_url?: TelegramLoginUrl;
  callback_data?: string;
  switch_inline_query?: string;
  switch_inline_query_current_chat?: string;
  callback_game?: {};
  pay?: boolean;
}

export interface TelegramLoginUrl {
  url: string;
  forward_text?: string;
  bot_username?: string;
  request_write_access?: boolean;
}

// =============================================================================
// FILE INFORMATION
// =============================================================================

/**
 * Information about a file attached to a Telegram message
 */
export interface FileInfo {
  /** Telegram file identifier - can be used to download the file */
  file_id: string;
  /** Unique file identifier which is supposed to be the same over time and for different bots */
  file_unique_id: string;
  /** Type of the file */
  file_type: FileType;
  /** File size in bytes */
  file_size: number;
  /** Direct URL to download the file from Telegram servers */
  file_url: string;
  /** Additional file-specific metadata */
  metadata: FileMetadata;
}

export type FileType =
  | "Photo"
  | "Audio"
  | "Voice"
  | "Video"
  | "VideoNote"
  | "Document"
  | "Sticker"
  | "Animation";

export type FileMetadata =
  | { Photo: { width: number; height: number } }
  | { Audio: { duration: number; performer?: string; title?: string } }
  | { Voice: { duration: number } }
  | { Video: { width: number; height: number; duration: number } }
  | { VideoNote: { length: number; duration: number } }
  | { Document: { file_name?: string; mime_type?: string } }
  | { Sticker: { width: number; height: number; emoji?: string } }
  | { Animation: { width: number; height: number; duration: number } };

// =============================================================================
// OUTGOING MESSAGE TYPES (Kafka OUT topic -> Telegram)
// =============================================================================

/**
 * Main wrapper for all outgoing messages to Telegram
 */
export interface OutgoingMessage {
  message_type: OutgoingMessageType;
  timestamp: string; // ISO 8601 datetime string
  target: MessageTarget;
}

/**
 * Union type for different kinds of outgoing messages
 */
export type OutgoingMessageType =
  | { type: "TextMessage"; data: TextMessageData }
  | { type: "ImageMessage"; data: ImageMessageData }
  | { type: "AudioMessage"; data: AudioMessageData }
  | { type: "VoiceMessage"; data: VoiceMessageData }
  | { type: "VideoMessage"; data: VideoMessageData }
  | { type: "VideoNoteMessage"; data: VideoNoteMessageData }
  | { type: "DocumentMessage"; data: DocumentMessageData }
  | { type: "StickerMessage"; data: StickerMessageData }
  | { type: "AnimationMessage"; data: AnimationMessageData }
  | { type: "EditMessage"; data: EditMessageData }
  | { type: "DeleteMessage"; data: DeleteMessageData }
  | { type: "TypingMessage"; data: TypingMessageData };

/**
 * Target information for outgoing messages
 */
export interface MessageTarget {
  platform: string; // "telegram"
  chat_id: number;
  thread_id?: number; // For forum groups
}

// =============================================================================
// OUTGOING MESSAGE DATA TYPES
// =============================================================================

export interface TextMessageData {
  text: string;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
  parse_mode?: string; // "HTML", "Markdown", etc.
  disable_web_page_preview?: boolean;
}

export interface ImageMessageData {
  image_path: string;
  caption?: string;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface AudioMessageData {
  audio_path: string;
  caption?: string;
  duration?: number;
  performer?: string;
  title?: string;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface VoiceMessageData {
  voice_path: string;
  caption?: string;
  duration?: number;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface VideoMessageData {
  video_path: string;
  caption?: string;
  duration?: number;
  width?: number;
  height?: number;
  supports_streaming?: boolean;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface VideoNoteMessageData {
  video_note_path: string;
  duration?: number;
  length?: number;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface StickerMessageData {
  sticker_path: string;
  emoji?: string;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface AnimationMessageData {
  animation_path: string;
  caption?: string;
  duration?: number;
  width?: number;
  height?: number;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface DocumentMessageData {
  document_path: string;
  filename?: string;
  caption?: string;
  buttons?: ButtonInfo[][];
  reply_keyboard?: ReplyKeyboardMarkup;
}

export interface EditMessageData {
  message_id: number;
  new_text?: string;
  new_buttons?: ButtonInfo[][];
}

export interface DeleteMessageData {
  message_id: number;
}

export interface TypingMessageData {
  action?: "typing"; // For extensibility, could support other actions like "upload_photo"
}

// =============================================================================
// KEYBOARD AND BUTTON TYPES
// =============================================================================

export interface ButtonInfo {
  text: string;
  callback_data: string;
}

export interface ReplyKeyboardButton {
  text: string;
  request_contact?: boolean;
  request_location?: boolean;
  request_poll?: RequestPoll;
  web_app?: WebApp;
}

export interface RequestPoll {
  type?: string; // "quiz" or "regular"
}

export interface WebApp {
  url: string;
}

export interface ReplyKeyboardMarkup {
  keyboard: ReplyKeyboardButton[][];
  is_persistent?: boolean;
  resize_keyboard?: boolean;
  one_time_keyboard?: boolean;
  input_field_placeholder?: string;
  selective?: boolean;
}

// =============================================================================
// LEGACY TYPES (for backwards compatibility)
// =============================================================================

/**
 * @deprecated Use OutgoingMessage with TextMessageData instead
 */
export interface LegacyOutgoingMessage {
  chat_id: number;
  text: string;
  buttons?: ButtonInfo[][];
  parse_mode?: string;
  disable_web_page_preview?: boolean;
}

// =============================================================================
// UTILITY TYPES
// =============================================================================

/**
 * Image information for downloaded images (legacy)
 */
export interface ImageInfo {
  file_id: string;
  file_unique_id: string;
  width: number;
  height: number;
  file_size: number;
  local_path: string;
}

/**
 * Callback message (legacy)
 * @deprecated Use IncomingMessage with CallbackQueryData instead
 */
export interface IncomingCallbackMessage {
  chat_id: number;
  user_id: number;
  message_id: number;
  callback_data: string;
  callback_query_id: string;
}
