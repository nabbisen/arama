# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Visible recoverable error UX

**Status.** RFC 017 proposed for review.

**Why now.** Recent cache, setup, and similarity resilience work removed
several panic paths, but some recoverable failures still degrade to empty,
default, or stderr-only states. The project now needs a consistent
user-visible policy before more implementation work spreads ad hoc handling
patterns.

**Planned design questions.**

- Which recoverable failures should be inline page errors, toasts, fatal startup
  errors, or developer diagnostics?
- How should settings load/save failures behave without losing the current
  in-memory session state?
- How should Cache page reload failures avoid presenting a false empty cache?
- Which errors should remain stderr-only because the fallback is truthful and
  safe?

## Recently implemented, pending owner-managed lifecycle

### Cache lifecycle

**Status.** RFC 015 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** RFC 002 moved arama from the old `file-feature-cache` engine to
`localcache` in v0.23.0. RFC 015 retires the temporary v1 migration path and
keeps cache-size/disk-pressure management split into a separate design.

**Follow-up status.** RFC 016 implementation reviewed; release/lifecycle
transition pending owner action.

## Later candidates

### Startup fatal-boundary resilience

After RFC 017 handles recoverable settings visibility, remaining startup work
should focus on failures that prevent a usable application shell from starting
at all, and on which of those should return an `iced::Result` error versus a
fallback shell.

### AI and video pipeline resilience

The video similarity pipeline still has failure modes that should be handled
with a deliberate policy: skip one item, mark one item failed, retry work, or
abort the whole pipeline.

### Audit exception burn-down

Temporary audit exceptions should be revisited periodically. This is maintenance
work unless the policy changes or a blocking advisory appears.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
