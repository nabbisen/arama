> **NOT AUTHORIZED YET.** Written 2026-11-14 by the architect. Do not start
> until this banner is replaced by the owner's authorization.
>
> **Urgent once authorized: the next push to `main` fails `Audit`.** Four
> advisories published since the last green run (`a6921ed`, 2026-11-09) reach
> `Cargo.lock`. Do this **before** T070's lockfile commits; if T070 has already
> committed its §2.6 `wasmtimer` retirement, build on it — do not combine the two.

# T072 Handoff — `rustix` unsound and three `quinn` advisories, cleared by lockfile updates

Governed by [RFC 049](../proposed/049-advisory-maintenance-policy.md) — §3
(the ignore list pairs with the ledger) and §4 (an ignore an update makes
unnecessary is retired, not kept).

## 1. Purpose

Return `cargo audit --deny warnings` to exit 0 by **updating the lockfile**,
and retire the five `quinn-proto` ignores the same update makes unnecessary.

## 2. Background

Run by the architect on 2026-11-14 against 1509 advisories:

| ID | Crate | In `Cargo.lock` | Kind | Fixed by |
|---|---|---|---|---|
| RUSTSEC-2026-0486 | `rustix` | 1.1.4 | unsound (`xattr` functions with a zero-length buffer) | 1.1.6 |
| RUSTSEC-2026-0523 | `quinn-proto` | 0.11.15 | vulnerability | 0.11.19 |
| RUSTSEC-2026-0524 | `quinn-proto` | 0.11.15 | vulnerability | 0.11.19 |
| RUSTSEC-2026-0538 | `quinn-udp` | 0.5.14 | vulnerability | 0.5.16 |

- **`rustix` 1.1.4** is reached on every desktop target through
  `async-io` ← `async-process` ← `zbus` ← `file-handle` / `mundy`. The localcache
  team flagged the same advisory in their 0.27.0 notice. arama does not call the
  six `xattr` functions; the update is the fix regardless.
- **`quinn-proto` and `quinn-udp`** are in the lockfile only. The architect
  found nothing for `cargo tree -i quinn-proto --target <t> -e normal,build` on
  `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` and
  `aarch64-apple-darwin`: reqwest's `http3` feature is not enabled — the ledger
  entry *"`quinn-proto` 0.11.15 — added 2026-10-21"* records why. **Updating is
  still right**: it is free, and it lets the existing ignores go (§3.2).

**The architect's dry run, on a copy** (both files restored afterwards):

```sh
cargo update -p rustix@1.1.4 -p quinn-proto -p quinn-udp
```

moved `rustix` 1.1.4 → 1.1.6, `quinn-proto` 0.11.15 → 0.11.19, `quinn-udp`
0.5.14 → 0.5.16; added `chacha20` 0.10.2, `rand` 0.10.3, `rand_core` 0.10.1 and
`rand_pcg` 0.10.2; removed `windows-sys` 0.60.2 and nine `windows*` 0.53 crates.
Then `cargo audit --deny warnings` exited 0 — **and still exited 0 with
RUSTSEC-2026-0338 to 0342 removed from `.cargo/audit.toml`.**

## 3. Required implementation

### 3.1 The update

Run the command above, against `main` as you find it. **Reproduce the package
list yourself** and report it; if it differs from §2's, explain each difference
or stop and report. `-p rustix@1.1.4` names the version because the lockfile
also holds `rustix` 0.38.44, which is not affected.

### 3.2 Retire the five `quinn-proto` ignores

Remove `RUSTSEC-2026-0338`, `0339`, `0340`, `0341` and `0342` from
`.cargo/audit.toml`, with the comment block that explains them. In
`rfcs/notes/audit-warning-burn-down.md`, move the entry *"`quinn-proto` 0.11.15 —
added 2026-10-21"* from **Remaining Warnings** to a dated **Resolved** section,
as the earlier retirements were, saying: retired by a lockfile update to
0.11.19, the commit, and that the three new advisories were cleared by the same
update rather than ignored.

**Confirm that each of the five is fixed in 0.11.19**, not merely silenced:
`cargo audit` exiting 0 with the ignore removed shows that. State it per ID.

### 3.3 Commits

**One commit:** the lockfile update, the removed ignores and the ledger change
together. They are one change: the ignores only become retirable because the
lockfile moved.

## 4. Change scope

- `Cargo.lock`
- `.cargo/audit.toml`
- `rfcs/notes/audit-warning-burn-down.md`

## 5. Non-change scope

- Any `Cargo.toml`. No dependency requirement changes; this is lockfile only.
- reqwest's features. Do not enable or disable `http3`.
- `localcache` 0.27.0 — arama reaches `rusqlite` only through it, so the
  upgrade needs no code; it is not this task.
- RUSTSEC-2026-0441 (`pin-utils`) — T070 §2.6 retires it.
- Any Rust source, `CHANGELOG.md`, any `version` field.

## 6. Documentation updates

The ledger (§3.2). Nothing user-facing.

## 7. Required tests

The gates — `cargo build`, `cargo test`, `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, all `--workspace --locked` — and
`cargo audit --deny warnings`. No new test: no code changes.

## 8. Acceptance criteria

- `cargo audit --deny warnings` exits 0.
- RUSTSEC-2026-0338 to 0342 are absent from `.cargo/audit.toml`, and the
  ledger records them retired.
- No new ignore was added.
- The package list matches §2, or each difference is explained.
- All gates green.

## 9. Prohibited shortcuts

- Ignoring any of the four new IDs instead of updating.
- Keeping the five `quinn-proto` ignores "just in case" — RFC 049 §4 is
  explicit that an unnecessary ignore hides the next advisory on the same crate.
- A broad `cargo update` with no `-p`: it moves crates this task does not need
  to move, and makes the diff unreviewable.

## 10. Module boundaries

None: lockfile and two documents.

## 11. Compatibility constraints

- MSRV: the workspace's `rust-version` is 1.91, and the dry run resolved with
  "highest Rust 1.91 compatible versions". The `MSRV` workflow must stay green.
- **Windows:** removing the `windows-sys` 0.60 and `windows*` 0.53 duplicates
  changes what links on Windows. The `-msvc` job is the only proof; the
  architect watches it.

## 12. Security constraints

- Three of the four are vulnerabilities in crates no shipped binary contains,
  and one is an unsound function arama does not call. The update closes all four
  anyway: a fix that costs nothing is never deferred behind a reachability
  argument.
- Retiring the five ignores **removes** suppression; it adds none.

## 13. Known risks

- **T070 also edits `Cargo.lock` and `.cargo/audit.toml`** (§2.6). This task
  goes first; whichever lands second rebases. Never resolve a lockfile conflict
  by hand: re-run the `cargo update` command on the rebased tree.
- The advisory database grows daily. If `cargo audit` reports something new
  when you run it, **stop and report**; do not widen this task to cover it.

## 14. Required evidence

### 14.1 Evidence plan

No GUI change, so no capture.

| Question | Level | Evidence |
|---|---|---|
| Which packages moved? | machine | the lockfile diff summary |
| Is each of the four new IDs cleared? | machine | `cargo audit --deny warnings` exiting 0 |
| Is each of the five old ignores unnecessary? | machine | the same, with them removed — stated per ID |
| Does the workspace still build and pass? | machine | every gate's output |

### 14.2 Deliverables

The four rows above, observed, and the commit.

## 15. Review-request format

A package under `.git-exclude/review-request/` (working copy, not in the
repository) with a `## Entry point` section: the commit, the `git show`
command, and full repo-relative paths. Report observed output; a check not run
is recorded as not run. You cannot push: report the commit, and the architect
pushes and watches every workflow on that SHA.

## 16. Handoff checklist

The 17 items the architect's operating instructions require of every Developer
Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | Companion line (RFC 049 §3, §4) |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §3 |
| Required tests | §7 |
| Required documentation updates | §6 |
| Acceptance criteria | §8 |
| Prohibited shortcuts | §9 |
| Relevant code or module boundaries | §10 |
| Compatibility constraints | §11 |
| Security constraints | §12 |
| Known risks | §13 |
| Required evidence | §14 |
| Required review-request format | §15 |
