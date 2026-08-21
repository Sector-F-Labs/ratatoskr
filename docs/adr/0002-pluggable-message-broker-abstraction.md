# 0002. Introduce a pluggable `MessageBroker` abstraction over the transport

Date: 2025-06-19

## Status

Accepted

## Context

Ratatoskr's job is to move Telegram updates and outgoing replies across some message transport.
Early on this meant Kafka specifically, with `rdkafka` calls inlined directly into the Telegram
handling code. That made the transport a fixed assumption baked into the handlers rather than a
swappable detail — every place that needed to publish or consume had to know it was talking to
Kafka.

The transport was not settled at this point: Kafka and MQTT were both plausible backends
(`rumqttc` support landed alongside this change), and the pipe-based, broker-free mode adopted
later (see [ADR-0003](0003-broker-free-named-pipe-transport.md)) was already foreseeable as a third
option.

## Decision

We will define a `MessageBroker` trait (`src/broker/mod.rs`) — `async fn publish(&self, ...)` /
`async fn subscribe(&self, ...) -> BoxStream<...>` — and have the Telegram handling code
(`telegram_handler`, `kafka_processing`) depend only on `dyn MessageBroker`, never on a concrete
transport. Each transport (Kafka, MQTT at the time, later pipes) implements the trait in its own
module under `src/broker/`, and `main.rs` is the only place that picks a concrete implementation.

## Consequences

- Swapping the transport (Kafka → pipes → Kafka again, see ADR-0003/ADR-0005) became a matter of
  writing a new `src/broker/*.rs` implementation and changing what `main.rs` constructs, not a
  rewrite of the handler code — which is exactly what played out over the following months.
  This is also what makes the crate's test suite able to substitute a `MockMessageBroker`
  (`src/broker/mod.rs`, `#[cfg(test)]`) instead of a real Kafka connection.
- The abstraction has one direction of leakage: `KafkaBroker::publish` takes an `Option<&str>` key
  used for partition routing (see ADR-0005), a concept that only really means something for Kafka.
  Other implementations either ignore it or approximate it, rather than the trait being a clean
  lowest-common-denominator across all possible transports.
- Only one concrete implementation (`KafkaBroker`) exists in the codebase today; MQTT and pipe
  support were both added and later removed as the transport decision was revisited (ADR-0003,
  ADR-0005). The trait boundary stayed in place the whole time even as implementations churned.
