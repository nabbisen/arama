# RFC 032: Cross-platform external FFmpeg

**Status.** Implemented (0.37.0) — implementation closeout accepted
2026-08-03. Supersedes [RFC 031](../archive/031-macos-ffmpeg-trust-boundary.md),
now archived.

**Native verification.** Linux x86_64 passed real-media smoke; Windows x86_64 and
Apple Silicon macOS are recorded `not run` under an explicit owner risk
acceptance; Intel macOS and Linux aarch64 are closed as out of scope. See
[`rfcs/notes/native-smoke-risk-acceptance.md`](../notes/native-smoke-risk-acceptance.md).
Archive and built-executable artifact-absence inspection is deferred to release
packaging and has not yet run.
**Tracks.** Replace arama-managed FFmpeg acquisition with user-managed external
toolchains on Linux, Windows, and macOS.
**Touches.** `crates/engine/sidecar`, `crates/ui/main`, `crates/ui/widgets`,
`crates/ai`, `crates/cache`, `env`, `app`, i18n, setup/settings UI, user and
developer documentation, `NOTICE`, release gates, and RFC 031.

## Summary

Arama will not bundle, mirror, download, extract, install, update, or invoke a
package manager for FFmpeg on any platform. Users obtain `ffmpeg` and
`ffprobe` independently. Arama only discovers or accepts a user-selected
directory, validates a compatible same-directory pair with bounded probes,
and uses the resulting `FfmpegToolchain` authority.

The required first-run gate becomes CLIP-only on Linux, Windows, and macOS.
FFmpeg and Wav2Vec2 are optional video capabilities. Their absence must not
block image-only use, reopen setup after restart, or prevent images from being
indexed when video is enabled.

This RFC supersedes RFC 031 only where RFC 031 retains or governs managed
Linux/Windows FFmpeg acquisition. RFC 031 remains the authority for the
accepted paired-toolchain validation, bounded-process, legacy-exclusion, and
macOS external-acquisition security work until both RFCs are reconciled at
implementation completion.

## Decision and motivation

The project owner approved the architect's Option A decision: external FFmpeg
on every platform.

The managed Linux/Windows design failed both operationally and as a durable
distribution contract:

- all three pinned GitHub release asset IDs became unavailable;
- the upstream `latest` release rotates binary identities and digests;
- the deleted archives cannot be tied to retained license material, exact
  build inputs, or corresponding source;
- rotating asset IDs would restore availability only temporarily;
- retaining a managed GPL or LGPL binary channel would require recurring
  provenance, source-delivery, license, availability, and incident-response
  work disproportionate to this project's capacity.

Arama and FFmpeg remain separate programs: arama does not link FFmpeg code and
uses ordinary process arguments, files, stdout, and stderr. External ownership
does not claim cryptographic authentication of a local installation. It removes
FFmpeg binary procurement and redistribution from arama's product contract.

## Goals

- Remove every production FFmpeg network-download and archive-install path.
- Use one external-acquisition policy on Linux, Windows, and macOS.
- Preserve one validated same-directory `ffmpeg`/`ffprobe` authority.
- Keep probe output, time, and descendant-process lifetime bounded.
- Give desktop users an explicit-directory fallback without filesystem scans.
- Make CLIP the only setup-completion requirement on every platform.
- Keep image indexing usable when video capabilities are absent.
- Ignore, retain, and clearly explain legacy arama-managed FFmpeg files.
- Remove dead artifact metadata and dependencies rather than leaving a dormant
  installer.
- Make release evidence prove that arama ships and procures no FFmpeg binary.

## Non-goals

- No arama-built, hosted, mirrored, cached, or signed FFmpeg distribution.
- No choice between upstream GPL and LGPL binary variants.
- No automatic package-manager, shell, privilege, quarantine, signing, or
  environment mutation.
- No recursive search of drives, program directories, registries, or package
  databases.
- No mixing `ffmpeg` and `ffprobe` from different logical directories.
- No cryptographic claim about executables selected from the user's machine.
- No automatic deletion of legacy `.arama-local/bin` files.
- No AI model, media format, cache schema, similarity algorithm, release,
  version, packaging, tag, commit, or push action in the proposal change.

## Acquisition invariant

On every target OS, production code must satisfy:

```text
arama network/setup/settings code
              │
              ├── must not fetch or install FFmpeg
              │
              ▼
user-owned acquisition outside arama
              │
              ▼
auto-discovered or explicitly selected directory
              │
              ▼
bounded same-directory pair validation
              │
              ▼
       FfmpegToolchain authority
              │
              ▼
 AI / cache / thumbnail / probe / similarity consumers
```

There is no platform exception. Enabling video, opening setup, pressing
re-check, selecting a directory, restoring settings, or encountering a missing
pair must not create an FFmpeg network request or install process.

Documentation may describe commands that the user can run independently. The
application must not execute them. Documentation must prefer stable
platform/package-manager guidance rather than directing delivery of a pinned
binary chosen by arama.

## Selection model

### Persisted preference

Add a serde-defaulted setting with two states:

```rust
enum FfmpegLocationPreference {
    Auto,
    SelectedDirectory(PathBuf),
}
```

Existing settings deserialize as `Auto`. Persist a directory, not two
executable paths and not an archive identity. The path is user configuration,
not evidence that a toolchain remains valid.

The application composition root loads and saves this preference and passes it
to discovery/UI consumers. Avoid a mutable process-global preference. The
locator returns an unforgeable `FfmpegToolchain` whose executable fields remain
private; production consumers obtain commands or paths only from that value.

### Auto mode

Auto mode tries a finite ordered candidate list:

| Platform | Candidates |
|---|---|
| Linux | inherited `PATH`, in order |
| Windows | inherited `PATH`, in order |
| Apple Silicon macOS | inherited `PATH`, then `/opt/homebrew/bin` |
| Intel macOS | inherited `PATH`, then `/usr/local/bin` |

One discovery attempt has all of these fixed bounds:

```text
MAX_RAW_PATH_ENTRIES = 64   (Linux, macOS)  /  256 (Windows)
MAX_PATH_CANDIDATES  = 32   (Linux, macOS)  /  128 (Windows)  unique normalized absolute directories
PROBE_TIMEOUT        = 2 seconds per executable, all platforms
AUTO_ATTEMPT_TIMEOUT = 6 seconds total on a monotonic clock, all platforms
```

**Amended by RFC 039 (2026-08-15): the two entry-count bounds are
platform-conditional; the two time bounds are not.**

These are reachability bounds, not performance bounds:
`.take(MAX_RAW_PATH_ENTRIES)` is applied to the raw `PATH` iterator before any
entry is inspected, so an entry beyond the cap is never canonicalized, never
checked, and never becomes a candidate — its position, not its validity,
decides whether discovery ever sees it. macOS has a native-prefix candidate
reserved *after* this cap (see below), so its own bound is never
reachability-blocking regardless of `PATH` length; Windows has no such
fallback; Linux has none either but no evidence suggests its `PATH`s
approach these numbers. Windows is therefore the one platform where these
bounds are load-bearing for reachability and the one platform with nothing
behind them — the asymmetry is deliberate, not a general license for
per-platform tuning of future bounds.

The Windows values are calibrated against a real measurement, not a round
increase: `windows-latest` (RFC 038's native-smoke runner) presented **78**
raw `PATH` entries and **66** unique, existing candidate directories on
2026-08-15, both above the original Linux/macOS defaults. 256 / 128 give
roughly 3x / 2x headroom over that measurement — a ceiling with room, not a
value fitted to one data point. The same measurement found Windows
filesystem collection cost at ~80µs per raw entry, so a full 256-entry scan
costs on the order of tens of milliseconds against the 6-second
`AUTO_ATTEMPT_TIMEOUT` — three orders of magnitude of headroom. The time
bounds therefore do not move: raising the entry-count bounds does not
reintroduce the risk they exist to prevent, and `AUTO_ATTEMPT_TIMEOUT`'s job
of bounding wall-clock and `PROBE_TIMEOUT`'s job of bounding subprocess cost
per executable are unaffected by how many raw entries are scanned before a
match or a confident `Missing`.

Raising `MAX_RAW_PATH_ENTRIES` alone without `MAX_PATH_CANDIDATES` would have
been a change with no observable effect on Windows: the real measurement
above (66 candidates from 78 raw entries) already exceeds the original
32-candidate cap, confirmed by execution during RFC 039's own measurement
step — an instrumented run with the raw cap raised and the candidate cap
left at 32 still returned `SearchLimitReached(CandidateCount)`. Both moved
together.

The applicable macOS native prefix is a separately reserved final candidate,
so PATH saturation cannot remove it; total candidates are at most 33. The
whole-attempt deadline still applies. Before each filesystem operation and
probe, discovery checks the remaining total budget. Each process deadline is
the smaller of `PROBE_TIMEOUT` and the remaining attempt budget. Budget expiry
kills/reaps the active probe group and returns `SearchLimitReached`, never
`Missing`. A coordinator-side monotonic timer also publishes
`SearchLimitReached(WholeAttempt)` at six seconds even if the blocking worker
is stuck inside a filesystem call, so the UI never displays Checking beyond
the attempt budget.

PATH processing is deterministic:

- inspect at most the first `MAX_RAW_PATH_ENTRIES` raw entries (64, or 256 on
  Windows per the RFC 039 amendment above) and remember truncation;
- reject empty entries rather than interpreting them as the current directory;
- reject every relative entry rather than joining it to a mutable working
  directory;
- accept only absolute directories;
- resolve an existing candidate once and deduplicate by directory file identity
  (same-file semantics), so symlink aliases are probed once;
- on Windows, logical equality must also normalize drive/UNC prefixes and
  separators and use Windows case-insensitive path semantics; case or separator
  spelling cannot cause a second probe;
- if identity resolution/access fails, skip the candidate and retain a typed
  filesystem diagnostic.

If raw or normalized candidate limits truncate the search, a later valid pair
inside the inspected candidates may still return Ready. If no pair succeeds,
truncation returns `SearchLimitReached(CandidateCount)`. Duplicate logical
directories are probed once. No other implicit prefixes, registry keys,
package-manager queries, or recursive scans are added without a future
reviewed RFC amendment.

The subprocess and candidate-count bounds are enforceable by arama. A platform
filesystem call may itself block inside the OS beyond the requested deadline;
the coordinator must still ignore stale publication, bound the number of such
operations, and record this residual limitation rather than claiming hard
cancellation of arbitrary kernel I/O.

### Selected-directory mode

The user selects one directory through the native directory picker. Arama
looks for the platform executable names for both tools directly inside that
directory. It does not accept selection of only one executable. A Selected
directory must be non-empty and absolute; a relative value returned by a
picker or manually placed in settings produces `InvalidSearchPath` and is
never resolved against the process working directory.

While selected-directory mode is active:

- the selected directory is the sole candidate;
- invalidation produces an actionable selected-directory error;
- arama does not silently fall back to `PATH` or a Homebrew prefix;
- **Change** selects another directory;
- **Clear selection / Use automatic discovery** returns to Auto mode.

The pair is revalidated after selection, on re-check, at application start
when video work is requested, and before a newly created long-running video
task captures its toolchain. A running task may retain its already validated
pair; later tasks observe the updated preference.

This explicit mode is required as a Windows desktop fallback and is available
consistently on every platform.

### Discovery outcomes

Discovery returns a typed outcome rather than `Option<FfmpegToolchain>`:

```rust
enum FfmpegDiscoveryOutcome {
    Ready { toolchain: FfmpegToolchain, source: DiscoverySource },
    Missing,
    InvalidPair(PairIssue),
    ProbeTimedOut,
    SearchLimitReached(SearchLimit),
    LegacyLocationExcluded,
    InvalidSearchPath,
    FilesystemUnavailable(FilesystemIssue),
}
```

`PairIssue` distinguishes a missing member, malformed version output, version
mismatch, and bounded-output failure. `SearchLimit` distinguishes raw/candidate
count, whole-attempt deadline, and an earlier timed-out worker still draining.
`FilesystemIssue` distinguishes access from other metadata/identity failures
without carrying private paths.

Selected mode reports the exact applicable class. Auto mode may encounter
multiple failures; if no pair is Ready, publish the highest-priority observed
class in this order: search-limit, probe-timeout, filesystem, invalid-pair,
invalid-search-path, excluded-legacy, then ordinary missing. UI text is
actionable but durable logs/review evidence contain only the class and source
category unless a path is explicitly redacted. Raw probe output is never shown.

Cancellation is an internal coordinator result, not a user-facing discovery
outcome.

### Preference transaction and startup ordering

The persisted preference is authority. Startup must load or default `Settings`
before constructing Setup, Settings UI, the locator, or any task capable of
FFmpeg discovery. The locator is initialized once from that loaded preference;
no preliminary Auto probe may start and later overwrite Selected mode.

A newly picked directory follows validate-before-persist semantics. An invalid
selection is not saved and does not replace the previous active preference.
A previously persisted Selected directory that later becomes invalid remains
the sole authority and enters Selected/invalid until the user changes or clears
it.

| Event | Validation and persistence order | Published authority/state |
|---|---|---|
| Startup Auto | Load preference, then discover | Auto + typed outcome |
| Startup Selected | Load preference, validate only selected directory | Selected + Ready or typed invalid outcome |
| Pick valid directory | Preflight JSON serialization, validate, save full settings, then publish | New Selected + Ready |
| Pick invalid directory | Preflight, validate; do not save | Prior authority retained; candidate error shown |
| Pick non-serializable directory | Reject before validation/save | Prior authority retained; recoverable persistence error |
| Picker cancel | No validation or save | Prior authority/state unchanged |
| Valid candidate but save fails | Do not publish candidate | Prior authority retained; recoverable save error |
| Re-check | No save; validate current authority | Same preference + new typed outcome |
| Clear selection succeeds | Save Auto, publish Auto, then discover | Auto + Checking, then typed outcome |
| Clear selection save fails | Do not publish Auto or discover it | Prior Selected authority retained |

Saving precedes in-memory publication so a crash after save is recovered by
startup loading and revalidating the new preference. A successful selection
does not launch a second redundant discovery: its validation result becomes
the first Ready result for the newly published Selected authority.

JSON settings cannot represent every non-Unicode native Unix path. The picker
result must pass an explicit serialization preflight; failure uses the same
recoverable, prior-authority-preserving rule as a settings save failure. No
in-memory-only selection is permitted.

### Asynchronous coordination

One `FfmpegDiscoveryCoordinator` owns generations, cancellation, and worker
lifetime. It permits at most one active worker and one replaceable pending
request:

- a newer request increments the generation, replaces any pending request, and
  sets the active worker's cancellation token;
- the worker checks cancellation before/after filesystem work and while polling
  each child;
- cancellation kills/reaps an active probe group and returns promptly within
  the process bound;
- the coordinator drains/joins the cancelled worker before starting the newest
  pending request;
- the coordinator's timer may publish `SearchLimitReached(WholeAttempt)` and
  mark the worker stale without waiting for a blocked kernel filesystem call;
- while such a stale worker has not drained, a newer request replaces the one
  pending slot and publishes `SearchLimitReached(WorkerDraining)` rather than
  displaying an unbounded Checking state or spawning another worker;
- only a result matching both the current generation and current preference may
  publish UI/toolchain state;
- dropping an iced task or rejecting a stale message alone is not considered
  cancellation evidence.

This serialization prevents repeated re-check/select events from accumulating
unbounded `spawn_blocking` workers. Under normal process cancellation the
newest generation remains Checking only while the older worker drains within
the process bound; a stuck filesystem worker instead produces the explicit
WorkerDraining diagnostic until the queued latest request can begin.

## Pair validation and process boundary

Retain RFC 031's accepted authority:

- both executable files must exist in the same logical candidate directory;
- canonical/logical comparison must permit normal symlinks without combining
  candidates;
- `ffmpeg -version` and `ffprobe -version` run off the render thread;
- stdout is bounded and stdin/stderr handling cannot deadlock the probe;
- each probe has a finite deadline;
- timeout/output/error paths kill and reap the process group or Windows child
  tree covered by the implementation contract;
- both commands must parse successfully and report the same exact
  release/build token;
- consumers cannot construct `FfmpegToolchain` from arbitrary paths.

Compatibility validation does not authenticate user-controlled executables.
The UI and security documentation must say that Auto mode trusts the user's
process environment and Selected mode trusts the user's explicit choice.

## Legacy managed files

On every platform, exclude the entire arama local-bin authority—including
`.arama-local/bin`, the `ffmpeg-managed` child, lexical descendants, and
resolvable filesystem aliases/canonical descendants—from both Auto and
Selected modes.

Legacy files:

- are never executed by arama;
- are never used as a fallback;
- are not automatically deleted, moved, quarantined, or modified;
- remain the user's responsibility to remove after installing an external
  pair;
- produce a specific migration explanation if explicitly selected.

This exclusion prevents the rejected managed distribution from surviving as
an undocumented trusted source. It does not claim to detect a copied binary or
an arbitrary hard link whose path/file identity cannot be resolved to the
legacy directory; explicit selection of such a user-controlled external copy
remains inside the documented local trust boundary.

## Setup and readiness

### Required capability

CLIP is the sole setup-completion requirement on every platform:

```text
setup_ready = clip_ready
```

Once CLIP is authenticated, the user can enter the application and subsequent
restarts must not reopen setup because FFmpeg or Wav2Vec2 is absent.

### Optional capabilities

Setup may show FFmpeg and Wav2Vec2 as optional video capabilities, but it must
not start either action without an explicit user request. FFmpeg offers only:

- platform-appropriate external installation guidance;
- **Re-check**;
- **Select directory**;
- **Continue image-only**.

There is no **Get**, **Download**, **Install**, progress bar, or retry-download
state for FFmpeg on any platform. Wav2Vec2 remains an authenticated model
download owned by arama and is clearly distinguished from external executable
acquisition.

## Settings and platform UX

Settings → AI exposes the same capability model after first run:

| State | Required behavior |
|---|---|
| Auto / checking | Non-blocking local probe and checking state |
| Auto / ready | Ready plus source category; re-check and select-directory actions |
| Auto / missing | External install guidance; re-check and select-directory actions |
| Selected / ready | Ready plus locally displayed selected directory; re-check/change/clear actions |
| Selected / invalid | Actionable reason; change/clear actions; no silent fallback |
| Legacy selected | Explicit unsupported legacy-location migration message |

Guidance matrix:

| Platform | UI guidance | Discovery fallback |
|---|---|---|
| Linux | Install `ffmpeg` and `ffprobe` with the system package manager | PATH or selected directory |
| Windows | Install a trusted paired FFmpeg distribution, then select its `bin` directory if PATH is unavailable | PATH or selected directory |
| macOS | `brew install ffmpeg` as the recommended user-run command | PATH, native Homebrew prefix, or selected directory |

The UI does not embed a mutable binary asset URL. Maintained user docs may add
reviewed package-manager examples, but commands are copyable text only and
arama never runs them.

Directory-picker cancel is a no-op. Invalid, non-serializable, or unsavable new
selections keep the prior preference and show a recoverable error. A selected
directory may be displayed locally, but durable review evidence must redact
private paths.

## Image and video behavior

- Image discovery, thumbnails, CLIP embedding, similarity, and cache work must
  continue when no FFmpeg pair exists.
- If video is enabled but FFmpeg is unavailable, skip video work and surface
  one actionable capability warning per indexing generation rather than one
  error per file.
- A missing pair must not abort unrelated images or discard existing image
  cache results.
- With FFmpeg ready and Wav2Vec2 absent, video frame extraction and CLIP video
  features remain usable; audio embeddings are optional.
- Existing video cache entries remain readable where FFmpeg is not required.
  Operations that require probing, normalization, or thumbnail regeneration
  report the missing capability truthfully.
- When the preference changes, new work uses a newly validated toolchain. An
  active task retains its captured authority or is cancelled through the
  existing task lifecycle; it must not read changing raw settings mid-command.

## API and dependency removal

Remove, rather than deprecate indefinitely, the managed installer surface once
all consumers migrate:

- `DownloadArtifact` and supported artifact constants/IDs/digests;
- `FfmpegDistribution::Managed` and managed-source status;
- `download_artifact()`, `download_and_install()`, digest verification,
  bounded artifact buffering, archive extraction, activation, and rollback;
- setup/Settings install commands, messages, states, progress, and tests;
- managed local-bin candidate preference;
- FFmpeg-specific HTTP/archive dependencies from `arama-sidecar`, including
  `reqwest`, `sha2`, `tar`, `xz2`, and `zip` when source search confirms no
  remaining use.

Retain `tokio` for blocking discovery, `command-group` and platform process
support for bounded probes, and only dependencies required by external
validation.

Before deletion, inventory all public compatibility APIs and downstream crate
consumers. No dormant download/extraction entry point or raw managed-local path
accessor may remain callable in production.

## Security and privacy model

### Removed risks

- network acquisition and execution of third-party FFmpeg bytes;
- rotating artifact URLs/digests and deleted-release availability;
- archive traversal/extraction and partial executable publication;
- project-owned FFmpeg binary license/source delivery;
- pressure to bypass verification when upstream assets disappear.

### Retained risks

- a malicious or compromised executable on user-controlled `PATH`;
- a malicious executable in a user-selected directory;
- package-manager/repository compromise outside arama;
- tool replacement after one validation and before later use;
- malicious media processed by FFmpeg with the user's permissions.

Mitigations are truthful provenance messaging, explicit selection, per-task
revalidation/capture, same-directory compatibility validation, bounded probes,
least-surprise candidate ordering, and no privilege escalation. Code-signing
or package-repository authentication is outside arama's current authority.

## Compatibility and migration

- Existing settings load as Auto through serde defaults.
- Existing image caches and embeddings require no migration.
- Existing video cache records remain; unavailable regeneration is reported as
  a capability limitation.
- Existing Linux/Windows users with only arama-managed binaries lose video
  processing until they install/select an external pair.
- Legacy binaries stay on disk untouched and may be removed manually.
- No automatic attempt converts a legacy directory into Selected mode.
- The app remains Apache-2.0; FFmpeg is a separately acquired external program
  with license/provenance determined by the user's source.

## Required automated test design

### Policy and API tests

- source/API assertions prove there is no production FFmpeg URL, artifact ID,
  digest table, HTTP request, archive extractor, installer, or managed-source
  status;
- compile/API tests prove callers cannot construct `FfmpegToolchain` from raw
  paths or install an archive;
- dependency inspection proves FFmpeg-only HTTP/archive crates are removed.

### Selection tests

- existing settings without the new field deserialize to Auto;
- settings are loaded before Setup/locator construction and no preliminary
  Auto generation can overwrite a persisted Selected preference;
- Auto preserves PATH order, rejects empty/relative entries without current-dir
  resolution, and deduplicates resolvable logical directory aliases;
- Windows tests cover case, separator, drive-prefix, and UNC-equivalent
  directory spellings without duplicate probes;
- raw-entry/candidate limits and the six-second total budget produce typed
  search-limit outcomes and terminate/reap active probe children;
- platform candidate matrices contain only the specified finite candidates;
- Selected mode uses only its directory and never silently falls back;
- pure transition tests cover valid/invalid selection, non-Unicode serialization
  failure, cancel, save failure, re-check, clear, restart, and two overlapping
  generations;
- cancellation tests prove at most one active/one pending worker, pending
  replacement, worker drain, child cleanup, and stale-publication rejection;
- both executable names must exist directly in one directory;
- mixed-directory, malformed, mismatched, and hanging candidates are rejected;
- symlinked same-directory pairs are accepted without combining candidates;
- lexical descendants and resolvable aliases/canonical descendants of arama
  local-bin are rejected in Auto and Selected modes;
- timeout descendants are killed/reaped and output remains bounded.

### Setup and settings tests

- CLIP-only readiness succeeds identically on Linux, Windows, and macOS;
- absence of FFmpeg/Wav2Vec2 does not reopen setup after reconstruction;
- no setup or Settings event can select a download/install command;
- re-check is local-only and stale completions cannot overwrite newer
  selection generations;
- each stable result class maps to an actionable state without leaking raw
  probe output or private paths into durable evidence;
- newly selected invalid directories retain prior authority, while persisted
  selections that become invalid remain selected without fallback;
- localization keys cover guidance and selection actions in English/Japanese.

### Consumer tests

- every AI/cache/widget/video consumer receives its command/path from a
  validated toolchain authority;
- image-only indexing succeeds when video is enabled but no pair exists;
- missing FFmpeg produces one bounded actionable warning and skips videos;
- frame-only video features work with FFmpeg ready and Wav2Vec2 absent;
- preference changes affect new tasks without mutating a captured running
  toolchain;
- no-download tests use a request-counting fixture and assert zero FFmpeg
  requests for setup, Settings, re-check, and selection on every policy seam.

## Native owner-smoke matrix

Run on each available supported platform/architecture and record `pass`,
`fail`, `not run`, or `environment-dependent` independently:

- inherited-PATH discovery and real video probe/extraction;
- platform default-prefix discovery where specified;
- explicit directory select, persistence, restart, re-check, change, and clear;
- missing pair with CLIP-only continuation/restart and image indexing;
- legacy-only location ignored and left untouched;
- mismatched/malformed/mixed pair rejected;
- controlled timeout with descendant cleanup;
- FFmpeg network observation showing no request during setup/Settings/re-check;
- release/package inspection showing no FFmpeg binary/archive.

Required platform rows are Linux x86_64, Linux aarch64 where available,
Windows x86_64, Apple Silicon macOS, and Intel macOS. Unavailable hardware is
never inferred from another platform.

## Release contract

Before release, evidence must show:

- no FFmpeg binary/archive in executable assets, source archives, or
  `cargo package --list` output;
- no production provider URL, asset ID/digest, downloader, installer, or
  archive activation code;
- setup/Settings/re-check cannot request FFmpeg bytes;
- user docs and UI describe external ownership on every platform;
- `NOTICE` does not claim arama redistributes FFmpeg and identifies it only as
  an optional external program;
- the shipped arama dependency/license audit remains consistent with
  Apache-2.0;
- focused/full format, check, clippy, tests, declared MSRV, docs build, and
  diff gates are observed;
- native smoke evidence is reviewed separately before RFC completion.

If a future release bundles, mirrors, procures, or directs delivery of a
particular FFmpeg binary, this RFC's contract is violated. That change requires
a new RFC and qualified license/compliance review.

## Implementation sequence

1. Add the serde-defaulted preference and pure locator/selection tests.
2. Make CLIP-only readiness platform-independent.
3. Add external guidance, re-check, directory select/change/clear, and
   generation-safe UI state.
4. Inject validated toolchain authority through every consumer and preserve
   image-only/frame-only degradation.
5. Exclude legacy local-bin on all platforms and add migration diagnostics.
6. Delete managed download/install APIs, constants, dependencies, and tests.
7. Update `NOTICE`, user/maintainer/security/release documentation and RFC 031
   cross-references.
8. Run automated gates and source/package absence checks.
9. Run or explicitly defer native owner smoke per platform.
10. Request architect completion review; only then perform lifecycle/release
    work separately.

## Acceptance criteria

- Architect review accepts external user-managed FFmpeg on every platform.
- Arama has no production ability to bundle, download, extract, install,
  update, or invoke a package manager for FFmpeg.
- CLIP alone completes setup on every platform.
- Auto and Selected-directory modes follow the finite, non-fallback rules.
- Auto applies raw/candidate/whole-attempt bounds and rejects empty/relative
  PATH entries without executing from the working directory.
- Preference validation, persistence, publication, startup, and cancellation
  follow the specified transaction/coordinator contract.
- Legacy arama-managed locations are ignored everywhere and never deleted.
- Pair/process validation and sole-authority consumption remain intact.
- Missing FFmpeg never blocks image-only work and is actionable for video.
- Managed installer APIs, dead metadata, and unused dependencies are removed.
- Documentation, `NOTICE`, release contracts, and localized UI match the
  external-only policy.
- Automated and owner evidence is recorded honestly without cross-platform
  inference.
- RFC 031's superseded Linux/Windows clauses cannot be mistaken for active
  policy.

## Alternatives rejected

### Rotate to current GPL asset IDs

Rejected. It repeats the mutable-release failure and does not establish exact
binary/source/license retention.

### Switch to upstream LGPL variants

Rejected for this project cycle. LGPL changes one copyleft dimension but still
requires feature-parity, provenance, transitive-license, corresponding-source,
availability, and rotation ownership.

### Owner-built or mirrored binaries

Rejected. Reproducible builds, hosting, signing, source delivery, security
response, and platform coverage exceed the value of one-click setup.

### PATH-only discovery

Rejected as insufficient for Windows and GUI-launched environments. Explicit
directory selection provides deterministic user control without broad scans.

### Automatically trust legacy managed files

Rejected. Their origin belongs to the retired acquisition authority and would
make the external-only claim false.

## Review questions resolved

- Use one directory picker rather than two executable pickers to preserve the
  pairing authority.
- Expose Selected mode on Linux and macOS as well as Windows for one consistent
  state contract.
- Emit one actionable warning per indexing generation and keep persistent
  capability status in Settings; no global banner is required.
- Inventory compatibility APIs before deletion. Stage removal only where a
  documented downstream compatibility contract exists; otherwise remove the
  managed surface in the implementation checkpoint.
