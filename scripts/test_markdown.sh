#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

echo "Testing complex Telegram markdown formatting..."
echo "=============================================="
echo "This script tests various markdown elements that need proper escaping for Telegram MarkdownV2"

# Test 1: Basic formatting (bold, italic, underline, strikethrough)
echo -e "\n1. Testing basic formatting:"
TMP_FILE1=$(mktemp)
TRACE_ID1=$(uuidgen | tr '[:upper:]' '[:lower:]')
TIMESTAMP1=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
cat > "$TMP_FILE1" << EOF
{
  "trace_id": "$TRACE_ID1",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 1: Basic Formatting\n\n*Bold text* should be bold\n_Italic text_ should be italic\n__Underlined text__ should be underlined\n~Strikethrough text~ should be crossed out\n\nMixed: *bold _italic* text_ and __*bold underlined*__",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "$TIMESTAMP1",
  "target": {
    "platform": "telegram",
    "chat_id": $CHAT_ID,
    "thread_id": null
  }
}
EOF

echo "$CHAT_ID:$(cat "$TMP_FILE1" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE1"
sleep 3

# Test 2: Headings (markdown headings - # characters need proper escaping)
echo -e "\n2. Testing markdown headings:"
TMP_FILE2=$(mktemp)
cat > "$TMP_FILE2" << 'EOF'
{
  "trace_id": "TRACE_ID_PLACEHOLDER",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 2: Markdown Headings\n\nLevel 1 heading:\n# Main Title\n\nLevel 2 heading:\n## Section Header\n\nLevel 3 heading:\n### Subsection\n\nLevel 4 heading:\n#### Sub-subsection\n\nLevel 5 heading:\n##### Deep Section\n\nLevel 6 heading:\n###### Deepest Section\n\nHeadings with special characters:\n# Title with *bold* and _italic_\n## Section with `code` and [links](https://example.com)\n### Mixed: #hashtag vs # heading",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "target": {
    "platform": "telegram",
    "chat_id": CHAT_ID_PLACEHOLDER,
    "thread_id": null
  }
}
EOF

# Replace placeholders
sed -i.bak "s/TRACE_ID_PLACEHOLDER/$(uuidgen | tr '[:upper:]' '[:lower:]')/g" "$TMP_FILE2"
sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$TMP_FILE2"
sed -i.bak "s/CHAT_ID_PLACEHOLDER/$CHAT_ID/g" "$TMP_FILE2"
rm "$TMP_FILE2.bak"

echo "$CHAT_ID:$(cat "$TMP_FILE2" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE2"
sleep 3

# Test 3: Code blocks and inline code
echo -e "\n3. Testing code formatting:"
TMP_FILE3=$(mktemp)
cat > "$TMP_FILE3" << 'EOF'
{
  "trace_id": "TRACE_ID_PLACEHOLDER",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 3: Code Formatting\n\nInline code: `console.log('Hello World!')`\n\nCode block:\n```javascript\nfunction greet(name) {\n  return `Hello, ${name}!`;\n}\n\nconst message = greet('Telegram');\nconsole.log(message);\n```\n\nAnother inline: `git status` command",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "target": {
    "platform": "telegram",
    "chat_id": CHAT_ID_PLACEHOLDER,
    "thread_id": null
  }
}
EOF

# Replace placeholders
sed -i.bak "s/TRACE_ID_PLACEHOLDER/$(uuidgen | tr '[:upper:]' '[:lower:]')/g" "$TMP_FILE3"
sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$TMP_FILE3"
sed -i.bak "s/CHAT_ID_PLACEHOLDER/$CHAT_ID/g" "$TMP_FILE3"
rm "$TMP_FILE3.bak"

echo "$CHAT_ID:$(cat "$TMP_FILE3" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE3"
sleep 3

# Test 4: Special characters that need escaping
echo -e "\n4. Testing special characters that need escaping:"
TMP_FILE4=$(mktemp)
cat > "$TMP_FILE4" << 'EOF'
{
  "trace_id": "TRACE_ID_PLACEHOLDER",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 4: Special Characters\n\nThese characters need escaping in MarkdownV2:\n_ * [ ] ( ) ~ ` > # + - = | { } . !\n\nExamples:\n- Underscores in file_names.txt\n- Asterisks in mathematical expressions: 2*3=6\n- Brackets in arrays: [1, 2, 3]\n- Parentheses in functions: func(param)\n- Tildes in ~approximate values~\n- Backticks in `code`\n- Greater than: value > 0\n- Hash tags: #hashtag\n- Plus/minus: +5 or -3\n- Equals: x = y\n- Pipes: option1 | option2\n- Braces: {key: value}\n- Dots: end of sentence.\n- Exclamation: Hello!",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "target": {
    "platform": "telegram",
    "chat_id": CHAT_ID_PLACEHOLDER,
    "thread_id": null
  }
}
EOF

# Replace placeholders
sed -i.bak "s/TRACE_ID_PLACEHOLDER/$(uuidgen | tr '[:upper:]' '[:lower:]')/g" "$TMP_FILE4"
sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$TMP_FILE4"
sed -i.bak "s/CHAT_ID_PLACEHOLDER/$CHAT_ID/g" "$TMP_FILE4"
rm "$TMP_FILE4.bak"

echo "$CHAT_ID:$(cat "$TMP_FILE4" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE4"
sleep 3

# Test 5: Links and URLs
echo -e "\n5. Testing links and URLs:"
TMP_FILE5=$(mktemp)
cat > "$TMP_FILE5" << 'EOF'
{
  "trace_id": "TRACE_ID_PLACEHOLDER",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 5: Links and URLs\n\nInline link: [Telegram](https://telegram.org)\nAnother link: [GitHub](https://github.com)\n\nRaw URLs that should be clickable:\nhttps://example.com\nhttp://test.org\n\nEmail: contact@example.com\n\nMarkdown link with special chars: [Test (Link)](https://example.com/path?param=value&other=123)",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "target": {
    "platform": "telegram",
    "chat_id": CHAT_ID_PLACEHOLDER,
    "thread_id": null
  }
}
EOF

# Replace placeholders
sed -i.bak "s/TRACE_ID_PLACEHOLDER/$(uuidgen | tr '[:upper:]' '[:lower:]')/g" "$TMP_FILE5"
sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$TMP_FILE5"
sed -i.bak "s/CHAT_ID_PLACEHOLDER/$CHAT_ID/g" "$TMP_FILE5"
rm "$TMP_FILE5.bak"

echo "$CHAT_ID:$(cat "$TMP_FILE5" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE5"
sleep 3

# Test 6: Tables (markdown tables need to be handled carefully)
echo -e "\n6. Testing table-like structures:"
TMP_FILE6=$(mktemp)
cat > "$TMP_FILE6" << 'EOF'
{
  "trace_id": "TRACE_ID_PLACEHOLDER",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 6: Table-like Structures\n\nMarkdown table (needs proper escaping):\n\n| Name | Age | City |\n|------|-----|------|\n| John | 25  | NYC  |\n| Jane | 30  | LA   |\n| Bob  | 35  | SF   |\n\nSimple aligned text table:\n```\nName     Age    City\n----     ---    ----\nJohn     25     NYC\nJane     30     LA\nBob      35     SF\n```",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "target": {
    "platform": "telegram",
    "chat_id": CHAT_ID_PLACEHOLDER,
    "thread_id": null
  }
}
EOF

# Replace placeholders
sed -i.bak "s/TRACE_ID_PLACEHOLDER/$(uuidgen | tr '[:upper:]' '[:lower:]')/g" "$TMP_FILE6"
sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$TMP_FILE6"
sed -i.bak "s/CHAT_ID_PLACEHOLDER/$CHAT_ID/g" "$TMP_FILE6"
rm "$TMP_FILE6.bak"

echo "$CHAT_ID:$(cat "$TMP_FILE6" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE6"
sleep 3

# Test 7: Complex nested formatting
echo -e "\n7. Testing complex nested formatting:"
TMP_FILE7=$(mktemp)
cat > "$TMP_FILE7" << 'EOF'
{
  "trace_id": "TRACE_ID_PLACEHOLDER",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 7: Complex Nested Formatting\n\n*Bold with _italic inside_*\n_Italic with *bold inside*_\n__Underlined with *bold* and _italic___\n\nCode with special chars: `const obj = {key: 'value', num: 42};`\n\nMixed formatting:\n*Bold* text with `inline code` and [links](https://example.com)\n\nList with formatting:\n• *First item* with bold\n• _Second item_ with italic\n• `Third item` with code\n• [Fourth item](https://example.com) with link",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "target": {
    "platform": "telegram",
    "chat_id": CHAT_ID_PLACEHOLDER,
    "thread_id": null
  }
}
EOF

# Replace placeholders
sed -i.bak "s/TRACE_ID_PLACEHOLDER/$(uuidgen | tr '[:upper:]' '[:lower:]')/g" "$TMP_FILE7"
sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$TMP_FILE7"
sed -i.bak "s/CHAT_ID_PLACEHOLDER/$CHAT_ID/g" "$TMP_FILE7"
rm "$TMP_FILE7.bak"

echo "$CHAT_ID:$(cat "$TMP_FILE7" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE7"
sleep 3

# Test 8: Edge cases and problematic content
echo -e "\n8. Testing edge cases:"
TMP_FILE8=$(mktemp)
cat > "$TMP_FILE8" << 'EOF'
{
  "trace_id": "TRACE_ID_PLACEHOLDER",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 8: Edge Cases\n\nEmpty code blocks: `` (empty inline code)\n\nConsecutive special chars: **bold** and __underline__\n\nMixed quotes: \"double quotes\" and 'single quotes'\n\nMath expressions: f(x) = x^2 + 2*x - 1\n\nFile paths: /home/user/file.txt and C:\\Users\\user\\file.txt\n\nRegex patterns: ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$\n\nJSON example: {\"name\": \"value\", \"array\": [1, 2, 3]}\n\nCombined: *Bold `code` text* and _italic [link](https://example.com)_",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "TIMESTAMP_PLACEHOLDER",
  "target": {
    "platform": "telegram",
    "chat_id": CHAT_ID_PLACEHOLDER,
    "thread_id": null
  }
}
EOF

# Replace placeholders
sed -i.bak "s/TRACE_ID_PLACEHOLDER/$(uuidgen | tr '[:upper:]' '[:lower:]')/g" "$TMP_FILE8"
sed -i.bak "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$TMP_FILE8"
sed -i.bak "s/CHAT_ID_PLACEHOLDER/$CHAT_ID/g" "$TMP_FILE8"
rm "$TMP_FILE8.bak"

echo "$CHAT_ID:$(cat "$TMP_FILE8" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE8"

echo -e "\n============================================="
echo "Complex markdown testing completed!"
echo "Check your Telegram chat to see how the messages are rendered."
echo ""
echo "What to look for:"
echo "- Test 1: Basic formatting should render correctly"
echo "- Test 2: Headings should appear bold/emphasized (# characters escaped properly)"
echo "- Test 3: Code blocks and inline code should be monospaced"
echo "- Test 4: Special characters should be properly escaped/displayed"
echo "- Test 5: Links should be clickable and formatted correctly"
echo "- Test 6: Tables should display in a readable format"
echo "- Test 7: Nested formatting should work without conflicts"
echo "- Test 8: Edge cases should not break the message formatting"
echo ""
echo "If any messages fail to send or display incorrectly, it indicates"
echo "areas where the format_telegram_markdown() function needs improvement."
echo ""
echo "Common issues to watch for:"
echo "- Messages not sending (parsing errors)"
echo "- Broken formatting (unescaped special characters)"
echo "- Missing formatting (over-escaped characters)"
echo "- Nested formatting conflicts"
