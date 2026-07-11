# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Cache lifecycle

**Status.** Proposed RFC pending review.

**Why now.** RFC 002 moved arama from the old `file-feature-cache` engine to
`localcache` in v0.23.0. The temporary v1 migration shim was intentionally kept
for one release cycle, and the implementation still says it is scheduled for
removal. arama is now past that compatibility window, so the cache lifecycle
policy should be reviewed deliberately instead of letting historical migration
code remain indefinitely.

**Planned design questions.**

- Should the v1 cache migration shim be removed now?
- Should the old `rusqlite` dependency leave `arama-cache` as part of the same
  change?
- How should users recover if they still have only a v1 cache database?
- Should cache-size or disk-pressure management be designed in the same RFC, or
  split into a follow-up RFC?
- What tests and release notes are required for safe removal?

**Candidate RFC.** RFC 015 — Cache lifecycle: retire v1 migration and define
cache-capacity direction.

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
