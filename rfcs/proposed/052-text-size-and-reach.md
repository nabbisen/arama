# RFC 052: A text-size setting, and reach for older readers

**Status:** **Accepted by the project owner, 2026-11-04** (*"Yes. Authorized.
Proceed on the improvements and enhancement."*)
**Raised by:** the owner, 2026-11-04, asking whether layout, typography and
colour could be improved *"especially for elder people"*
**Evidence:** [`elder-usability-review-2026-11-04`](../notes/elder-usability-review-2026-11-04.md)
**Relates to:** [RFC 043](../done/043-typography-roles-and-prose-readability.md),
[RFC 011](../done/011-theme-setting.md),
[RFC 047](./047-indexing-cancellation.md)

## Summary

The review measured all three areas the owner asked about. **Colour and target
sizes need nothing** — `text_primary` 17.81:1 and `text_secondary` 8.80:1 both
clear AAA, and the navigation button measures ≈52 × 76 logical px against a
44 × 44 enhanced threshold. Effort spent there is effort taken from the one real
gap.

**arama has no text-size setting**, and no theme preset can substitute: not one
preset file overrides `Typography::default_roles()`, so all four render text at
identical sizes. snora 0.52 removed hardcoded chrome sizes precisely so a host
application's setting would reach them — arama took that upgrade in T052 and
left the capability unused.

This RFC adds the setting, plus two smaller reach improvements.

## 1. The constraint that shapes the design

Theme switching already works at runtime: `set_theme` is a global the lookup
consults (`app/src/core/update/component.rs:312`), the same pattern as the i18n
locale. **Text size cannot simply copy it**, because of how text sizes reach the
screen by two different routes:

| Route | Who reads it | Changeable at runtime? |
|---|---|---|
| `iced::Settings.default_text_size` | arama's own unannotated `text(...)` sites | **No** — read once at startup |
| `arama_theme::tokens()` | snora's chrome on the `design` path, and the 2 annotated sites | Yes, like `set_theme` |

Measured: **118 of 120 non-test `text(` sites inherit `default_text_size`.** Only
2 carry an explicit size.

So scaling tokens at runtime would resize snora's chrome and 2 call sites while
leaving 118 at 16 px. **That is worse than not scaling at all** — a half-scaled
interface is harder to read than a small consistent one, and it is exactly the
kind of self-inconsistency the owner's second principle rules out.

## 2. Design

**2.1 Scale the whole token set once, at startup.** A `text_scale` setting feeds
both routes from one source: `iced::Settings.default_text_size` and the token
set arama hands snora. Everything scales together — arama's 118 sites and
snora's chrome — because both derive from the same scaled tokens.

**Three steps, named for what they do**, matching snora's own cited precedent
(*"orbok, for example, offers Default / Large / Larger"*): **Default / Large /
Larger**. Not a free-form number — three named choices are easier to reason
about and bound what the layout must survive.

**2.2 It requires a restart, and must say so.** `Settings` is read once at
application start (`app/src/core/settings.rs:9`, RFC 043 §3.1's own note). A
setting that silently does nothing until restart is precisely the confusion the
owner's second principle forbids, so **the control states it in both locales**,
visibly, at the point of choosing.

**2.3 The layout must survive Larger.** Scaling text in a window that does not
grow is where text clips — the defect [RFC 047](./047-indexing-cancellation.md)'s
sibling work found in the Japanese footer, which was the same cause (content
wider than its allotted share). **Acceptance requires the narrow case checked at
the largest step, in both locales.**

**2.4 Why not annotate all 118 sites instead.** Giving every `text(` site an
explicit role would let the size change at runtime with no restart, and is the
tidier end state — it is also 118 mechanical edits across the UI crates, each a
chance to pick the wrong role. **Deferred, not rejected.** §2.1 does not block
it: if runtime switching is later wanted, annotation is the way, and the setting
built here keeps working unchanged.

## 3. Navigation labels

`app/src/core/view.rs:55-82` builds all three navigation buttons as
`button(icon_*())` wrapped in `tooltip(text(t("nav.*")))`. The labels
**already exist and are already translated in both locales** — they are shown
only on hover.

Hover is the wrong gate for this audience: it needs a steady pointer, it is slow
with tremor, it does not exist on touch, and it is invisible to a user who does
not know to try. Three glyphs with no persistent text is a recognition task on
every visit.

**Show the labels** — beside the icons, or behind an expand control. No new
strings, no new translations.

## 4. The disabled pairs button — deferred behind RFC 047

snora 0.52 dims disabled labels to 45 %, which puts disabled text at **2.92:1**
(`text_primary`) and **2.23:1** (`text_secondary`) on the light preset.

WCAG 1.4.3 exempts inactive components, so this is **not a conformance failure**
— but the question asked was about older readers, and arama's most meaningful
disabled control is the Similar Pairs button, which
[RFC 047](./047-indexing-cancellation.md) §3.4 and §7.1 are about to make carry
*more* information: it will mean "embedding completed", and its greyed state
will be the visible consequence of a partial index.

**A control whose state is the message should be readable when it is off.**

**Deliberately not scheduled yet.** T061 is authorized and undelivered and edits
exactly this control. Two tasks changing one button would make either one's
result unattributable. **Revisit once T061 lands**, when the button's final
behaviour is known.

## 5. Non-goals

- **Changing colour tokens or target sizes.** Both measure well; §Summary.
- **A free-form font-size field.** Three named steps bound what the layout must
  survive; an arbitrary number does not.
- **Per-role scaling.** One scale over the whole set keeps the type hierarchy
  intact by construction.
- **Runtime switching.** §2.4 — deferred with a stated route.

## 6. Acceptance

- A text-size setting with three steps appears in Settings → General, beside
  theme and language, in both locales.
- Choosing a step and restarting changes **both** arama's own text and snora's
  chrome, consistently — no element left at the old size.
- The control states that a restart is needed, in both locales.
- **At the largest step, in both locales, at a narrow window**: nothing clips,
  and the footer in particular still renders whole.
- The navigation labels are visible without hover, in both locales.
- `cargo test --workspace`, `clippy -D warnings`, `fmt --check` clean, and the
  `Untranslated strings` CI job passes.
