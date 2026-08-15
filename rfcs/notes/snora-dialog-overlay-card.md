# Upstream report: snora's dialog overlay draws no card

**Status.** **Sent to snora by the project owner, 2026-08-10.** Retained here as
arama's record of what was reported and why. Any snora-side outcome — a fix, a
documentation change, or a decision that the current behaviour is intended —
should be recorded by appending to this note rather than rewriting it.
**Format.** Downstream bug report, written from arama's evidence.
**Subject.** `snora` 0.25.0, `src/overlay/dialog.rs`.
**Relates to.** arama [RFC 036](../done/036-similarity-dialog-absence-states.md),
whose rendered captures are the evidence for this report.

---

## Summary

`render_dialog` is documented as producing "the centered modal card", but draws
no card — no container, no background, no padding. Dialog content is centred
directly over whatever the application is already rendering, so its legibility
depends on what happens to be behind it.

## The code

`snora-0.25.0/src/overlay/dialog.rs`, in full:

```rust
//! Dialog — the centered modal card.

use iced::{Element, widget::center};
use snora_core::Dialog;

/// Center the dialog content in the window. The surrounding dim layer is
/// owned by [`crate::render::render`].
pub(crate) fn render_dialog<'a, Message>(
    dialog: Dialog<Element<'a, Message>, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    center(dialog.content).into()
}
```

The module doc says *card*. The body centres content and nothing else. The
surrounding dim layer is owned elsewhere and is not in question — the dim
backdrop works.

## Observed effect in a real application

arama is an image and video browser: its dialogs open over a gallery of
thumbnails. Two states from arama's own release evidence:

1. A dialog whose message lands over dark thumbnails — dark text on dark blue,
   red, and green — is difficult to read. The surrounding chrome collides too:
   in one capture a path label overlaps the toolbar and a control label overlaps
   an image.
2. The same dialog, when its message happens to land on empty backdrop, reads
   cleanly.

The difference between the two is **where the application's own content sits**,
not anything the dialog controls. Screenshots available on request; arama can
supply both.

## Why this is worth reporting rather than working around

A consumer can work around it by wrapping every dialog's content in its own
container. arama could do that. But:

- every consumer would have to do it, identically, forever;
- the doc comment already promises the behaviour, so consumers reasonably do
  not check;
- the failure is **content-dependent**, so it survives testing on any
  application whose background is plain. It is invisible until someone puts a
  dialog over a photo grid.

That last point is why arama did not find it for a long time: the defect only
became visible once a dialog was given text to show in a state where it
previously showed nothing.

## Suggested fix

Give the centred content a surface: a container with the design system's
background token, padding, and whatever corner radius Snora Design already
defines for raised surfaces. arama has no opinion on the exact tokens — the
point is that the card the doc describes should exist.

If the current behaviour is deliberate — some consumers may want to compose
their own surface — then a variant, or a documented note that the caller owns
the surface, would resolve the mismatch just as well. **The doc comment and the
behaviour disagreeing is the actual defect**; either half can move.

## What arama is not asking for

- No API break. If a card cannot be added without one, a `Dialog` option is
  fine.
- No change to the dim backdrop, which is correct.
- No urgency. arama has shipped around it and this is not blocking.

## Reply — 2026-08-15, and what it changed

snora upheld the report. Their reply is at
`.git-exclude/tmp/reply-to-report-2026-08-15.md`. Summary of what binds arama:

**The report's premise was wrong, and the correction is against snora.** At
0.25.0 `guides/overlays.md` line 43 said *"`Dialog` does not own the card
chrome… snora is a positioner, not a styler"* — eleven lines below the line 32
"centered modal card" claim. So the behaviour did match a documented contract.
snora recorded this as their documentation contradicting **itself**, in one file,
for four releases, rather than as arama missing the docs. Seven contradiction
sites were corrected in 0.28.1.

**The card already exists.** `snora::design::render(layout, &tokens)`, shipped
**0.27.0** — two minors after the version arama reported against. Token-derived
fill, border, radius; border-defined with no shadow, deliberately, because
shadows carry little information in high-contrast presets.

**Both resolutions this note offered now exist** — the variant *and* the
documentation — and both stated constraints hold: no API break
(`design::render` is a sibling entry point), and no change to the default dim
backdrop.

**A second defect, found while verifying this report:** since 0.27.0 the
identifier `snora-dialog-card` was attached to the centring wrapper — a
window-filling container — rather than to the card. Fixed in 0.29.0. Does not
affect arama at 0.25.0.

### The finding that matters most to arama, verified here

snora's reply flagged, as an aside:

> Before 0.27.0 the modal dim was a hardcoded 40% black, which over a
> pure-black background composites to pure black — invisible.

**This applies to arama today.** Verified at source:

- `snora-0.25.0/src/render.rs:192` paints the dim as
  `Color::from_rgba(0.0, 0.0, 0.0, 0.4)` — hardcoded, and its style closure
  takes `|_theme|`, so it is theme-independent by construction.
- `snora-design-0.25.1/src/presets/high_contrast_dark.rs:7` sets
  `background: Color::rgb(0.0, 0.0, 0.0)` — pure black.
- arama exposes `HighContrastDark` as a user-selectable theme
  (`crates/theme/src/lib.rs:49`).

40% black over pure black is pure black. **For any arama user on the
high-contrast dark theme, opening a modal produces no modality signal at all** —
the dim is invisible and, at 0.25.0, there is no card either. Dialog content
appears floating over an apparently unchanged screen.

This is an accessibility defect in the preset chosen by the users who most need
visual clarity, and it is shipped in 0.39.0. It is carried by
[RFC 040](../done/040-snora-0.29-upgrade-and-dialog-surface.md).

### What snora asked for in return

Recorded so it is not lost:

1. **Screenshots** — before/after over arama's thumbnail gallery. The card has
   almost no downstream exercise; arama would be the first evidence of how it
   reads over arbitrary image content rather than a flat background.
2. **Whether the card is enough.** If dialog text over a photo grid is still
   hard to read with it, snora wants to know.
3. **Which parts of `AppLayout` arama uses and ignores.** Both downstream teams
   so far adopted the engine and none of the prefab widgets.

## Misdirected bundles — 2026-08-15, raised and resolved

Three snora "app team" upgrade bundles arrived
(`app-team-snora-0.25.2-to-0.28.0`, `-0.28.0-to-0.29.0`,
`-0.29.0-to-0.30.0`). **All three were addressed to a different downstream
consumer**, not arama, and carried that team's correspondence — their
post-adoption report, their architecture, their feature requests, and snora's
assessment of them.

Identified from internal evidence before the content was mined: two READMEs
carry an explicit addressee that is not arama; the third targets a team on
`0.25.2` where arama was on `0.25.0`; and the 0.30.0 bundle states "you are on
0.28.0", replies to a report arama never sent, and thanks the recipient for an
`AppLayout` breakdown that was not arama's. "arama" appears in exactly one
file across all three — `contributing/feature-gating-criteria.md`, a general
snora policy document that records both consumers as examples of the same
documentation gap.

**snora confirmed the error the same day**
(`.git-exclude/tmp/note-2026-08-15-disregard-misdirected-bundle.md`) and asked
that the 0.30.0 bundle be disregarded. Their stated cause is worth recording
because it is structural rather than clerical: *"release news and correspondence
with a named team were the same document, which is how a document addressed to
one team became something we sent to three. They are separate artifacts from
now on."*

That note names the 0.30.0 bundle specifically. The other two were addressed to
the same team and should be treated the same way.

**Nothing from those bundles informs any arama decision.** The version facts
below are taken from snora's note to arama, not from them.

## Version position — 2026-08-15

Confirmed by snora directly:

- **153 public items at 0.25.0, 157 at 0.30.0, none removed or renamed.**
- **MSRV unchanged at 1.88** — below arama's declared 1.91.
- **0.30.0 adds one example and no library change**; the diff over
  `crates/*/src/` between 0.29.0 and 0.30.0 is empty.

**arama is on 0.29.0** as of RFC 040 (`e3ab14b`), which snora does not yet know
— nothing has been sent back. The manifest pin is `snora = "0.29"`, which will
not resolve 0.30.0.

**Staying on 0.29.0 is deliberate.** 0.30.0 contains no library change, so
moving would be churn against a dependency we have just verified byte-identical
across an upgrade. A future reader wondering why arama sits a minor behind
current should read this rather than assume neglect.

## Addition to what we owe snora — the byte-identity result

Beyond the three items above, one further piece of evidence is worth sending
because it is stronger than what was promised:

**arama's commit-1 checkpoint was byte-identical across the upgrade.** The same
dialog, on the same preset, captured at `snora 0.25` and at `0.29` with the
render call unchanged, produced screenshots with identical MD5
(`daae7534fc2a219d58e145339a9ea236`). Not "indistinguishable" — the same bytes.

That is a pixel-level confirmation of snora's no-visual-change guarantee across
four minors, which is a stronger claim than the before/after captures alone
support. It is arama's own evidence and stands on its own merits.

## snora 0.31.0–0.33.0 — 2026-08-15, assessed

Two bundles arrived, both **correctly addressed to arama** this time
(`note-arama-0.33.0`, plus a `release-0.33.0` notes document explicitly
identical for all consumers). Addressees checked before content was read, per
the misdirection above.

**snora's assessment: none of the three affects arama.** Verified rather than
accepted:

- **0.31.0** adds `snora::design::responsive_render`. arama does not use
  responsive layout.
- **0.32.0** extracts the token→iced style bridge into a new `snora-style`
  crate, making `design` and `widgets` **independent features**. No public path
  changed; a `widgets` + `design` build is byte-for-byte identical before and
  after.
- **0.33.0** removes the `snora_widgets::design::{style, theme}` compatibility
  re-exports. **arama does not import them** — confirmed by grep: every `snora`
  path arama uses is `snora::design::style::button::*`, `snora::design::Tokens`,
  `snora::design::render`, `snora::toast::*`, and the layout types. Nothing from
  `snora_widgets` or `snora::widget` anywhere in the workspace.

`snora` itself is unchanged across the span — 19 public items at 0.30.0, 19 at
0.33.0 — and MSRV stays 1.88.

### One finding they did not flag

**arama enables `features = ["widgets", "design"]` and imports nothing from the
widgets crate.** Since 0.32.0 that combination is no longer required: `design`
without `widgets` is a newly expressible configuration, added precisely for
consumers in this position.

```toml
snora = { version = "0.33", default-features = false, features = ["design"] }
```

Dropping an entire unused crate should reduce binary size and build time across
all five shipped executable variants. **The saving is unquantified** — snora
measured that widgets+design is unchanged, not what widgets costs — and it
requires moving 0.29 → 0.33 first.

Optional, not urgent, and not proposed as work here. Recorded so it is available
the next time there is reason to touch this dependency.

### Version currency

Where snora's earlier reply says "0.29.0 is current", read **0.33.0**. arama
remains deliberately on **0.29.0**: nothing in 0.30–0.33 fixes anything arama
has, and the only reason to move is the optional feature-drop above.

### The note is written on stale information, and that is arama's doing

It advises arama on an upgrade path already taken, offers screenshots "if the
offer stands", and closes by warning that *if* arama ships `high_contrast_dark`,
modals have no modality signal — a defect arama diagnosed, fixed, and released
in **0.39.1** before this note was written.

snora does not know because **nothing has been sent back**. They have now asked
twice for the two items gathered in package 097, one of which — the first
pixel-level confirmation of their no-visual-change guarantee — they have said no
consumer has ever provided.

## Provenance

Found during arama RFC 036, which changed its similarity dialogs to always
render text explaining an empty result. Before that change those dialogs
rendered nothing at all in that state — so the missing card was invisible,
because there was no content for it to fail to frame. The fix to one exposed
the other.
