# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

No active implementation theme is selected. The most recent reviewed work is
recorded below as implemented but unreleased; release timing remains
owner-managed.

## Recently implemented, unreleased

### Distribution and version contract reconciliation

**Status.** RFC 030 implemented; unreleased.

**Why now.** Source, executable, and crates.io distribution contracts are now
distinct, while the version helper remains independent of workspace topology
and updates only the package version inherited by members.

### Release smoke evidence template

**Status.** RFC 029 implemented; unreleased.

**Why now.** The RFC 025 checklist now has stable smoke IDs and a reusable
owner evidence template for recording pass, fail, not-run, and
environment-dependent results without adding desktop UI automation or
performing release actions.

### Source TODO hygiene

**Status.** RFC 028 implemented; unreleased.

**Why now.** Stale source TODO comments were removed or replaced with current
design-boundary rationale, and the undeclared gallery subscription legacy source
was deleted without changing runtime behavior.

### Audit warning ledger refresh

**Status.** RFC 027 implemented; unreleased.

**Why now.** The audit-warning ledger now matches the current `cargo audit`
allowed-warning surface: `bincode`, `paste`, `rustybuzz`, and `ttf-parser`.
Release-gate docs distinguish explicit `.cargo/audit.toml` ignores from
allowed warnings with recorded owner paths and revisit conditions.

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
