> **ACCEPTED 2026-11-18 — review 187** (`92cf7d6`, ledger fix by the architect).

> **AUTHORIZED 2026-11-17 — owner:** *"Authorized."* Written 2026-11-17 by the
> architect, after `pt2safetensors` 0.2.0 was published (owner and architect, the
> same day).
>
> **Independent of T083.**

# T082 Handoff — adopt `pt2safetensors` 0.2.0, and retire the eight `zip` ignores

## 1. Purpose

`pt2safetensors` 0.2.0 depends on `candle-core` 0.11 (T081). Adopting it removes
`zip` 7.2.0, the last `zip` below 9, from arama's graph. The eight ignores that
existed only for it are then retired, as RFC 049 requires: proven unneeded first,
then removed, and recorded as resolved.

## 2. Background

- **Published 2026-11-17:** `pt2safetensors` 0.2.0, `rust-version` 1.94,
  `candle-core` 0.11, `safetensors` 0.8. Tag `0.2.0` (signed). The converted CLIP
  model is byte-identical to 0.1.3's output (review 185, reproduced by the
  architect).
- arama declares it once: `Cargo.toml:80`,
  `[workspace.dependencies] pt2safetensors = "0.1.3"`. `crates/ai` uses
  `{ workspace = true }`.
- **Measured by the architect** (the manifest edit plus
  `cargo update -p pt2safetensors`, then reverted): `pt2safetensors` 0.1.3 →
  0.2.0. Removed: `candle-core` 0.10.2, `onig` 6.5.3, `onig_sys` 69.9.3,
  `safetensors` 0.7.0, `tokenizers` 0.22.2, `typed-path` 0.12.3, `zip` 7.2.0.
  Nothing else moved. Afterwards: one `zip` (9.5.1), one `candle-core` (0.11.1).
- **The ignores this retires** (`.cargo/audit.toml`): RUSTSEC-2026-0367, 0368,
  0369, 0429 (ledger: `zip` 7.2.0 / 8.6.0 entry) and 0545, 0546, 0547, 0548 (ledger:
  `zip` 7.2.0 — RUSTSEC-2026-0545 through 0548). Each entry's revisit condition is
  "`pt2safetensors` on `candle-core` ≥ 0.11.1 (no `zip` 7.x in the lockfile)",
  and this task meets it.
- **T066's script does not flag them**, by design: going from 0.1 to 0.2 is a
  semver-major change, which its own note says it cannot see. §6.3 proves each
  retirement directly.

## 3. Applicable RFC and requirements

[RFC 049](../done/049-advisory-maintenance-policy.md) §3, §4 (an ignore
whose condition is met is retired, with evidence), §8 (ignore list and ledger in
sync).

## 4. Change scope

- `Cargo.toml` (the one line)
- `Cargo.lock`
- `.cargo/audit.toml`
- `rfcs/notes/audit-warning-burn-down.md`

## 5. Explicit non-change scope

- Any `.rs` file. `pt2safetensors`' API is unchanged; if anything fails to build,
  stop and report.
- Any other dependency: the update is `-p pt2safetensors` only.
- The other seven ignores (`paste`, `ttf-parser`, `lru`, the three `zbus`
  family, `chrono`).

## 6. Required implementation

1. `pt2safetensors = "0.2.0"` in `[workspace.dependencies]`, then
   `cargo update -p pt2safetensors`. **List every package that moves**, and
   compare it with §2's measured list. **Stop if anything differs.** Confirm:
   `grep -A1 '^name = "zip"$' Cargo.lock` gives only 9.x, and only one
   `candle-core` remains.
2. **Prove each of the eight is unneeded:** remove all eight from
   `.cargo/audit.toml`. `cargo audit --deny warnings` then exits 0, and its
   output names none of them. Report that output.
3. **The ledger:** move both `zip` entries into a new `## Resolved:` section
   dated 2026-11-17. Name the commit range, the before/after (`zip` 7.2.0
   gone), and the eight IDs. Do it the way the existing *Resolved* sections do,
   for example the `rustix`/`quinn-proto` one.
4. T066's script: **7** ignored, 0 retirable, in sync.

## 7. Required tests

- `cargo audit --deny warnings` exits 0 with the eight removed.
- `cargo build`, `clippy --all-targets -D warnings`, `fmt --check`, all
  `--locked`. `cargo test --workspace`, with `ARAMA_DATA_HOME` and the four
  `XDG_*` homes set into `.git-exclude/tmp/`. **If T083 has not landed yet**, its
  test fails under `ARAMA_DATA_HOME`. Then run with the `XDG_*` homes only, and
  say so.
- `cargo +1.94 check --workspace --locked`, with the project-local
  `RUSTUP_HOME="$PWD/.git-exclude/tmp/rustup-1.94"` (already installed; check
  `rustup toolchain list` under it first).
- **One real conversion through arama:** the `arama-ai` tests that load a model
  must pass. If any test converts a `.bin` with `pt2safetensors`, name it. If
  none does, say so. The byte-identical comparison in review 185 already covers
  0.2.0's output.

## 8. Required documentation updates

The ledger, as §6.3.

## 9. Acceptance criteria

- `pt2safetensors` 0.2.0. The lockfile changes exactly as §2 measured. No `zip`
  below 9.
- Eight ignores removed, shown unneeded by `cargo audit`, and recorded as
  resolved. T066's script: 7, in sync.
- All gates green, including the 1.94 check. `Audit` and `MSRV` green after the
  architect pushes.

## 10. Prohibited shortcuts

- Removing an ignore without the `cargo audit` run that shows it is unneeded.
- A bare `cargo update`.
- Deleting the ledger entries instead of moving them to *Resolved*.
- Any `cargo +<version>` or `rustup` call without the project-local `RUSTUP_HOME`.

## 11. Module boundaries

Workspace manifest, lockfile, audit config, ledger.

## 12. Compatibility constraints

`rust-version` 1.94 matches arama's baseline (T080). No MSRV change.

## 13. Security constraints

The point of the task: no `zip` below 9 in the graph, so the eight advisories are
gone, not only shown to be unreachable.

## 14. Known risks

- **`tokenizers` 0.22.2 leaves.** It was `candle-core` 0.10.2's. The remaining
  0.23.2 is `candle-core` 0.11.1's, and arama declares no `tokenizers` of its own.
  Confirm the build and the `arama-ai` tests; do not add anything back.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Did exactly the measured set move? | machine | the lockfile diff, compared with §2 |
| Is each of the eight unneeded? | machine | `cargo audit` with them removed |
| Ledger and ignore list in sync? | machine | T066's script |
| Anything regressed? | machine | gates, the 1.94 check, the `arama-ai` tests |

No GUI involved.

### 15.2 Deliverables

The rows above, observed, and the commit.

## 16. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the commit, the `git show` command, and full repo-relative paths, one per line.
Report observed output; a check not run is recorded as not run. You cannot push:
report the commit, and the architect pushes.

## 17. Handoff checklist

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | §3 |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §6 |
| Required tests | §6, §7 |
| Required documentation updates | §8 |
| Acceptance criteria | §9 |
| Prohibited shortcuts | §10 |
| Relevant code or module boundaries | §11 |
| Compatibility constraints | §12 |
| Security constraints | §13 |
| Known risks | §14 |
| Required evidence | §15 |
| Required review-request format | §16 |
