#!/usr/bin/env bash
set -euo pipefail

# Black-box check: write a sample OutgoingMessage JSON into the outbound pipe.
# Requires the Ratatoskr service to be running in pipe mode and a valid CHAT_ID.

PIPE_OUTBOUND_PATH=${PIPE_OUTBOUND_PATH:-./ratatoskr_out.pipe}
CHAT_ID=${CHAT_ID:-}
TEXT=${TEXT:-"Hello from pipe test"}

if [[ -z "$CHAT_ID" ]]; then
  echo "CHAT_ID is required (target Telegram chat ID)" >&2
  exit 1
fi

if [[ ! -p "$PIPE_OUTBOUND_PATH" ]]; then
  echo "Pipe not found at $PIPE_OUTBOUND_PATH; create it with 'mkfifo $PIPE_OUTBOUND_PATH' and ensure Ratatoskr is running." >&2
  exit 1
fi

TRACE_ID=$(uuidgen 2>/dev/null || echo "00000000-0000-0000-0000-000000000000")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat >"$PIPE_OUTBOUND_PATH" <<EOF
{"trace_id":"$TRACE_ID","message_type":{"type":"TextMessage","data":{"text":"$TEXT","buttons":null,"reply_keyboard":null,"parse_mode":null,"disable_web_page_preview":null}},"timestamp":"$TIMESTAMP","target":{"platform":"telegram","chat_id":$CHAT_ID,"thread_id":null}}
EOF

echo "Wrote outbound message to $PIPE_OUTBOUND_PATH with trace_id=$TRACE_ID"
