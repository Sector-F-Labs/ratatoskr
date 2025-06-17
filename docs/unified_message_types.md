# Unified Message Types Documentation

This document describes the unified message type system implemented in Ratatoskr for handling both incoming and outgoing messages through Kafka topics.

## Overview

Ratatoskr now uses a single unified type system for all Kafka messages:
- **`IncomingMessage`** - All messages sent TO Kafka (from Telegram to your application)
- **`OutgoingMessage`** - All messages sent FROM Kafka (from your application to Telegram)

This replaces the previous multiple separate types and provides a consistent, extensible structure for all message handling.

## Incoming Messages (`KAFKA_IN_TOPIC`)

All messages from Telegram are wrapped in the `IncomingMessage` type:

```json
{
  "message_type": {
    "type": "TelegramMessage",
    "data": {
      "message": { /* Full Telegram Message object */ },
      "downloaded_images": [
        {
          "file_id": "AgACAgIAAxkDAAIC_mF...",
          "file_unique_id": "abc123def456",
          "width": 1920,
          "height": 1080,
          "file_size": 245760,
          "local_path": "/absolute/path/to/images/-123456789_42_abc123def456_1703123456.jpg"
        }
      ]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": 123456789,
    "bot_username": "my_bot"
  }
}
```

### Incoming Message Types

#### 1. TelegramMessage
Standard Telegram messages (text, photos, documents, etc.)

```json
{
  "message_type": {
    "type": "TelegramMessage",
    "data": {
      "message": {
        "message_id": 123,
        "from": { "id": 456, "first_name": "User", "username": "testuser" },
        "chat": { "id": 789, "type": "private" },
        "date": 1678901234,
        "text": "Hello bot!"
      },
      "downloaded_images": []
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": null,
    "bot_username": null
  }
}
```

#### 2. CallbackQuery
Button click events from inline keyboards

```json
{
  "message_type": {
    "type": "CallbackQuery",
    "data": {
      "chat_id": 123456789,
      "user_id": 987654321,
      "message_id": 54321,
      "callback_data": "action_1",
      "callback_query_id": "1234567890123456789"
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": null,
    "bot_username": null
  }
}
```

#### 3. MessageReaction
Emoji reactions that users add to or remove from messages

```json
{
  "message_type": {
    "type": "MessageReaction",
    "data": {
      "chat_id": 123456789,
      "message_id": 54321,
      "user_id": 987654321,
      "date": "2023-12-01T10:30:00Z",
      "old_reaction": ["👍"],
      "new_reaction": ["👍", "❤️"]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": null,
    "bot_username": null
  }
}
```

## Outgoing Messages (`KAFKA_OUT_TOPIC`)

All messages to Telegram are wrapped in the `OutgoingMessage` type:

```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Hello from your application!",
      "buttons": [
        [
          {"text": "Button 1", "callback_data": "action_1"},
          {"text": "Button 2", "callback_data": "action_2"}
        ]
      ],
      "parse_mode": "HTML",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
    "thread_id": null
  }
}
```

### Outgoing Message Types

#### 1. TextMessage
Send text messages with optional formatting and buttons

```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Hello! This supports <b>HTML</b> formatting.",
      "buttons": [
        [
          {"text": "Yes", "callback_data": "confirm_yes"},
          {"text": "No", "callback_data": "confirm_no"}
        ]
      ],
      "parse_mode": "HTML",
      "disable_web_page_preview": true
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
  "thread_id": null
  }
}
```

Longer texts are automatically split into multiple messages if they exceed 4096 characters.

#### 2. ImageMessage
Send images stored on the local filesystem

```json
{
  "message_type": {
    "type": "ImageMessage",
    "data": {
      "image_path": "/path/to/image.jpg",
      "caption": "Check out this image!",
      "buttons": [
        [{"text": "Like", "callback_data": "like_image"}]
      ]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
  "thread_id": null
  }
}
```

If a caption is longer than 1024 characters, the remaining text is sent as follow-up messages after the media.

#### 3. DocumentMessage
Send documents/files stored on the local filesystem

```json
{
  "message_type": {
    "type": "DocumentMessage",
    "data": {
      "document_path": "/path/to/document.pdf",
      "filename": "report.pdf",
      "caption": "Here's your requested report",
      "buttons": null
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
    "thread_id": null
  }
}
```

#### 4. EditMessage
Edit previously sent messages

```json
{
  "message_type": {
    "type": "EditMessage",
    "data": {
      "message_id": 42,
      "new_text": "This message has been updated!",
      "new_buttons": [
        [{"text": "Updated Button", "callback_data": "new_action"}]
      ]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
    "thread_id": null
  }
}
```

#### 5. DeleteMessage
Delete messages from the chat

```json
{
  "message_type": {
    "type": "DeleteMessage",
    "data": {
      "message_id": 42
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
    "thread_id": null
  }
}
```

## Backwards Compatibility

The old message formats are still supported for backwards compatibility:

### Legacy Outgoing Format (Deprecated)
```json
{
  "chat_id": 123456789,
  "text": "Hello from legacy format!",
  "buttons": [
    [
      {"text": "Button 1", "callback_data": "action_1"}
    ]
  ]
}
```

This will be automatically converted to the new `OutgoingMessage` format internally.

## Common Fields

### ButtonInfo
```json
{
  "text": "Button Label",
  "callback_data": "action_identifier"
}
```

### ImageInfo
```json
{
  "file_id": "AgACAgIAAxkDAAIC_mF...",
  "file_unique_id": "abc123def456",
  "width": 1920,
  "height": 1080,
  "file_size": 245760,
  "local_path": "/absolute/path/to/images/downloaded_image.jpg"
}
```

### MessageSource
```json
{
  "platform": "telegram",
  "bot_id": 123456789,
  "bot_username": "my_bot_username"
}
```

### MessageTarget
```json
{
  "platform": "telegram",
  "chat_id": 123456789,
  "thread_id": 456
}
```

## Benefits of Unified Types

1. **Consistency** - All messages follow the same structure
2. **Extensibility** - Easy to add new message types without breaking changes
3. **Type Safety** - Clear distinction between different message types
4. **Metadata** - Rich context information (timestamps, source/target info)
5. **Backwards Compatibility** - Legacy formats still work
6. **Platform Agnostic** - Structure supports future platforms beyond Telegram

## Migration Guide

### From Legacy Incoming Messages

**Old:**
```json
{
  "message_id": 123,
  "text": "Hello",
  "downloaded_images": [...]
}
```

**New:**
```json
{
  "message_type": {
    "type": "TelegramMessage",
    "data": {
      "message": { "message_id": 123, "text": "Hello", ... },
      "downloaded_images": [...]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": { "platform": "telegram", ... }
}
```

### From Legacy Outgoing Messages

**Old:**
```json
{
  "chat_id": 123,
  "text": "Hello",
  "buttons": [...]
}
```

**New:**
```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Hello",
      "buttons": [...],
      "parse_mode": null,
      "disable_web_page_preview": null
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": { "platform": "telegram", "chat_id": 123, "thread_id": null }
}
```

## Examples

### Processing Incoming Messages

```rust
match incoming_message.message_type {
    IncomingMessageType::TelegramMessage(data) => {
        // Handle regular message
        let telegram_msg = &data.message;
        let images = &data.downloaded_images;
        // Process message...
    }
    IncomingMessageType::CallbackQuery(data) => {
        // Handle button click
        let callback_data = &data.callback_data;
        let user_id = data.user_id;
        // Process callback...
    }
}
```

### Creating Outgoing Messages

```rust
// Text message with buttons
let msg = OutgoingMessage::new_text_message(
    chat_id,
    "Choose an option:".to_string(),
    Some(vec![
        vec![
            ButtonInfo { text: "Option A".to_string(), callback_data: "opt_a".to_string() },
            ButtonInfo { text: "Option B".to_string(), callback_data: "opt_b".to_string() }
        ]
    ])
);

// Send image
let img_msg = OutgoingMessage::new_image_message(
    chat_id,
    "/path/to/image.jpg".to_string(),
    Some("Beautiful sunset!".to_string()),
    None
);
```

This unified system provides a robust foundation for handling all types of Telegram interactions while maintaining backwards compatibility and enabling future enhancements.