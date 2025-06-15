#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

echo "Testing Telegram MarkdownV2 edge cases..."
echo "========================================"
echo "This script tests the most challenging cases for Telegram MarkdownV2 escaping"

# Helper function to send a test message
send_test_message() {
    local test_name="$1"
    local message_text="$2"
    local description="$3"

    echo -e "\n$test_name:"
    echo "Description: $description"

    TMP_FILE=$(mktemp)
    cat > "$TMP_FILE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "$message_text",
      "buttons": null,
      "parse_mode": "Markdown",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "target": {
    "platform": "telegram",
    "chat_id": $CHAT_ID,
    "thread_id": null
  }
}
EOF

    echo "Sending: $message_text"
    echo "$CHAT_ID:$(cat "$TMP_FILE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
    rm "$TMP_FILE"
    sleep 2
}

# Test 1: All reserved characters that MUST be escaped
send_test_message "Test 1: Reserved Characters" \
"Reserved chars that need escaping:\n_underscore_ *asterisk* [bracket] (paren) ~tilde~ \`backtick\` >greater #hash +plus -minus =equals |pipe {brace} .dot !exclamation" \
"Every special character that requires escaping in MarkdownV2"

# Test 2: Consecutive special characters
send_test_message "Test 2: Consecutive Special Chars" \
"Consecutive specials: ** __ ]] (( ~~ \`\` >> ## ++ -- == || {{ }} .. !!" \
"Multiple consecutive special characters that could break parsing"

# Test 3: Mixed formatting with special chars
send_test_message "Test 3: Mixed Formatting + Special Chars" \
"*Bold with _nested italic_* and \`code with {special} chars\`\nLink: [Text (with parens)](https://example.com/path?param=value&other=test)" \
"Nested formatting combined with special characters"

# Test 4: Code blocks with all special characters
send_test_message "Test 4: Code Block Edge Cases" \
"Inline code with specials: \`const obj = {key: 'value', arr: [1,2,3], fn: () => true};\`\n\nBlock code:\n\`\`\`javascript\nif (x > 0 && y < 10) {\n  console.log('Test: ' + result);\n  return {success: true, data: [x, y]};\n}\n\`\`\`" \
"Code blocks containing all types of special characters"

# Test 5: URLs with special characters
send_test_message "Test 5: Complex URLs" \
"URLs with specials:\nhttps://example.com/path?q=test&filter=value#section\nftp://user:pass@server.com:21/path/file.txt\n\nMarkdown links:\n[Complex URL](https://api.example.com/v1/users/{id}/posts?limit=10&offset=0)\n[Query Params](https://search.com/?q=hello+world&type=exact)" \
"URLs and links containing special characters and query parameters"

# Test 6: Mathematical expressions
send_test_message "Test 6: Mathematical Expressions" \
"Math expressions:\nf(x) = x^2 + 2*x - 1\ng(x) = (x + 5) * (x - 3)\nh(x) = |x| + sqrt(x^2 + 1)\n\nInequalities: x > 0, y < 10, z >= 5, w <= 100\nSet notation: {x | x ∈ ℝ, x > 0}" \
"Mathematical formulas with parentheses, operators, and special symbols"

# Test 7: Programming code snippets
send_test_message "Test 7: Programming Snippets" \
"Python dict: \`{'name': 'John', 'age': 30, 'skills': ['python', 'rust']}\`\n\nRegex: \`^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$\`\n\nSQL: \`SELECT * FROM users WHERE age > 18 AND status = 'active';\`\n\nJSON: \`{\"id\": 123, \"data\": [1, 2, 3], \"meta\": {\"count\": 3}}\`" \
"Real programming code with brackets, quotes, and special syntax"

# Test 8: File paths and system commands
send_test_message "Test 8: File Paths & Commands" \
"Unix paths: /home/user/documents/file.txt\nWindows: C:\\Users\\User\\Documents\\file.txt\n\nCommands:\n\`ls -la /tmp/*.log\`\n\`grep -r \"pattern\" ./src/ --include=\"*.rs\"\`\n\`find . -name \"*.txt\" -exec rm {} \\;\`" \
"File system paths and shell commands with various special characters"

# Test 9: Markdown table with special chars
send_test_message "Test 9: Tables with Special Chars" \
"Table with specials:\n\n| Function | Syntax | Example |\n|----------|--------|----------|\n| Add | x + y | 5 + 3 = 8 |\n| Multiply | x * y | 4 * 2 = 8 |\n| Compare | x > y | 10 > 5 |\n| Object | {key: val} | {id: 123} |" \
"Markdown table containing various special characters in cells"

# Test 10: Deeply nested formatting
send_test_message "Test 10: Deeply Nested Formatting" \
"*Bold containing _italic with \`code and {special} chars\` inside_ text*\n\n__Underlined with *bold and _italic_* nested__\n\n~Strikethrough with [link](https://example.com) and \`code\`~" \
"Multiple levels of nested formatting with special characters"

# Test 11: Email addresses and mentions
send_test_message "Test 11: Emails & Mentions" \
"Email formats:\nuser@domain.com\nfirst.last+tag@sub.domain.co.uk\nuser+123@example-site.org\n\nSocial mentions:\n@username_123\n#hashtag_test\n@user.name" \
"Email addresses and social media mentions with special characters"

# Test 12: Escape sequences and edge cases
send_test_message "Test 12: Escape Edge Cases" \
"Edge cases:\nBackslash: \\\\ (should show single backslash)\nQuotes: \"double\" and 'single'\nNewlines and \\n literal\nTabs and \\t literal\n\nCombined: \\n\\t\\r (literal escape sequences)" \
"Backslashes, quotes, and literal escape sequences"

# Test 13: Unicode and emoji with formatting
send_test_message "Test 13: Unicode & Emoji" \
"Unicode: ñáéíóú ÑÁÉÍÓÚ çÇ ¿¡\nEmoji: 🚀 *bold emoji* 📝 _italic emoji_ 🔧\nSymbols: ∀∃∈∉⊂⊃∪∩ ∞≠≤≥±×÷\nCurrency: $100 €50 £30 ¥1000" \
"Unicode characters, emoji, and symbols combined with formatting"

# Test 14: Long text with many special chars
send_test_message "Test 14: Long Text Stress Test" \
"Stress test with everything:\n\n*This* message contains _every_ type of __special__ character: ()[]{}~\`>#+-=|.!\n\nCode: \`if (obj.key === 'value' && arr[0] > 10) { return {success: true}; }\`\n\nURL: [Complex Link](https://api.example.com/v1/data?query={search}&filter[type]=exact&limit=100#results)\n\nMath: f(x) = (x^2 + 2*x - 1) / (x + 5)\n\nFile: C:\\Program Files\\App\\config.json\n\nEmail: user+test@sub.domain.co.uk" \
"Long message combining all types of special characters and formatting"

echo -e "\n========================================"
echo "Telegram MarkdownV2 edge case testing completed!"
echo ""
echo "What to check in your Telegram chat:"
echo ""
echo "✅ SUCCESS indicators:"
echo "- All messages send without errors"
echo "- Formatting (bold, italic, code) renders correctly"
echo "- Special characters display as intended"
echo "- Links are clickable"
echo "- Code blocks preserve formatting"
echo ""
echo "❌ FAILURE indicators:"
echo "- Messages fail to send (400 errors)"
echo "- Formatting is broken (unformatted or garbled text)"
echo "- Special characters are doubled (e.g., \\\\. instead of .)"
echo "- Missing characters (over-escaped)"
echo "- Nested formatting conflicts"
echo ""
echo "Common MarkdownV2 requirements:"
echo "- These chars MUST be escaped outside code: _*[]()~\`>#+-=|{}.!"
echo "- Inside code blocks/inline code: NO escaping needed"
echo "- Links: escape everything except the URL part"
echo "- Nested formatting: careful order of escape/format operations"
echo ""
echo "If any tests fail, your format_telegram_markdown() function needs fixes!"
