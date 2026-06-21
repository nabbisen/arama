# RFC 012 — Workspace housekeeping: manifest inheritance, orphan removal, changelog & doc reconciliation

**Status.** Implemented (v0.35.0)
**Tracks.** A maintenance release (v0.35.0) bundling six independent,
behaviour-neutral cleanups:
(a) single-source versioning and workspace metadata inheritance across
all member manifests;
(b) removal of the orphan `arama-storage` crate;
(c) CHANGELOG reconciliation — add the missing `[0.34.0]` history;
(d) repository-URL and `readme`-path corrections;
(e) `version.sh` simplification;
(f) release-documentation correction (archive layout + version bump).
**Touches.** Root `Cargo.toml`; every member `Cargo.toml`;
`crates/engine/storage/` (removed); `CHANGELOG.md`; `version.sh`;
`docs/src/dev/release.md`; `rfcs/README.md`. No `.rs` source changes,
no behaviour changes.

---

## Summary

v0.35.0 is a pure housekeeping release. It changes project metadata,
tooling, and documentation only — not a single line of application
logic. Every item below was surfaced during the v0.34.0 baseline review
and is independently shippable; they are grouped into one release
because each is small and the QA story is shared: **the build graph and
runtime behaviour are identical before and after.**

The unifying theme is *single source of truth*. Today the workspace
version is duplicated across eleven manifests, package metadata is
copied into some and missing from others, a dead crate lingers outside
the build graph, and two documents describe procedures that no longer
match the project's rules. After this release each fact lives in exactly
one place.

## Why

The v0.34.0 review found:

- **Version duplicated 11×.** Every member hard-codes
  `version = "0.34.0"`. A bump touches eleven files and is the reason
  `version.sh` exists.
- **Inconsistent package metadata.** `app`, `arama-ai`, and
  `arama-ui-layout` carry full `[package]` blocks (authors, license,
  repository, …) copied verbatim; the other seven members carry only
  `name`/`version`/`edition`. The copies have already drifted (see (d)).
- **Orphan crate.** `crates/engine/storage` (`arama-storage`, v0.12.7)
  is the pre-`localcache` storage engine superseded by RFC 002
  (v0.23.0). It is absent from `members`, from `workspace.dependencies`,
  from every dependency list, from `Cargo.lock`, and from the docs. No
  `.rs` file references it. It is dead source that never compiles.
- **Missing changelog history.** The workspace is at 0.34.0 but the
  CHANGELOG's top section is still `[Unreleased]`; the published 0.34.0
  has no section. That block also has two separate `### Added`
  subsections and a forward-looking `### Planned` under a shipped range.
- **Stray repository URL.** Root `[workspace.package].repository` points
  to `github.com/nabbisen/orbok`; every per-crate `repository`, the
  README badges, and `app` use `…/arama`.
- **Wrong `readme` paths.** The three crates that declare `readme` all
  point at non-existent files (`app/README.md`, `crates/README.md`,
  `crates/ui/layout/README.md`).
- **Stale release doc.** `docs/src/dev/release.md` instructs the
  packager to wrap the archive in an `arama-vX.Y.Z/` parent directory
  and to bump via `version.sh` — both now contradict project policy
  (archives must have **no** parent directory; version becomes
  single-source).

## Design

### (a) Single-source versioning + metadata inheritance

The version moves to one location — `[workspace.package].version` —
and every member inherits it. The root `[workspace.package]` table
(which already declares `version`, `edition`, `rust-version`, `license`,
`authors`, `categories`, `keywords`, `repository`) becomes the single
source for all *common* fields.

Each member `[package]` is rewritten to inherit the common fields and
keep only what is genuinely per-crate: `name`, `description`, `readme`.

```toml
# every member, e.g. crates/ai/Cargo.toml
[package]
name        = "arama-ai"
description = "Offline AI inference pipeline (CLIP + wav2vec2) for arama"
readme      = "../../README.md"           # per-crate: correct relative depth
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
authors.workspace      = true
license.workspace      = true
repository.workspace   = true
categories.workspace   = true
keywords.workspace     = true
```

Per-crate `description` values (each crate gains a meaningful one;
most currently have none):

| Crate | `description` |
|---|---|
| `arama` (app) | Image / video viewer and similarity finder powered by offline AI |
| `arama-ai` | Offline AI inference pipeline (CLIP + wav2vec2) for arama |
| `arama-cache` | Embedding and thumbnail cache facade over localcache for arama |
| `arama-env` | Environment constants, path helpers, and persisted settings for arama |
| `arama-i18n` | Lightweight internationalisation (locale tables and `t()`) for arama |
| `arama-theme` | Snora Design token-driven styling for arama |
| `arama-sidecar` | ffmpeg sidecar binary management for arama |
| `arama-ui-layout` | Application shell layout (header, aside, footer) for arama |
| `arama-ui-main` | Core views (gallery, setup wizard, cache page) for arama |
| `arama-ui-widgets` | Reusable iced widgets and dialogs for arama |

Per-crate `readme` paths (corrected to the actual depth to the root
`README.md`):

| Manifest | `readme` |
|---|---|
| `app`, `env` | `../README.md` |
| `crates/ai`, `crates/cache`, `crates/i18n`, `crates/theme` | `../../README.md` |
| `crates/engine/sidecar`, `crates/ui/layout`, `crates/ui/main`, `crates/ui/widgets` | `../../../README.md` |

A bump is now a one-line edit to `[workspace.package].version`. The
canonical `authors` string is consolidated to
`["nabbisen <nabbisen@scqr.net>"]` (the fuller form already present in
most crates); see Open questions.

### (b) Remove the orphan `arama-storage` crate

Delete `crates/engine/storage/` in its entirety. Because the crate is
already outside the build graph (not a member, no dependents, absent
from `Cargo.lock`), removal changes nothing that compiles. `cargo
check --workspace` output is byte-for-byte unaffected.

`crates/engine/` then contains only `sidecar/`. The directory grouping
is retained as-is (the member path `crates/engine/sidecar` and the docs
that reference it are unchanged); flattening is out of scope (see Open
questions).

The permanent record of *why* this crate existed and why it was removed
is this RFC, which lives in `rfcs/done/` forever once shipped — no
separate note is needed.

### (c) CHANGELOG reconciliation

Rename the current `[Unreleased]` block to `[0.34.0]` (it is exactly the
set of changes published as 0.34.0), and normalise it to Keep-a-Changelog
sections:

```
## [0.34.0]
### Added
- Snora recipe: `Theme::custom` from design tokens (note)
- Smoke tests for arama-i18n
### Changed
- RFC 011 high-contrast caveat sharpened
- lucide-icons 0.576.0 → 1.17.0
- candle-{core,nn,transformers} 0.9.2 → 0.10.2
```

The two duplicate `### Added` subsections merge; the dependency-update
block (currently under a bespoke heading) moves under `### Changed`; the
existing detailed bullet prose is preserved. A fresh `[Unreleased]`
section is opened for the v0.35.0 cycle and will carry this RFC's
entries (Changed / Removed / Fixed). The forward-looking "relative-time
rendering" item moves from `### Planned` into the new `[Unreleased]`.

### (d) Repository URL + readme path fixes

- Root `[workspace.package].repository`: `…/orbok` → `…/arama`.
- `readme` paths corrected per the table in (a); this is also the fix
  for the three currently-broken paths.

### (e) `version.sh` simplification

With the version single-sourced, the script's entire premise (rewrite
the `version =` line in every manifest, via `cargo metadata` + `jq`) is
obsolete and would in fact no-op against the inheriting members. The
recommended replacement is a small script that reads/edits the single
`[workspace.package].version` line in the root `Cargo.toml` — no
`cargo metadata`, no `jq` (which removes the standing "jq unavailable"
friction). `--list` reports the one version; `--update X.Y.Z` rewrites
the one line. See Open questions for the retire-entirely alternative.

### (f) Release-doc correction

`docs/src/dev/release.md` is brought in line with current policy:

- **Step 2 (bump):** replace the `version.sh --update` per-manifest
  description with the one-line `[workspace.package].version` edit (and
  the simplified `version.sh`).
- **Step 5 (package):** replace the `--transform` recipe that injects an
  `arama-vX.Y.Z/` parent with a no-parent-directory recipe so files
  unpack straight into the destination, e.g.

  ```sh
  cd <workspace-root>
  tar --exclude='./target' -czf ../arama-vX.Y.Z.tar.gz .
  ```

- **Step 6 (verify):** confirm the archive's top-level entries are the
  project files themselves (`./Cargo.toml`, `./app/`, …), **not** a
  wrapping directory.
- **Checklist:** "Version bumped in all `Cargo.toml` files" →
  "Version bumped in `[workspace.package]`"; update the archive-layout
  verification line.

`docs/src/dev/workspace.md` already documents `crates/engine/` as
holding only `sidecar/`, so removing the orphan makes the docs *more*
accurate with no edit required.

## Non-goals (deferred)

- **ELOC splits.** `app/src/core/update.rs` (~543 ELOC) and
  `crates/cache/tests/integration_tests.rs` (~615 ELOC) exceed the
  500-ELOC "strongly recommended split" line. These are live-code/test
  refactors with their own review and regression surface; folding them
  into a metadata release would muddy the "zero behaviour change" QA
  story. Recommended: a focused follow-up (RFC 013). See Open questions.
- **Flattening `crates/engine/sidecar`.** Out of scope; retained as-is.
- **Adding `ROADMAP.md`.** `rfcs/` already serves planning; not needed.

## Task breakdown / PR plan

The items are independent and can land as separate commits or one PR:

1. Root `Cargo.toml`: consolidate `[workspace.package]` (repository fix,
   canonical authors); set `version = "0.35.0"` at release time.
2. Rewrite the eleven member manifests to inherit + per-crate
   `description`/`readme`.
3. Delete `crates/engine/storage/`.
4. `CHANGELOG.md`: add `[0.34.0]`, open fresh `[Unreleased]`.
5. `version.sh`: simplify (or retire).
6. `docs/src/dev/release.md`: correct steps 2/5/6 + checklist.
7. `rfcs/`: move this RFC `proposed/ → done/`, set
   `Implemented (v0.35.0)`, update `rfcs/README.md`.

## Acceptance / QA checklist

- [ ] `cargo check --workspace` succeeds and resolves the same package
      set as before (10 `arama*` members; **no** `arama-storage`).
- [ ] `cargo metadata --no-deps` shows every member at `0.35.0`.
- [ ] `cargo test --workspace` passes (existing i18n / env / cache
      tests unchanged).
- [ ] `cargo fmt --check` clean.
- [ ] No `.rs` file changed (diff is manifests + docs + CHANGELOG +
      version.sh only).
- [ ] Each member manifest resolves its `readme` to the existing root
      `README.md`.
- [ ] Release archive has **no** parent directory (top entries are the
      project files).

## Resolved decisions

The four open questions were resolved by the maintainer before
implementation:

1. **`version.sh`:** simplified to a single-field, jq-free bump (reads
   and writes only `[workspace.package].version`).
2. **ELOC splits:** deferred to a focused RFC 013; v0.35.0 stays a
   zero-behaviour-change release.
3. **Canonical `authors`:** `["nabbisen"]` — the bare-name form, chosen
   over the email-bearing form for privacy / anti-spam reasons. The
   workspace already declared `["nabbisen"]`; the three crates that
   carried `["nabbisen <nabbisen@scqr.net>"]` now inherit the bare form.
4. **`crates/engine/` grouping:** retained; `engine/sidecar` is
   unchanged (no flattening).

## Implementation notes

- One detail differed from the RFC's *recommendation* (not its design):
  decision 3 chose `["nabbisen"]` over the email-bearing string. This
  drops the email previously present in `arama-ai`, `arama-ui-layout`,
  and `app`.
- The migration of v0.34.0's changelog content into a `[0.34.0]`
  section also normalised it to Keep-a-Changelog `Added` / `Changed`
  groups (the published block had two `### Added` subsections and a
  bespoke dependency-update heading).
- After implementation it was found that `workspace.dependencies` entries for internal crates also require an explicit `version` field (alongside `path`) to satisfy `cargo publish`, and by extension deps.rs and docs.rs. All nine internal entries were updated to `{ version = "0.35.0", path = "…" }`; `version.sh` was extended to update both locations atomically on a bump.
- No `.rs` files were touched; `cargo fmt` was a no-op. Validation:
  `cargo metadata --no-deps` resolves all ten members at `0.35.0` with
  `arama-storage` gone; representative leaf crates compile and their
  unit tests pass (see the handoff's QA section).
