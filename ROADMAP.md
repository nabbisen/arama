# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Image codec dependency minimization

**Status.** RFC 024 implementation proposed for review.

**Why now.** The remaining `paste` warning has several broad owners, but one
path enters through `image`'s default AVIF stack (`ravif` / `rav1e`) even
though arama's current image allowlist is limited to PNG, JPEG, WebP, GIF, and
BMP. Pruning unused image codecs is a narrow dependency-surface reduction with
clear product boundaries.

**Planned design questions.**

- Can `iced` switch from `image` to `image-without-codecs` while arama enables
  only the image formats it accepts?
- Which image crate features are required for the current allowlist and
  generated JPEG thumbnails?
- What decode/thumbnail/cache tests prove PNG, JPEG, WebP, GIF, and BMP still
  work after disabling default formats?
- Which dependency graph owners are removed, and which `paste`/font warnings
  intentionally remain?

## Recently implemented, pending owner-managed lifecycle

### Cache serialization dependency strategy

**Status.** RFC 023 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** The current `localcache` 0.20 bincode-backed cache payload path was
retained because no published or local bincode-free `localcache` dependency
route is available yet.

### Image similarity search dependency strategy

**Status.** RFC 022 implementation reviewed; release/lifecycle transition
pending owner action.

**Why now.** `hnsw_rs` was replaced with exact bounded pairwise image search,
removing the `bincode` 1.3 warning while preserving a deterministic top-50
similar-pairs contract.

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

The remaining `bincode`, Candle/transitive `paste`, and font/rendering stack
warnings should be revisited when upstream releases expose compatible fixes or
when a replacement design is intentionally proposed.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
