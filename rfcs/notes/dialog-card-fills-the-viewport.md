# The media dialog's card fills the viewport, by arama's own choice

**Found:** 2026-08-20, while reviewing Task 028's captures. Cause established
from source the same day, after a first pass that guessed at it.
**Status:** understood, not a defect, and not scheduled. Recorded because it
changes what evidence a dialog capture can produce.

## What happens

With the Media Focus dialog open, the dialog card **spans the entire screen**
and its border sits on the screen edge. Row scan at `y=700` of a light-preset
capture:

```text
x=0     #898C8F   ← the dialog card's border
x=2..   #FFFFFF   ← the card's own surface
x=2559  #898C8F   ← the border again
```

Same in `dark` (`#69717D` border, `#1F242B` surface), and vertically too.

## Why — and it is neither the window size nor the image

snora sizes the card **from its content**
(`snora-0.38.0/src/overlay/dialog.rs:39-46`): `center(container(dialog.content)
.padding(...))`. A `container` with no explicit width shrinks to its child, so
the card is exactly as large as the application makes its content.

arama's content asks for all of it —
`crates/ui/widgets/src/dialog/media_focus_dialog/view.rs:33-47`:

```rust
scrollable(container(img).width(Fill).center(Fill))
    .width(Fill)
    .height(Fill)
```

`Fill` expands to the full available space, so the card covers the viewport at
**any** window size and for **any** image.

The first version of this note said a smaller window would settle whether this
was an artifact. It would not have — that guess is corrected here rather than
quietly replaced.

## This is not a defect

A full-viewport image viewer is a reasonable design for looking at a photograph,
and nothing about it is broken.

**What it does mean:** snora's card, border and dim contribute **nothing** to
the Media Focus dialog. There is no visible backdrop for the dim to act on and
no field for the border to be seen against. arama adopted RFC 040's card and
0.34.0's border repair and 0.37.0's stronger dim, and **none of them changes
what this dialog looks like.**

Worth holding deliberately rather than assuming those upstream repairs improved
every modal.

## The consequence that actually cost something

Task 028 §5 asks — because snora asked, and their own suite structurally cannot
— whether a 3.1:1 border reads over photographic content. That is a question
about a card **with a dimmed gallery behind it**.

**The Media Focus dialog cannot answer it**, and a full capture pass was spent
before anyone noticed. The Similar Pairs dialog can: it sets no `Fill` and its
thumbnails are fixed at `MAX_THUMBNAIL_SIZE`, so its card is content-sized and
the dim stays visible around it — and it is the dialog RFC 036 and RFC 040 were
actually about.

## Left open

Whether any other arama dialog is greedy in the same way. Only two exist today
(`MediaFocusDialog`, `SimilarPairsDialog`) and the settings surface is a page
now, so the answer is currently "no" — but the next dialog added will not
inherit that automatically, and nothing checks it.
