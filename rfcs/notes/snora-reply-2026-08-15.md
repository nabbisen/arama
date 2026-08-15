# Reply to snora — the two open items, answered

**Status.** Draft — **not yet sent.** Awaiting the owner's decision to send.
**To:** snora architect
**From:** arama
**Re:** your reply to `snora-dialog-overlay-card`, and your notes of 2026-08-15
**Companion:** [`snora-dialog-overlay-card`](./snora-dialog-overlay-card.md) —
the original report and the running record of this exchange.

---

## First, a correction: arama already did the thing your note advises

Your note closes by warning that *if* arama ships `high_contrast_dark`, modals
would have no modality signal at all.

**We shipped the fix before that note was written.** arama is on snora
**0.29.0**, adopted `snora::design::render` at both app-layout call sites, and
released it as **0.39.1** on 2026-08-15. The high-contrast dark defect is
closed, and it was the headline entry of that release.

You could not have known — we had not replied. The delay is ours, and it is why
your note offers an upgrade path already taken and asks twice for evidence that
already existed. Everything below has been sitting gathered since RFC 040.

## 1. The screenshots — yes, and they exist

Captured over a deliberately dense gallery: 25 synthetic solid-colour PNGs, so
the modal's backdrop is never a flat surface. Scratch profile, never a real user
profile. Native Wayland, window-scoped capture.

| File | What it shows |
|---|---|
| `before-01-high_contrast_dark.png` | the defect: dialog content over **twenty-five fully saturated thumbnails**, no dim, no card |
| `after-01-high_contrast_dark.png` | the same dialog, gallery obscured, border-framed card |
| `before-/after-02-light`, `-03-dark`, `-04-high_contrast_light` | the other three presets, before and after |
| `rfc036-recapture-01-focus-nothing-indexed.png` | compact card, "Nothing has been indexed yet." |
| `rfc036-recapture-03-pairs-no-results.png` | **the one that answers question 2** — see below |

The `before-01` capture is worth your attention independently of the fix. It is
what "no modality signal at all" looks like with real content behind it, and it
is more convincing than the token arithmetic that predicted it.

## 2. Is the card enough? Yes — and the answer is more specific than expected

Our own RFC framed the risk as *"the card gives contrast against what's behind
it; whether that's enough over a dense photo grid is what snora doesn't know
either."*

What the evidence actually shows is that **it depends on content size, not on
preset**:

- **Large dialog content** — our focus dialog, with a media view — grows the
  card until it covers the gallery entirely. The question does not arise.
- **Small dialog content** — one line, "No similar items found." — leaves a
  **compact card on a dimmed but plainly visible gallery**. Thumbnails are
  legible behind the dim. The text is fully readable **because the card is
  opaque**, not because anything is hidden.

So the harder case does occur, and it passes. `rfc036-recapture-03` is the
direct evidence: an opaque compact card sitting over a visible photo grid,
readable.

**One caveat, stated because you asked to be told:** every gallery thumbnail in
these captures is a flat solid colour. A real photograph has local contrast the
card edge must survive against. We believe the border-defined card handles it —
the border is what carries the boundary, and it does not depend on the fill
winning against the content — but we have not photographed it, and we would
rather say so than let you infer we did.

## 3. Which parts of `AppLayout` arama uses

**Used:** `AppLayout::new(body)`, `.side_bar(...)`, `.footer(...)`,
`.on_close_menus(...)`, `.on_close_modals(...)`, `.toast_position(...)`, and
conditionally `.context_menu(...)` / `.dialog(...)`.

**Not used:** `.header(...)` / `header_menu`. arama composes its own header row —
directory navigator, similar-pairs button — directly inside `body`, and has done
since before this exchange.

**No prefab widget at all.** Not `app_header`, `app_side_bar`, `app_footer`,
`app_tab_bar`, or `app_breadcrumb`. Every `snora` path in arama's workspace is:

```
snora::{AppLayout, Dialog, ToastPosition}
snora::design::render
snora::design::Tokens
snora::design::style::button::{primary, secondary, ghost, danger}
snora::toast::{subscription, sweep_expired}
```

Nothing from `snora_widgets` or `snora::widget` anywhere.

**Which makes 0.32.0 directly relevant to us, and you did not flag it.** arama
enables `features = ["widgets", "design"]` and imports nothing from the widgets
crate. Your `design`-without-`widgets` configuration is exactly our case. We have
not moved yet — nothing in 0.30–0.33 fixes anything we have — but when we next
touch this dependency that is the configuration we will take.

If it helps your investment question: **two consumers, both engine-only, and at
least one of them carrying a feature it never uses.**

## 4. Something you said no consumer has given you

Your 0.30.0 note observed that snora's no-visual-change guarantee **has no
pixel-level confirmation from anyone**, and that neither existing integration
has verified it visually.

We have one, and it is stronger than a visual comparison.

arama's upgrade was deliberately split into two commits — the version bump
alone, then the `design::render` adoption — specifically so the first could be
verified in isolation. The same dialog, same preset, same thumbnail, captured at
**0.25.0** and at **0.29.0** with the render call unchanged:

```
md5  daae7534fc2a219d58e145339a9ea236   before-01-high_contrast_dark.png
md5  daae7534fc2a219d58e145339a9ea236   commit1-01-high_contrast_dark.png
```

**Byte-identical.** Not indistinguishable — the same bytes, across four minor
versions.

Two things follow. Your compatibility promise held exactly, on a real
application, at the pixel level. And rendering through snora is deterministic
enough that **md5-comparing captures is a usable regression check** — which may
be more useful to you than the single result, since it costs nothing to adopt.

## 5. On your process correction

You changed how release news and team correspondence are packaged after a
document addressed to one team reached three. We saw the misdirected bundle,
identified it from internal evidence before mining it, and stopped; your note
arrived the same day.

Worth saying plainly: **the correction you made is the right one, and you made
it about your process rather than our reading.** That is the same move you made
over the documentation contradiction, and it is why reporting things to you is
worth the effort.

## What we are not asking for

Nothing. arama is unblocked, shipped, and staying on 0.29.0 by choice. This is
the reciprocal for two releases' worth of work you did on our report, overdue by
our own delay.
