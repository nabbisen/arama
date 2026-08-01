# Development Workflow

## Prerequisites

- Rust toolchain via [rustup.rs](https://rustup.rs/) (stable channel), Rust
  1.91 or newer — the workspace's verified contributor-setup baseline
  (`[workspace.package].rust-version` in the root `Cargo.toml` is the single
  normative declaration; this line and `docs/src/users/installation.md`
  mirror it)
- `cargo` in `PATH`

## Daily loop

```sh
# Check the whole workspace (fast — no linking)
cargo check --workspace

# Run the app in debug mode
cargo run -p arama

# Run tests for a specific crate
cargo test -p arama-cache

# Run all tests
cargo test --workspace
```

`cargo run` (debug profile) is fine for UI work; use
`cargo run -p arama --release` when measuring AI inference speed since
debug builds are significantly slower for SIMD-heavy candle kernels.

## Code conventions

### Language

All source code, comments, documentation, and RFC text must be in
**English**, except for user-facing locale strings and historical RFC records.
Older production comments are translated as they are touched.

### File size

| Threshold | Guidance |
|---|---|
| > 300 ELOC | Consider splitting at logical boundaries |
| > 500 ELOC | Strongly recommended to split |

The same thresholds apply to test files under `tests/`.

### Module layout

Rust 2018+ module style: a `foo.rs` and a `foo/` subdirectory may
coexist. `mod.rs` is not used.

Tests for a module live in a sibling `tests.rs` file or, if large, in
`tests/` subdirectory modules.

### Error handling

- Library crates use `thiserror`-derived typed error enums.
- Application code uses `.expect(...)` for invariants and `push_error_toast`
  for user-visible errors in the UI.
- `todo!()` and `eprintln!` are acceptable placeholders; `unwrap()`
  without a comment is a code smell in production paths.

### Async and tasks

Long-running work (AI inference, cache writes) runs inside
`Task::perform` or `Task::run` so it does not block the iced event loop.
Both use `Task::abortable` so the handle can be stored and cancelled
when the user changes directories.

The AI embedding loop calls `tokio::task::yield_now().await` at each
file boundary to allow responsive cancellation.

### Formatting and linting

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

There is no CI enforcement for formatting or linting at this stage; run both
before opening a pull request. CI does verify the Rust baseline declared
above — see [MSRV verification](#msrv-verification) below.

## MSRV verification

The `MSRV` GitHub Actions workflow (`.github/workflows/msrv.yaml`) installs
the exact declared `[workspace.package].rust-version` toolchain — not
`stable` or another floating channel — and runs `cargo check --workspace
--locked` and `cargo test --workspace --locked` on it. It triggers on every
push and pull request against `main`.

This is the only CI job in the repository. It exists to keep the declared
baseline (above) true by construction rather than by memory: a
`rust-version` with no automated check is a claim that can silently drift
out of date as dependencies are bumped. It is intentionally narrow —
formatting, Clippy, `cargo audit`, feature-matrix, and cross-target
verification remain manual, run locally before opening a pull request, as
described above and in the [release process](./release.md).

## Workspace version bumps

All workspace packages inherit `[workspace.package].version`. `version.sh`
updates that single field atomically in the root `Cargo.toml`:

```sh
./version.sh --list              # show the workspace version
./version.sh --update 0.25.0     # bump the inherited package version
./version.sh --update 0.25.0 --dry-run  # preview only
```

Internal workspace dependency requirements are intentionally outside the
helper. Adding or removing a crate therefore does not require a script change.
The helper also does not edit member manifests, `Cargo.lock`, the changelog, or
the Git index. After a real release bump, refresh and review `Cargo.lock`
explicitly as described in the [release process](./release.md).

## Adding a new crate

1. `cargo new --lib crates/<category>/<name>`
2. Add to `[workspace.members]` in the root `Cargo.toml`.
3. Add workspace-level dependency entries for any new third-party crates
   under `[workspace.dependencies]`.
4. Keep the crate focused: one clear responsibility, explicit public API
   surface in `src/lib.rs`, and an `//!` crate-level doc comment.
