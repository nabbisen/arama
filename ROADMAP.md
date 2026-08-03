# Roadmap

This roadmap records near-term project themes that are specific enough to
guide planning but not yet implementation promises. Non-trivial themes move
through the RFC process before code changes begin.

## Current focus

### Cross-platform external FFmpeg closeout

**Status.** Complete. RFC 032 implemented (unreleased); implementation closeout
accepted 2026-08-03; RFC 031 archived as superseded in the same change.

**Outcome.** Arama removed managed FFmpeg acquisition and uses a user-managed
`ffmpeg`/`ffprobe` pair on every supported platform. The source-build baseline
is truthful and CI-enforced (RFC 033), the automated absence tooling passes for
source and Cargo package listings, Linux x86_64 passed real-media smoke, and
Windows/macOS carry an explicit owner risk acceptance recorded in
[`rfcs/notes/native-smoke-risk-acceptance.md`](./rfcs/notes/native-smoke-risk-acceptance.md).

Three defects were found and fixed during closeout — a release-blocking
Selected-directory startup hang, a missing required Setup action, and a
prohibited download affordance — none of which automated gates detected. Two
items are carried into release prep: archive and built-executable
artifact-absence inspection, which has never run because no artifacts existed,
and the ~15 release-smoke rows outside the FFmpeg subset.

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
   [RFC 033](./rfcs/done/033-cache-dependency-and-rust-baseline.md) is
   Implemented (Unreleased) and moved to `rfcs/done/` on 2026-08-01; the
   deferred dialog-routing work is recorded in its Status section pending a
   follow-up RFC.
2. **Implementation closeout review — accepted 2026-08-03.** The
   external-FFmpeg contract check, Linux real-media smoke, rendered setup and
   Settings smoke, user and maintainer documentation, source and package
   absence evidence, and the MSRV gate were reviewed as one bounded checkpoint.
   Three defects were found, fixed, and re-verified before acceptance. Archive
   and built-executable inspection is carried into release prep.
3. **Native evidence review — decided 2026-08-01, re-confirmed 2026-08-03.**
   Linux x86_64 is the executed target and passed real-media and rendered smoke;
   Windows x86_64 and Apple Silicon macOS are `not run` with explicit owner risk
   acceptance, re-confirmed knowing what closeout found; Intel macOS and Linux
   aarch64 are closed on near-zero incremental value. The acceptance, the
   specific risks, and the supporting evidence are recorded in
   [`rfcs/notes/native-smoke-risk-acceptance.md`](./rfcs/notes/native-smoke-risk-acceptance.md).
   No result is inferred from another platform.
4. **RFC reconciliation — done 2026-08-03.**
   [RFC 031](./rfcs/archive/031-macos-ffmpeg-trust-boundary.md) archived as
   superseded; [RFC 032](./rfcs/done/032-cross-platform-external-ffmpeg.md)
   moved to `done/` as Implemented (Unreleased); the RFC index, handoff, this
   roadmap, and all cross-references updated in the same atomic change.
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

**Why now.** The current `localcache`/bincode-backed cache payload path was
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

### Native smoke on CI runners

**Owner-accepted 2026-08-01; to land before the next release, not this one.**

Extend automated native smoke to the `windows-latest` and `macos-latest`
runners the release workflow already uses, covering the two highest-value
targets without owning hardware. Needs its own RFC: RFC 033 Part F fenced CI
expansion as a separate theme, and the runners must install a trusted
`ffmpeg`/`ffprobe` pair with the ignored smoke parameterised for them. It
cannot cover Finder-launch `PATH` inheritance or rendered UI, which stay
desktop-only. Supersedes the corresponding rows in
[`rfcs/notes/native-smoke-risk-acceptance.md`](./rfcs/notes/native-smoke-risk-acceptance.md)
once it lands.

### Similarity-dialog cache-error tier routing

Deferred from RFC 033 Part B. The similarity dialogs route **every**
`CacheError` variant to `eprintln!` and render a partial or empty result. The
gap predates RFC 033 and is variant-agnostic, so the RFC should classify all
variants under RFC 017 rather than only the `Poisoned` case. Requires UX
decisions — partial-versus-empty semantics, toast versus inline placement, new
localized strings.

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
