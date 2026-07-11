# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### AI and video pipeline resilience

**Status.** RFC 018 proposed for review.

**Why now.** Video indexing is the most failure-prone AI workflow: ffmpeg,
ffprobe, frame extraction, audio extraction, model inference, and cache writes
can fail independently. The project needs an explicit policy for whether to
skip one file, use one successful modality, warn the user, or abort the whole
run.

**Planned design questions.**

- Which failures are fatal setup errors versus per-file recoverable failures?
- When should a video with only frame or only audio embeddings remain usable?
- How should cache write failures be reported without aborting unrelated files?
- What concise user-visible warning should summarize partial indexing results?

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

## Later candidates

### Startup fatal-boundary resilience

After RFC 017 handles recoverable settings visibility, remaining startup work
should focus on failures that prevent a usable application shell from starting
at all, and on which of those should return an `iced::Result` error versus a
fallback shell.

### Audit exception burn-down

Temporary audit exceptions should be revisited periodically. This is maintenance
work unless the policy changes or a blocking advisory appears.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
