# RFC 032 Handoff — Cross-platform external FFmpeg

Companion to [RFC 032](../done/032-cross-platform-external-ffmpeg.md).
RFC 032 is **Implemented (Unreleased)**; its implementation closeout was accepted
2026-08-03 and RFC 031 was archived as superseded in the same change. This
handoff is retained as the historical execution record for that work.

## 1. Design authority and precedence

The implementation and its reviews must be checked against:

1. [RFC 032 — Cross-platform external FFmpeg](../done/032-cross-platform-external-ffmpeg.md),
   the primary product and cross-platform policy;
2. [RFC 031 — macOS ffmpeg trust boundary](../archive/031-macos-ffmpeg-trust-boundary.md),
   for the paired-toolchain, bounded-process, legacy-exclusion, and macOS
   security requirements retained by RFC 032;
3. [RFC 030 — Distribution and version contract reconciliation](../done/030-distribution-and-version-contracts.md),
   for the distinct source archive, executable asset, and Cargo package
   contracts; and
4. [RFC 029 — Release smoke evidence template](../done/029-release-smoke-evidence-template.md)
   and [RFC 025 — Release smoke checklist](../done/025-release-smoke-checklist.md),
   for owner-recorded native and rendered-UI evidence.

When the documents overlap, RFC 032 controls the acquisition policy: arama
must not bundle, download, extract, install, update, or invoke a package manager
for FFmpeg on any platform. RFC 031 remains authority only for the security
mechanics that RFC 032 explicitly retains.

Implementation review packages must cite these design paths and this handoff.
They must also cite the applicable accepted design-review records available in
the review workspace so a reviewer who did not participate in design review can
reconstruct the decision history.

## 2. Implementation handoff

**Goal.** Finish and verify the migration from arama-managed FFmpeg to one
external, user-owned `ffmpeg`/`ffprobe` policy on Linux, Windows, and macOS.

The non-negotiable implementation invariants are:

- setup, Settings, re-check, and directory selection never acquire FFmpeg
  bytes or invoke an installer/package manager;
- auto discovery and explicit selection validate a compatible same-directory
  pair under bounded path, output, time, and descendant-process policies;
- only a validated `FfmpegToolchain` can reach AI, cache, thumbnail, probe, or
  similarity consumers;
- a running task retains its captured authority even if the preference changes;
- CLIP alone completes setup, and absent FFmpeg preserves image-only use;
- legacy `.arama-local/bin` tools are ignored, retained, and never executed or
  deleted; and
- release artifacts and Cargo package listings contain no FFmpeg executable or
  archive payload, while production source contains no FFmpeg acquisition
  implementation.

Primary implementation seams:

- `env/src/config/settings/ffmpeg_location.rs` — persisted Auto/Selected
  preference contract;
- `crates/engine/sidecar/src/media/video/video_engine.rs` and
  `video_engine/discovery/` — private toolchain construction, bounded probing,
  path policy, coordination, and preference publication;
- `app/src/core.rs` and `app/src/core/update/ffmpeg.rs` — application-owned
  discovery runtime and published authority;
- `crates/ai/src/pipeline/extract/video_extractor.rs`,
  `crates/ai/src/pipeline/encode/image/embeddings.rs`, and
  `crates/ai/src/pipeline_manager/video_similarity_pipeline.rs` — captured
  consumer authority and image-only degradation;
- setup/settings UI and i18n — external guidance, re-check, select/change, and
  clear actions without a managed-install branch;
- `scripts/check-external-ffmpeg-contract.sh` — reproducible source, Cargo
  package, archive, and executable absence checks; and
- `crates/engine/sidecar/tests/external_ffmpeg_smoke.rs` — ignored owner-run
  native selected-pair real-media smoke.

## 3. Closeout task breakdown

1. **Resolve the source-build baseline.**
   - Reconcile arama's declared Rust version with the supported contract of
     `localcache` and its locked `rusqlite`/`libsqlite3-sys` graph.
   - Test the chosen exact toolchain; do not infer support from a newer compiler.
2. **Review automated absence evidence.**
   - Check production source for removed provider, downloader, installer,
     digest, archive, and activation identities.
   - Check every workspace `cargo package --list` result.
   - Inspect representative source and executable archives plus the built
     executable when available.
3. **Review real-media and native behavior.**
   - Run the ignored selected-directory smoke with a trusted native pair.
   - Record PATH/default-prefix, persistence/restart/re-check/change/clear,
     missing, legacy-only, mismatch, timeout cleanup, and no-network results per
     available native platform.
   - Record rendered setup/Settings behavior separately from headless tests.
4. **Request implementation completion review.**
   - Scope the request to observed evidence.
   - List unavailable native rows as `not run` or `environment-dependent`.
   - Keep lifecycle, versioning, packaging publication, commit, tag, and push
     outside the requested decision.
5. **Reconcile RFC lifecycle after acceptance.**
   - Archive RFC 031 with `Superseded by RFC 032`.
   - Move RFC 032 to `done/` with the correct implemented version/status.
   - In this handoff, update the header status note, change the RFC 031 link to
     `../archive/`, change the RFC 032 link to `../done/`, and replace the QA
     assertions that both RFCs remain Proposed.
   - Update the `rfcs/handoffs/README.md` index row from `Proposed; closeout
     pending` to the shipped version/status.
   - Update `rfcs/README.md`, ROADMAP, and all affected cross-references in the
     same atomic lifecycle change.
6. **Prepare a release only with separate owner authorization.**
   - Run the full release, audit, documentation, archive, and smoke gates.
   - Do not treat completion review as permission to version, commit, tag,
     publish, or release.

## 4. Acceptance and QA checklist

### Design and authority

- [ ] Implementation review cites RFCs 032, 031, 030, 029, and 025 plus this
  handoff and applicable accepted design reviews.
- [ ] No production seam can construct an FFmpeg authority without successful
  bounded same-directory validation.
- [ ] Every consumer receives a captured validated `FfmpegToolchain`.
- [ ] Missing FFmpeg keeps setup and image-only indexing usable.
- [ ] Legacy managed tools remain ignored, unexecuted, and undeleted.

### Acquisition and artifacts

- [ ] Production code contains no FFmpeg provider URL, artifact identity,
  downloader, installer, archive extraction, activation, or package-manager
  invocation.
- [ ] Sidecar dependencies contain no HTTP, digest, or archive acquisition
  stack.
- [ ] Cargo package listings contain no FFmpeg executable/archive payload.
- [ ] Representative source and executable archives contain no FFmpeg payload.
- [ ] The application executable contains no managed-acquisition identities.

### Evidence

- [ ] The chosen exact Rust baseline compiles and tests the locked workspace.
- [ ] Available non-Linux cross-target checks are recorded as compile evidence,
  not native-runtime evidence.
- [ ] Native smoke rows are recorded independently for each available
  platform/architecture.
- [ ] Every required native row has an explicit result; failures block release,
  unavailable rows require owner risk acceptance, and no result is inferred.
- [ ] At least one available native target passes real-media and
  artifact-absence checks.
- [ ] Real video probe and frame extraction use only the returned validated
  toolchain.
- [ ] Rendered setup/Settings and missing-tool degradation are observed or
  explicitly deferred.
- [ ] Format, check, Clippy, full tests, docs build, audit/release gates, and
  diff hygiene are reported only when their output was observed.

### Lifecycle and release boundaries

- [x] RFC 031 archived as superseded and RFC 032 moved to `done/` — done
      2026-08-03, after the closeout contract was accepted.
- [ ] Lifecycle reconciliation is performed as one reviewed change.
- [ ] Release/version/package publication remains separately owner-authorized.
