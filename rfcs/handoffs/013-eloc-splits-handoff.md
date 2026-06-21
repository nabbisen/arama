# RFC 013 Handoff — ELOC splits

Companion to [RFC 013](../done/013-eloc-splits.md). Shipped in **v0.36.0**.
Pure structural refactor — no logic changes, no API changes.

## 1. Implementation Handoff

**Goal.** Bring every `.rs` file in the project under 300 ELOC by splitting two
over-threshold files along natural logical seams.

**`app/src/core/update.rs` (was 543 ELOC → now 4 files):**

- `update.rs` — thin router (~35 ELOC); `match message` delegates to handlers
- `update/cache.rs` — cache pipeline handlers + dir helpers (~273 ELOC)
- `update/component.rs` — component delegation (Setup, Gallery, Header, …) (~232 ELOC)
- `update/ui.rs` — nav + housekeeping (toast, cursor, dialog close) (~45 ELOC)

All handler functions use `pub(super)` visibility, keeping the interface
contained. The router calls them as `self.handle_*(…)`.

**`crates/cache/tests/integration_tests.rs` (was 615 ELOC → now 5 files):**

- `integration_tests.rs` — module-level doc only (0 ELOC)
- `helpers.rs` — shared fixtures: `TempFile`, writers, `MINIMAL_JPEG`
- `image.rs` — 9 image-namespace tests (~163 ELOC)
- `video.rs` — 7 video-namespace tests (~141 ELOC)
- `cross.rs` — 11 cross-namespace / session / parallel / dir tests (~226 ELOC)

Each of `image.rs`, `video.rs`, `cross.rs` is a standalone Cargo test binary.
They share fixtures via `#[path = "helpers.rs"] mod helpers`.
`autotests = false` in `crates/cache/Cargo.toml` prevents Cargo from treating
`helpers.rs` itself as a test binary; four explicit `[[test]]` entries enumerate
the real binaries.

**Key pitfall (module resolution in tests/):** The Rust 2018+ "foo.rs + foo/"
rule only applies when the declaring file's stem matches the subdirectory name.
For `integration_tests.rs`, `mod image` looks for `tests/image.rs` (sibling),
not `tests/integration_tests/image.rs`. Using `#[path]` in each binary avoids
any ambiguity.

## 2. Task Breakdown

1. `app/src/core/update.rs` → router + `update/{cache,component,ui}.rs`
2. `crates/cache/tests/integration_tests.rs` → `integration_tests.rs` + `{helpers,image,video,cross}.rs`
3. `crates/cache/Cargo.toml` — `autotests = false` + four `[[test]]` entries
4. `cargo fmt` once; `cargo check --workspace`; `cargo test`
5. RFC lifecycle: `proposed/ → done/`, index, this handoff

## 3. Acceptance / QA Checklist

- [ ] `cargo check --workspace` — clean.
- [ ] `cargo test -p arama-cache -p arama-i18n -p arama-env` — 27 + 4 tests pass.
- [ ] `cargo fmt --check` — clean.
- [ ] Every `.rs` file under 300 ELOC (none flagged by the ELOC sweep).
- [ ] `git diff --stat` shows no files outside `app/src/core/update*`,
      `crates/cache/tests/*`, `crates/cache/Cargo.toml`, and RFC files.
