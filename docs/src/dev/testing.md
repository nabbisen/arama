# Testing

## Test organisation

Test code is kept separate from implementation code:

| Pattern | Where |
|---|---|
| Unit tests for a module | `src/<module>/tests.rs` or inline `#[cfg(test)] mod tests` |
| Integration tests for a crate | `tests/integration_tests.rs` |
| When `tests.rs` grows large | Split into `tests/<category>.rs` submodules |

The same ELOC limits (300 / 500) apply to test files.

## Running tests

```sh
# All tests
cargo test --workspace

# One crate
cargo test -p arama-cache

# Specific test by name
cargo test -p arama-cache image_lookup_invalidated
```

## Release gates

The default CPU release gate is:

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

Do not report `cargo clippy --workspace --all-targets --all-features`
as clean unless it was run in an environment with the CUDA toolkit
available. The `cuda` feature enables Candle CUDA support through
`cudarc`, whose build script requires `nvcc`.

`cargo audit` uses `.cargo/audit.toml` for explicitly reviewed
temporary advisory exceptions, and may also print allowed warnings such
as unmaintained-crate warnings under the current policy. Keep each
ignored advisory and each allowed warning documented with its dependency
path, rationale, and revisit condition.

## `arama-cache` integration tests

T076: `cargo test -p arama-cache` runs in CI on all three platforms —
Linux (`msrv.yaml`), and macOS and Windows (`native-smoke.yaml`) — the
latter two specifically so `thumbnail.rs`'s process-group/job-object
reaping tests (including a Windows grandchild-reaping test) actually run
on the platform their code is for, not only compile there.

`crates/cache/tests/integration_tests.rs` is the **API compatibility
contract** for the cache facade. The test file was written against the
v1 (`file-feature-cache`) implementation and passes unchanged against
the v2 (`localcache`) implementation — this is the RFC 002
compatibility proof. Do not remove or weaken existing tests when
refactoring the cache internals.

Tests cover:
- Upsert and lookup (image and video)
- Invalidation when a file changes (different-length overwrite for
  deterministic size-based detection)
- COALESCE semantics: `None` vectors in an upsert preserve existing values
- Parallel lookup via cloned readers sharing the same read pool
- Partial failure in batch upsert (`upsert_all`)
- Thumbnail generation to a directory (`.jpg` suffix required for
  `image::open` format detection)
- Directory-scoped queries (`all_in_dir`, `all_in_dir_and_sub_dirs`)
- Persistence across writer/reader lifecycles

## `arama-ai` tests

`crates/ai/src/config/video_similarity_config.rs` has unit tests for
the timestamp computation logic (`compute_sample_timestamps`). These
are small and fast; they do not require model files.

## Headless layout probe (T074 Phase 0)

RFC 053 defines a headless layout matrix - every text size, language,
theme and window size, checked by test instead of by capture. Before
that matrix is built, `app/src/core/tests/layout_probe.rs` establishes
what `iced_test::Simulator` can actually see, each through a planted
defect shown failing before it is trusted to pass:

- **Clipping is visible** to `visible_bounds` only through a
  `scrollable` ancestor's viewport - a plain `Container::clip(true)` is
  not: `iced_selector`'s own tree-walker narrows its tracked viewport
  only inside `Finder::scrollable`, never for a plain container's own
  bounds.
- **A non-clipping overflow (the Japanese-footer shape, T055) is
  invisible to every bounds-based check**, because a widget's `bounds`
  is the box its parent allocated to it, not its intrinsic content size
  - true with or without disabling word-wrap. A pixel check past the
  container's edge, using `Simulator::snapshot` and decoding the PNG
  `Snapshot::matches_image` writes (its raw pixels have no other public
  accessor), does detect it.
- **A font's presence changes measured width** - confirmed for the
  bundled Lucide icon font (`Settings::fonts`); requesting a system font
  *by name* that was never loaded into cosmic-text's own font database
  does not (an unmatched name falls through to the same fallback
  regardless of which name was given). `iced_graphics::text::
  font_system()` is a process-wide singleton with no unload, so a test
  comparing "with" and "without" a loaded font must measure "without"
  first in the same test binary.
- **Which font face draws a kanji vs. a kana in the same label**,
  queried directly through `iced::advanced::graphics::text::cosmic_text`'s
  shaping API rather than inferred from measured width - confirming RFC
  053 §2.4's finding for real: a kanji can select a Simplified-Chinese
  face while a kana in the same string selects the Japanese one, since
  cosmic-text's fallback-by-script reads the **system** locale
  (`sys_locale`), never arama's own language setting.
- **The default headless backend (`wgpu`) costs roughly 100x a CPU
  render** (`ICED_TEST_BACKEND=tiny-skia`) for the identical result on
  every property above - relevant to keeping the full matrix inside its
  CI time budget.

**A Japanese font is required to run the probe's font items as
intended** - `fonts-noto-cjk` on Linux (installed in `msrv.yaml` before
`cargo test`, its installed version printed into the log rather than
pinned to an exact package, since pinning breaks the next time the
runner image updates); macOS and Windows runners use their own system
fonts, installing nothing.

**Not yet built**: the matrix itself (RFC 053 §2.1, every screen state
× text size × locale × theme × window size), the enforced minimum/launch
window size's effect on the matrix's own results, and
`KNOWN_LAYOUT_DEFECTS` (combinations that fail at the 760 × 520 minimum
today, each naming the task that owns fixing it). This section will be
extended when that delivery lands.

## Release smoke with the UI

There are no automated UI tests. For releases that include UI, setup,
cache, first-run, or recoverable-error changes, the owner should run a
short manual smoke pass after the automated gates and before versioning
or archive work.

Keep this pass release-smoke sized. It is a confidence check for critical
workflows, not an exhaustive exploratory QA script.

### Preconditions

- Use a release build when checking responsiveness or AI inference speed:
  `cargo run -p arama --release`.
- Use a small fixture directory with at least a few images and, when video
  behavior is in scope, one short video.
- The normal smoke pass may use the owner's existing settings/models/cache
  state at its real platform locations (RFC 041; see
  [installation.md's Data locations](../users/installation.md#data-locations)).
- Clean first-run checks need an isolated profile, not the owner's real
  state. Set `ARAMA_DATA_HOME` (`env/src/dir.rs:24`) to a scratch directory
  before launching — it scopes settings, models and cache under that one
  root, replacing the CWD/exe-relative isolation trick RFC 036/040 used
  before RFC 041 removed the anchors that trick relied on. Do not delete the
  owner's real cache/settings unless that is the explicit test.
- Model download checks require network access and available upstream
  artifacts. In offline/headless environments, record them as not run rather
  than blocking automated gate evidence. ffmpeg re-check is local-only.

### First-run and setup

- **`SMOKE-SETUP-READY`** — Existing configured state: app opens without the
  setup wizard and the footer/model status reflects ready local artifacts.
- **`SMOKE-SETUP-FIRST-RUN`** — Clean first-run state,
  environment-dependent: setup wizard presents model download actions and the
  external ffmpeg prerequisite, model downloads complete, and checksum or
  local path failures surface as visible setup errors instead of crashes.
- **`SMOKE-SETUP-AI-SETTINGS`** — Settings -> AI: existing models show ready
  state; absent models show the expected load/get actions.

### External ffmpeg trust boundary

Run these on each available native platform when ffmpeg/setup behavior
changes. Record unavailable platforms as `not run` or `environment-dependent`;
never infer native Windows or macOS results from Linux tests.
Missing/legacy-only cases need a disposable host or VM with no valid native
Homebrew fallback; never rename or replace an owner's normal Homebrew tools.
For controlled rejection/timeout candidates, use marker-producing wrappers in
a disposable directory to prove which commands ran. Record the selected-pair
diagnostic when available, a process listing or marker proving timeout-child
cleanup, and a network capture scoped to arama's re-check interval. If the
environment cannot safely produce that evidence, use `environment-dependent`
rather than an inferred pass.

- **`SMOKE-FFMPEG-PATH`** — With a compatible `ffmpeg` and `ffprobe`
  pair first on inherited `PATH`, re-check finds that exact pair, reports Ready,
  and probes/extracts one short fixture video.

  The ignored native smoke test exercises selected-directory validation and
  uses only the returned toolchain to generate, probe, and extract a real
  fixture. Run it with a trusted installed pair, for example on Linux:

  ```sh
  ARAMA_FFMPEG_SMOKE_DIR=/usr/bin \
    cargo test -p arama-sidecar --test external_ffmpeg_smoke \
    -- --ignored --exact selected_external_pair_generates_probes_and_extracts_real_video
  ```
- **`SMOKE-MACOS-FFMPEG-PREFIX`** — On macOS, launch from Finder or an environment whose
  `PATH` omits Homebrew. Re-check finds the native prefix pair
  (`/opt/homebrew/bin` on Apple Silicon, `/usr/local/bin` on Intel) and a short
  video probe succeeds.
- **`SMOKE-FFMPEG-MISSING`** — With no eligible pair, setup/Settings show
  external-install guidance and re-check only; image-only continuation remains
  available and no ffmpeg network request occurs.
- **`SMOKE-FFMPEG-LEGACY`** — With only legacy
  `.arama-local/bin/ffmpeg`/`ffprobe` files, discovery stays Missing, leaves the
  files untouched, and does not execute or download them.
- **`SMOKE-FFMPEG-MISMATCH`** — With missing, mixed-directory, malformed,
  or incompatible-version commands, discovery rejects the whole pair and the
  UI remains actionable rather than combining candidates.
- **`SMOKE-FFMPEG-TIMEOUT`** — With a controlled candidate that exceeds
  the probe deadline, the child and descendants are terminated/reaped and the
  UI does not remain blocked. Re-check reports Missing when no later valid
  candidate exists, or Ready for the independently validated later pair.

### Gallery and indexing

- **`SMOKE-GALLERY-INDEX`** — Select a fixture directory. Gallery rows populate
  and processing indicators stop after indexing finishes.
- **`SMOKE-GALLERY-SWITCH`** — Switch directories while indexing. The previous
  run stops and the new directory starts indexing without stale progress
  indicators.
- **`SMOKE-GALLERY-FOCUS`** — Open a gallery item. The focus view opens and
  similar media are ordered by score when enough cache data exists.
- **`SMOKE-GALLERY-EMPTY`** — Select a directory containing no media files.
  The gallery reports no files and the footer shows a zero count, with no
  error toast and no processing indicator left running.

### Similarity dialogs

- **`SMOKE-SIMILARITY-PAIRS`** — Open similar pairs from the header. The dialog
  renders image/video pairs when cache data exists.
- **`SMOKE-SIMILARITY-SPARSE`** — With sparse or partial cache data, the dialog
  remains usable and shows **visible, readable text** for an empty result —
  "Nothing has been indexed yet." for an unindexed target/directory, or "No
  similar items found." for a search that ran and found nothing (RFC 036). A
  dialog that renders no text at all, even with no crash, **fails this check**.
- **`SMOKE-SIMILARITY-ERROR`** — With the cache genuinely unreadable (e.g. the
  cache database file replaced or made unreadable before the dialog opens),
  both the similar-pairs and focus dialogs show a single inline "some files
  could not be read" message rather than silently reporting no matches.
- **`SMOKE-SIMILARITY-FFMPEG-ABSENT`** — With indexed videos and no
  ffmpeg/ffprobe pair available, both dialogs show a single statement that
  video comparison did not run, alongside any image-only results, without
  a call to action (RFC 036).

### Cache page

- **`SMOKE-CACHE-SUMMARY`** — Open Cache. Directory summaries load, and source
  media size is distinct from cache footprint.
- **`SMOKE-CACHE-PRUNE`** — Run a small manual prune target. The result reports
  deleted entries or the remaining unreclaimable footprint clearly.
- **`SMOKE-CACHE-RELOAD`** — Trigger a cache reload after normal navigation.
  Stale rows remain visible while recoverable reload errors are shown inline.
- **`SMOKE-CACHE-DELETE`** — Settings -> File System -> Cache delete: confirm
  the cache directory is removed or a visible error is shown.

### Settings and theme

- **`SMOKE-SETTINGS-MEDIA`** — Settings -> General: toggle media types and
  confirm re-indexing follows the selected media policy.
- **`SMOKE-SETTINGS-THEME`** — Change light, dark, and high-contrast theme
  presets. Standard iced widgets remain readable, with no obvious low-contrast
  controls.
- **`SMOKE-SETTINGS-PERSIST`** — Save settings, restart, and confirm the
  selected settings reload. If saving fails in the test environment, the app
  should show an error toast instead of crashing.

### Exit and restart

- **`SMOKE-RESTART-VALID-ROOT`** — Quit and restart with a valid saved root.
  The shell opens on that directory.
- **`SMOKE-RESTART-INVALID-ROOT`** — Invalid saved root,
  environment-dependent: point settings at a missing directory and restart.
  The app should open a usable shell with visible startup feedback instead of
  aborting.

Record an owner-run pass with the
[release smoke evidence template](./release-smoke-evidence-template.md). The
template is reusable confidence evidence; completing it is not an automated
gate or a release action.

### Future automation candidates

Do not add a desktop UI automation harness just for this checklist. If a later
RFC adds smoke automation, good first candidates are startup/exit in an
already-configured profile, settings load/save round trips, and cache page data
reloads through existing logic seams.

## Real-UI sessions

`scripts/gui-session.sh` is shared launch-and-capture infrastructure for any
task that needs a real, running `arama` window — not an automated version of
the checklist above (see "Future automation candidates"): it starts the app
and gets out of the way; a task's own script sources it and drives the UI.

It guarantees, every time:

- **Preflight** — refuses to start if an `arama` instance is already running,
  a stale legacy `.arama-local`/`.arama-cache` folder sits beside the binary
  under test, or `ARAMA_DATA_HOME`/any `XDG_*_HOME` already resolves outside
  the project.
- **Isolation** — a fresh scratch profile, with every directory `arama` or the
  toolkit reads (`ARAMA_DATA_HOME` and the four `XDG_*_HOME` variables)
  pointed into it. Nothing is created in the maintainer's real home directory
  (owner's rule) — the binary is never launched any other way from this
  script.
- **A fixed window size**, obtained by sizing a nested compositor before the
  app launches into it, configured to open its one window already
  fullscreen — never a resize command run against a window that already
  exists.
- **The right window** — the nested compositor is private to one session, so
  the only window in it (confirmed: none before launch, exactly one after)
  is arama's, cross-checked by pid on Wayland. Never "the first window on
  screen," and never an X11 client query against the session's own display.
- **Capture and teardown** — a `name` → screenshot helper, and a teardown
  that kills only the exact pid this script itself launched and confirms it
  is gone — never a kill by process name, which could hit an arama the
  maintainer opened for real, on their own desktop, during the session.

See the script's own header for full usage and environment variables, and the
project's own GUI test rules (not public) for why each of these exists.

Reusable fixtures — synthetic gallery images, the two AI models pre-installed
so a session skips the network download, and a previous release's build for
upgrade checks — live in the project's git-ignored `.git-exclude/fixtures/`,
built or fetched once and recorded with their checksums (see that directory's
own `README.md`).

## Avoiding regressions

Design specs (RFCs) are the source of truth for test design. When an
RFC implementation note records a behaviour change (e.g. "Invalidated
no longer deletes the stale row"), add a test comment explaining the
expected behaviour and why it differs from the original description.
