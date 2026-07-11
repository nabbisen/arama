# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Image similarity search dependency strategy

**Status.** RFC 022 proposed for review.

**Why now.** The remaining `bincode` 1.3 audit warning is owned by `hnsw_rs`,
and `hnsw_rs` is used only by the image similar-pairs search path. This is the
smallest remaining warning-owner surface that can be designed without replacing
the cache engine, UI stack, or model artifact source.

**Planned design questions.**

- Should image similar-pairs use exact bounded pairwise search, a maintained
  ANN crate, or retain `hnsw_rs` for now?
- What result-cap and ordering contract prevents exact search from flooding the
  dialog?
- What fixture tests prove threshold filtering, duplicate avoidance, and stable
  ordering?
- What performance evidence is enough before removing `hnsw_rs`?

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

### CLIP SafeTensors source strategy

**Status.** RFC 021 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** Runtime PyTorch-to-SafeTensors conversion is intentionally retained
until a trustworthy pinned SafeTensors source or owner-managed mirror exists.
The decision is recorded in
[`rfcs/notes/clip-safetensors-source-decision.md`](./rfcs/notes/clip-safetensors-source-decision.md).

## Later candidates

### Remaining audit-warning owners

`localcache`, Candle/transitive `paste`, `proc-macro-error2`, and the
font/rendering stack should be revisited when upstream releases expose
compatible fixes or when a replacement design is intentionally proposed.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
