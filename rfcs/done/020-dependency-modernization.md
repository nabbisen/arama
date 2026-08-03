# RFC 020 - Dependency modernization: Candle and sidecar archive stack

**Status.** Implemented (0.37.0)
**Tracks.** Roadmap follow-up after the audit warning burn-down: modernize
direct dependencies deliberately, with platform-specific gate evidence where
dependency behavior can affect AI inference or ffmpeg sidecar setup.
**Touches.** `crates/ai/Cargo.toml`,
`crates/engine/sidecar/Cargo.toml`, `Cargo.lock`,
`crates/ai/src/pipeline/encode/**`,
`crates/engine/sidecar/src/media/video/video_engine.rs`,
`rfcs/notes/audit-warning-burn-down.md`, `CHANGELOG.md`.

## Summary

The audit warning burn-down resolved compatible patch-level warnings and left
five allowed warnings documented with dependency owners. The next dependency
work should not be a broad "update everything" pass. This RFC proposes a
bounded modernization batch:

1. Update the first-party Candle dependency set from 0.10 to 0.11.
2. Update the Windows/macOS sidecar `zip` dependency from the 4.x line to the
   latest stable 8.x line.
3. Keep unresolved audit-warning owners tracked unless an implementation pass
   proves a compatible direct update removes them.
4. Require platform-aware evidence for AI and sidecar extraction behavior.

The proposal is intentionally sequencing-oriented. It defines which dependency
families belong in the first modernization batch and which ones should remain
notes or future RFCs.

## Why

`cargo outdated --workspace` currently shows direct modernization candidates
that are not simple patch-level maintenance:

- `candle-core`, `candle-nn`, and `candle-transformers` are on 0.10.2 while
  the latest stable line is 0.11.0.
- The sidecar ZIP extraction dependency is declared as `zip = "4"` for
  non-Linux platforms, while the latest stable line observed during planning is
  8.6.0. The crate also has a 9.0.0 pre-release, which should not be adopted
  unless a blocking reason appears.

These dependencies sit on sensitive boundaries:

- Candle drives local CLIP and wav2vec2 inference, including CUDA and Metal
  feature wiring.
- The sidecar ZIP path handles ffmpeg setup on Windows and macOS.
- Candle currently brings in `zip` 7.2 transitively, so the workspace already
  carries multiple ZIP major lines when non-Linux targets are considered.

At the same time, the remaining audit warnings do not all have direct compatible
updates:

- `hnsw_rs` 0.3.4 is the latest observed direct owner of `bincode` 1.3.3.
- `localcache` 0.20.0 is the latest observed direct owner of `bincode` 2.0.1.
- `paste` is spread through Candle/gemm, tokenizers, image/rav1e, and iced
  rendering paths.
- `ttf-parser` 0.25.1 is the latest observed font/parser line.
- `proc-macro-error2` remains in the lockfile without a normal all-target
  workspace owner found during the audit pass.

A reviewed dependency-modernization RFC keeps the first implementation batch
useful without turning every transitive warning into replacement work.

## Design

### Part A - First modernization batch

The first implementation pass should attempt these direct updates together only
if they fit within one reviewable diff:

| Dependency family | Current declaration | Target | Boundary |
|-------------------|---------------------|--------|----------|
| Candle | `candle-core`, `candle-nn`, `candle-transformers` `0.10` | `0.11` stable | AI inference, CUDA, Metal |
| Sidecar ZIP | `zip = "4"` on `cfg(not(target_os = "linux"))` | `8` stable | Windows/macOS ffmpeg archive extraction |

If Candle 0.11 requires non-trivial code changes beyond API compatibility, the
implementation should split the work and bring only the independently safe
part to review.

### Part B - Candle migration checks

The implementation should inspect and update these Candle usage surfaces:

- `Device::new_cuda` / `Device::new_metal` fallback behavior in
  `ModelManager::device()`;
- `VarBuilder::from_mmaped_safetensors` calls for CLIP and wav2vec2;
- `ClipConfig`, `ClipModel`, and tensor construction/normalization paths;
- audio feature-extractor layers using `candle_nn`;
- CUDA feature propagation through the `arama-ai` `cuda` feature;
- macOS Metal dependency declarations.

The minimum acceptance bar is successful default Linux gates plus a clear
statement of whether CUDA and Metal were run or not run in the implementation
environment.

### Part C - Sidecar ZIP migration checks

The sidecar ZIP update should stay focused on archive extraction behavior:

- preserve `default-features = false`;
- keep only the decompression features required for current ffmpeg ZIP assets;
- confirm `ZipArchive::new(...).extract(...)` still compiles or adjust the
  extraction code with a focused compatibility change;
- keep Linux `tar`/`xz2` extraction untouched.

Because the active development platform may be Linux, implementation review
should include at least `cargo check --workspace --target x86_64-pc-windows-gnu`
or another available non-Linux target if installed. If no non-Linux Rust target
is installed, the review package must state that limitation explicitly.

### Part D - Audit-warning owners remain tracked

The implementation should not replace `hnsw_rs`, `localcache`, the iced/font
stack, or the image stack merely to silence warnings. Those are separate design
decisions unless a compatible dependency update naturally removes a warning.

After implementation, update `rfcs/notes/audit-warning-burn-down.md` only with
observed changes:

- remove or revise warning-owner entries that are actually resolved;
- keep unresolved entries with current observed owners;
- do not add broad new `.cargo/audit.toml` ignores without a release-gate
  policy review.

### Part E - Pre-release dependency policy

Pre-release dependency lines are out of scope for this RFC. For example, the
`zip` crate currently reports a 9.0.0 pre-release through `cargo info`, while
`cargo outdated` identifies 8.6.0 as the stable modernization target. Adopt a
pre-release only in a separate proposal or for a blocking advisory with no
stable fix.

## Touches in detail

### `crates/ai/Cargo.toml`

Update the default, CUDA, and Metal Candle declarations consistently. Avoid
splitting Candle crates across incompatible minor lines.

### `crates/ai/src/pipeline/encode/**`

Adjust only compatibility issues caused by Candle 0.11. Avoid model, scoring,
embedding-shape, or threshold behavior changes.

### `crates/engine/sidecar/Cargo.toml`

Update the non-Linux `zip` dependency to the selected stable major line and
keep feature selection narrow.

### `crates/engine/sidecar/src/media/video/video_engine.rs`

Touch only if the new ZIP API requires a compatibility adjustment.

### `Cargo.lock`

Expect dependency churn from Candle and ZIP updates. The review package should
summarize meaningful dependency graph changes, especially any impact on the
five tracked audit warnings.

### `CHANGELOG.md`

Record the implementation under `[Unreleased]` as dependency modernization,
not as a release point.

## Non-goals

- No replacement of `hnsw_rs`, `localcache`, iced, image, or font-stack crates.
- No scoring, model-selection, embedding-shape, or cache schema change.
- No setup UX redesign.
- No adoption of dependency pre-releases by default.
- No new audit ignores unless a separate release-gate policy review approves
  them.
- No release action or RFC lifecycle movement.

## Risks

- Candle 0.11 may change API details or backend feature behavior. Mitigation:
  keep the code diff focused and require AI tests plus explicit CUDA/Metal
  evidence status.
- ZIP 8 may alter default features or extraction behavior. Mitigation: keep
  `default-features = false`, preserve the required decompression feature, and
  check at least one non-Linux target when available.
- A broad lockfile update could hide unrelated changes. Mitigation: use
  targeted dependency edits and summarize lockfile churn in the review package.
- Audit warnings may remain after modernization. Mitigation: update the audit
  note based only on observed outcomes and keep unresolved owners visible.

## Test plan

Required default gates:

```sh
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

Dependency-specific checks:

```sh
cargo tree -i candle-core
cargo tree --target all -i zip
cargo outdated --workspace --depth 1
```

Platform evidence:

- Run the all-features CUDA gate if `nvcc` and the required CUDA environment
  are available.
- Run a macOS/Metal check on a macOS host if available.
- Run a non-Linux sidecar target check if the Rust target is installed.
- If a platform gate is not available in the implementation environment, record
  it under "Not run" rather than implying coverage.
