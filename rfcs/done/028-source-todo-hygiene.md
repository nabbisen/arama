# RFC 028 - Source TODO hygiene and orphan cleanup

**Status.** Implemented (0.37.0)
**Tracks.** Maintenance follow-up: remove stale source TODOs and orphaned
legacy source that no longer describe actionable implementation work.
**Touches.** `env/src/file_system.rs`, `crates/ai/src/lib.rs`,
`crates/ai/src/store/file.rs`, `crates/ai/src/pipeline/encode/image.rs`,
`crates/ui/widgets/src/dialog/settings_dialog/view.rs`,
`crates/ui/main/src/core/views/gallery/subscription.rs`, `ROADMAP.md`,
`rfcs/README.md`.

## Summary

The current source tree still contains a small set of `todo` comments and one
orphaned source file that are no longer useful as planning signals.

Observed source TODOs on 2026-07-13:

```text
env/src/file_system.rs:2
crates/ai/src/lib.rs:19
crates/ai/src/store/file.rs:1
crates/ai/src/pipeline/encode/image.rs:26
crates/ui/widgets/src/dialog/settings_dialog/view.rs:28
crates/ui/main/src/core/views/gallery/subscription.rs:105
```

The gallery subscription file is not declared by `crates/ui/main/src/core/views/gallery.rs`.
It contains a dormant, commented-out legacy worker path and imports used only
by that commented code. The live app subscription is in `app/src/core/subscription.rs`.

This RFC records a narrow hygiene pass:

1. Remove context-free or stale TODO comments.
2. Replace meaningful-but-deferred TODOs with current rationale and RFC
   references where useful.
3. Delete orphaned commented legacy source that is not compiled or reachable.
4. Avoid behavior, API, dependency, release, or policy changes.

## Why

The project now uses RFCs and review packages as the source of truth for
non-trivial work. Bare TODO comments are easy to misread as active scope even
when the design has already been settled elsewhere.

Examples:

- cache disk-pressure policy was addressed by RFC 016 as explicit prune controls
  with no automatic low-space guard in the first pass;
- CLIP SafeTensors sourcing was addressed by RFC 021 and its decision note;
- video similarity defaults already flow through `VideoSimilarityConfig`;
- the `arama-ai` file-store module visibility question should not be changed
  casually because workspace crates use current module paths;
- the gallery subscription source is orphaned and should not continue to imply
  a dormant image-similarity worker path.

## Proposal

### Part A - Remove stale TODO comments

Remove no-action TODOs where surrounding code is self-explanatory:

- `crates/ui/widgets/src/dialog/settings_dialog/view.rs`

Replace stale TODOs with current rationale where deleting them would hide
useful context:

- `env/src/file_system.rs`: low-disk cache-update behavior is not part of the
  current explicit-prune design; any automatic disk-pressure guard needs a
  future RFC.
- `crates/ai/src/lib.rs`: private video similarity defaults feed
  `VideoSimilarityConfig`; changing runtime configurability belongs in a
  future AI-quality/settings RFC.
- `crates/ai/src/store/file.rs`: leave current public module paths intact for
  compatibility; any public API cleanup requires a separate design.
- `crates/ai/src/pipeline/encode/image.rs`: model loading currently depends on
  the locally prepared SafeTensors path selected by the model manager; trusted
  source/mirror replacement remains governed by RFC 021.

### Part B - Remove orphaned gallery subscription source

Delete `crates/ui/main/src/core/views/gallery/subscription.rs` if review
confirms it is not declared by the gallery module and has no live callers.

This should not remove the live app subscription at `app/src/core/subscription.rs`.

### Part C - Keep behavior stable

The implementation must not:

- change module visibility;
- change AI/video scoring defaults;
- change model loading behavior;
- add automatic disk-pressure pruning or warnings;
- change gallery/indexing behavior;
- change dependencies or audit policy.

## Non-goals

- No public API redesign.
- No AI quality/settings UI.
- No CLIP model source replacement.
- No cache disk-pressure automation.
- No gallery subscription rewrite.
- No dependency update.
- No release action, version bump, tag, publish, or push.

## Risks

- Removing a TODO could erase useful intent. Mitigation: keep intent as a
  normal comment when it points to a real existing design boundary.
- Deleting the orphaned gallery subscription file could hide a future idea.
  Mitigation: rely on RFCs/handoffs for future work, not commented-out source.
- A module-visibility cleanup could accidentally become a public API change.
  Mitigation: explicitly avoid visibility changes in this RFC.

## Acceptance criteria

- Source TODO comments listed in this RFC are removed or replaced with
  rationale.
- Orphaned gallery subscription source is deleted if still unreferenced.
- `rg -n "todo|TODO|FIXME|fixme" env app crates` reports only intentional
  historical comments or none in live source.
- Runtime behavior is unchanged.
- No dependency, source policy, release, or version change is included.

## Review evidence

Required for proposal review:

```sh
rg -n "todo|TODO|FIXME|fixme" env app crates
rg -n "mod subscription|pub mod subscription|subscription\\(" crates/ui/main/src app/src
mdbook build docs
git diff --check
```

Required for implementation review:

```sh
rg -n "todo|TODO|FIXME|fixme" env app crates
cargo fmt --check
cargo check --workspace --all-targets
mdbook build docs
git diff --check
```

## Implementation notes

The implementation removed the context-free settings-dialog TODO, deleted the
orphaned gallery subscription source, and replaced the remaining stale TODO
markers with current design-boundary comments. It did not change module
visibility, AI/video scoring defaults, model loading behavior, cache
disk-pressure behavior, gallery/indexing behavior, dependencies, audit policy,
release state, version, tag, publish, or push state.
