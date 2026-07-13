# RFC 027 - Audit warning ledger refresh

**Status.** Implemented (Unreleased)
**Tracks.** Release-gate hygiene follow-up: reconcile the audit-warning ledger
and release-gate wording with the current `cargo audit` output.
**Touches.** `rfcs/notes/audit-warning-burn-down.md`,
`docs/src/dev/testing.md`, `docs/src/dev/release.md`, `ROADMAP.md`,
`rfcs/README.md`.

## Summary

At proposal time, the `cargo audit` gate passed, but the recorded warning
ledger was stale against the latest observed output.

On 2026-07-13, `cargo audit` reported no blocking vulnerabilities and four
allowed warnings:

```text
bincode   2.0.1  RUSTSEC-2025-0141  unmaintained
paste     1.0.15 RUSTSEC-2024-0436  unmaintained
rustybuzz 0.20.1 RUSTSEC-2026-0206  unmaintained
ttf-parser 0.25.1 RUSTSEC-2026-0192 unmaintained
```

The existing audit note already tracked `bincode`, `paste`, and `ttf-parser`,
but it did not yet track the newer `rustybuzz` warning. It also had
proposal-era/count drift: one paragraph said five allowed warnings from an
earlier pass, then later paragraphs described three.

This RFC records a documentation-only reconciliation:

1. Update the audit-warning ledger to state the current four allowed warnings.
2. Add `rustybuzz` ownership details through the `usvg`/`resvg`/iced rendering
   path.
3. Keep `bincode`, `paste`, and `ttf-parser` owner notes current with the
   observed tree paths.
4. Clarify release/testing docs so reviewers check both explicit audit ignores
   and allowed warnings with recorded rationale.
5. Do not change dependencies, add new audit ignores, or claim any warning is
   resolved.

## Why

The release gate is only useful if its exceptions and allowed warnings are
understandable to reviewers. `cargo audit` currently exits successfully, but
the human-facing ledger should match what maintainers see when they run it.

The distinction matters:

- `.cargo/audit.toml` contains explicit ignored advisories for scoped
  quick-xml/Wayland issues.
- `cargo audit` also reports unmaintained-crate warnings that are allowed by
  policy unless the project chooses to deny them.
- Those allowed warnings still need owner paths, rationale, and revisit
  conditions so "audit passed" does not hide dependency risk.

## Observed dependency facts

On 2026-07-13:

- `cargo audit` passed and reported four allowed warnings: `bincode`, `paste`,
  `rustybuzz`, and `ttf-parser`.
- `cargo info localcache` reported `localcache` 0.20.0 as the current published
  version; it still owns the `bincode` 2.0.1 path through `arama-cache`.
- `cargo info bincode` reported the locked 2.0.1 line and a newer 3.0.0 line,
  but arama reaches `bincode` through `localcache`, not a direct workspace
  dependency.
- `cargo info paste`, `cargo info rustybuzz`, and `cargo info ttf-parser`
  reported the same locked versions as the audit output.
- `cargo tree --target all -i bincode@2.0.1` showed:
  `bincode <- localcache <- arama-cache`.
- `cargo tree --target all -i rustybuzz@0.20.1` showed:
  `rustybuzz <- usvg <- resvg <- iced_tiny_skia/iced_wgpu <- iced`.
- `cargo tree --target all -i ttf-parser@0.25.1` showed font/rendering paths
  through `fontdb`, `cosmic-text`, `owned_ttf_parser`, `ab_glyph`, `usvg`,
  `resvg`, and `rustybuzz`.
- `cargo tree --target all -i paste@1.0.15` still showed Candle/gemm,
  tokenizers, and target-qualified rendering owners.

These facts should be rechecked during implementation because registry and
advisory state can change.

## Proposal

### Part A - Refresh the audit note

Update `rfcs/notes/audit-warning-burn-down.md` to:

- state that the current observed surface is four allowed warnings;
- remove or rewrite stale count statements from earlier passes;
- add a `rustybuzz` section with owner path and revisit condition;
- keep `ttf-parser` as a separate font-stack warning because it is reported
  independently by `cargo audit`;
- keep `bincode` and `paste` unresolved, with owner paths and revisit
  conditions.

### Part B - Clarify release-gate wording

Update `docs/src/dev/testing.md` and `docs/src/dev/release.md` so release-gate
instructions distinguish:

- explicit ignored advisories in `.cargo/audit.toml`;
- allowed audit warnings printed by `cargo audit`;
- the requirement that both have recorded rationale and revisit conditions.

This should be wording-only. It must not make the owner-managed release flow
stricter than the actual gate.

### Part C - Keep policy stable

Do not add new audit ignores for the four warnings. If the project later wants
to deny warnings or replace one of these dependency paths, that should be a
separate RFC with implementation impact.

## Non-goals

- No dependency update.
- No cache-engine replacement.
- No rendering stack replacement.
- No new `.cargo/audit.toml` ignore.
- No `cargo audit --deny warnings` policy change.
- No release action, version bump, tag, publish, or push.

## Risks

- Advisory state may change between proposal and implementation. Mitigation:
  rerun `cargo audit` and the targeted `cargo info`/`cargo tree` checks during
  implementation.
- The docs could imply that allowed warnings are harmless. Mitigation: describe
  them as accepted risk with owner path and revisit condition.
- The docs could imply release blockers beyond current policy. Mitigation: keep
  the release-gate wording descriptive and owner-reviewed, not stricter than
  the command behavior.

## Acceptance criteria

- `rfcs/notes/audit-warning-burn-down.md` matches the current `cargo audit`
  warning list and includes `rustybuzz`.
- `docs/src/dev/testing.md` and `docs/src/dev/release.md` distinguish ignored
  advisories from allowed warnings.
- No dependency, source-code, or audit-ignore policy change is included.
- Documentation builds cleanly.

## Review evidence

Required for proposal review:

```sh
cargo audit
cargo info localcache
cargo info bincode
cargo info paste
cargo info rustybuzz
cargo info ttf-parser
cargo tree --target all -i bincode@2.0.1
cargo tree --target all -i paste@1.0.15
cargo tree --target all -i rustybuzz@0.20.1
cargo tree --target all -i ttf-parser@0.25.1
mdbook build docs
git diff --check
```

Implementation should rerun the same evidence commands and call out any changed
advisory or registry state.

## Implementation notes

The implementation refreshed `rfcs/notes/audit-warning-burn-down.md` to record
the current four allowed warnings and added a `rustybuzz` owner section. It also
updated release/testing documentation to distinguish explicit ignored advisories
from allowed warnings with recorded dependency paths, rationale, and revisit
conditions. No dependency, source-code, `.cargo/audit.toml`, release, version,
tag, publish, or push change is included.
