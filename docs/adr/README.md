# Architecture Decision Records

This directory holds ADRs for ratatoskr — short records of significant architecture decisions,
written at the time they're made, following Michael Nygard's format as popularized by Martin
Fowler: <https://martinfowler.com/bliki/ArchitectureDecisionRecord.html>.

## When to write one

Write an ADR when a decision is:

- Hard, or expensive, to reverse once other code depends on it.
- Not obvious from reading the code — a "why", not a "what".
- The kind of thing that'll get re-litigated later without a record ("didn't we already decide
  this?").

Small, reversible choices don't need one. Rule of thumb: if you'd want to explain the choice to a
new contributor in a paragraph beyond what the code itself shows, it's ADR-sized.

This is a different job from the README's living "why" documentation (the "🧠 Why Ratatoskr?"
section, the message-type reference, etc.) — that stays as living documentation of *how the system
currently works and why*, and gets edited as the system evolves. An ADR is a point-in-time record
of *a decision as it was made*, including the options considered and rejected. It does not get
rewritten later — a decision that reverses or replaces an earlier one gets its own new ADR that
supersedes the old one, which stays in place with its status updated.

## Convention

- One file per decision: `NNNN-short-kebab-title.md`, numbered sequentially, zero-padded to 4
  digits. Numbers are never reused, even for a decision that gets reversed.
- Start from [`0000-template.md`](0000-template.md) — copy it, don't write from scratch.
- `Status` is one of: `Proposed`, `Accepted`, `Rejected`, `Deprecated`, or `Superseded by
  ADR-00NN` (linked to the superseding record).
- Every new ADR gets a row in the index below.

## Index

| # | Title | Status |
|---|-------|--------|
| [0001](0001-convert-markdown-to-telegram-html-instead-of-markdownv2.md) | Convert Markdown to Telegram-compatible HTML instead of relying on MarkdownV2 | Accepted |
| [0002](0002-pluggable-message-broker-abstraction.md) | Introduce a pluggable `MessageBroker` abstraction over the transport | Accepted |
| [0003](0003-broker-free-named-pipe-transport.md) | Move to a broker-free named-pipe transport | Superseded by [ADR-0005](0005-replace-pipe-broker-with-kafka.md) |
| [0004](0004-auth-system-with-silent-drop-and-username-auto-promotion.md) | Add an opt-in auth system with silent-drop and username-based auto-promotion | Accepted |
| [0005](0005-replace-pipe-broker-with-kafka.md) | Replace the pipe broker with Kafka | Accepted |
| [0006](0006-cross-compile-to-musl-for-raspberry-pi-deployment.md) | Cross-compile to `aarch64-unknown-linux-musl` for the Raspberry Pi deployment | Accepted |
| [0007](0007-defer-rich-messages-until-teloxide-support.md) | Defer Telegram Rich Messages support until teloxide adds Bot API 10.1+ support | Accepted |
