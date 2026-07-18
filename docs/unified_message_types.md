# Unified Message Types Documentation

This document describes the unified message type system implemented in Ratatoskr for handling both incoming and outgoing messages through Kafka topics.

## Overview

Ratatoskr uses a single unified type system for all Kafka messages:
- **`IncomingMessage`** - All messages sent TO Kafka (from Telegram to your application, plus delivery confirmations for messages your application sent - see `MessageSent`), published to the `{prefix}.in` topic
- **`OutgoingMessage`** - All messages sent FROM Kafka (from your application to Telegram), consumed from the `{prefix}.out` topic

`{prefix}` defaults to `ratatoskr` and is configurable via the `KAFKA_TOPIC_PREFIX` environment variable.

## Incoming Messages (`{prefix}.in`)

All messages from Telegram are wrapped in the `IncomingMessage` type:

```json
{
  "trace_id": "b3b3b3b3-b3b3-b3b3-b3b3-b3b3b3b3b3b3",
  "message_type": {
    "type": "TelegramMessage",
    "data": {
      "message": { /* Full Telegram Message object */ },
      "file_attachments": [
        {
          "file_id": "AgACAgIAAxkDAAIC_mF...",
          "file_unique_id": "abc123def456",
          "file_type": "Photo",
          "file_size": 245760,
          "file_url": "https://api.telegram.org/file/bot<token>/photos/file_1.jpg",
          "metadata": { "type": "Photo", "width": 1920, "height": 1080 }
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

Files are **not** downloaded by Ratatoskr — `file_attachments` gives you the Telegram `file_id` and a direct `file_url` so your own handler can fetch the bytes if it needs them.

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
      "file_attachments": []
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

#### 4. EditedMessage
A previously sent message that the user edited

```json
{
  "message_type": {
    "type": "EditedMessage",
    "data": {
      "message": { /* Full Telegram Message object, with updated content */ },
      "file_attachments": [],
      "edit_date": 1678901300
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

#### 5. MessageSent
Confirms an `OutgoingMessage` was actually delivered, carrying the real Telegram `message_id` it
was sent as - producers of `OutgoingMessage` never otherwise learn that id, since sending happens
entirely on ratatoskr's side. Delivered on the IN topic rather than a separate one, since it flows
in the same direction (ratatoskr -> consumer) as everything else there. Unlike the other incoming
types, the envelope's `trace_id` here is the *outgoing* message's own `trace_id` (not freshly
generated), so a producer can correlate this back to whichever message it published. Currently only
emitted for `TextMessage` sends - not the other `OutgoingMessageType` variants.

```json
{
  "trace_id": "the outgoing message's own trace_id, not a new one",
  "message_type": {
    "type": "MessageSent",
    "data": {
      "chat_id": -1001234567890,
      "message_id": 456
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

## Outgoing Messages (`{prefix}.out`)

All messages to Telegram are wrapped in the `OutgoingMessage` type:

```json
{
  "trace_id": "b3b3b3b3-b3b3-b3b3-b3b3-b3b3b3b3b3b3",
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

`trace_id` is optional on the way in — if omitted, Ratatoskr generates one.

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

#### 2. ImageMessage
Send images stored on the local filesystem (the path must be reachable from where Ratatoskr runs)

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

#### 3. AudioMessage / VoiceMessage / VideoMessage / VideoNoteMessage / StickerMessage / AnimationMessage
Send other media types from the local filesystem, following the same `*_path` + optional `caption`/`buttons` shape as `ImageMessage`, plus type-specific fields (`duration`, `width`, `height`, `performer`, `title`, `supports_streaming`, `length`, `emoji` as applicable). See `src/kafka_processing/outgoing.rs` for the exact fields per type.

#### 4. DocumentMessage
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

#### 5. EditMessage
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

#### 6. DeleteMessage
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

#### 7. TypingMessage
Show the "typing..." indicator

```json
{
  "message_type": {
    "type": "TypingMessage",
    "data": {
      "action": "typing"
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

## Common Fields

### ButtonInfo
```json
{
  "text": "Button Label",
  "callback_data": "action_identifier"
}
```

### FileInfo (incoming file attachments)
```json
{
  "file_id": "AgACAgIAAxkDAAIC_mF...",
  "file_unique_id": "abc123def456",
  "file_type": "Photo",
  "file_size": 245760,
  "file_url": "https://api.telegram.org/file/bot<token>/photos/file_1.jpg",
  "metadata": { "type": "Photo", "width": 1920, "height": 1080 }
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
5. **Platform Agnostic** - Structure supports future platforms beyond Telegram

## Examples

### Processing Incoming Messages

```rust
match incoming_message.message_type {
    IncomingMessageType::TelegramMessage(data) => {
        // Handle regular message
        let telegram_msg = &data.message;
        let attachments = &data.file_attachments;
        // Process message...
    }
    IncomingMessageType::CallbackQuery(data) => {
        // Handle button click
        let callback_data = &data.callback_data;
        let user_id = data.user_id;
        // Process callback...
    }
    IncomingMessageType::MessageReaction(data) => {
        // Handle emoji reaction change
    }
    IncomingMessageType::EditedMessage(data) => {
        // Handle an edited message
    }
    IncomingMessageType::MessageSent(data) => {
        // Correlate incoming_message.trace_id back to the OutgoingMessage
        // you published with the same trace_id, and record data.message_id
        // as the real Telegram id it was sent as.
    }
}
```

### Creating Outgoing Messages

```rust
use ratatoskr::kafka_processing::outgoing::{
    ButtonInfo, MessageTarget, OutgoingMessage, OutgoingMessageType, TextMessageData,
};

let msg = OutgoingMessage {
    trace_id: uuid::Uuid::new_v4(),
    message_type: OutgoingMessageType::TextMessage(TextMessageData {
        text: "Choose an option:".to_string(),
        buttons: Some(vec![vec![
            ButtonInfo { text: "Option A".to_string(), callback_data: "opt_a".to_string() },
            ButtonInfo { text: "Option B".to_string(), callback_data: "opt_b".to_string() },
        ]]),
        reply_keyboard: None,
        parse_mode: None,
        disable_web_page_preview: None,
    }),
    timestamp: chrono::Utc::now(),
    target: MessageTarget {
        platform: "telegram".to_string(),
        chat_id,
        thread_id: None,
    },
};
```

Serialize `msg` to JSON and publish it to the `{prefix}.out` Kafka topic (or use `ratatoskr send`, see the main [README](../README.md)).
