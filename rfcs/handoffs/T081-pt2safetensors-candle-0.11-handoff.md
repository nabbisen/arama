> **REWORK 2026-11-17 — review 185.** The conversion is byte-identical to 0.1.3's
> output (reproduced by the architect). Owner, on raising the minimum to Rust
> 1.94 in both crates: *"Authorized."* (`candle-core` 0.11.1 compiles on x86_64
> only from Rust 1.94.0; see review 184 and T080's rework banner.) In the
> pt2safetensors repository, **one commit on top of `13a2677`**:
>
> 1. **Remove the empty `[workspace]`** from `Cargo.toml`. It existed only because
>    the clone sits inside arama's tree, and it must not ship in the owner's
>    manifest. arama now excludes `.git-exclude` from its workspace, which is
>    what makes cargo work there without it. Confirm `cargo build` still runs.
> 2. **`rust-version = "1.94"`.** Prove it with the toolchain T080's rework
>    installs under `.git-exclude/tmp/rustup-1.94` (same `RUSTUP_HOME`, no
>    home-directory write): `cargo +1.94 check` and `cargo +1.94 test` in the
>    crate. **Also show that `cargo +1.93` fails** if you can install it the same
>    way. The crate has no CI, so this is the only proof of the floor it declares.
>    If 1.93 cannot be installed that way, say so.
> 3. All gates again (build, test, clippy `-D warnings`, `fmt --check`). There is
>    no need to repeat the real-model comparison: `rust-version` and `[workspace]`
>    do not reach the conversion.
>
> `version` still unchanged; no push, tag or publish.

> **AUTHORIZED 2026-11-17 — owner:** *"You and dev team may update it. It is my
> project."* Written 2026-11-17 by the architect.
>
> **This task works in a different repository:** the owner's `pt2safetensors`,
> cloned at `.git-exclude/projects/pt2safetensors` (its own git history, remote
> `github.com/nabbisen/pt2safetensors`). Commit there, not in arama.
> **Pushing it, tagging and publishing to crates.io are the owner's and the
> architect's** — the dev team does none of them, and does not change the
> crate's `version`.
>
> **Independent of T080**, which keeps arama green meanwhile. **T082** (arama
> adopts the release and retires the `zip` ignores) follows the release.

# T081 Handoff — `pt2safetensors` on `candle-core` 0.11

## 1. Purpose

`pt2safetensors` 0.1.3 depends on `candle-core = "0.10"`. That keeps `zip` 7.x
in every consumer's graph, and `zip` < 9 carries eight advisories:
RUSTSEC-2026-0367, 0368, 0369, 0429 and 0545–0548 (T080). Moving to
`candle-core` 0.11.1 brings `zip` 9.6.1, which fixes all of them. Do it, and prove
that the conversion still produces the same file.

## 2. Background

- The crate is 62 lines (`src/lib.rs`, `src/core.rs`). It uses one `candle-core`
  API: `candle_core::pickle::read_all(path)`. That API keeps the same signature in
  0.11 (`candle-core` 0.11.0 `src/pickle.rs:839`). The crate's public API exposes
  no `candle` type: `convert` returns `anyhow::Error`. **No public API change is
  expected.**
- **No tests, and no CI**, in the repository. §6 adds the evidence this release
  needs.
- **The real-model fixture** lives in arama's working copy:
  `.git-exclude/fixtures/models-raw/pytorch_model.bin` (the CLIP model as
  downloaded, SHA-256 pinned in arama's
  `crates/ai/src/model/model_container/clip.rs`), and
  `.git-exclude/fixtures/models/.arama-local/model/clip-vit-base-patch32/model.safetensors`
  — **the output of `pt2safetensors` 0.1.3 on that file** (T073). That is the
  reference.
- `rust-version = "1.90.0"`. `candle-core` 0.11 declares none.

## 3. Applicable RFC and requirements

arama's RFC 049 (advisory maintenance), applied upstream at the source.

## 4. Change scope (in `.git-exclude/projects/pt2safetensors`)

- `Cargo.toml` — `candle-core = "0.11"`
- `Cargo.lock`
- `README.md`, only if it names the `candle` version
- tests, if you add one that needs no large file in the repository (§6.3)

## 5. Explicit non-change scope

- `version` in `Cargo.toml`, tags, `CHANGELOG`, publishing — the release is the
  owner's and the architect's.
- Any other dependency, beyond what the `candle-core` move pulls in.
- The public API, and the conversion's behaviour.
- arama's repository (T080 and T082 are separate).

## 6. Required implementation and tests

1. **The move:** `candle-core = "0.11"`, then `cargo update -p candle-core`.
   **List every package that moves** by name/version pair. `zip` 9.x is
   expected; **no `zip` < 9 may remain** (`grep '^name = "zip"' -A1 Cargo.lock`).
   Run `cargo audit --deny warnings` in the crate and report it.
2. **Same output, on the real model:** convert
   `.git-exclude/fixtures/models-raw/pytorch_model.bin` with the updated crate,
   through a throwaway binary or example **in `.git-exclude/tmp/`**, with its own
   `CARGO_TARGET_DIR`, depending on the crate by `path`. Compare the output with
   the 0.1.3 reference:
   - **byte-identical** (`sha256sum` of both) — report it; or, if not,
   - **tensor-identical**: the same names, dtypes, shapes and bytes per tensor,
     compared by a small program (`safetensors` can read both). Report which, and
     if the bytes differ, why (for example, header ordering), with evidence.
   Copy nothing from the fixtures into the pt2safetensors repository, and do not
   modify the fixtures.
3. **A test in the crate** that needs no large file: at least that a path which
   is not a PyTorch file is refused with an error, not a panic or an empty
   output. If you can build a tiny valid input from code without Python, add a
   round trip too; if not, say so.
4. **Gates in the crate:** `cargo build`, `cargo test`, `cargo clippy
   --all-targets -- -D warnings`, `cargo fmt --check`, and `cargo +1.90.0 check`
   (the declared `rust-version`). If 0.11 needs a newer Rust, **stop and
   report** — raising `rust-version` is the owner's decision.

## 7. Required documentation updates

`README.md` only if it states the `candle` version or anything §6 changes.

## 8. Acceptance criteria

- `candle-core` 0.11.x; no `zip` < 9 in the crate's lockfile; `cargo audit`
  clean.
- The real CLIP model converts to a byte- or tensor-identical file, shown.
- A new test, and all gates, including the MSRV check.
- No version, tag or publish.

## 9. Prohibited shortcuts

- Asserting equivalence without the real-model comparison.
- Committing a model file, or anything from `.git-exclude/fixtures/`, to the
  pt2safetensors repository.
- Pushing, tagging, publishing, or changing `version`.
- Raising `rust-version` without the owner.

## 10. Module boundaries

The pt2safetensors repository only; scratch work in arama's `.git-exclude/tmp/`.

## 11. Compatibility constraints

Same public API; same output for the same input; `rust-version` 1.90.0 unless
the owner decides otherwise.

## 12. Security constraints

The point of the task: `zip` ≥ 9 for every consumer. The crate reads only the
file its caller names. That is arama's pinned, checksum-verified model, but other
users may pass anything, which is why the advisories matter upstream more than in
arama.

## 13. Known risks

- **`candle` 0.11's pickle reader may order or name tensors differently.** §6.2
  decides whether that is a byte difference only, or a real one.
- **Memory:** the CLIP model is about 600 MB; the conversion holds it in memory.
  Close other heavy work while it runs.

## 14. Required evidence

### 14.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| What moved, and is `zip` ≥ 9? | machine | the lockfile diff, `cargo audit` |
| Is the output the same on the real model? | machine | the §6.2 comparison |
| Does it still build where it promises? | machine | the gates, `cargo +1.90.0 check` |

No GUI involved.

### 14.2 Deliverables

The rows above, observed, and the commit in the pt2safetensors repository.

## 15. Review-request format

A package under arama's `.git-exclude/review-request/` with a `## Entry point`
section: the repository path (`.git-exclude/projects/pt2safetensors`), the
commit, `git -C .git-exclude/projects/pt2safetensors show <commit>`, and full
paths, one per line. Report observed output; a check not run is recorded as not
run.

## 16. Handoff checklist

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | §3 |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §6 |
| Required tests | §6 |
| Required documentation updates | §7 |
| Acceptance criteria | §8 |
| Prohibited shortcuts | §9 |
| Relevant code or module boundaries | §10 |
| Compatibility constraints | §11 |
| Security constraints | §12 |
| Known risks | §13 |
| Required evidence | §14 |
| Required review-request format | §15 |
