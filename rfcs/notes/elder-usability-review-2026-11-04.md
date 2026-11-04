# Legibility and reach for older users — a measured review

**Date:** 2026-11-04
**Reviewer:** architect
**Asked by:** the owner — view layout and what is visible by default, typography,
and colour, *"especially for elder people."*

Everything below is measured against the code and the shipped tokens, not
recalled. **Two of the three areas asked about are already in good shape**, and
saying so matters as much as the findings: inventing problems would cost effort
that the real gap deserves.

---

## Colour — already strong. No change recommended.

| Role | On background | On surface | AA (4.5) | AAA (7.0) |
|---|---|---|---|---|
| `text_primary` | **17.81:1** | 16.44:1 | pass | pass |
| `text_secondary` | **8.80:1** | 8.12:1 | pass | pass |

Light preset, computed from `snora-design-0.52.0/src/presets/light.rs`. Both
roles clear **AAA**, not merely AA. Two high-contrast presets exist besides, and
snora 0.52 asserts contrast for every toast intent on both rendering paths.

The owner asked specifically about label, button and message backgrounds. Those
are the same two text roles over `surface`/`surface_raised`, and they measure
above. **There is no contrast problem to fix here.**

### The one exception — disabled text

snora 0.52 dims disabled labels and borders to **45 %**. At that factor, on the
light preset:

| Role | Dimmed contrast | AA (4.5) |
|---|---|---|
| `text_primary` | **2.92:1** | fail |
| `text_secondary` | **2.23:1** | fail |

**This is not a WCAG violation** — 1.4.3 exempts inactive components, and that
exemption is deliberate. **But the question asked was about older users, not
conformance**, and 2.23:1 is hard to read for anyone whose contrast sensitivity
has declined.

It matters here specifically because **arama's most meaningful disabled control
is the Similar Pairs button**, and [RFC 047](../proposed/047-indexing-cancellation.md)
is about to make that state carry *more* information — §3.4 makes it mean
"embedding completed", and §7.1 makes it the visible consequence of a partial
index. A control whose state is the message should be readable when it is off.

---

## Target sizes — already fine. No change recommended.

Measured from a real capture (T057's gallery frame, 1560×1020 physical at this
display's 1.2 scale): the active navigation button is **≈52 × 76 logical px**.

That clears WCAG 2.5.8's 24 × 24 minimum and also **2.5.5's enhanced 44 × 44**.
Nothing to do.

---

## Typography — the real gap, and it is one thing

**arama has no text-size setting.** A user may change theme, locale, thumbnail
size, similarity threshold and ffmpeg location. **Not text size.**

And it cannot be worked around by changing theme: no preset file in
`snora-design-0.52.0/src/presets/` mentions `typography` at all, so **all four
share `Typography::default_roles()`** — `body` 16, `body_small` 14, `label` 14,
`title` 18. Light, dark and both high-contrast presets render text at identical
sizes.

**The plumbing already exists and we are not using it.** snora 0.52's own
migration guide states the intent plainly:

> *"the literal sizes ignored your application's text-size setting (orbok, for
> example, offers Default / Large / Larger). They now follow it."*

That release removed hardcoded sizes from chrome precisely so a host
application's setting would reach them. arama sets `default_text_size` once at
startup from `tokens().typography.body.size`
(`app/src/core/settings.rs:16`) — a constant — and offers no way to change it.
**We took the upgrade and left the feature on the table.**

**Recommendation: a text-size setting, three steps, in Settings → General**,
beside the existing theme and language controls. It is the single highest-value
change for the audience asked about, and `14 px` body text — what
`body_small` renders secondary information at today — is small for a reader over
sixty.

---

## Layout — one finding, one correction

### Navigation is icon-only, and the labels already exist

`app/src/core/view.rs:55-82` builds all three navigation buttons as
`button(icon_*())` wrapped in `tooltip(text(t("nav.*")))`. So
`nav.explorer`, `nav.cache` and `nav.settings` **are already translated in both
locales and shown only on hover.**

Hover is the wrong gate for this audience: it needs a steady pointer, it is slow
with tremor, it does not exist on touch, and it is invisible to someone who does
not know to try. Three glyphs with no persistent text is a recognition task
every time.

**Recommendation: show the labels**, either permanently beside the icons or
behind an expand control. The strings, the translations and the i18n keys all
exist — this is layout, not new content.

### What is hidden by default is defensible

`aside_open: false` (`app/src/core.rs:286`) — the tree pane starts closed. That
is the right call under *"less is more"*: the gallery is the point, and the tree
is a secondary navigation aid.

### A correction to my own earlier review

Review 152 said the gallery's thumbnail slider sits *"near its minimum"*. It
does not. `DEFAULT_THUMBNAIL_SIZE` is **224** in a range of **128–384**
(`env/src/media.rs:1-3`) — 37.5 % along, not near the bottom. The 42 % empty
canvas I measured there is real, but attributing it to an unusually small
default was wrong.

---

## Ranked recommendations

1. **A text-size setting** — Default / Large / Larger, in Settings → General.
   The design system already supports it; arama simply does not expose it. By
   far the highest value for older users.
2. **Visible navigation labels** — the strings exist and are hidden behind
   hover.
3. **Reconsider the disabled treatment for the pairs button** — either a gentler
   dim than 45 %, or pair the disabled state with a visible reason, so the
   control's meaning survives its being off.

**Not recommended:** any change to colour tokens or target sizes. Both measure
well, and effort spent there is effort not spent on item 1.
