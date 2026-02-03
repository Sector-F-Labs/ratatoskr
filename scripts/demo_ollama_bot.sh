#!/usr/bin/env bash
set -euo pipefail

# Demo: read inbound Kafka messages, run Ollama, and reply via ratatoskr send.
# Requirements: jq, ollama, ratatoskr
# Env vars:
#   OLLAMA_MODEL (required)   e.g. denny
#   ALLOWED_CHAT_IDS (required, comma-separated) e.g. -100123456789,123456789
#   KAFKA_BROKERS (optional)  default: localhost:9092
#   KAFKA_TOPIC_PREFIX (optional) default: ratatoskr

if [[ -z "${OLLAMA_MODEL:-}" ]]; then
  echo "Missing OLLAMA_MODEL" >&2
  exit 1
fi

if [[ -z "${ALLOWED_CHAT_IDS:-}" ]]; then
  echo "Missing ALLOWED_CHAT_IDS (comma-separated)" >&2
  exit 1
fi

IFS=',' read -r -a allowed_ids <<< "${ALLOWED_CHAT_IDS}"

echo "Starting demo bot (model=${OLLAMA_MODEL})..." >&2

to_upper() {
  printf '%s' "$1" | tr '[:lower:]' '[:upper:]'
}

is_allowed_chat_id() {
  local chat_id="$1"
  for allowed in "${allowed_ids[@]}"; do
    allowed="$(printf '%s' "$allowed" | tr -d '[:space:]')"
    if [[ "$chat_id" == "$allowed" ]]; then
      return 0
    fi
  done
  return 1
}

while true; do
  raw_json="$(ratatoskr read --wait)"
  message_type="$(printf '%s' "$raw_json" | jq -r '.message_type.type')"

  if [[ "$message_type" != "TelegramMessage" ]]; then
    continue
  fi

  chat_id="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.chat.id')"
  text="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.text // empty')"
  username="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.from.username // empty')"
  first_name="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.from.first_name // empty')"

  if [[ -z "$text" ]]; then
    continue
  fi

  if [[ "$chat_id" == -* ]] && [[ "$text" != *"@"* ]]; then
    continue
  fi

  if ! is_allowed_chat_id "$chat_id"; then
    echo "Skipping chat_id=${chat_id} (not in allowlist)" >&2
    continue
  fi

  sender_label="$first_name"
  if [[ -n "$username" ]]; then
    sender_label="${sender_label} (@${username})"
  fi

  prompt="User: ${sender_label}\nMessage: ${text}";

  response="$(ollama run --hidethinking "${OLLAMA_MODEL}" "$prompt")"
  response="$(printf '%s' "$response" | sed -e 's/[[:space:]]*$//')"

  if [[ -z "$response" ]]; then
    continue
  fi

  printf '%s' "$response" | ratatoskr send --chat-id "$chat_id" >/dev/null
  echo "Replied to chat_id=${chat_id}" >&2
 done
