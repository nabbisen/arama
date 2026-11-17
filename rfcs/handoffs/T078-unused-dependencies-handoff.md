> **ACCEPTED 2026-11-17 — review 181.**

> **AUTHORIZED 2026-11-17 — owner:** *"The three are approved."* (T077, T078,
> T079). Written 2026-11-17 by the architect.
>
> **Independent of every open task.**

# T078 Handoff — remove the four unused dependencies

## 1. Purpose

Cargo reports four dependencies that the crate declaring them never uses. Each one
is code compiled and audited for nothing, plus an edge that keeps a crate in the
graph without a reason. Remove them, and prove each was really unused.

## 2. Background

`cargo check --workspace --all-targets --locked` (Cargo's `unused_dependencies`
lint), observed 2026-11-17:

```text
unused dependency `swdir`             @ crates/ai/Cargo.toml:41
unused dependency `anyhow`            @ crates/cache/Cargo.toml:41
unused dependency `arama-ui-widgets`  @ crates/ui/layout/Cargo.toml:20
unused dependency `arama-ui-widgets`  @ crates/ui/main/Cargo.toml:20
```

Recorded since review 163 (2026-11-14) and noted in several packages as
*"pre-existing, out of scope"*.

## 3. Applicable RFC and requirements

[RFC 049](../proposed/049-advisory-maintenance-policy.md) (a smaller graph is
fewer advisories to track). No new design.

## 4. Change scope

- `crates/ai/Cargo.toml`, `crates/cache/Cargo.toml`,
  `crates/ui/layout/Cargo.toml`, `crates/ui/main/Cargo.toml`
- `Cargo.lock`

## 5. Explicit non-change scope

- Any `.rs` file. If removing a dependency breaks a build, the lint was wrong
  for that one: **put it back and report it**; do not rewrite code to free it.
- Any version, feature, or other dependency.
- The workspace `[workspace.dependencies]` table — unless an entry becomes unused
  by every member, in which case remove it too and say so.

## 6. Required implementation

For **each** of the four, before removing it, show it is unused in every place
Cargo compiles for that crate:

- `grep` of the crate's `src/`, `tests/`, `benches/`, `examples/` and `build.rs`
  for its Rust name (`swdir`, `anyhow`, `arama_ui_widgets`) — `use`, paths and
  macros;
- no `[features]` entry of that crate forwards to it (`dep:` or `name/feature`);
- no target-specific section (`[target.'cfg(…)'.dependencies]`) depends on it.

Then remove the line, and run the gates.

## 7. Required tests

- `cargo check --workspace --all-targets --locked` prints **no**
  `unused dependency` warning.
- `cargo build`, `cargo test --workspace`, `cargo clippy --all-targets -D
  warnings`, `cargo fmt --check`, all `--locked` where they take it;
  `cargo audit --deny warnings`.
- **The lockfile diff, by name/version pair:** removed edges are expected. If a
  **package** leaves the graph entirely (for example, `swdir` if nothing else
  uses it), list it. **If any package version moves, stop and report** — a
  removal must not update anything.

## 8. Required documentation updates

None.

## 9. Acceptance criteria

- The four lines removed; each shown unused first (§6).
- No `unused dependency` warning remains.
- The lockfile changes only by removed edges and, if any, removed packages,
  listed.
- All gates green.

## 10. Prohibited shortcuts

- Silencing the lint (`[lints]`, `cargo:` flags, `#[allow]`) instead of removing
  the dependency.
- Removing a dependency the grep shows in use, because "the lint said so".
- `cargo update` of anything.

## 11. Module boundaries

Four manifests and the lockfile.

## 12. Compatibility constraints

No behaviour change, no MSRV change. The `MSRV` workflow must stay green.

## 13. Security constraints

None beyond §7's audit. A package that leaves the graph is one fewer to audit.

## 14. Known risks

- **The lint is new to Cargo** and may misjudge a dependency used only through a
  macro or a feature. §6's checks exist for that; §5 says what to do.
- **`arama-ui-widgets` in two UI crates** may be wanted again by a near-future
  UI task. Removing an unused edge is still right; adding it back is one line.

## 15. Required evidence

### 15.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Was each truly unused? | source | §6's three checks, per dependency |
| Is the warning gone? | machine | `cargo check` output |
| Did anything else move? | machine | the lockfile diff by name/version pair |
| Does everything still build and pass? | machine | the gates |

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
