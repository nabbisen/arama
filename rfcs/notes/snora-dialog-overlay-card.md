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

## Provenance

Found during arama RFC 036, which changed its similarity dialogs to always
render text explaining an empty result. Before that change those dialogs
rendered nothing at all in that state — so the missing card was invisible,
because there was no content for it to fail to frame. The fix to one exposed
the other.
