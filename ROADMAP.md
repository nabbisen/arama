# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Startup fatal-boundary resilience

**Status.** RFC 019 proposed for review.

**Why now.** RFC 017 made recoverable runtime and page-load failures visible,
and RFC 018 made AI/video indexing failures explicit. The remaining resilience
boundary is application startup: arama should distinguish failures that prevent
opening a usable shell from failures that can recover into a truthful degraded
startup state with visible feedback.

**Planned design questions.**

- Which startup failures should return an `iced::Result` instead of opening the
  shell?
- Which local setup, settings, gallery, or root-directory failures can recover
  with a startup toast?
- Should invalid configured root directories keep the path visible while using
  an empty Explorer state, or reset the session root to `"."`?
- Which startup `expect()` paths are true developer invariants versus ordinary
  recoverable filesystem failures?

## Recently implemented, pending owner-managed lifecycle

### Cache lifecycle

**Status.** RFC 015 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** RFC 002 moved arama from the old `file-feature-cache` engine to
`localcache` in v0.23.0. RFC 015 retires the temporary v1 migration path and
keeps cache-size/disk-pressure management split into a separate design.

**Follow-up status.** RFC 016 implementation reviewed; release/lifecycle
transition pending owner action.

### Visible recoverable error UX

**Status.** RFC 017 implementation reviewed; release/lifecycle transition
pending owner action.

### AI and video pipeline resilience

**Status.** RFC 018 implementation reviewed; release/lifecycle transition
pending owner action.

## Later candidates

### Audit exception burn-down

Temporary audit exceptions should be revisited periodically. This is maintenance
work unless the policy changes or a blocking advisory appears.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
