# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Audit warning ledger refresh

**Status.** RFC 027 proposed.

**Why now.** The current `cargo audit` gate passes, but its allowed-warning
output no longer matches the recorded audit-warning ledger exactly. The latest
observed audit surface is four allowed warnings: `bincode`, `paste`,
`rustybuzz`, and `ttf-parser`. The existing note tracks `ttf-parser` but not
the newer `rustybuzz` warning, and the release docs mostly describe explicit
`.cargo/audit.toml` ignores rather than the broader "allowed warning with
recorded owner/rationale" state.

This theme is documentation and release-gate hygiene only: reconcile the audit
ledger with observed output, clarify the release-gate wording, and avoid adding
new ignores or dependency changes without a separate design.

## Recently implemented, unreleased

### Explorer tree maintenance

**Status.** RFC 026 implemented; unreleased.

**Why now.** The workspace now locks `iced-swdir-tree` 0.9.3 on the accepted
0.9 line, and the cache update path documents that media `DirNode` discovery is
separate from the folder-only aside tree UI state.

### Cache serialization dependency strategy

**Status.** RFC 023 implemented; unreleased.

**Why now.** The current `localcache` 0.20 bincode-backed cache payload path was
retained because no published or local bincode-free `localcache` dependency
route is available yet.

### Release smoke checklist

**Status.** RFC 025 implemented; unreleased.

**Why now.** The release-readiness review called out manual GUI smoke as a
reasonable owner-managed check before a release point. The developer testing
docs now provide a concise smoke checklist for setup, gallery/indexing,
similarity, cache, settings/theme, and restart behavior.

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
