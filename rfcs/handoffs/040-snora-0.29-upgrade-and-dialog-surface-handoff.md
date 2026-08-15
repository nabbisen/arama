# RFC 040 Handoff — snora 0.29 upgrade and dialog surface

Companion to [RFC 040](../done/040-snora-0.29-upgrade-and-dialog-surface.md),
shipped in **0.39.1** and moved to `rfcs/done/` with that cut, per
[RFC 000](../done/000-rfc-lifecycle-policy.md).

**Do this before RFC 039.** It carries a defect already shipped to users.

## 1. Design authority

1. [RFC 040](../done/040-snora-0.29-upgrade-and-dialog-surface.md);
2. [`snora-dialog-overlay-card`](../notes/snora-dialog-overlay-card.md) — the
   report, snora's reply, and the verification of the defect;
3. [RFC 036](../done/036-similarity-dialog-absence-states.md) — the limitation
   this closes. **Its decisions are not reopened.**

## 2. What is actually broken

Not the card. **On the high-contrast dark theme a modal has no modality signal
at all.**

`snora 0.25` paints the dim as `Color::from_rgba(0.0, 0.0, 0.0, 0.4)` with a
`|_theme|` closure — theme-independent by construction — and
`high_contrast_dark.background` is `Color::rgb(0.0, 0.0, 0.0)`. 40% black over
pure black is pure black. With no card either, the dialog is content over an
apparently unchanged screen.

**Verify this yourself before changing anything.** Capture a modal on
high-contrast dark at the current pin. That capture is the "before" half of the
evidence and the proof the defect is real rather than inferred from token
values.

## 3. Two commits, in this order

**3.1 Upgrade only.** `snora` 0.25 → 0.29 in `Cargo.toml`, lockfile refreshed.
**No call-site change.** Run the gates and capture a modal per theme preset.

snora states no breaking changes across the span (153 → 157 public items, none
removed or renamed) and that untouched screens cannot change appearance. **Treat
that as a claim to verify, not a guarantee to rely on.** If anything moves here,
it is far easier to see when it is the only change in the diff.

The rustc floor is ≥ 1.88; arama declares 1.91, so nothing to do — but confirm
rather than assume.

**3.2 Adopt the surface.** Switch the app-layout render call from
`snora::render` to `snora::design::render(layout, &tokens)`.

**All call sites, in one change** (design question 1). The defect is not
specific to the similarity dialogs — it affects every modal on that preset.
Adopting piecemeal would leave two dialog appearances and a partially-fixed
accessibility problem.

Migration guides exist per minor: `migration-0.25-to-0.26` through
`-0.28-to-0.29` in snora's `docs/src/guides/`. Read them; do not infer the API
from the type signatures alone.

## 4. Non-change scope

- **snora's prefab widgets.** arama uses the engine. That stays true here.
- **Any dialog's content or layout.** This is a surface change.
- **RFC 036's decisions** about which states render which text.
- Product logic, discovery, cache, i18n strings.
- RFC 039. Separate, and it follows this.

## 5. Verification — gates prove almost nothing here

This is a visual change with an accessibility motive. `cargo test` and `clippy`
are necessary and nowhere near sufficient.

**Rendered captures are the deliverable**, using RFC 036's method:
scratch-profile isolation under `.git-exclude/tmp/`, never the owner's real
profile, native Wayland window-scoped capture.

Required, **before and after**, with a modal open over a populated thumbnail
gallery:

| Preset | Expectation |
|---|---|
| **High-contrast dark** | **must change** — this is the defect |
| High-contrast light | must not regress |
| Light | must not regress |
| Dark | must not regress |

Also re-capture RFC 036's own two states — the captures where absence text
landed unreadably over thumbnails. Those are the direct before/after for the
card and are owed to snora regardless.

**If the card does not make the text readable over a dense photo grid, say so.**
That is a real result, not a failure of this work, and snora explicitly asked to
be told. Do not quietly declare success because the card is present.

## 6. What we owe snora — gather it, do not send it

They acted on arama's report across two releases and credited it. The reciprocal
is cheap and is part of this task's deliverable:

1. **Before/after screenshots over the thumbnail gallery** — falls out of §5 at
   no extra cost.
2. **Whether the card is enough** — your honest read from those captures.
3. **Which parts of `AppLayout` arama uses and ignores** — a paragraph. Both
   downstream teams so far adopted the engine and none of the prefab widgets,
   and it is changing where they invest.

Collect these into the review package. **Do not send anything to snora** —
outward communication is the owner's, as this report itself was.

## 7. Acceptance criteria

- A modal is visibly modal on **every** preset, high-contrast dark included,
  proven by before/after captures.
- No untouched screen changed appearance.
- RFC 036's absence text is legible over gallery thumbnails — or the finding
  that it still is not, stated plainly.
- Upgrade and adoption are separate commits.
- Gates clean.
- The three items in §6 gathered.

## 8. Known risks

- **A four-minor dependency jump.** Mitigated by §3.1's split and by snora's
  item-set comparison — which you should spot-check, not take on faith.
- **The card may not be sufficient.** If so, this work has still fixed the
  modality defect and produced the evidence snora needs. Report it.
- **Theme regressions on presets that were fine.** §5's per-preset captures are
  the guard, which is why all four are required and not just the broken one.

## 9. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include both commits, all before/after captures,
and §6's three items. Report observed output; a check not run is recorded as not
run.
