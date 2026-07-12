# RFC 026 - Explorer tree maintenance and scan ownership

**Status.** Implemented (Unreleased)
**Tracks.** Small explorer-tree maintenance pass after the release-smoke
checklist work: keep the `iced-swdir-tree` patch dependency current and clarify
the ownership boundary between the aside directory tree and arama's media cache
scan.
**Touches.** `Cargo.lock`, `app/src/core/update/cache.rs`, `ROADMAP.md`,
`rfcs/README.md`.

## Summary

The workspace depends on `iced-swdir-tree` through the UI layout crate. The
workspace manifest allows the compatible `0.9` line, but `Cargo.lock` currently
pins `iced-swdir-tree` 0.9.1. A current dependency check found 0.9.3 available
on the same compatible line.

The app also contains this cache-update note:

```rust
// todo dir_node should be got from dir_tree
```

That note is misleading. The aside tree is a folder-only UI navigation widget.
The cache/gallery path needs a separate media-aware scan so it can apply
arama's accepted image/video extensions and produce the `DirNode` used by cache
and gallery processing. Reusing the aside tree directly would either lose media
entries or force the explorer widget to own cache/indexing concerns.

This RFC records a narrow maintenance pass:

1. Update the lockfile to the latest compatible `iced-swdir-tree` 0.9 patch
   release after review.
2. Keep the workspace manifest on the existing `0.9` requirement unless review
   requests an exact pin.
3. Replace the misleading cache TODO with an ownership note or remove it if the
   surrounding code is already clear.
4. Avoid changing explorer UX, cache scan behavior, or directory selection
   flow.

## Why

`iced-swdir-tree` is part of arama's first-screen navigation surface. Keeping a
compatible patch release current is low-risk maintenance when the package
surface is verified and the dependency remains inside the accepted major/minor
line.

The TODO is more important than its size suggests. It points future developers
toward coupling two different responsibilities:

- the aside tree owns folder navigation UI state;
- arama's cache/gallery scan owns media discovery, extension filtering, and
  processing inputs.

The code should make that boundary explicit before later explorer or cache work
tries to "fix" the TODO in the wrong direction.

## Observed dependency facts

On 2026-07-13, these checks were observed:

- `cargo info iced-swdir-tree@0.9.3` reported `iced-swdir-tree` 0.9.3 as an
  available Apache-2.0 crate on the same `0.9` line.
- `cargo update -p iced-swdir-tree --dry-run` reported:
  `Updating iced-swdir-tree v0.9.1 -> v0.9.3`.
- A local diff of the unpacked registry crates showed no differences under
  `src/` or `tests/` between 0.9.1 and 0.9.3.
- The visible package delta is documentation/package metadata. The packaged
  changelog records mdBook documentation restructuring under the 0.9.1 section
  and does not expose separate 0.9.2/0.9.3 behavioral notes.

These observations should be rechecked during implementation because registry
state can change.

## Proposal

### Part A - Patch-update `iced-swdir-tree`

Run the targeted lockfile update:

```sh
cargo update -p iced-swdir-tree
```

Expected outcome:

- `Cargo.lock` moves from `iced-swdir-tree` 0.9.1 to the latest compatible
  0.9 patch release.
- No unrelated dependency churn is accepted unless Cargo requires it and the
  diff is reviewed.
- `Cargo.toml` remains on `version = "0.9"` so the root workspace does not need
  patch-level internal version churn.

### Part B - Clarify scan ownership

Update `app/src/core/update/cache.rs` around `on_dir_changed` so the media scan
ownership is clear:

- either remove the stale TODO;
- or replace it with a short note that the aside tree is folder-only UI state
  and cache/gallery require a media-extension-aware `DirNode` scan.

No behavior change is intended.

### Part C - Validate the narrow surface

Because this is dependency/UI-adjacent maintenance, implementation should run:

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test -p arama-ui-layout
cargo test -p arama --lib
git diff --check
```

If the lockfile update pulls unrelated transitive changes, implementation must
call that out in the review package instead of treating it as invisible churn.

## Non-goals

- No explorer UI redesign.
- No cache/gallery scan rewrite.
- No attempt to source media `DirNode` data from `iced-swdir-tree`'s
  `DirectoryTree`.
- No dependency major/minor upgrade.
- No release action, tag, publish, or version bump.

## Risks

- The patch release may include undocumented behavioral changes. Mitigation:
  compare unpacked registry `src/` and run focused layout/app checks before
  review.
- Cargo may update more than the intended package. Mitigation: review
  `Cargo.lock` closely and keep the update targeted.
- Removing the TODO may hide a legitimate future optimization. Mitigation:
  replace it with an ownership note if review prefers preserving the rationale.

## Acceptance criteria

- RFC 026 is accepted before implementation begins.
- Implementation updates only the accepted patch-level dependency surface and
  scan-ownership comment.
- `Cargo.lock` changes are explained in the implementation review package.
- The app still builds and the focused UI/app tests pass in the implementation
  environment.
- No product behavior change is introduced.

## Review evidence

Required for proposal review:

```sh
mdbook build docs
git diff --check
```

Required for implementation review:

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test -p arama-ui-layout
cargo test -p arama --lib
git diff --check
```

## Implementation notes

The implementation updated only the targeted lockfile entry for
`iced-swdir-tree` 0.9.1 -> 0.9.3 and replaced the stale cache TODO with a short
ownership note. The workspace manifest remains on `version = "0.9"`. No
explorer UX, cache/gallery scan behavior, release action, tag, publish, or
version bump is included.
