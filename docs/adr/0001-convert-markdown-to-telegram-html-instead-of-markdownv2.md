# 0001. Convert Markdown to Telegram-compatible HTML instead of relying on MarkdownV2

Date: 2025-06-15

## Status

Accepted

## Context

Ratatoskr accepts outgoing text as Markdown and has to render it in a Telegram message. Telegram
offers two `parse_mode`s for this: `MarkdownV2` and `HTML`. `MarkdownV2` requires escaping a long,
easy-to-miss list of characters (`_*[]()~`\>#+-=|{}.!`) anywhere they appear outside of an
intentional formatting marker, and its parser rejects the whole message on a single unescaped
character — a message that renders fine everywhere else in Markdown (a stray `.` or `-` in normal
prose) can silently fail to send. It also has no representation at all for some common Markdown
constructs, tables in particular.

The alternative considered and taken instead: pre-convert Markdown to Telegram's `HTML` parse
mode ourselves, rather than pass MarkdownV2 straight through to Telegram's own parser.

## Decision

We will convert Markdown to Telegram-compatible HTML in `src/utils.rs`
(`format_telegram_markdown`) before sending, rather than relying on Telegram's `MarkdownV2` parser:

- Headings (`#`) are downgraded to `<b>bold</b>` — Telegram's formatting model has no heading
  concept.
- Bold/italic/strikethrough/inline-code/code-block map onto their HTML equivalents
  (`<b>`/`<i>`/`<s>`/`<code>`/`<pre>`).
- Markdown tables have no HTML equivalent Telegram renders as a table either, so they're
  reformatted into aligned plain text and wrapped in `<pre>` for a monospace layout instead of
  being dropped.
- A dedicated `escape_html_except_tags` step HTML-escapes `&`/`<`/`>` in the surrounding text while
  leaving the tags we just inserted alone.

## Consequences

- Outgoing messages stop failing outright on unescaped MarkdownV2 special characters — the most
  common source of silent send failures under the old approach.
- Telegram's formatting model still has no real heading or table primitive, so both are lossy
  downgrades (bold text, monospace text) rather than a faithful rendering — this is a workaround
  for a real gap, not a full fix.
- `format_telegram_markdown` is a growing pile of regex-based, hand-rolled Markdown parsing that
  has to be kept in sync with whatever Markdown producers actually send; it isn't a general-purpose
  Markdown engine and doesn't try to be one.
- If Telegram ever ships a richer formatting model with native headings/tables, this conversion
  step — and its lossy downgrades — becomes unnecessary. See [ADR-0007](0007-defer-rich-messages-until-teloxide-support.md).
