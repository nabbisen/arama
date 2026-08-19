# A large image makes the media dialog cover the whole viewport

**Found:** 2026-08-20, while reviewing Task 028's captures. The finding is
incidental to that work and independent of it.
**Status:** recorded, not scheduled. Needs one measurement before it can be
called a defect.

## What was observed

In every Task 028 capture with the Media Focus dialog open over a gallery of
real photographs, the dialog card **spans the entire screen**. Its border sits
on the screen edge.

Row scan at `y=700` of `after-0.38/light-dialog-bright.png`:

```text
x=0     #898C8F   ← the dialog card's border
x=2..   #FFFFFF   ← the card's own surface
x=2559  #898C8F   ← the border again
```

The same holds in `dark` (`#69717D` at `x=0`, `#1F242B` surface) and vertically
— a sample at `y=1430` is still card surface.

**The card sizes to its content**, and the fixture images are full-resolution
phone photographs.

## Why it may matter

RFC 036 gave the similarity dialogs their absence states. Their text was legible
only where it happened to land on neutral background, because snora's overlay
drew no card — reported upstream, fixed upstream, and adopted in RFC 040.
snora then went further: 0.34.0 raised the card border's contrast and 0.37.0
strengthened the modal dim, both because a modal must be distinguishable from
what is behind it.

**If the card covers everything, none of that carries information.** There is
nothing behind to be dimmed, and no edge to see the border against. That is
close to the defect RFC 040 fixed — *"a modal has no modality signal at all"* —
arriving by a different route: not because the dim is invisible, but because
there is no visible backdrop for it to act on.

It also means arama currently **cannot answer the question snora asked** — does
a 3.1:1 border read over photographic content — because in these captures there
is no photographic content visible outside the card.

## What is not yet known, and it is the whole question

**Whether this happens at ordinary window sizes with ordinary images.** The
captures were taken fullscreen at 2560×1440 with large photographs. Both of
those push the card outward, and neither is necessarily what a user has.

Task 028's §5 retake — the same two captures at a smaller window — will show
this directly, at no extra cost. **Wait for it.** If the dim is visible there,
this is an artifact of the capture setup and the note can be closed. If the card
still fills the viewport, it is a real sizing question worth an RFC.

## What not to do yet

Do not cap the dialog's size, or change how the image is fitted, on the strength
of this note. Both would be reasonable-sounding fixes to a problem that has not
been shown to affect a user, and the measurement that settles it is already
scheduled.
