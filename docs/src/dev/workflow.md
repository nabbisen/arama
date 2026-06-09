# Development Workflow

## Prerequisites

- Rust toolchain via [rustup.rs](https://rustup.rs/) (stable channel)
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
**English**. (Some older files contain Japanese comments; these are
translated as they are touched.)

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

There is no CI enforcement at this stage; run both before opening a
pull request.

## Workspace version bumps

`version.sh` updates the `version` field in every member `Cargo.toml`
atomically:

```sh
./version.sh --list              # show current versions
./version.sh --update 0.25.0     # bump all crates
./version.sh --update 0.25.0 --dry-run  # preview only
```

The script also `git add`s the modified manifests and `Cargo.lock`.

## Adding a new crate

1. `cargo new --lib crates/<category>/<name>`
2. Add to `[workspace.members]` in the root `Cargo.toml`.
3. Add workspace-level dependency entries for any new third-party crates
   under `[workspace.dependencies]`.
4. Keep the crate focused: one clear responsibility, explicit public API
   surface in `src/lib.rs`, and an `//!` crate-level doc comment.
