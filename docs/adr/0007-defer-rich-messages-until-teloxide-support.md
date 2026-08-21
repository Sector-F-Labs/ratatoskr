# 0007. Defer Telegram Rich Messages support until teloxide adds Bot API 10.1+ support

Date: 2026-08-21

## Status

Accepted

## Context

Telegram shipped Bot API 10.1 (2026-06-11) and 10.2 (2026-07-14), introducing "Rich Messages":
`sendRichMessage`, `sendRichMessageDraft` (streaming, including `InputRichBlockThinking`), and a
`rich_message` param on `editMessageText`, backed by 20 rich-text formatting classes and 16 block
types — real headings, tables, lists, quotes, dividers, math, and media galleries, plus (in 10.2)
per-user ephemeral messages.

This lands squarely on a known workaround: [ADR-0001](0001-convert-markdown-to-telegram-html-instead-of-markdownv2.md)
converts Markdown to Telegram HTML precisely because the old formatting model has no real heading
or table primitive, downgrading headings to bold text and tables to aligned monospace `<pre>`
blocks. Rich Blocks would let that lossy downgrade go away entirely.

The blocker: `teloxide` — ratatoskr's Rust Telegram client (see [ADR-0006](0006-cross-compile-to-musl-for-raspberry-pi-deployment.md)
for its role in the deployment pipeline) — has no Rich Messages support. Its latest release
(0.17.0, 2025-07-11) tops out at Bot API 9.1, and there is no open issue or pull request against
the teloxide repository touching Rich Messages at all (by contrast, Python's `python-telegram-bot`
already has full support). This isn't a routine dependency bump like the teloxide 0.15→0.17,
rdkafka, or toml bumps done alongside this decision — the underlying Rust types simply don't exist
yet.

Three ways to get there were considered:

- **Bypass teloxide with raw HTTP** for just the rich-message calls, keeping teloxide for
  everything else. Unblocks ratatoskr immediately, but means hand-writing and maintaining the
  Rich Message JSON schema ourselves, decoupled from teloxide's types and duplicating auth/base-URL
  plumbing teloxide already provides.
- **Contribute the Rich Message payload/response types upstream to teloxide.** Benefits the whole
  ecosystem and keeps ratatoskr on teloxide's types long-term, but is the slowest path into
  ratatoskr — blocked on review and merge of a nontrivial PR against a project we don't maintain,
  with no guaranteed timeline.
- **Wait for teloxide to add support upstream, keep the current workaround.** Lowest effort and
  zero new maintenance surface, at the cost of being blocked on someone else's roadmap with no
  known timeline.

## Decision

We will keep `format_telegram_markdown` (ADR-0001) as-is for now and not attempt Rich Messages
support via either the raw-HTTP bypass or an upstream teloxide contribution. This will be revisited
once teloxide adds Rich Messages support upstream — at that point, adopting it is expected to look
like the mechanical dependency-bump workflow already used for teloxide 0.15→0.17: bump the pin,
fix whatever the compiler flags, verify against the test suite.

## Consequences

- No new maintenance surface or duplicated Telegram API schema-tracking is taken on, and today's
  formatting stays exactly as reliable (or unreliable) as ADR-0001 already made it.
- Headings and tables remain lossy downgrades (bold text, monospace `<pre>` blocks) in every
  outgoing message until this is revisited — the actual cost of waiting.
- ratatoskr does not get streaming rich-message drafts (`sendRichMessageDraft` /
  `InputRichBlockThinking`), which would be directly useful for the assistant-reply use case that
  originally motivated looking at Rich Messages, until teloxide catches up.
- This decision is coupled to a dependency we don't control; teloxide's release cadence is the
  trigger for revisiting it, not a date. If teloxide's pace on this turns out to be slow enough
  to matter, the raw-HTTP-bypass option above is the fallback, not a dead end.
