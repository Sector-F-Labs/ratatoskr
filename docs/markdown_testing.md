# Telegram Markdown Testing Guide

This document explains how to test the Telegram markdown formatting functionality in Ratatoskr.

## Overview

Telegram uses a strict subset of markdown called MarkdownV2, which requires specific character escaping and has particular rules for nested formatting. The `format_telegram_markdown()` function in `src/utils.rs` handles this conversion.

## Test Scripts

### 1. Simple Markdown Test (`test_simple_markdown`)

Quick test for basic markdown functionality:

```bash
# Test with default message containing common markdown elements
make test_simple_markdown

# Test with custom message
make test_simple_markdown TEXT="Your *markdown* message here"
```

**Use this for**: Quick validation during development

### 2. Complex Markdown Test (`test_markdown`)

Comprehensive test suite covering all markdown features:

```bash
make test_markdown
```

**Tests included**:
- Basic formatting (bold, italic, underline, strikethrough)
- Code blocks and inline code
- Special characters that need escaping
- Links and URLs
- Table-like structures
- Complex nested formatting
- Edge cases and problematic content

**Use this for**: Full feature validation

### 3. Edge Cases Test (`test_markdown_edge_cases`)

Focused on the most challenging MarkdownV2 scenarios:

```bash
make test_markdown_edge_cases
```

**Tests included**:
- All reserved characters: `_ * [ ] ( ) ~ ` > # + - = | { } . !`
- Consecutive special characters
- Mixed formatting with special chars
- Code blocks with all special characters
- Complex URLs and query parameters
- Mathematical expressions
- Programming code snippets
- File paths and system commands
- Markdown tables with special chars
- Deeply nested formatting
- Email addresses and mentions
- Escape sequences and edge cases
- Unicode and emoji with formatting
- Long text stress test

**Use this for**: Debugging specific escaping issues

## How to Interpret Results

### ✅ Success Indicators

- **All messages send**: No HTTP 400 errors from Telegram API
- **Formatting renders correctly**: Bold, italic, code formatting appears as expected
- **Special characters display properly**: Characters like `()[]{}` appear normally
- **Links are clickable**: URL links work and display correctly
- **Code blocks preserve formatting**: Monospace font and proper line breaks

### ❌ Failure Indicators

- **Messages fail to send**: HTTP 400 "Bad Request" errors
- **Broken formatting**: Text appears unformatted or garbled
- **Double-escaped characters**: Seeing `\\.` instead of `.`
- **Missing characters**: Characters disappeared due to over-escaping
- **Nested formatting conflicts**: Formatting breaks when combined

## MarkdownV2 Requirements

### Characters That MUST Be Escaped

Outside of code blocks, these characters must be escaped with backslash:

```
_ * [ ] ( ) ~ ` > # + - = | { } . !
```

### Characters That Should NOT Be Escaped

Inside code blocks (`code` or ```code blocks```), do NOT escape any characters.

### Special Cases

1. **Links**: Escape everything except the URL part
   ```
   [Escaped text with \. and \!](https://example.com/no-escaping-here)
   ```

2. **Nested formatting**: Order of operations matters
   ```
   *Bold with \_escaped underscore\_*
   ```

3. **Code spans**: Content inside backticks should not be escaped
   ```
   `{unescaped: "content", array: [1,2,3]}`
   ```

## Development Workflow

1. **Start with simple test**: Run `make test_simple_markdown` first
2. **Implement basic escaping**: Handle the required characters
3. **Test edge cases**: Run `make test_markdown_edge_cases`
4. **Fix specific issues**: Focus on failing test categories
5. **Full validation**: Run `make test_markdown` for complete coverage

## Common Implementation Pitfalls

### Over-escaping
```rust
// WRONG: Escaping inside code blocks
text.replace("`", r"\`");

// RIGHT: Don't escape inside code blocks
// Handle code blocks separately
```

### Under-escaping
```rust
// WRONG: Missing required escapes
text.replace(".", r"\."); // Only escapes dots, misses other chars

// RIGHT: Escape all required characters
for char in ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'] {
    // ... proper escaping logic
}
```

### Nested Formatting Issues
```rust
// WRONG: Escaping after applying formatting
let formatted = format!("*{}*", text);
escape_markdown(&formatted); // Breaks the formatting

// RIGHT: Escape content, then apply formatting
let escaped_content = escape_markdown(text);
format!("*{}*", escaped_content);
```

## Testing Environment Setup

Ensure these environment variables are set (copy from `.envrc.example`):

```bash
export CHAT_ID="your_telegram_chat_id"
export KAFKA_BROKER="localhost:9092"
export KAFKA_OUT_TOPIC="com.sectorflabs.ratatoskr.out"
```

## Debugging Tips

1. **Check Telegram API errors**: Look for specific error messages in logs
2. **Test incrementally**: Start with single character escaping
3. **Use the simple test**: Isolate issues with custom messages
4. **Compare with working examples**: Look at messages that render correctly
5. **Check nested formatting**: Test each combination separately

## Example Implementation Structure

```rust
pub fn format_telegram_markdown(text: &str) -> String {
    // 1. Identify code blocks and inline code spans
    // 2. Escape special characters OUTSIDE code blocks only
    // 3. Handle nested formatting carefully
    // 4. Preserve link formatting
    // 5. Return properly formatted text
}
```

## Running All Tests

```bash
# Run all markdown tests
make test_all_message_types

# Run just markdown-specific tests
make test_markdown test_simple_markdown test_markdown_edge_cases
```

## Need Help?

If tests are failing:
1. Check the specific error messages in Telegram
2. Run the simple test with isolated examples
3. Review the MarkdownV2 specification
4. Look at successful examples from other Telegram bots

The test scripts provide detailed output about what to look for and common issues to watch out for.