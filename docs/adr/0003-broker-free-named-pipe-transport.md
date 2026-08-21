# 0003. Move to a broker-free named-pipe transport

Date: 2025-12-15

## Status

Superseded by [ADR-0005](0005-replace-pipe-broker-with-kafka.md)

## Context

Running ratatoskr against Kafka or MQTT meant standing up and operating a broker just to get
Telegram updates into a handler process — real infrastructure for what could, for a single-process
handler chained via a shell pipeline, be plain stdio. The `MessageBroker` abstraction from
[ADR-0002](0002-pluggable-message-broker-abstraction.md) made the transport swappable in principle;
this was the point it got exercised in the other direction, toward less infrastructure rather than
more.

## Decision

We will drop the Kafka and MQTT broker implementations and replace them with a broker-free mode:
incoming Telegram updates stream out as newline-delimited JSON on **stdout**, and outgoing replies
are read as `OutgoingMessage` JSON lines from a named pipe (`PIPE_OUTBOUND_PATH`). No broker process
required — a handler can be anything that reads JSONL from stdin and writes JSONL to the pipe,
including a plain shell pipeline.

## Consequences

- Running ratatoskr for local development or a single-consumer handler no longer needs a Kafka or
  MQTT broker at all — `README.md`'s description of this mode called it "broker-free — great for
  chaining with shell pipelines," which was the actual appeal.
- The `MessageBroker` trait boundary from ADR-0002 held: this was a new `src/broker/pipe.rs`
  implementation plus a `main.rs` wiring change, not a rewrite of the Telegram handling code.
- Named pipes push coordination problems onto the filesystem and the process model instead: a pipe
  per consumer to manage, and no persistence, replay, or multi-consumer fan-out the way a real
  broker provides. This tradeoff is exactly what motivated moving back to Kafka less than two
  months later — see [ADR-0005](0005-replace-pipe-broker-with-kafka.md) for the reversal and its
  reasoning.
