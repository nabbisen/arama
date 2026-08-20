# What actually makes arama's dialog card edge legible

**Measured:** 2026-08-20, from Task 028's Similar Pairs captures at snora
0.38.0, over a real 27-photo gallery, `light` and `dark`.
**Why it exists:** snora asked a question their own test suite structurally
cannot answer, and this is the answer. It is also a correction to two earlier
attempts that measured the wrong pixels.

## The question, as snora put it

> Our contrast assertions measure the border against snora's own surfaces, not
> against whatever image sits behind the modal dim. If it still disappears over
> bright imagery, we want to know.

Specifically: **is a 3.1:1 border enough over photographic content?**

## The measurement

Border pixels sampled where the card edge meets the dim with no photo between
them (`x=1200, y=0` — the card spans the full height, so its top edge lies on
the screen edge). Dim values derived from `DIM_ALPHA = 0.44` and confirmed
against the screen edge at `x=0`.

| | border ǀ surface | **border ǀ dim** | dim ǀ surface |
|---|---|---|---|
| **light**, before snora 0.34.0 | 1.39:1 | 2.49:1 | — |
| **light**, at 0.38.0 | **3.38:1** | **1.02:1** | **3.46:1** |
| **dark**, before 0.34.0 | 1.19:1 | 3.27:1 | — |
| **dark**, at 0.38.0 | **3.17:1** | **1.23:1** | **3.89:1** |

WCAG relative luminance, computed from measured pixels. The
`border ǀ surface` column lands on snora's own published figures, which is the
check that the method is right.

## What it means

**The 1 px border is invisible against the dim** — 1.02:1 and 1.23:1. It is not
what outlines the card and cannot be.

**The dim-to-surface step is what carries the edge** — 3.46:1 and 3.89:1, both
clearing SC 1.4.11's 3:1 minimum for a boundary that identifies a component.

**This confirms snora's own decision.** Their 0.37.0 note recorded that `light`
failed *"by either its border or its fill"* at 1.19 and 2.85, and they repaired
it by strengthening the dim — the **fill** route, not the border route. That
route now passes at 3.46:1. The border route is still ~1:1 and always was.

**So snora's 0.34.0 border repair does its work at the card's *inner* edge**,
against the card's own surface, where a bordered container meets its own
content. Not at the outline. That is not where a reader would assume, and it is
worth knowing before anyone treats "border contrast repaired" as "the card
outline is now stronger".

## Confirmed and generalised upstream, 2026-08-20

snora re-derived the figure before believing it — the dim over light content
composites to `0.56 × content`, and `border` is `0.537`, so from pure white it
is **1.04:1** against our measured 1.02. **Arithmetic, not an artefact of one
gallery.**

They then swept the full greyscale content range for all four presets:

| preset | `border ǀ dim` minimum | `dim ǀ fill` range |
|---|---|---|
| `light` | **1.00** | 3.24 – 21.00 |
| `dark` | **1.00** | 3.16 – 15.61 |
| `high_contrast_light` | **1.00** | 3.24 – 21.00 |
| `high_contrast_dark` | **1.00** | 4.01 – 19.80 |

**The border reaches 1.00:1 against the dim in every preset** — only the content
that does it differs. They expected the high-contrast presets to invert the
finding; they do not.

**They withdrew the rationale, not the repair.** Their RFC-058 had justified the
border change with *"a border is that boundary for the dialog card"* — against
the dim it is not. The repair stays necessary because it works at the card's
inner edge (3.38:1 and 3.17:1 against fill), which is where it always did.
**Nothing in the palette changed:** no value, no `DIM_ALPHA`, no assertion. Only
the explanation was wrong.

*Their sheet surface, measured because arama's report prompted it, is not
token-styled at all — it takes iced's `extended_palette()`, so its border does
1.02–1.35:1 against its own fill and is carried entirely by the fill-versus-dim
route. arama uses no sheets; recorded because it is the same finding one surface
over.*

## A detail that dissolves the original question

**No arama dialog puts the border directly against raw photo pixels.** Both
dialogs pad their content, so the order is always *dim → border → card surface →
padding → image*. The literal scenario snora described — a border stroke with a
bright photograph immediately on one side — does not occur in arama's UI.

## Two earlier attempts that measured the wrong thing

Recorded because both looked conclusive and neither was.

1. **Media Focus dialog** (Task 028's first pass). Both samples were *inside*
   the card. That dialog's content is `Fill`-sized, so its card covers the
   viewport at any window size and there is no visible dim at all — see
   [`dialog-card-fills-the-viewport`](dialog-card-fills-the-viewport.md).
2. **Similar Pairs, first reading.** The sampled "boundary" was the dimmed page
   background — `0.56 × #F4F6F8 = #898A8B` in light — about 140 px from the
   real card edge. The same value appears at `x=0` and `x=2559`, which is the
   tell.

**Both produced plausible prose from pixels that were not the thing being
measured.** The guard that would have caught either is cheap: derive what the
value *should* be from the known constants first, then check the pixel matches.
The dim is computable from `DIM_ALPHA` and the background; the border is a
published token value. Neither needs to be discovered by looking.
