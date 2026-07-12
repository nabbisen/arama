# RFC 025 - Release smoke checklist

**Status.** Implemented (Unreleased)
**Tracks.** Release-readiness follow-up: define a repeatable manual GUI smoke
checklist for owner-managed release preparation without performing release
actions.
**Touches.** `docs/src/dev/testing.md`, `docs/src/dev/release.md`,
`ROADMAP.md`, `rfcs/README.md`.

## Summary

The unreleased RFC 015-024 batch is technically clean against the observed
automated gates, including the default CPU release gate and the all-features
clippy gate in the current CUDA-capable environment. The release-readiness
review still called out a useful gap: no manual GUI smoke was run.

This RFC proposes turning the existing short manual UI checklist in
`docs/src/dev/testing.md` into a clearer release-smoke checklist that covers
the workflows most affected by recent changes:

1. First-run/setup state and recoverable setup errors.
2. Gallery indexing, cancellation/restart, and focus/similarity dialogs.
3. Cache page footprint, prune, reload, stale data, and recoverable errors.
4. Settings persistence/error feedback and theme/high-contrast rendering.
5. Release-owner boundaries: versioning, archive, tag, and publish remain
   manual owner actions outside the smoke checklist.

## Why

The recent implementation batch improved important first-run and UI reliability
surfaces:

- setup/download error resilience;
- visible recoverable errors;
- cache lifecycle and capacity controls;
- AI/video pipeline partial-failure behavior;
- startup fatal-boundary resilience;
- high-contrast theme completion;
- cache and similarity dialog error resilience;
- image codec dependency minimization.

Automated tests now cover many logic paths, but the release owner still benefits
from a compact, repeatable GUI checklist before selecting a release point. The
current testing doc has useful manual steps, but they predate several of these
surfaces and do not distinguish:

- clean first-run checks from already-configured local checks;
- online/download checks from offline/no-network checks;
- release-smoke checks from broader exploratory testing;
- owner release actions from technical smoke evidence.

## Proposal

### Part A - Replace the short UI checklist with release-smoke sections

Update `docs/src/dev/testing.md` so the UI section is split into:

1. **Preconditions.** Build mode, data fixtures, optional network/model state,
   and when a clean `.arama-local` / `.arama-cache` profile is needed.
2. **First-run smoke.** Setup wizard, model/ffmpeg readiness, recoverable local
   path/disk/download errors, and clear user feedback.
3. **Gallery/indexing smoke.** Directory selection, image/video indexing,
   processing indicators, cancellation/restart after directory switch, and
   focus view.
4. **Similarity smoke.** Similar-pairs dialog, partial cache entries, and
   user-visible degradation if cache lookups fail.
5. **Cache page smoke.** Summary reload, footprint distinction, prune target,
   stale rows after reload error, and cache deletion path.
6. **Settings/theme smoke.** General/File System/AI settings, save feedback,
   persisted reload, light/dark/high-contrast presets, and readable iced
   widgets.
7. **Exit/restart smoke.** Saved root handling, invalid saved root fallback,
   and visible startup notices.

The checklist should stay short enough for a release owner to execute in one
focused pass. It should not become an exhaustive QA plan.

### Part B - Link release docs to the smoke checklist

Update `docs/src/dev/release.md` to mention the manual GUI smoke checklist as
an owner-managed optional/recommended check after automated gates and before
version/changelog/archive actions.

The wording must avoid making release impossible without GUI smoke in every
environment. A headless CI environment may still run automated gates only.

### Part C - Keep release actions separate

The smoke checklist must not perform or instruct agents to perform owner-only
release actions:

- version bump;
- changelog finalization;
- RFC `Implemented (vX.Y.Z)` stamping;
- archive creation;
- tag, publish, or push.

Those remain in `docs/src/dev/release.md` as release-owner steps.

### Part D - Optional future automation

Do not build a UI automation harness in this RFC. The implementation may add a
short "future automation candidates" note for checks that could become scripted
later, such as:

- app starts and exits without panic in an already-configured profile;
- settings file load/save round trip;
- cache page data reload through existing logic tests.

Any Playwright/GUI/system automation should be a later RFC because iced desktop
UI automation can be brittle and environment-dependent.

## Implementation outline

### `docs/src/dev/testing.md`

Replace the previous seven-step "Testing with the UI" section with the
release-smoke structure above.

### `docs/src/dev/release.md`

Add one short step or checklist item that points release owners to the manual
GUI smoke checklist when a release includes UI, setup, cache, or first-run
behavior changes.

### `ROADMAP.md`

Record RFC 025 as implemented once the documentation update lands.

### `rfcs/README.md`

List RFC 025 under Implemented once the documentation update lands.

## Non-goals

- No version bump, changelog finalization, archive, tag, publish, or push.
- No automated GUI harness.
- No new test framework dependency.
- No product behavior change.
- No requirement that every release environment must have network access,
  model downloads, or a clean first-run profile.

## Risks

- The checklist can become too long to run. Mitigation: keep it release-smoke
  focused and split exploratory checks out of scope.
- Some checks require network/model/ffmpeg state. Mitigation: mark those as
  optional or environment-dependent and provide offline alternatives where
  possible.
- Manual checks can drift. Mitigation: tie each section to named product
  surfaces and keep automated gates as the primary release-readiness baseline.
- Agents may misread smoke as permission to release. Mitigation: state release
  actions remain owner-managed and out of scope.

## Acceptance criteria

- `docs/src/dev/testing.md` has a release-smoke checklist covering first-run,
  gallery/indexing, similarity, cache page, settings/theme, and restart
  behavior.
- `docs/src/dev/release.md` points to the manual smoke checklist without making
  it an automated gate.
- The checklist distinguishes required local checks from environment-dependent
  clean first-run/download checks.
- No release action is performed.
- Documentation builds cleanly.

## Review evidence

Required:

```sh
mdbook build docs
git diff --check
```

Optional if the implementation only changes Markdown:

```sh
cargo fmt --check
```

## Implementation notes

The implementation replaced the old seven-step UI checklist with a release
smoke checklist in `docs/src/dev/testing.md`, and added a release-process
pointer in `docs/src/dev/release.md`. No release action, version bump, archive,
tag, publish, UI automation harness, or product behavior change is included.
