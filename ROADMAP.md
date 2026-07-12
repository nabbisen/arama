# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Release smoke checklist

**Status.** RFC 025 proposed for review.

**Why now.** The unreleased batch is technically clean against observed
automated gates, but the release-readiness review called out manual GUI smoke
as a reasonable owner-managed check before a release point. The current testing
doc has a short UI checklist, but it predates the recent setup, cache,
first-run, theme, and recoverable-error work.

**Planned design questions.**

- Which manual GUI workflows are release-critical enough to check before an
  owner-selected release point?
- Which checks require local model/ffmpeg setup, and which can run without
  network or clean first-run state?
- Where should the checklist live so it helps release preparation without
  implying that release actions are automated or delegated?
- Can any low-risk smoke checks be scripted later without building a brittle UI
  automation harness?

## Recently implemented, unreleased

### Cache serialization dependency strategy

**Status.** RFC 023 implemented; unreleased.

**Why now.** The current `localcache` 0.20 bincode-backed cache payload path was
retained because no published or local bincode-free `localcache` dependency
route is available yet.

### Image codec dependency minimization

**Status.** RFC 024 implemented; unreleased.

**Why now.** The workspace now disables unused default image codecs and keeps
only arama's accepted PNG, JPEG, WebP, GIF, and BMP decode path active. This
removes the AVIF/ravif/rav1e owner path from the active dependency graph while
leaving the remaining `paste` owners tracked.

### Image similarity search dependency strategy

**Status.** RFC 022 implemented; unreleased.

**Why now.** `hnsw_rs` was replaced with exact bounded pairwise image search,
removing the `bincode` 1.3 warning while preserving a deterministic top-50
similar-pairs contract.

### Cache lifecycle

**Status.** RFC 015 implemented; unreleased.

**Why now.** RFC 002 moved arama from the old `file-feature-cache` engine to
`localcache` in v0.23.0. RFC 015 retires the temporary v1 migration path and
keeps cache-size/disk-pressure management split into a separate design.

**Follow-up status.** RFC 016 implemented; unreleased.

### Visible recoverable error UX

**Status.** RFC 017 implemented; unreleased.

### AI and video pipeline resilience

**Status.** RFC 018 implemented; unreleased.

### Startup fatal-boundary resilience

**Status.** RFC 019 implemented; unreleased.

### Audit warning burn-down

**Status.** Maintenance pass implemented; unreleased.

**Why now.** Compatible patch-level RustSec warnings for `anyhow` and `memmap2`
were resolved. Remaining allowed warnings are tracked in
[`rfcs/notes/audit-warning-burn-down.md`](./rfcs/notes/audit-warning-burn-down.md).

### Dependency modernization

**Status.** RFC 020 implemented; unreleased.

**Why now.** First-party Candle dependencies moved to 0.11 and non-Linux
sidecar ZIP extraction moved to stable `zip` 8.6.0. `pt2safetensors` remains
as the only Candle 0.10 owner.

### CLIP SafeTensors source strategy

**Status.** RFC 021 implemented; unreleased.

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
