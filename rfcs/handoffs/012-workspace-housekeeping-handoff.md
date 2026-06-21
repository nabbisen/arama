# RFC 012 Handoff — Workspace housekeeping

Companion to [RFC 012](../done/012-workspace-housekeeping.md). Shipped in
**v0.35.0**. This is a metadata / tooling / documentation release with
**no `.rs` changes** and no behaviour change.

## 1. Implementation Handoff

**Goal.** Make every project fact live in one place: single-source the
workspace version, inherit common package metadata, delete dead code,
and bring the changelog and release doc back in line with policy.

**Key mechanics.**

- **Version is single-sourced** in `[workspace.package].version`. Every
  member uses `version.workspace = true`. A bump is now one edit (use
  `./version.sh --update X.Y.Z`).
- **Metadata inheritance.** Members inherit `version`, `authors`,
  `repository`, `license`, `edition`, `rust-version`, `categories`,
  `keywords` via `{ workspace = true }`. Each member keeps only its own
  `name`, `description`, and `readme`.
- **`readme` is per-crate and depth-correct.** `app`/`env` →
  `../README.md`; `crates/<x>` → `../../README.md`; `crates/<a>/<b>` →
  `../../../README.md`. (The previous `readme` values were broken paths.)
- **`arama-storage` is deleted** (`crates/engine/storage/`). It was
  outside the build graph, so nothing that compiles changed.

**Pitfalls to avoid.**

- Do **not** reintroduce a literal `version = "…"` into a member
  manifest — it shadows the workspace inheritance and silently drifts.
- `authors` is the bare `["nabbisen"]` (privacy / anti-spam). Do not
  re-add an email address when touching manifests.
- Release archives must have **no** parent directory (files at the
  archive root). The old `--transform` recipe is gone.

## 2. Task Breakdown / PR Plan

Independent, can land as one PR or several commits:

1. Root `Cargo.toml`: `[workspace.package].version` → `0.35.0`;
   `repository` `orbok` → `arama`.
2. Rewrite the ten member `[package]` blocks (inherit + per-crate
   `description`/`readme`).
3. Delete `crates/engine/storage/`.
4. `CHANGELOG.md`: add `[0.34.0]`, add `[0.35.0]`, open fresh
   `[Unreleased]`.
5. `version.sh`: single-field, jq-free rewrite.
6. `docs/src/dev/release.md`: steps 2 / 5 / 6 + checklist.
7. RFC lifecycle: `proposed/ → done/`, `rfcs/README.md`, this handoff.

## 3. Acceptance / QA Checklist

Reusable as a regression pass:

- [ ] `cargo metadata --no-deps` resolves; exactly ten `arama*` members,
      all at `0.35.0`; **no** `arama-storage`.
- [ ] `git grep -n 'version = "' -- '*/Cargo.toml'` finds the version
      only in the root `[workspace.package]` (members inherit).
- [ ] `cargo fmt --check` clean; `git diff --stat` shows **no** `.rs`
      files.
- [ ] Representative leaf crates compile (`cargo check -p arama-i18n -p
      arama-env -p arama-theme`) and their unit tests pass
      (`cargo test -p arama-i18n -p arama-env`).
- [ ] Each member resolves `readme` to the root `README.md`.
- [ ] `tar tzf arama-v0.35.0.tar.gz | head` shows project files at the
      root, not a wrapping directory.
