# Ratatoskr Type Definitions

This directory contains TypeScript definitions for all Ratatoskr message types, designed to work with [quicktype](https://quicktype.io/) for generating client types in multiple programming languages.

## Overview

Ratatoskr uses a unified message type system for consistent handling of all Kafka communications. The TypeScript definitions in `ratatoskr-types.ts` provide complete type safety and can be used to generate equivalent types in your preferred programming language using quicktype.

> **Note:** This directory only stores the canonical TypeScript type definitions. You are responsible for generating types for your target language using [quicktype](https://quicktype.io/) or a similar tool. No pre-generated types or build scripts are provided here.

## Usage

To use these types in your project, download or copy `ratatoskr-types.ts` and use [quicktype](https://quicktype.io/) to generate types for your preferred language. See the [quicktype documentation](https://quicktype.io/) for usage instructions and options.

Example quicktype commands:

```bash
# Python
yarn global add quicktype # or npm install -g quicktype
quicktype --src ratatoskr-types.ts --lang python --out ratatoskr_types.py

# Java
quicktype --src ratatoskr-types.ts --lang java --out RatatoskrTypes.java

# Go
quicktype --src ratatoskr-types.ts --lang go --out ratatoskr_types.go
```

For more options and supported languages, refer to the [quicktype documentation](https://quicktype.io/).

## Message Types Reference

### Incoming Messages (Telegram → Kafka IN topic)

All incoming messages are wrapped in the `IncomingMessage` type:

- **TelegramMessage**: Regular messages from users
- **CallbackQuery**: Button press interactions
- **MessageReaction**: Emoji reactions to messages

Example structure:
```json
{
  "message_type": {
    "type": "TelegramMessage",
    "data": { ... }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": null,
    "bot_username": null
  }
}
```

### Outgoing Messages (Kafka OUT topic → Telegram)

All outgoing messages are wrapped in the `OutgoingMessage` type:

- **TextMessage**: Send text with optional formatting and buttons
- **ImageMessage**: Send images from local filesystem
- **AudioMessage**: Send audio files
- **VoiceMessage**: Send voice messages
- **VideoMessage**: Send video files
- **VideoNoteMessage**: Send video notes (circular videos)
- **DocumentMessage**: Send documents/files
- **StickerMessage**: Send stickers
- **AnimationMessage**: Send GIFs/animations
- **EditMessage**: Edit previously sent messages
- **DeleteMessage**: Delete messages from chat

Example structure:
```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Hello!",
      "buttons": [[{"text": "Click me", "callback_data": "action_1"}]]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789
  }
}
```

## Using Generated Types

### Python Example

After generating Python types:

```python
import json
from ratatoskr_types import IncomingMessage, OutgoingMessage, TextMessageData, MessageTarget

# Parse incoming message
incoming_json = '{"message_type": {...}, "timestamp": "...", "source": {...}}'
incoming_msg = IncomingMessage.from_dict(json.loads(incoming_json))

# Create outgoing message
outgoing_msg = OutgoingMessage(
    message_type={"type": "TextMessage", "data": TextMessageData(
        text="Hello from Python!",
        buttons=[[{"text": "Button", "callback_data": "action"}]]
    )},
    timestamp="2023-12-01T10:30:00Z",
    target=MessageTarget(platform="telegram", chat_id=123456789)
)

# Serialize to JSON
outgoing_json = json.dumps(outgoing_msg.to_dict())
```

### Java Example

After generating Java types:

```java
import com.fasterxml.jackson.databind.ObjectMapper;

ObjectMapper mapper = new ObjectMapper();

// Parse incoming message
String incomingJson = "{...}";
IncomingMessage incomingMsg = mapper.readValue(incomingJson, IncomingMessage.class);

// Create outgoing message
OutgoingMessage outgoingMsg = new OutgoingMessage();
outgoingMsg.setMessageType(new OutgoingMessageType());
// ... set properties

// Serialize to JSON
String outgoingJson = mapper.writeValueAsString(outgoingMsg);
```

### Go Example

After generating Go types:

```go
import (
    "encoding/json"
    "time"
)

// Parse incoming message
var incomingMsg IncomingMessage
err := json.Unmarshal([]byte(incomingJSON), &incomingMsg)

// Create outgoing message
outgoingMsg := OutgoingMessage{
    MessageType: OutgoingMessageType{
        Type: "TextMessage",
        Data: TextMessageData{
            Text: "Hello from Go!",
            Buttons: [][]ButtonInfo{{
                {Text: "Button", CallbackData: "action"},
            }},
        },
    },
    Timestamp: time.Now().Format(time.RFC3339),
    Target: MessageTarget{
        Platform: "telegram",
        ChatID:   123456789,
    },
}

// Serialize to JSON
outgoingJSON, err := json.Marshal(outgoingMsg)
```

## Advanced Usage

### Custom Serialization

Some languages may require custom serialization logic for the union types. Quicktype handles most cases automatically, but you may need to adjust the generated code for specific requirements.

### Validation

Consider adding validation logic to ensure messages conform to Ratatoskr's expectations:

- Required fields are present
- Enum values are valid
- File paths exist (for outgoing media messages)
- Chat IDs are valid

### Error Handling

Always handle JSON parsing errors gracefully in your client code:

```python
try:
    message = IncomingMessage.from_dict(json.loads(json_string))
except (json.JSONDecodeError, ValueError) as e:
    print(f"Failed to parse message: {e}")
```

## Supported Languages

Quicktype supports generation for:

- **Strongly typed**: TypeScript, Java, C#, Kotlin, Swift, Go, Rust, C++
- **Dynamically typed**: Python, JavaScript, Ruby, Dart
- **Functional**: Elm, Haskell
- **Other**: JSON Schema, GraphQL, Protocol Buffers

## Troubleshooting

### Union Type Issues

If your target language has issues with the union types (`IncomingMessageType`, `OutgoingMessageType`), you can:

1. Use the `--union-style` flag with quicktype
2. Manually adjust the generated code
3. Create wrapper functions for type discrimination

### Large File Generation

For languages that generate very large files, consider:

1. Splitting types into multiple files manually
2. Using quicktype's `--top-level` flag to generate only specific types
3. Creating a build process that generates and organizes types

### Performance Considerations

For high-throughput applications:

1. Consider using streaming JSON parsers
2. Implement message pooling to reduce allocations
3. Use binary serialization formats like Protocol Buffers or MessagePack

## Contributing

If you find issues with the type definitions or have suggestions for improvements:

1. Check that your changes match the Rust source types
2. Test with quicktype generation for your target language
3. Update this documentation if needed
4. Submit a pull request

## Version Compatibility

These type definitions are versioned and should match your Ratatoskr installation:

- Type definitions version: `1.0.0`
- Compatible with Ratatoskr: `>= 1.0.0`

Always regenerate your client types when updating Ratatoskr to ensure compatibility.