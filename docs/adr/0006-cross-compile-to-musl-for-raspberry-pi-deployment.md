# 0006. Cross-compile to `aarch64-unknown-linux-musl` for the Raspberry Pi deployment

Date: 2026-07-17

## Status

Accepted

## Context

Ratatoskr runs on Muninn, an aarch64 Raspberry Pi — a resource-constrained device where compiling
the dependency tree directly (`rdkafka`, `teloxide`, `openssl`) is slow and puts unwanted load on
the box. Building on a dev machine and shipping a finished binary is clearly preferable, but the
first attempt at that — cross-compiling to the default `aarch64-unknown-linux-gnu` target — failed
at runtime on the device with `GLIBC_2.38 not found`: the `gnu` cross image ships a newer glibc
than Debian 12 (what Muninn actually runs), and a `gnu`-targeted binary is tied to whatever glibc
version it was linked against.

## Decision

We will cross-compile with `cross` (Docker-based cross-compilation) targeting
`aarch64-unknown-linux-musl` instead of `-gnu`:

- musl statically links libc into the binary, so it isn't tied to the target's glibc version at
  all — sidestepping the `GLIBC_2.38` class of failure entirely rather than trying to match
  versions.
- `rdkafka-sys` vendors and builds librdkafka from C source via its own build system, which doesn't
  reliably pick up the cross toolchain unprompted. `Cross.toml` passes the musl `CC`/`AR`/etc. env
  vars through to the build container so librdkafka is actually compiled for aarch64 instead of
  silently linking host-arch object code.
- `openssl` became a direct dependency with the `vendored` feature, so OpenSSL is built from source
  for the target instead of requiring target-arch `libssl-dev` to be cross-installed in the build
  container.

Together this produces a self-contained aarch64 binary built on x86 and shipped straight to Muninn,
with no compiler or build dependencies required on the device itself.

## Consequences

- Deploying a new build no longer costs a slow, load-bearing on-device compile — the Pi just
  receives a finished binary.
- Any new dependency's default TLS/crypto backend and build system now matter for this pipeline
  specifically: a dependency that doesn't cross-compile cleanly to musl, or that pulls in a
  heavier TLS stack by default (as `reqwest`'s newer defaults do — a live consideration when
  evaluating dependency bumps), is a real cost to this deployment path, not just an abstract
  dependency-weight concern.
- The build now requires Docker (for `cross`) on the dev machine doing the cross-compile — a new
  local prerequisite that didn't exist when builds only ever targeted the host architecture.
- musl's static linking sidesteps glibc version skew, but any future dependency that assumes glibc
  specifically (rather than a POSIX-ish libc in general) could reintroduce a version of the same
  problem this ADR solved.
