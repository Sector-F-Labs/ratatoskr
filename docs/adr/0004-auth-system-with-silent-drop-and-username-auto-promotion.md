# 0004. Add an opt-in auth system with silent-drop and username-based auto-promotion

Date: 2026-02-02

## Status

Accepted

## Context

Ratatoskr had no access control: any Telegram user who found the bot could send it messages and
have them forwarded to the handler on the other side of the broker. As ratatoskr started being
used to bridge to system-level handlers (mapping a Telegram identity to a system user), an
allowlist became necessary — but it needed to work for operators who know a user's Telegram
*username* in advance but not their stable, immutable `telegram_user_id`, since usernames are
attacker-changeable but IDs are not.

## Decision

We will add a `ratatoskr users` CLI (`add`/`list`/`remove`) that manages a `users.toml` allowlist,
and gate all incoming Telegram messages against it (`src/config.rs`, `src/auth.rs`):

- Each entry maps a `system_user` to a Telegram identity, matched by either a stable
  `telegram_user_id` or a set of `allowed_usernames`.
- An entry created with `--promote` (`promote_on_first_auth`) starts matched by username only;
  the first message from a matching username captures that user's `telegram_user_id` into the
  entry and clears `allowed_usernames`, so every subsequent auth check is by immutable ID rather
  than a changeable username.
- A message from a Telegram user that doesn't match any entry is **silently dropped** — no error
  reply is sent back to an unauthorized sender.
- When `users.toml` is empty or missing, the auth gate is disabled entirely and all messages pass
  through, preserving the pre-auth behavior as the zero-config default.

## Consequences

- Operators get a workable bootstrap path (allowlist by username, get promoted to ID on first
  contact) without needing to already know a numeric Telegram user ID up front.
- Silently dropping unauthorized messages avoids leaking to a prober that the bot exists and is
  gating access, at the cost of unauthorized users getting no feedback at all if they mistype a
  username or haven't been added yet — indistinguishable, from their side, from the bot being
  offline.
- The "empty allowlist = auth disabled" default means a fresh or misconfigured deployment fails
  open, not closed. This matches every other optional-config convention in this codebase, but it
  is a real security-relevant default worth naming explicitly: an operator who intends to run with
  auth must remember to actually populate `users.toml`.
- Auto-promotion trusts whichever username claims to be the operator's expected user on *first
  contact* — if an attacker claims a still-unclaimed allowlisted username before the real user
  does, they get promoted instead. This is an accepted risk for a self-hosted single-operator bot,
  not a suitable model for a public-facing one with unclaimed usernames on the allowlist.
