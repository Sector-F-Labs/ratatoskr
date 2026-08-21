# 0005. Replace the pipe broker with Kafka

Date: 2026-02-03

## Status

Accepted

## Context

The broker-free named-pipe transport from [ADR-0003](0003-broker-free-named-pipe-transport.md)
traded broker infrastructure for filesystem coordination: a pipe per consumer to manage, and no
persistence, replay, or multi-consumer fan-out. As usage grew past a single local handler process,
those gaps started to matter — there was no way for a consumer to catch up on messages sent while
it was down, and no clean way to have more than one consumer of the same stream.

## Decision

We will remove the pipe broker and reintroduce Kafka (via `rdkafka`) as the (now sole) transport,
implementing the existing `MessageBroker` trait from
[ADR-0002](0002-pluggable-message-broker-abstraction.md):

- Incoming Telegram updates publish to a `{prefix}.in` topic; outgoing replies are consumed from a
  `{prefix}.out` topic (`src/broker/kafka.rs`).
- Messages are keyed by `telegram_user_id` (`MessageBroker::publish`'s `Option<&str>` key), so
  Kafka's partition routing guarantees all messages from a given user land on the same partition
  and are processed in order relative to each other.
- MQTT is not reintroduced alongside this — Kafka alone is judged sufficient for the persistence
  and fan-out properties that motivated the reversal.

## Consequences

- Consumers get persistence and replay (a consumer that was down catches up from the topic instead
  of losing messages sent in the meantime) and can fan out to multiple consumers of the same
  stream — the two gaps that motivated this reversal.
- Running ratatoskr again requires operating a Kafka broker, undoing the "broker-free" appeal of
  ADR-0003. This is treated as the more valuable tradeoff for how ratatoskr is actually deployed
  (persistent, always-on bridge), rather than a hosted-anywhere-instantly tool.
- Per-user ordering depends on partition-key consistency: `telegram_user_id` must stay the publish
  key everywhere a message enters the `.in` topic, or ordering guarantees silently stop holding for
  whichever path skips it.
- The `MessageBroker` trait boundary again absorbed the swap without touching the Telegram handler
  code — the same abstraction from ADR-0002 has now survived two full transport reversals.
