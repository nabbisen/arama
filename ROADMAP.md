# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Cache lifecycle

**Status.** RFC 015 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** RFC 002 moved arama from the old `file-feature-cache` engine to
`localcache` in v0.23.0. RFC 015 retires the temporary v1 migration path and
keeps cache-size/disk-pressure management split into a separate design.

**Current follow-up.** RFC 016 — Cache capacity and disk-pressure management.

**Planned design questions.**

- How should arama measure actual cache footprint?
- What user-visible controls should exist for cache limits and pruning?
- Should pruning be explicit/manual first, or automatic after indexing runs?
- What eviction order should be used when entries must be removed?
- How should low-disk-space warnings differ from cache-size limits?

## Later candidates

### Visible recoverable error UX

Recent cache and similarity resilience work removed several panic paths, but
some failures still degrade to empty or partial UI states. A future RFC should
define which recoverable failures deserve inline errors, toasts, retry buttons,
or silent fallback.

### Startup and settings persistence resilience

Remaining app-level initialization and settings-save panic paths should be
classified into fatal startup errors versus recoverable UI errors before code
changes begin.

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
