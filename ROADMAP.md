# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Cross-platform external FFmpeg closeout

**Status.** RFC 032 proposed; production migration committed; closeout
evidence and lifecycle acceptance pending.

**Why now.** Arama has removed managed FFmpeg acquisition and now uses a
user-managed `ffmpeg`/`ffprobe` pair on every supported platform. The remaining
work is to make the source-build baseline truthful, review the automated
artifact-absence and real-media smoke tooling, record or explicitly defer
native-platform evidence, and reconcile RFCs 031/032 before any release
decision.

RFC 031 remains proposed only as retained history and security context. RFC 032
supersedes its managed Linux/Windows policy; the lifecycle transition is
deliberately deferred until RFC 032 completion evidence is accepted.

### Near-term milestones

1. **Source-build baseline (RFC 033) — implemented, unreleased.**
   `localcache` moved to 0.21.0: the `rusqlite`/toolchain resolution defect is
   fixed, and `ReadPool`'s silent poisoned-guard recovery is replaced with a
   reported error, proven by a crate-internal test at pool size 1 (the
   condition under which `ReadPool::checkout`'s blocking fallback, not its
   `try_lock` scan, reports poisoning). Rust 1.91 is declared as arama's
   verified contributor-setup baseline (`[workspace.package].rust-version`,
   the workspace's only normative declaration) and enforced by a new,
   single `MSRV` CI job that reads the declaration from `Cargo.toml` rather
   than restating it, on push and pull request against `main`. The baseline
   is deliberately **not** the lowest version the graph permits — arama is
   an application, not a library, so the declaration exists for contributor
   setup and release stability. Similarity-dialog cache-error tier routing
   was explicitly deferred to a follow-up RFC (RFC 033 Part B); the Cache
   page already satisfied its blocking-view case with no change needed.
   [RFC 033](./rfcs/proposed/033-cache-dependency-and-rust-baseline.md)
   remains in `rfcs/proposed/` pending a separate, owner-authorized
   lifecycle move to `rfcs/done/`.
2. **Implementation closeout review.** Review the external-FFmpeg contract
   check, native real-media smoke, maintainer documentation, package/source
   absence evidence, and available MSRV/cross-target gates as one bounded
   implementation checkpoint.
3. **Native evidence review.** Record Linux, Windows, and macOS owner-smoke
   results independently. Mark unavailable hardware `not run` or
   `environment-dependent`; never infer a native result from another platform.
4. **RFC reconciliation.** After completion evidence is accepted, archive RFC
   031 as superseded by RFC 032, move RFC 032 to implemented, and update the RFC
   index, roadmap, and cross-references atomically.
5. **Release checkpoint.** Treat the completed external-FFmpeg theme and the
   accumulated unreleased RFC batch as a candidate release boundary. Audit,
   packaging, versioning, and publication remain separate owner-authorized
   work. Before release authorization, every required native target row must
   record `pass`, `fail`, `not run`, or `environment-dependent`; every executed
   row must pass, any failure blocks release, and every unavailable row requires
   an explicit owner risk acceptance rather than an inferred pass. At least one
   available native target must pass the real-media and artifact-absence
   checks.

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

### ELOC remeasurement and focused splits

Re-run an effective-line-count sweep before the next broad implementation
batch. Prioritize files that exceed the 300-ELOC consideration threshold, but
open a split RFC only after exact measurements identify a coherent scope. A
preliminary sweep found no file above the 500-ELOC strongly recommended
threshold.

### Remaining audit-warning owners

The remaining `bincode`, Candle/transitive `paste`, and font/rendering stack
warnings should be revisited when upstream releases expose compatible fixes or
when a replacement design is intentionally proposed.

### Release prep

Release prep remains owner-driven. The roadmap does not make a release point;
it only identifies when a coherent reviewed batch may be ready for release
consideration.
