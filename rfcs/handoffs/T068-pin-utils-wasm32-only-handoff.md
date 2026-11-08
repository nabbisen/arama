> **AUTHORIZED 2026-11-08 — owner:** *"Authorized."* Written 2026-11-08 by the
> architect.
>
> **Governed by [RFC 049](../proposed/049-advisory-maintenance-policy.md)
> §3** — this task adds an ignore.
>
> **This is the last thing between `main` and a green `Audit`.** T065 and T067
> both landed (`d930756`) and `Audit` fails on exactly one denied warning — this
> one — confirmed in CI run 42473961033 and locally against 1422 advisories.
>
> **Moved 2026-11-08** from `.git-exclude/tasks/dev-team/` into `rfcs/handoffs/`, at the owner's direction, and completed against the 17-item Developer Handoff structure. The authorization above is unchanged.

# T068 Handoff — RUSTSEC-2026-0441, `pin-utils`: wasm32-only

Companion to [RFC 049](../proposed/049-advisory-maintenance-policy.md) — maintenance under its §3 and §6.

Found by the dev team during T065 (review package 158) and correctly left alone
there as out of scope. **Its reachability is already determined** — by the
architect, below — so this task is the ignore and the ledger entry, not an
investigation.

## 1. The finding

> `pin-utils` 0.1.0 — *unmaintained*, RUSTSEC-2026-0441, dated 2026-10-01.

Package 158 traced it with `cargo tree --target all`:

```
pin-utils 0.1.0 <- wasmtimer 0.4.3 <- iced_futures 0.14.0 <- iced <- arama
```

**`--target all` includes a target arama never builds.** Per target, the
architect found:

| Target | `cargo tree -i pin-utils --target <t> -e normal,build` |
|---|---|
| `x86_64-unknown-linux-gnu` | nothing to print |
| `x86_64-pc-windows-msvc` | nothing to print |
| `aarch64-apple-darwin` | nothing to print |
| `wasm32-unknown-unknown` | `pin-utils 0.1.0 <- wasmtimer 0.4.3` |

`iced_futures` 0.14.0 declares it under
`[target.'cfg(target_arch = "wasm32")'.dependencies.wasmtimer]`.

**arama is a desktop application and ships no wasm32 build** — the five release
variants are Linux ×2, Windows ×2 and macOS. So `pin-utils` is in the lockfile
and in **no binary we ship**. The same shape as `quinn-proto` (T059), reached by
a different route: that one was an optional feature not enabled; this one is a
target not built.

**Confirm the per-target result yourself** before relying on it — RFC 049 §3
asks for a determination, and a table in a task is a claim until reproduced.

## 2. What to do

Add `RUSTSEC-2026-0441` to `.cargo/audit.toml`'s `ignore` list **and** a ledger
entry, in the same commit (§3.1).

- **Path:** as above, with the `cfg(target_arch = "wasm32")` gate named.
- **Reachability:** determined — absent from every target arama ships.
- **Revisit condition, observable:** `cargo tree -i pin-utils --target <t>`
  printing a path for any of `x86_64-unknown-linux-gnu`,
  `x86_64-pc-windows-msvc` or `aarch64-apple-darwin` — which would mean either
  arama gained a target or `iced_futures` widened the dependency. **Not "when
  upstream fixes it."**

## 3. Record the class, briefly

The ledger now has two entries that `cargo audit` reports and no shipped binary
contains, for two different reasons — an optional feature not enabled
(`quinn-proto`) and a target not built (`pin-utils`). **Say so in one sentence**
in the new entry, so the next person reading a `cargo tree --target all` result
knows to ask which targets.

## 4. Verification

```sh
cargo audit --deny warnings
```

**Exit 0** — the first time since 2026-11-04. Plus `cargo build`, `cargo test`,
`cargo fmt --check`, `cargo clippy -D warnings`, all `--workspace --locked`.

No lockfile change is expected. If one appears, stop and report.

## 5. Change scope

- `.cargo/audit.toml`
- `rfcs/notes/audit-warning-burn-down.md`

## 6. Non-change scope

- `Cargo.lock`, any Rust source, `CHANGELOG.md`, any `Cargo.toml` version field

## 7. Documentation updates

The ledger entry only (§2, §3). No user-facing documentation.

## 8. Compatibility constraints

No lockfile change is expected. If one appears, stop and report (§4).

## 9. Security constraints

- `pin-utils` is *unmaintained*, not vulnerable, and absent from every binary arama ships. Ignoring it adds no exposure.
- **The revisit condition is what keeps that true**: it fires if a shipped target ever gains the dependency.

## 10. Known risks

- Reading `cargo tree --target all` as if every listed target were built — the error this handoff corrects.
- If arama ever adds a wasm32 target, the ignore becomes wrong silently unless the revisit condition is checked. That is why it is phrased as a per-target `cargo tree` result.

## 11. Acceptance criteria

- `cargo audit --deny warnings` **exits 0**.
- `RUSTSEC-2026-0441` is in `.cargo/audit.toml` **and** the ledger, in the same commit.
- The per-target table in §1 is reproduced.
- No lockfile change.

## 12. Required deliverables

Review request with `## Entry point`, your reproduction of §1's per-target
table, the `.cargo/audit.toml` diff, and `cargo audit --deny warnings` exiting 0.

You cannot push: report the commit and the architect pushes and watches every
workflow on that SHA.

Report observed output; a check not run is recorded as not run.

## 13. Handoff checklist

The 17 items the architect's operating instructions require of every Developer Handoff, and where each is covered here.

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §1 |
| Applicable RFC and requirements | Authorization banner and companion line, §2 |
| Change scope | §5 |
| Explicit non-change scope | §6 |
| Required implementation | §2 |
| Required tests | §4 |
| Required documentation updates | §7 |
| Acceptance criteria | §11 |
| Prohibited shortcuts | §2, §6 |
| Relevant code or module boundaries | §5 |
| Compatibility constraints | §8 |
| Security constraints | §9 |
| Known risks | §10 |
| Required evidence | §1, §4 |
| Required review-request format | §12 |
