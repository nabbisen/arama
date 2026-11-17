> **REWORK 2026-11-17 — review 184: raise the Rust baseline to 1.94.** Owner, on
> option A ("raise the minimum to Rust 1.94 in both crates"): *"Authorized."*
> `candle-core` 0.11.1 compiles on x86_64 only from Rust 1.94.0: `__cpuid_count`
> became safe there (stdarch PR #1935; `pub unsafe fn` at the 1.93.0 tag, `pub fn`
> at 1.94.0). Your `dfea607` stands. Add **one commit on top of it**:
>
> 1. `[workspace.package] rust-version = "1.94"` in the root `Cargo.toml`, the one
>    normative declaration (`MSRV` reads it). Its mirrors:
>    `docs/src/users/installation.md` (line 18), `docs/src/dev/workflow.md`
>    (line 6), and the comment in `.github/workflows/clippy.yaml` (line 20). No
>    other file names the baseline (`grep -rn '1\.91'` outside `rfcs/`,
>    `CHANGELOG.md` and `Cargo.lock`, to confirm).
> 2. `cargo update -p candle-nn@0.11.0 -p candle-transformers@0.11.0`, so the
>    candle family is on one patch level (my §6.1 expected this from
>    `-p candle-core`, and it does not reach them). List every package that moves.
>    **Stop if anything beyond `candle-*` and their own new dependencies moves.**
> 3. **Prove 1.94 without writing to the home directory:** install the toolchain
>    into the project, using
>    `RUSTUP_HOME="$PWD/.git-exclude/tmp/rustup-1.94" rustup toolchain install 1.94 --profile minimal`.
>    Then run, with the same `RUSTUP_HOME`, `cargo +1.94 check --workspace --locked`
>    and `cargo +1.94 test -p arama-ai --locked`. If rustup writes anything
>    outside that folder, stop and report. Keep the folder: T081 uses it too.
> 4. All gates on the default toolchain again, plus T066's script (still 15, in
>    sync). The architect pushes `dfea607` and this commit together, so `MSRV`
>    never sees one without the other.
>
> The architect added `exclude = [".git-exclude"]` to the workspace (pushed
> separately). Rebase onto `origin/main` before you start; `dfea607` gets a new
> hash, so report it.

> **AUTHORIZED 2026-11-17 — owner:** *"Authorized."* Written 2026-11-17 by the
> architect. The end state (no `zip` 7.x) follows through T081 (pt2safetensors) and
> T082 (arama adopts its release); this task gets `main` green meanwhile.
>
> **`main` is red on `Audit` until this lands.** Every push since `89a69a3` fails
> that one workflow. Independent of every other task.

# T080 Handoff — four new `zip` advisories, and `chrono` unmaintained

## 1. Purpose

Bring `cargo audit --deny warnings` back to exit 0. Clear what an update can clear,
and record the rest under RFC 049 with per-identity reasoning and a concrete revisit
condition — never a blanket ignore.

## 2. Background

Observed 2026-11-17 (`Audit` on `89a69a3`, reproduced locally with a fresh
database):

| ID | Crate | Summary | Patched |
|---|---|---|---|
| RUSTSEC-2026-0545 | `zip` 7.2.0, 8.6.0 | AES entries returned without checking their authentication code | ≥ 9.0.0 |
| RUSTSEC-2026-0546 | `zip` 7.2.0, 8.6.0 | `ZipArchive::extract` can write outside the destination via symlinks | ≥ 9.0.0 |
| RUSTSEC-2026-0547 | `zip` 7.2.0, 8.6.0 | Shrink/Reduce/Implode decoders preallocate the declared size | ≥ 9.0.0 |
| RUSTSEC-2026-0548 | `zip` 7.2.0, 8.6.0 | `ZipStreamReader` uses local-header names that can differ from the central directory | ≥ 9.0.5 |
| RUSTSEC-2026-0543 | `chrono` 0.4.45 | unmaintained (warning) | — |

**Paths** (`cargo tree -i … -e normal`):

- `zip` 8.6.0 ← `candle-core` 0.11.0 ← `arama-ai`.
- `zip` 7.2.0 ← `candle-core` 0.10.2 ← `pt2safetensors` 0.1.3 ← `arama-ai`.
- `chrono` 0.4.45 ← `arama-ui-main` (directly: `crates/ui/main/src/core/views/cache_page/view.rs`,
  local-time formatting) and ← `trash` 5.2.6 ← `file-handle`.

**What an update reaches** (`cargo update --dry-run`, architect, 2026-11-17):
`candle-core` 0.11.0 → 0.11.1 (with `candle-kernels`, `candle-metal-kernels`,
`candle-nn`, `candle-transformers`), which **replaces `zip` 8.6.0 with 9.6.1**
and removes `candle-ug`. `zip` 7.2.0 can only move to 7.4.0 — below every fix —
because `pt2safetensors` 0.1.3 pins `candle-core` 0.10.

**The end state needs the owner, not this task:** `pt2safetensors` is the
owner's crate. A release on `candle-core` ≥ 0.11.1 removes `zip` 7.x from
arama's graph entirely — the four new IDs, **and** the existing
RUSTSEC-2026-0367/0368/0369/0429 ignores, whose recorded revisit condition is
exactly *"candle-core depending on zip ≥ 9.0.0"*. Publishing is the owner's
action. This task only states the condition.

## 3. Applicable RFC and requirements

[RFC 049](../proposed/049-advisory-maintenance-policy.md) §3 (each ignore with
its path, its reasoning and its revisit condition, in
`rfcs/notes/audit-warning-burn-down.md`, in sync with `.cargo/audit.toml` —
checked by T066's script), §4.

## 4. Change scope

- `Cargo.lock`
- `.cargo/audit.toml`
- `rfcs/notes/audit-warning-burn-down.md`

## 5. Explicit non-change scope

- Any `.rs` file, any `Cargo.toml`. In particular, `chrono` stays: `trash`
  keeps it in the graph regardless, so replacing arama's own use buys nothing
  for the audit.
- `pt2safetensors` (the owner's crate).
- Any package beyond what `cargo update -p candle-core` moves.

## 6. Required implementation

1. **`cargo update -p candle-core@0.11.0`** (only that). **List every package
   that moves**, by name/version pair. Expected: the five `candle-*` crates to
   0.11.1, `zip` 8.6.0 → 9.6.1, `candle-ug` removed, plus whatever those pull
   in. Explain anything else, or stop. Then confirm: no `zip` 8.x remains, and
   the four new IDs now match **only** `zip` 7.2.0.
2. **For each of RUSTSEC-2026-0545 … 0548, show by source** (in `candle-core`
   0.10.2 and `pt2safetensors` 0.1.3, the only consumers of `zip` 7.2.0) whether
   the affected API is reached:
   - 0546: is `ZipArchive::extract` (or `extract_unwrapped_root_dir`) called?
   - 0548: is `ZipStreamReader` used?
   - 0545: is `zip`'s `aes-crypto` feature enabled at all
     (`cargo tree -e features -i zip@7.2.0`), and does anything open an
     encrypted entry?
   - 0547: are the legacy decoders (`legacy-zip` feature or equivalent)
     compiled in?
   And, for every ID: what input reaches the parser. arama feeds it only the
   model file it downloaded and verified against the SHA-256 pinned in
   `crates/ai/src/model/model_container/clip.rs`, so no crafted archive can
   reach it. Say so per ID, but **do not let it stand in for the per-ID
   check**: an unreachable API is the stronger reason.
3. **Ignore each of the four in `.cargo/audit.toml`**, with a one-line comment,
   and a ledger entry each (or one grouped entry — T066's matcher reads
   comma- and "through"-joined identities) with the path, the per-ID finding and
   the **revisit condition: `pt2safetensors` on `candle-core` ≥ 0.11.1 (no
   `zip` 7.x in the lockfile)**.
4. **RUSTSEC-2026-0543 (`chrono`)**: an ignore and a ledger entry with both
   paths, and the revisit condition *"`trash` drops `chrono`, or a maintained
   successor is named in the advisory"*. Note arama's own direct use (one view)
   for a future decision.
5. Run T066's script: it must report **0 retirable** and the ignore list and
   ledger in sync, with the new count.

## 7. Required tests

`cargo audit --deny warnings` exits 0. `cargo build`, `cargo test --workspace`,
`clippy --all-targets -D warnings`, `fmt --check`, all `--locked`.
`.github/workflows/scripts/detect-retirable-ignores.sh` output as §6.5 says.
**One real model run** is not needed — `candle-core` is a patch release — but
the `arama-ai` tests that load a model must pass.

## 8. Required documentation updates

The ledger (`rfcs/notes/audit-warning-burn-down.md`), as §6.

## 9. Acceptance criteria

- `zip` 8.x gone; the four new IDs match only `zip` 7.2.0, each with a per-ID
  reachability finding.
- Five new ignores, each with a ledger entry and a revisit condition; T066's sync
  check green.
- `cargo audit --deny warnings` exits 0; all gates green; `Audit` green after the
  architect pushes.

## 10. Prohibited shortcuts

- Ignoring an ID without its per-ID source check.
- A bare `cargo update` (it also moves `zip` 7.2.0 → 7.4.0 and unrelated
  packages; this task moves only what it names).
- Relying on the checksum argument alone.
- Editing `pt2safetensors` or any `.rs` file.

## 11. Module boundaries

The lockfile, the audit config, the ledger.

## 12. Compatibility constraints

`candle-core` 0.11.1 must build on the MSRV toolchain (`cargo +<rust-version>
check`) and on the macOS/Windows CI runners (the architect watches them).

## 13. Security constraints

The point of the task. The pinned SHA-256 on every model file is the integrity
guarantee that bounds what any `zip` parser in this graph can be fed.

## 14. Known risks

- **A `candle-*` patch can change numerics.** The model-loading tests must pass;
  if an embedding test's expected values move, stop and report.
- **`zip` 9.6.1 is a new major in the graph.** Only `candle-core` 0.11.1 uses it.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| What moved in the lockfile? | machine | the name/version diff |
| Is each `zip` advisory's API reached? | source | per-ID findings (§6.2) |
| Is the audit green and the ledger in sync? | machine | `cargo audit`, T066's script |
| Did anything regress? | machine | the gates, the `arama-ai` tests |

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
| Required tests | §7 |
| Required documentation updates | §8 |
| Acceptance criteria | §9 |
| Prohibited shortcuts | §10 |
| Relevant code or module boundaries | §11 |
| Compatibility constraints | §12 |
| Security constraints | §13 |
| Known risks | §14 |
| Required evidence | §15 |
| Required review-request format | §16 |
