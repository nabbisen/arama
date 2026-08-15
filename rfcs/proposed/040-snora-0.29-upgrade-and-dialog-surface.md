# RFC 040: snora 0.29 upgrade and dialog surface

**Status.** Proposed — **accepted for implementation by the project owner
2026-08-15**. Remains in `rfcs/proposed/` until the work ships, per RFC 000.
Design questions 1–3 are settled in the handoff. **Takes precedence over
[RFC 039](./039-windows-path-search-reachability.md)** — this one carries a
defect already shipped to users.
**Tracks.** Two things at once, because one change fixes both: a **live
accessibility defect** — modals have no modality signal at all on arama's
high-contrast dark theme — and [RFC 036](../done/036-similarity-dialog-absence-states.md)'s
recorded limitation, that dialog text is legible only where it lands on neutral
background.
**Touches.** `Cargo.toml` (`snora` 0.25 → 0.29), the call sites that render the
app layout, and rendered evidence. No product logic.

## Summary

arama pins `snora = "0.25"`. Since **0.27.0** snora has offered
`snora::design::render(layout, &tokens)`, which wraps dialog content in a
token-derived card — the exact surface RFC 036 concluded was missing. Since
0.27.0 it also derives the modal dim from the active theme instead of painting a
hardcoded 40% black.

Both matter to arama, and the second is a shipped defect rather than a
limitation.

## The live defect

**On arama's high-contrast dark theme, opening any modal produces no modality
signal whatsoever.** Verified at source:

| Fact | Location |
|---|---|
| The dim is `Color::from_rgba(0.0, 0.0, 0.0, 0.4)`, hardcoded | `snora-0.25.0/src/render.rs:192` |
| Its style closure takes `\|_theme\|` — theme-independent by construction | same |
| `high_contrast_dark.background` is `Color::rgb(0.0, 0.0, 0.0)` | `snora-design-0.25.1/src/presets/high_contrast_dark.rs:7` |
| arama exposes `HighContrastDark` as a user-selectable theme | `crates/theme/src/lib.rs:49` |

40% black composited over pure black is pure black. The backdrop paints and
changes nothing.

At 0.25.0 there is also no card. So a modal on this theme is content appearing
over an apparently unchanged screen — no dim, no frame, no boundary.

**This lands on the preset chosen by the users who most need visual clarity**,
and it shipped in 0.39.0. It was not found by arama; it arrived as an aside in
snora's reply to an unrelated report, and was confirmed here.

## Why now

snora's reply to `snora-dialog-overlay-card` (2026-08-15) upheld that report and
supplied the upgrade path. Their stated position:

- **No breaking changes across 0.25 → 0.29.** Public item sets compared at both
  tags: 153 items at 0.25.0, 157 at 0.29.0, none removed or renamed.
- **One prerequisite: rustc ≥ 1.88.** arama's declared baseline is **1.91**
  (RFC 033), so this is already satisfied.
- **Adoption is per call site.** Switching only the screens that need it is a
  valid end state; untouched screens cannot change appearance, and snora
  enforces that with a test suite that passes unmodified across the span.
- Migration guides exist for each minor.

RFC 036 explicitly deferred this: *"making that text reliably readable is
upstream work in snora."* The upstream work is done and released.

## Goals

- A modal is visibly modal on **every** arama theme, high-contrast dark
  included.
- RFC 036's dialog messages are legible over gallery thumbnails.
- The upgrade is verified by rendered evidence, not by the dependency resolving.
- snora gets the three things they asked for in return.

## Non-goals

- Adopting snora's prefab widgets. arama uses the engine; that stays true unless
  separately proposed.
- Redesigning any dialog's content or layout. This is a surface change.
- Re-opening RFC 036's decisions about which states render which text.
- Any product-logic change.

## Design questions this RFC must settle

### 1. Which call sites adopt `design::render`?

Adoption is per call site, so the choice is real.

**Recommendation: all of them, in one change.** The defect above is not specific
to the similarity dialogs — it affects every modal on the high-contrast dark
theme. Adopting piecemeal would leave arama with two dialog appearances and a
partially-fixed accessibility problem, and "which screens got the card" would
become a thing to remember.

The counter-argument — smaller blast radius — is weakened by snora's guarantee
that untouched screens cannot change and by the fact that a per-site rollout
still has to verify every site eventually.

### 2. Does the upgrade land separately from the adoption?

**Recommendation: yes, two commits.** Upgrade 0.25 → 0.29 with no call-site
change first, and verify nothing moved; then switch the render entry point.

If something does shift on the upgrade alone — despite the no-breaking-change
guarantee — that is far easier to see when it is the only change in the diff.

### 3. What does "verified" mean here?

This is a visual change with an accessibility motive, so gates prove almost
nothing.

**Rendered evidence is the deliverable**, using RFC 036's method. At minimum,
each theme preset — including **high-contrast dark specifically** — with a modal
open over a populated gallery, before and after.

## Testing and verification

- `cargo test --workspace`, `clippy -D warnings` — necessary, not sufficient.
- **Rendered captures per theme preset**, modal open over a thumbnail gallery.
  High-contrast dark is the one that must change; the others must not regress.
- **Re-capture RFC 036's own states** — the `01` and `03` captures where text
  landed unreadably over thumbnails are the direct before/after for the card.
- Confirm no untouched screen changed appearance, per snora's guarantee.

## What we owe snora

Recorded so it is not lost when this ships. They asked for three things:

1. **Before/after screenshots over arama's thumbnail gallery.** The card has
   almost no downstream exercise; arama would be the first evidence of how it
   reads over arbitrary image content rather than a flat background.
2. **Whether the card is enough.** If dialog text over a photo grid is still
   hard to read with it, they want to know — that is a snora problem.
3. **Which parts of `AppLayout` arama uses and ignores.** Both downstream teams
   so far adopted the engine and none of the prefab widgets, and it is changing
   where they invest.

(1) and (2) fall out of this RFC's own verification at no extra cost. (3) is a
paragraph. **This should be sent, not merely gathered** — they acted on arama's
report across two releases and credited it; the reciprocal is cheap.

## Risks

- **A four-minor dependency jump.** Mitigated by the no-breaking-change
  comparison, arama's baseline already exceeding the rustc floor, and question
  2's split.
- **The card may not be sufficient.** It gives a surface with contrast against
  what is behind it; whether that is enough over a dense photo grid is exactly
  what snora does not know either. If it is not, this RFC has still fixed the
  modality defect and produced the evidence snora asked for.
- **Theme regressions on presets that were fine.** Question 3's per-preset
  captures are the guard.
- **Scope creep into adopting snora's widgets.** Fenced in Non-goals.

## Open questions

- Should the high-contrast-dark modality defect be called out in the CHANGELOG
  as a fix in its own right? Recommendation: **yes, separately from the card.**
  Users on that preset experienced a specific, describable problem, and burying
  it inside a dependency-upgrade line would understate it.
- Does this warrant a release of its own, or ride the next cut? It is an
  accessibility fix affecting a shipped preset, which argues for not sitting on
  it long.
