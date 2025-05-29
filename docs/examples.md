# Examples: Using Unified Message Types

This document provides practical examples of using Ratatoskr's unified message types for common scenarios.

## Example 1: Simple Text Message

Send a basic text message to a Telegram chat:

```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Welcome to our service! How can we help you today?",
      "buttons": null,
      "parse_mode": null,
      "disable_web_page_preview": null
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

## Example 2: Rich Text with HTML Formatting

Send a message with HTML formatting and inline buttons:

```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "<b>Order Confirmation</b>\n\nYour order #12345 has been confirmed!\n\n📦 <i>Estimated delivery:</i> Tomorrow, 2:00 PM\n💰 <i>Total:</i> $29.99",
      "buttons": [
        [
          {"text": "📍 Track Package", "callback_data": "track_12345"},
          {"text": "📞 Contact Support", "callback_data": "support"}
        ],
        [
          {"text": "📋 View Order Details", "callback_data": "details_12345"}
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

## Example 3: Send an Image with Caption

Send a locally stored image with a caption and interactive buttons:

```json
{
  "message_type": {
    "type": "ImageMessage",
    "data": {
      "image_path": "/absolute/path/to/var/ratatoskr/images/product_photo.jpg",
      "caption": "🌟 Featured Product: Premium Wireless Headphones\n\n✅ Noise cancellation\n✅ 30-hour battery life\n✅ Free shipping\n\n💸 Special price: $99.99",
      "buttons": [
        [
          {"text": "🛒 Buy Now", "callback_data": "buy_headphones_001"},
          {"text": "❤️ Add to Wishlist", "callback_data": "wishlist_headphones_001"}
        ],
        [
          {"text": "📋 More Details", "callback_data": "details_headphones_001"}
        ]
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

## Example 4: Send a Document

Send a PDF report or document file:

```json
{
  "message_type": {
    "type": "DocumentMessage",
    "data": {
      "document_path": "/absolute/path/to/var/ratatoskr/reports/monthly_report_nov_2023.pdf",
      "filename": "Monthly_Report_November_2023.pdf",
      "caption": "📊 Your monthly analytics report is ready!\n\nThis report includes:\n• User engagement metrics\n• Revenue summary\n• Growth analysis",
      "buttons": [
        [
          {"text": "📈 View Dashboard", "callback_data": "dashboard"},
          {"text": "📅 Schedule Meeting", "callback_data": "schedule_meeting"}
        ]
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

## Example 5: Edit an Existing Message

Update a previously sent message with new content and buttons:

```json
{
  "message_type": {
    "type": "EditMessage",
    "data": {
      "message_id": 42,
      "new_text": "🔄 <b>Status Updated</b>\n\nYour order #12345 is now out for delivery!\n\n🚚 <i>Tracking:</i> TRK789456123\n📍 <i>Current location:</i> Distribution Center\n⏰ <i>Estimated arrival:</i> 1-2 hours",
      "new_buttons": [
        [
          {"text": "🔄 Refresh Status", "callback_data": "refresh_12345"},
          {"text": "📞 Call Driver", "callback_data": "call_driver_12345"}
        ],
        [
          {"text": "📍 Live Tracking", "callback_data": "live_track_12345"}
        ]
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

## Example 6: Delete a Message

Remove a message from the chat:

```json
{
  "message_type": {
    "type": "DeleteMessage",
    "data": {
      "message_id": 38
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

## Example 7: Processing Incoming Telegram Messages

Here's what you receive when a user sends a message with an image:

```json
{
  "message_type": {
    "type": "TelegramMessage",
    "data": {
      "message": {
        "message_id": 156,
        "from": {
          "id": 987654321,
          "is_bot": false,
          "first_name": "John",
          "username": "john_doe",
          "language_code": "en"
        },
        "chat": {
          "id": 123456789,
          "first_name": "John",
          "username": "john_doe",
          "type": "private"
        },
        "date": 1701432600,
        "photo": [
          {
            "file_id": "AgACAgIAAxkDAAIBXGV...",
            "file_unique_id": "AQADyBUAAhoFqFNy",
            "width": 320,
            "height": 240,
            "file_size": 12543
          },
          {
            "file_id": "AgACAgIAAxkDAAIBXGV...",
            "file_unique_id": "AQADyBUAAhoFqFN-",
            "width": 1280,
            "height": 960,
            "file_size": 89032
          }
        ],
        "caption": "Look at this sunset!"
      },
      "downloaded_images": [
        {
          "file_id": "AgACAgIAAxkDAAIBXGV...",
          "file_unique_id": "AQADyBUAAhoFqFN-",
          "width": 1280,
          "height": 960,
          "file_size": 89032,
          "local_path": "/absolute/path/to/images/123456789_156_AQADyBUAAhoFqFN-_1701432600.jpg"
        }
      ]
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

## Example 8: Processing Incoming Button Clicks

When a user clicks a button, you receive a callback query:

```json
{
  "message_type": {
    "type": "CallbackQuery",
    "data": {
      "chat_id": 123456789,
      "user_id": 987654321,
      "message_id": 42,
      "callback_data": "buy_headphones_001",
      "callback_query_id": "1701432600123456789"
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

## Example 9: Multi-Step Conversation Flow

### Step 1: Initial Menu
```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "🏪 <b>Welcome to TechStore!</b>\n\nWhat would you like to do today?",
      "buttons": [
        [
          {"text": "🛒 Browse Products", "callback_data": "browse"},
          {"text": "📦 Track Orders", "callback_data": "track"}
        ],
        [
          {"text": "💬 Customer Support", "callback_data": "support"},
          {"text": "👤 My Account", "callback_data": "account"}
        ]
      ],
      "parse_mode": "HTML",
      "disable_web_page_preview": null
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

### Step 2: Product Categories (after user clicks "Browse Products")
```json
{
  "message_type": {
    "type": "EditMessage",
    "data": {
      "message_id": 42,
      "new_text": "📱 <b>Product Categories</b>\n\nChoose a category to explore:",
      "new_buttons": [
        [
          {"text": "📱 Smartphones", "callback_data": "cat_phones"},
          {"text": "💻 Laptops", "callback_data": "cat_laptops"}
        ],
        [
          {"text": "🎧 Audio", "callback_data": "cat_audio"},
          {"text": "⌚ Wearables", "callback_data": "cat_wearables"}
        ],
        [
          {"text": "⬅️ Back to Menu", "callback_data": "main_menu"}
        ]
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

## Example 10: Error Handling Response

When something goes wrong, provide helpful feedback:

```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "⚠️ <b>Oops! Something went wrong</b>\n\nWe couldn't process your request right now. This might be due to:\n\n• Temporary server maintenance\n• High traffic volume\n• Network connectivity issues\n\nPlease try again in a few moments, or contact our support team if the issue persists.",
      "buttons": [
        [
          {"text": "🔄 Try Again", "callback_data": "retry_last_action"},
          {"text": "💬 Contact Support", "callback_data": "support"}
        ],
        [
          {"text": "🏠 Back to Menu", "callback_data": "main_menu"}
        ]
      ],
      "parse_mode": "HTML",
      "disable_web_page_preview": null
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

## Best Practices

1. **Use descriptive callback_data**: Make button callbacks self-explanatory (`buy_product_123` instead of `b123`)

2. **Limit button rows**: Keep buttons organized in 1-3 rows for better user experience

3. **Include timestamps**: Always include proper timestamps for message tracking

4. **Handle file paths carefully**: Ensure image/document paths are absolute and accessible

5. **Use proper formatting**: Leverage HTML formatting for better message presentation

6. **Plan for errors**: Always provide fallback options and clear error messages

7. **Keep message IDs**: Store message IDs when you need to edit or delete messages later

8. **Validate file existence**: Check that files exist before sending ImageMessage or DocumentMessage

9. **Use absolute paths**: All file paths in messages are absolute paths for easy access by consuming applications

These examples should help you implement common Telegram bot interactions using Ratatoskr's unified message types.