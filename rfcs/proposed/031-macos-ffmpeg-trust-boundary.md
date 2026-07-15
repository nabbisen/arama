# RFC 031: macOS ffmpeg trust boundary

**Status.** Proposed
**Tracks.** Remove automatic execution of unverified macOS ffmpeg downloads.
**Touches.** `crates/engine/sidecar`, `crates/ai`, `crates/cache`,
`crates/ui/main`, `crates/ui/widgets`, setup/settings UI and i18n, user and
developer security documentation, `NOTICE`, and release-smoke evidence.

## Summary

Arama currently downloads and executes ffmpeg binaries automatically on every
supported platform. Linux and Windows artifacts use pinned GitHub release asset
identities and committed SHA-256 digests. macOS instead uses moving third-party
URLs from evermeet.cx and osxexperts.net with no expected digest.

This RFC removes arama-managed ffmpeg downloads on macOS. macOS users provide a
paired `ffmpeg` and `ffprobe` installation, with Homebrew as the documented
route. Arama discovers the pair on `PATH` and in the native default Homebrew
prefix, but never invokes Homebrew, changes quarantine/signing policy, or
downloads an executable for the user.

Linux and Windows retain their existing verified automatic-download path. The
artifact type is hardened so every future arama-managed download must carry a
required SHA-256 digest.

## Why now

The architecture preparation review identified macOS ffmpeg as the remaining
explicit executable supply-chain exception. Downloaded bytes cross a network
trust boundary and are later executed against user-selected private media, so
an undetected substitution can compromise both the machine and the local-first
privacy promise.

The current exception also has operational drift:

- Intel uses the moving `https://evermeet.cx/ffmpeg/getrelease/zip` endpoint;
- Apple Silicon uses `https://www.osxexperts.net/ffmpeg80arm.zip`, while the
  provider page now advertises a different release;
- neither branch supplies `expected_sha256`;
- the providers distribute `ffmpeg` and `ffprobe` separately, while arama's
  video-duration path requires both;
- `NOTICE` attributes downloaded ffmpeg only to `yt-dlp/FFmpeg-Builds`, which
  is not the current macOS source.

## Current trust boundary

`VideoEngine::download_artifact()` selects a platform URL. Both the first-run
setup stream and Settings → AI can download that artifact. Verification is
optional: `expected_sha256: None` makes the verifier return success without
checking identity. The archive is extracted into `.arama-local/bin`, marked
executable, and later receives paths to user media.

The current boundary is therefore:

```text
moving third-party URL
        │ HTTPS only; no expected artifact identity
        ▼
arama download + extraction
        ▼
.arama-local/bin/ffmpeg
        │ executed against private user media
        ▼
local process and filesystem access
```

TLS authenticates the current server connection but does not make a moving
artifact immutable or independently identify the reviewed bytes.

## Research evidence

Evidence was observed on 2026-07-15:

- [evermeet.cx](https://evermeet.cx/ffmpeg/) publishes Intel binaries,
  versioned releases, and GPG signatures. Its page states that it does not plan
  to provide native Apple Silicon binaries and documents quarantine handling.
- [OSXExperts](https://www.osxexperts.net/) currently publishes separate Apple
  Silicon `ffmpeg` and `ffprobe` files with SHA-256 values and user-side
  quarantine/ad-hoc-signing instructions. Its current advertised files differ
  from arama's hardcoded `ffmpeg80arm.zip` URL.
- [eugeneware/ffmpeg-static](https://github.com/eugeneware/ffmpeg-static)
  republishes macOS binaries from evermeet.cx and osxexperts.net. Adding that
  intermediary would change hosting but not the original build provenance.
- The official [Homebrew ffmpeg formula](https://formulae.brew.sh/formula/ffmpeg.html)
  supplies bottles for Apple Silicon and Intel macOS and installs both tools
  through `brew install ffmpeg`.
- Homebrew documents native default prefixes as `/opt/homebrew` on Apple
  Silicon and `/usr/local` on Intel macOS in its
  [installation documentation](https://docs.brew.sh/Installation).
- Apple documents that Gatekeeper uses developer identity, notarization, and
  provenance controls for downloaded software in its
  [platform security guide](https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-sec5599b66df/web).

None of the inspected third-party static-binary routes provides one stable,
owner-controlled, notarized archive containing both architectures and both
tools. Pinning current third-party bytes would improve integrity but would
still require arama to own provider review, licensing, macOS execution-policy
handling, paired-artifact updates, and recurring digest maintenance.

## Threat model

### Protected assets

- private image and video filenames and contents;
- user filesystem access available to the arama process;
- application settings, models, cache, and executable-adjacent data;
- integrity of video metadata, thumbnails, audio extraction, and embeddings.

### Adversaries and failures

- compromise of a third-party build or distribution server;
- silent replacement behind a moving URL;
- provider/operator mistakes that change archive content or layout;
- network or proxy substitution not caught by an expected digest;
- mixed `ffmpeg` and `ffprobe` installations with incompatible behavior;
- a corrupt or hostile candidate executable that never exits;
- stale legacy binaries left by older arama versions.

### Security invariants

1. Arama never automatically executes newly downloaded bytes unless their
   identity is checked against a required, reviewed digest.
2. macOS executable acquisition remains an explicit user/package-manager
   action outside the application process.
3. Arama selects `ffmpeg` and `ffprobe` as a pair from one logical candidate
   directory, checks a matching release/build identity, and gives both probes
   a finite deadline.
4. A missing macOS toolchain is visible and actionable; it never silently
   falls back to the old unverified download.
5. Migration never deletes a user's existing files automatically.
6. Every video process and cache-normalization consumer obtains its executable
   path from the same validated toolchain authority.

## Decision

### Part A — Disable arama-managed macOS downloads

Remove both macOS branches from the automatic artifact policy. On macOS,
`download_artifact()` returns a stable actionable error explaining that
automatic download is unavailable and that the user must install ffmpeg.

Do not retain evermeet.cx or osxexperts.net URLs in executable source code. Do
not invoke `brew`, `curl`, `xattr`, `codesign`, a shell, or privileged commands
from arama.

Linux x86_64, Linux aarch64, and Windows x86_64 keep their current pinned
`yt-dlp/FFmpeg-Builds` assets.

### Part B — Make verified downloads structurally mandatory

Change the arama-managed `DownloadArtifact` contract so its expected SHA-256 is
required rather than optional. The verifier must always compare downloaded
bytes before writing or extracting them.

Digest presence alone is insufficient while `unpack_archive()` remains a
public path-based entry point. Replace that split API with one of these
equivalent structural designs:

- extraction is private to one download, verify, and install operation; or
- extraction consumes a verified-archive type whose fields are private and
  whose only production constructor performs the digest comparison.

No public production API may install an archive selected only by path or
accept a caller's assertion that verification occurred. A mismatch must be
rejected before any archive entry is extracted or final executable is
replaced. Setup and Settings must use this same verified install authority;
progress reporting must not recreate a separate unverified extraction path.

The generic setup download stream may continue using an optional digest for
other call sites, but constructing an ffmpeg artifact without a digest must be
impossible through the sidecar API.

Add tests that every supported automatic-download platform exposes a 64-digit
lowercase hexadecimal SHA-256 value and that mismatch prevents installation.

### Part C — Discover a user-managed macOS tool pair

Refactor discovery around one `FfmpegToolchain`-style authority containing the
validated concrete `ffmpeg` and `ffprobe` paths rather than one `ffmpeg`
readiness flag. On macOS, consider candidates in this order:

1. `ffmpeg` and `ffprobe` resolved together from the inherited `PATH`;
2. `/opt/homebrew/bin/ffmpeg` and `/opt/homebrew/bin/ffprobe` on Apple Silicon;
3. `/usr/local/bin/ffmpeg` and `/usr/local/bin/ffprobe` on Intel macOS.

Both paths must come from the same logical candidate directory. Directory
equality is evaluated on the selected candidate paths before symlink
canonicalization, so Homebrew's two prefix-level symlinks remain one pair even
when their Cellar targets differ. Do not combine one tool from `PATH` with the
other from Homebrew.

Probe both commands with `-version` away from the UI/render thread. Each child
has a documented short deadline; on expiry arama kills and reaps it, records
the candidate as unavailable, and continues without blocking startup or a
Settings re-check. Parse the first version line from each successful command
and require the release/build token following `ffmpeg version` to exactly
match the token following `ffprobe version`. Missing, malformed, non-UTF-8,
non-successful, or mismatched output rejects the whole candidate. Exact token
matching is deliberately conservative: a user can repair or replace the pair
rather than arama guessing cross-release compatibility.

Successful discovery, not the presence of Homebrew, is the capability test. A
valid non-Homebrew pair selected together from `PATH` must pass the same
validation and remain supported.

The explicit Homebrew candidates are needed because a `.app` launched from
Finder may not inherit the interactive shell's `PATH`.

Linux and Windows keep the current preference for the verified arama-managed
pair, followed by a paired system `PATH` installation. All platforms return
the same authority type after validation.

Every process spawn and every consumer needing the ffmpeg path must use that
authority. This includes video probing and extraction in `crates/ai`, video
cache normalization in `crates/cache`, and similar-pair/media-focus paths in
`crates/ui/widgets`. Raw managed-local path helpers become private
installation details and cannot be used as a discovery bypass.

### Part D — Fail closed for legacy macOS sidecars

After implementation, macOS discovery does not automatically execute
`.arama-local/bin/ffmpeg` or `.arama-local/bin/ffprobe`. Those files may have
been downloaded by an older arama version without a verified identity.

Do not delete or rewrite them. Documentation tells users they may remove the
legacy files after installing a user-managed pair. This deliberately trades a
one-time migration inconvenience for a closed executable trust boundary.

### Part E — Make setup and settings actionable

The setup wizard must distinguish an external prerequisite from a downloadable
item. On macOS when the pair is absent:

- model downloads may still proceed;
- ffmpeg shows an external-action-required state, not a download progress bar;
- the user sees the localized `brew install ffmpeg` recommendation and may
  continue without video support;
- no network request is made for ffmpeg.

Settings → AI replaces the macOS **Get** action with localized installation
guidance and a re-check action. Re-check performs discovery only. It does not
run Homebrew or open a privileged installer.

Video features remain unavailable until both tools are discovered. Image-only
use continues normally.

This continuation is durable without storing a session-only skip flag. On
macOS, setup readiness gates the application only on the CLIP image model.
Wav2Vec2 and the ffmpeg pair are optional video capabilities shown by setup
when setup is otherwise needed and by Settings afterward. Once CLIP is ready,
reconstructing or restarting the application must not reopen setup solely
because Wav2Vec2 or ffmpeg is absent. The restart path performs discovery only
and makes no ffmpeg download request.

Linux and Windows retain their existing setup-readiness policy in this RFC.

### Part F — Reconcile documentation and notices

Update:

- first-run, settings, installation, and FAQ pages with the macOS prerequisite
  and Linux/Windows automatic-download distinction;
- the workspace/security documentation with the executable trust boundary and
  legacy migration behavior;
- `NOTICE` so `yt-dlp/FFmpeg-Builds` is explicitly scoped to Linux/Windows and
  no obsolete macOS provider attribution remains;
- release smoke evidence with a macOS user-managed pair case and a missing-pair
  case, both environment-dependent when no macOS runner is available.

The implementation should add a concise `docs/src/dev/security.md` threat-model
page and link it from mdBook navigation, because executable artifact trust is a
durable project security boundary rather than only an RFC implementation note.

## Alternatives considered

### Pin the current provider files and SHA-256 values

Rejected for this RFC. It would close silent replacement for the selected
bytes, but the two architectures use different providers, each tool is
distributed separately, current URLs and versions drift, and execution may
require quarantine or signing actions. Arama would inherit recurring binary,
license, and macOS policy maintenance.

### Verify evermeet GPG signatures at runtime

Rejected. It covers Intel only, adds a GPG verification/key-rotation subsystem,
and does not solve Apple Silicon provenance or paired-artifact maintenance.

### Republish third-party binaries in arama releases

Deferred. An owner-controlled mirror can provide immutable URLs and hashes, but
it also makes the project responsible for reproducible build provenance,
license/source-offer obligations, notarization/signing, update cadence, and
incident response. A future RFC may select this only with a documented build
and release pipeline.

### Use eugeneware/ffmpeg-static GitHub assets

Rejected as a trust-boundary solution. The project republishes binaries sourced
from the same macOS providers, so GitHub immutability would add integrity after
republication without establishing a stronger original build provenance.

### Keep the current exception but improve `NOTICE`

Rejected. Attribution is necessary but does not authenticate executable bytes.

## Compatibility and migration

- Linux and Windows first-run behavior remains unchanged.
- macOS users with `ffmpeg` and `ffprobe` already available on `PATH` should
  continue without action.
- Native Homebrew installations are found even when the GUI process lacks the
  shell's full `PATH`.
- macOS users relying only on the old `.arama-local/bin` files must install a
  user-managed pair before video analysis resumes.
- No cache schema, model artifact, media format, or similarity algorithm
  changes.
- Existing image caches and video embeddings remain usable; only new video
  probing/extraction requires the discovered tools.

## Risks and mitigations

- **First-run ease decreases on macOS.** Mitigation: show one copyable command,
  allow image-only continuation, and provide a re-check action.
- **Homebrew is not installed.** Mitigation: accept any valid paired tools on
  `PATH`; document Homebrew as the supported recommendation, not an automatic
  prerequisite installer.
- **GUI launch cannot see shell paths.** Mitigation: check native Homebrew
  prefixes explicitly.
- **Existing users lose automatic legacy-sidecar use.** Mitigation: do not
  delete files; explain the security reason and migration command.
- **A malicious user-controlled `PATH` can select a malicious tool.** This is
  part of the user-managed execution environment, not a network download by
  arama. Pair validation rejects cross-build and cross-candidate combinations
  but does not authenticate a user's local installation.
- **A candidate hangs during validation.** Mitigation: probe off the render
  thread, enforce a per-process deadline, then kill and reap the child before
  reporting the candidate unavailable.
- **Homebrew updates can change behavior.** Mitigation: validate both commands
  at runtime and include real video probe/extraction in owner smoke evidence.

## Non-goals

- No implementation in the proposal change.
- No automatic Homebrew installation or update.
- No shell, privilege, quarantine, Gatekeeper, or code-signing bypass.
- No bundled or mirrored macOS ffmpeg binaries.
- No Linux/Windows source or digest rotation.
- No change to supported media formats or AI models.
- No cache migration.
- No release, version bump, archive, executable asset, tag, publish, commit, or
  push action.

## Acceptance criteria

- RFC review accepts user-managed macOS ffmpeg/ffprobe as the trust policy.
- macOS production code contains no evermeet.cx or osxexperts.net download URL.
- No macOS ffmpeg network request is possible through setup or settings.
- Every arama-managed ffmpeg artifact requires a valid expected SHA-256.
- Discovery selects a same-directory pair and verifies both commands.
- Discovery requires matching release/build tokens, bounds both probes, and
  never runs them on the UI/render thread.
- The validated toolchain is the sole path authority for AI, cache, setup,
  settings, and widget consumers; no public raw local-path accessor bypasses
  it.
- macOS discovery ignores legacy `.arama-local/bin` sidecars without deleting
  them.
- Extraction is private to verified installation or consumes an
  unforgeable verified-archive value; no public production path can install a
  pre-positioned unverified archive.
- Setup and settings provide localized external-install and re-check behavior.
- On macOS, CLIP readiness permits image-only use across application
  reconstruction/restart even when Wav2Vec2 and the tool pair are absent.
- `NOTICE`, user docs, workspace docs, security docs, and smoke evidence match
  the implemented platform split.
- Linux and Windows verified downloads retain current behavior.
- Mac testing is recorded as passed, failed, not run, or
  environment-dependent rather than inferred from Linux checks.

## Required test design

The implementation must include automated tests for boundaries that do not
require a real macOS host:

- reconstruct setup state with CLIP ready but Wav2Vec2 and ffmpeg absent;
  assert that macOS enters the application, does not reopen setup, and issues
  no ffmpeg download request;
- inject PATH, Homebrew-prefix, missing, legacy-local-only, mismatched-version,
  malformed-version, and valid non-Homebrew candidate pairs;
- use a probe fixture that outlives the deadline; assert that it is killed and
  reaped, discovery returns unavailable, and the UI-facing task completes;
- verify logical candidate-directory comparison accepts Homebrew-style
  symlinks without mixing candidates;
- assert every production ffmpeg/ffprobe process factory and cache path is
  obtained from `FfmpegToolchain`, with raw managed paths inaccessible outside
  the installation module;
- reject malformed and mismatched digests before extraction and assert the
  previous installed pair remains unchanged;
- prove the public API cannot install a pre-positioned archive by path; test
  the private extraction operation only through a successfully constructed
  verified result;
- retain Linux and Windows discovery and verified-install regression cases.

macOS owner smoke must separately record inherited-PATH, native Homebrew
prefix, missing-pair, legacy-only, incompatible-pair, and timeout outcomes on
each available architecture. A missing host is reported as not run or
environment-dependent, never inferred from unit tests.

## Proposed implementation sequence

1. Introduce the sole-authority paired toolchain API with bounded compatible
   probes, then route every AI, cache, setup, settings, and widget consumer
   through it.
2. Couple digest verification structurally to extraction/installation.
3. Remove macOS automatic artifact branches and make legacy local sidecars
   ineligible on macOS.
4. Add restart-stable image-only readiness plus setup/settings
   external-prerequisite states and localized text.
5. Update user, maintainer, security, notice, and smoke documentation.
6. Run local Rust/documentation gates.
7. Run or explicitly defer macOS Apple Silicon and Intel smoke covering
   inherited-PATH and Homebrew-prefix discovery, missing and legacy-only pairs,
   incompatible pairs, probe timeout, restart-stable image-only readiness,
   verified-install enforcement, `ffprobe` duration, thumbnail extraction,
   and video similarity.

## Review evidence

Required for proposal review:

```sh
mdbook build docs
git diff --check
```

Proposal review should also verify:

- current source contains the two unverified macOS branches described here;
- Homebrew's current official formula and prefix documentation support the
  proposed discovery route;
- setup and settings call paths are included in implementation scope;
- all direct AI, cache, and widget path consumers are included in scope;
- the legacy migration is fail-closed but non-destructive;
- setup completion without ffmpeg remains stable across reconstruction;
- extraction cannot be reached without verified bytes;
- pair probes are compatible, bounded, and off the render thread;
- no release or implementation action is included.
