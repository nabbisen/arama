# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Dependency modernization

**Status.** RFC 020 proposed for review.

**Why now.** The audit warning burn-down resolved the compatible patch-level
warnings and recorded the remaining upstream/transitive owners. The next
dependency work should be explicit modernization rather than ad hoc version
bumps: direct AI/runtime dependencies can affect model loading, CUDA/Metal
builds, archive extraction, and release-gate evidence.

**Planned design questions.**

- Which direct dependency upgrades are small enough for one reviewed batch?
- Which updates require Linux, macOS/Metal, Windows, or CUDA-specific evidence?
- Which remaining audit warnings are blocked by upstream crates and should stay
  tracked rather than forced through replacement work?
- Should pre-release dependency lines be excluded unless they fix a blocking
  advisory?

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

### Startup fatal-boundary resilience

**Status.** RFC 019 implementation reviewed; release/lifecycle transition
pending owner action.

### Audit warning burn-down

**Status.** Maintenance pass reviewed; release/lifecycle transition pending
owner action.

**Why now.** Compatible patch-level RustSec warnings for `anyhow` and `memmap2`
were resolved. Remaining allowed warnings are tracked in
[`rfcs/notes/audit-warning-burn-down.md`](./rfcs/notes/audit-warning-burn-down.md).

## Later candidates

### Remaining audit-warning owners

`hnsw_rs`, `localcache`, Candle/transitive `paste`, `proc-macro-error2`, and
the font/rendering stack should be revisited when upstream releases expose
compatible fixes or when a replacement design is intentionally proposed.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
