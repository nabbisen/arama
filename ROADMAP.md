# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### CLIP SafeTensors source strategy

**Status.** RFC 021 implementation proposed for review.

**Why now.** RFC 020 moved first-party Candle use to 0.11, but
`pt2safetensors` still keeps a transitive Candle 0.10 line in the lockfile
because the pinned OpenAI CLIP source is a PyTorch `.bin` artifact. Removing
that duplicate AI stack is no longer a dependency bump; it requires a trust and
artifact-source decision.

**Implementation decision.**

- Selected outcome: retain runtime PyTorch-to-SafeTensors conversion for now.
- Preserve exact source evidence for the pinned OpenAI CLIP revision.
- Keep `pt2safetensors` as an intentional dependency until a trustworthy
  pinned SafeTensors source or owner-managed mirror exists.
- Revisit removal only with provenance, checksum, and embedding-regression
  evidence.

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

### Dependency modernization

**Status.** RFC 020 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** First-party Candle dependencies moved to 0.11 and non-Linux
sidecar ZIP extraction moved to stable `zip` 8.6.0. `pt2safetensors` remains
as the only Candle 0.10 owner.

## Later candidates

### Remaining audit-warning owners

`hnsw_rs`, `localcache`, Candle/transitive `paste`, `proc-macro-error2`, and
the font/rendering stack should be revisited when upstream releases expose
compatible fixes or when a replacement design is intentionally proposed.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
