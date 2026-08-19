# The thumbnail-size slider writes `settings.json` on every step

**Found:** 2026-08-20, by the dev team, while diagnosing a corrupted capture
pass during Task 028. Not looked for.
**Status:** recorded, not scheduled. Small, real, and it has already cost
something.

## What happens

`crates/ui/layout/src/footer/thumbnail_size_slider/view.rs:17-23` binds the
slider to `Message::ValueChanged` with **no `.on_release`**, so iced emits a
message on every drag increment. That routes through
`crates/ui/layout/src/footer/update.rs:17-22` and
`app/src/core/update/component.rs` to `app/src/core.rs:315-318`:

```rust
fn thumbnail_size_update(&mut self, thumbnail_size: u16) {
    self.settings.thumbnail_size = thumbnail_size;
    self.save_settings();
}
```

`save_settings` (`app/src/core.rs:279`) serialises the whole `Settings` struct
and writes it to disk synchronously through `app-json-settings`. **There is no
debounce anywhere in the chain.**

The range is 128–384 with `SLIDER_STEP = 32` (`env/src/media.rs:2-3`;
`thumbnail_size_slider.rs:5`), so a full drag is **eight** serialise-and-write
cycles, on the UI thread.

## Why it is worth recording rather than shrugging at

Eight small writes is not a performance problem and nobody has reported one.
What makes it worth a note is the **failure it enabled**, which was neither
small nor visible:

During Task 028's capture work a misdirected click landed near the footer of an
unexpectedly-small window. It moved the slider. `settings.json` was rewritten
immediately. Every subsequent launch in that pass patched only `theme`, so the
altered `thumbnail_size` **carried forward through all four presets**, and the
whole 16-capture pass was silently wrong — a grid of two giant thumbnails
instead of twenty-seven small ones.

That is the shape this project keeps meeting: not an error, but a plausible
wrong result. A settings write triggered by a transient interaction is
persistent state created by an accident.

## What a fix probably looks like

`.on_release` on the slider, so the value updates live in the UI but is
persisted once when the drag ends. iced supports it directly; it is close to a
one-line change plus a message.

**Check before assuming that is enough:** the same shape may exist on other
controls that write settings from a continuous interaction. The similarity
threshold slider is the obvious candidate and was not examined.

## Not doing it now, and why

Nothing user-facing is broken — a user dragging the slider gets the size they
chose, saved. The cost falls on automated capture work, which now guards against
it by verifying window geometry before every click.

Worth folding into whatever next touches the footer, rather than interrupting
the queue for it.
