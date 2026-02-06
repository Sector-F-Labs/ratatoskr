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

ingest_message() {
  local chat_id="$1"
  local trace_id="$2"
  local role="$3"
  local content="$4"
  printf '%s' "$content" | reservoir ingest --partition="$chat_id" --instance="$chat_id" --role="$role" --trace-id="$trace_id" >/dev/null
}

fetch_context() {
  local chat_id="$1"
  local query="$2"
  local thread semantic output=""
  # Thread context: synapse-connected messages + minimum recent (hybrid strategy)
  thread="$(reservoir thread --partition="$chat_id" --instance="$chat_id" 2>/dev/null || true)"
  # Semantic search: long-term memory lookup for related messages
  semantic="$(reservoir search --semantic --link "$query" --partition="$chat_id" --instance="$chat_id" 2>/dev/null || true)"
  if [[ -n "$semantic" ]]; then
    output="<system>The following messages are semantically related to the current query from your long-term memory:</system>
${semantic}"
  fi
  if [[ -n "$thread" ]]; then
    if [[ -n "$output" ]]; then
      output="${output}

"
    fi
    output="${output}<system>The following is your current conversation thread:</system>
${thread}"
  fi
  printf '%s' "$output"
}

while true; do
  raw_json="$(ratatoskr read --wait)"
  message_type="$(printf '%s' "$raw_json" | jq -r '.message_type.type')"

  if [[ "$message_type" != "TelegramMessage" ]]; then
    continue
  fi

  chat_id="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.chat.id | tostring')"
  chat_id="$(printf '%s' "$chat_id" | tr -d '[:space:]')"
  trace_id="$(printf '%s' "$raw_json" | jq -r '.trace_id // empty')"
  text="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.text // empty')"
  username="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.from.username // empty')"
  first_name="$(printf '%s' "$raw_json" | jq -r '.message_type.data.message.from.first_name // empty')"

  if [[ -z "$text" ]]; then
    continue
  fi

  if [[ "${DEBUG_CHAT_IDS:-}" == "1" ]]; then
    echo "DEBUG_CHAT_IDS: chat_id=${chat_id}" >&2
  fi

  if [[ "$chat_id" == -* ]] && [[ "$text" != *"@"* ]]; then
    continue
  fi

  if ! is_allowed_chat_id "$chat_id"; then
    echo "Skipping chat_id=${chat_id} (not in allowlist)" >&2
    continue
  fi

  if [[ -z "$trace_id" ]] || [[ "$trace_id" == "null" ]]; then
    trace_id="$(cat /proc/sys/kernel/random/uuid)"
  fi

  ingest_message "$chat_id" "$trace_id" "user" "$text"

  context="$(fetch_context "$chat_id" "$text")"

  if [[ -n "$context" ]]; then
    echo "--- Working memory for chat_id=${chat_id} ---" >&2
    printf '%s\n' "$context" >&2
    echo "--- End working memory ---" >&2
  fi

  sender_label="$first_name"
  if [[ -n "$username" ]]; then
    sender_label="${sender_label} (@${username})"
  fi

  prompt="User: ${sender_label}\nMessage: ${text}";
  if [[ -n "$context" ]]; then
    prompt="Relevant context:\n${context}\n\nCurrent user question:${prompt}"
  fi

  response="$(ollama run --hidethinking "${OLLAMA_MODEL}" "$prompt")"
  response="$(printf '%s' "$response" | sed -e 's/[[:space:]]*$//')"

  if [[ -z "$response" ]]; then
    continue
  fi

  ingest_message "$chat_id" "$trace_id" "assistant" "$response"
  printf '%s' "$response" | ratatoskr send --chat-id="$chat_id" --parse-mode Markdown >/dev/null
  echo "Replied to chat_id=${chat_id}" >&2
 done
