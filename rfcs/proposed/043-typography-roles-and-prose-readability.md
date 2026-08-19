# RFC 043: Typography roles and prose readability

**Status.** Proposed — requested by the project owner 2026-08-17 and
**accepted for implementation the same day**. Remains in `rfcs/proposed/` until
the work ships, per RFC 000. Not blocked on anything.
**Handoff:** [043 handoff](../handoffs/043-typography-roles-and-prose-readability-handoff.md).
**Tracks.** Adopt snora's six-role text scale, and give wrapping prose a
line-height, for the text arama renders itself.
**Touches.** View code in `crates/ui` and `app/src/core/view.rs`, and one line
in `app/src/core/settings.rs`. **No dependency change. No product logic.**

## Summary

snora 0.33.1 documents a six-role text scale that has existed since snora 0.20
and that **no documentation had ever mentioned** — snora's own release notes
call it "a documentation failure, not a missing capability," and predict that
applications built on snora will have "flat, uniform text."

arama is that prediction. **102 of arama's 108 `text(` call sites set no size at
all**, and **nothing in the application sets a line-height anywhere.** There is
no title, no heading, no section boundary — every piece of text in arama renders
at one size.

**The prerequisite is already met and costs nothing.** snora's note says arama
is "on the engine path at 0.25.0, so it is not reachable from your current
build." That was true when written; it is no longer. RFC 040 moved arama to the
`design` feature in 0.39.1, and Task 022 took it to `snora = "0.33"`. The
typography API is **verified present in the exact version already in
`Cargo.lock`** — `snora-style-0.33.0/src/text.rs:27-57` defines all six size
helpers, `snora-design-0.33.0/src/typography.rs:32` defines `Typography`.

**This RFC still makes no dependency change of its own** — but the version it
builds on is no longer the one in `Cargo.lock` when this was written. snora has
since shipped 0.34 through 0.38, two of them with rendered changes arama must
photograph separately, so the upgrade was split into **Task 028** and sequenced
ahead of this work. The typography API is unchanged across that range and 0.38
adds line-height helpers this RFC uses (§5.2).

The point the paragraph above made still holds: nothing here is blocked on
snora, and the scale was reachable the whole time.

## The honest caveat, stated first

**This will not make ordinary text bigger, and expecting that will disappoint.**

iced 0.14's default text size is `Pixels(16.0)`
(`iced_core-0.14.0/src/settings.rs:51`). snora's `body` role is **16.0**. They
are the same number by coincidence of two projects picking the same sensible
default. So arama's 102 unsized text sites are *already* rendering at `body`
size.

The readability gain is real but it is not size. It is four things:

1. **Hierarchy, which arama has none of.** No dialog title, page heading, or
   section boundary is visually distinguished from the paragraph beneath it.
   `title` (18.0) and `heading` (24.0) are unused because they were
   undocumented.
2. **Line-height on wrapping prose.** iced's default is `Relative(1.3)`
   (`iced_core-0.14.0/src/text.rs:215-219`); snora's `body` is **1.4**. Modest
   per line, compounding across a paragraph — and snora is right that it is
   "the single strongest lever on whether wrapping prose is comfortable to
   read."
3. **Six magic numbers retired**, two of which are defects (§2).
4. **Typography becomes theme-driven** rather than literal. Today a preset
   cannot change text size even in principle.

Anyone reviewing the before/after captures should be told to look for
hierarchy and paragraph comfort, not for larger text. Otherwise the honest
result — "most text is unchanged" — reads as a failure.

## 1. What arama renders today — measured, not estimated

| Measure | Value |
|---|---|
| `text(` call sites in `app/src` + `crates/ui` | **108** |
| …that set a size | **6** |
| …that set a line-height | **0** |
| Roles from the scale in use | **0** — every size is a literal |
| Distinct literal sizes | 20, 14, 14, 13, 12, 12 |
| English i18n strings ≥ 60 characters (wrapping prose) | **13** |
| English i18n strings ≥ 30 characters | **51** |

*(English only. The equivalent count for `ja.rs` cannot be taken from a byte
measure — CJK is three bytes per character in UTF-8 — and is not quoted here
rather than quoted wrongly. §3 is the reason it still matters.)*

The six literals, in full:

| Site | Size | What it is |
|---|---|---|
| `app/src/core/view.rs:189` | 20 | fatal-startup **title** |
| `crates/ui/main/src/core/components/setup/downloader/view.rs:55` | 14 | per-model status line |
| `…/downloader/view.rs:58` | 14 | ffmpeg external help **prose** |
| `crates/ui/main/src/core/views/cache_page/view.rs:176` | 13 | a directory **path** |
| `crates/ui/widgets/src/similarity_badge/view.rs:17` | 12 | `"97.3 %"` badge over a thumbnail |
| `crates/ui/widgets/src/dialog/settings_dialog/tab/general_settings/view.rs:105` | 12 | high-contrast explanation **prose** |

**Nothing in this table came from the scale.** 20 is not a role. 13 is not a
role. The two 12s are the same number for two unrelated reasons.

## 2. One site is a genuine defect, and it is the ironic one

`crates/ui/widgets/src/dialog/settings_dialog/tab/general_settings/view.rs:104-106`:

```rust
let theme_note = text(t("settings.general.theme.hc_note"))
    .size(12)
    .style(text::secondary);
```

The string is 104 characters — *"High-contrast maps core colors into standard
widgets; arama controls use the full high-contrast palette."* — wrapping prose
in a narrow dialog column.

It is rendered at **12 px, the smallest text in the application and exactly
snora's stated accessibility floor**, with **`text::secondary`**, which lowers
contrast, and **no line-height**, so its wrapped lines sit at 1.3.

**It is the note that explains the high-contrast themes.** The text a
low-vision user reads to understand arama's accessibility feature is the least
readable text arama renders. Per snora's readability guide this is
`body_small` at absolute minimum and arguably `body`; the guide's own rule is
"if none of these fit, the answer is almost always `body` or `body_small` — not
a custom size."

This alone justifies the work independently of the aesthetics, and it was found
by reading rather than by looking — which is why §7 requires captures.

`cache_page/view.rs:176` at 13 px is the second worst: a filesystem path, the
one kind of string where a misread character changes the meaning.

## 3. Japanese is not a footnote

arama ships two locales — `crates/i18n/src/en.rs` (202 lines) and
`crates/i18n/src/ja.rs` (276 lines).

CJK glyphs are denser and taller within the same em box than Latin ones, and
line-height matters **more** for them, not less, for exactly the reason snora
gives: tracking from the end of one wrapped line to the start of the next.

§2's defect is worse in Japanese, and the same file shows it —
`crates/i18n/src/ja.rs:259`:

> ハイコントラストは標準ウィジェットの基本色にも反映されます。arama 独自のコントロールには完全なハイコントラストパレットが適用されます。

Sixty-odd CJK characters, rendered at **12 px with reduced contrast and 1.3
leading**, in a narrow column. This is the project owner's own locale, and it
is the single strongest argument in this RFC.

**Evidence captured only in English proves half of this RFC.** Every capture
required in §7 is required in both locales. An English-only evidence set is not
a partial pass; it is a pass on the easier half.

## 4. The seam is already built, and this is why the work is cheap

The usual expensive part of adopting a type scale is threading `Tokens` to
every view. **arama has already paid it.**

`crates/theme/src/lib.rs:64`:

```rust
pub fn tokens() -> Tokens { tokens_for_preset(current_theme()) }
```

Zero arguments, over a global `AtomicU8`, safe from any thread, deliberately
returning owned `Tokens` to avoid lifetime friction in `view()`. **Any call site
in arama can reach the scale with one `use` and no signature change.**

That is a real dividend from RFC 010 and RFC 011, and it is the reason this RFC
is a view-code change and not an architecture change.

## 5. Do not annotate all 108 sites — the design question that matters

Because `body` == iced's default size, mechanically annotating every site would
produce a **108-site diff that changes only line-height**, and a large diff
whose visible effect is small is exactly the shape that hides a regression.

**Recommended: set the global default from the tokens, then annotate only
departures.**

**5.1 One line makes the default token-driven.** `app/src/core/settings.rs`
currently takes iced's default:

```rust
Settings { fonts: vec![LUCIDE_FONT_BYTES.into()], ..Default::default() }
```

Setting `default_text_size` from `tokens().typography.body.size` makes every
unannotated site track the scale rather than silently depending on iced and
snora agreeing on 16.0. The values are equal **today**; this makes them equal
*by construction*.

**Its limitation, named rather than discovered later:** `Settings` is read once
at application start, so a preset with different typography would not take
effect until restart. Today all four built-in presets share
`Typography::default_roles()`, so there is no live difference to observe — but
this is the reason a future per-preset type scale would need a second
mechanism, and it should be written in the code, not left for someone to find.

**5.2 Line-height cannot be defaulted.** `LineHeight::default()` is a fixed
`Relative(1.3)` with no settings hook in iced 0.14. **Every wrapping-prose site
must be touched individually** — there is no global lever. That is the ~13
strings ≥60 characters and a judgement call on the ~51 ≥30, not all 108.

> **snora 0.38.0 (RFC-068) adds six line-height helpers to
> `snora-style::text`**, one per role, alongside the existing size helpers.
> Purely additive, no rendered change.
>
> **This does not change the design** — there is still no global lever and every
> prose site is still touched individually. It changes the *call*: read the
> multiplier through the helper rather than reaching into
> `tokens.typography.<role>.line_height` directly, matching how sizes are
> already read. **Verify the helper names against the crate**; they are not
> quoted here because I have not seen them.
>
> Task 028 lands 0.38 for this reason, so this work needs no dependency change
> of its own.

**5.3 Departures get an explicit role.** Titles, headings, secondary metadata,
and the six literals in §1. Ordinary single-line body text is left alone.

The alternative — annotate everything — is defensible and is the one thing here
I would not overrule if the owner prefers it. See §11.

## 6. Non-goals

- **Any change to what text says.** Roles and line-height only. The i18n
  strings are not touched, in either locale.
- **Custom `Tokens`.** arama takes snora's presets. Supplying arama's own
  typography values is a separate decision and needs evidence this does not
  yet have.
- **Layout, spacing, padding, or colour.** A size change will shift layout as a
  consequence; that is a result to observe, not a licence to retune.
- **snora's prefab widgets.** arama uses the engine and dropped
  `snora-widgets` in Task 022. snora's own `label`/`body`-only chrome coverage
  does not reach arama.
- **The icon `.size()` calls.** Five sites size Lucide glyphs, not text. They
  are not in the scale and are out of scope.
- **RFC 042.** Unrelated and unaffected.

## 7. Verification — the gates prove nothing here

This is a visual change with a readability motive, exactly as RFC 040 was.
`cargo test` and `clippy` are necessary and nowhere near sufficient.

**Rendered captures are the deliverable**, by RFC 040's established method:
scratch-profile isolation (per RFC 041's replacement mechanism, since the
CWD-relative method is gone), native Wayland window-scoped capture, md5 to
prove a capture actually changed.

Required **before and after**, in **both locales**:

| Surface | Why |
|---|---|
| Settings → General, high-contrast dark | §2's defect. **Must change.** |
| Setup wizard, ffmpeg help prose | the other prose literal |
| Cache page, path column | the 13 px path |
| A similarity dialog over a populated gallery | RFC 036's absence text, and the densest background |
| Fatal-startup screen | the only near-`heading` in the app |

Four presets are **not** required per surface here — typography is
preset-independent today (§5.1) — **except** for the settings surface in §2,
where the defect and the high-contrast theme are the same subject.

**md5 every pair.** A capture that did not change must be proven not to have
changed, not asserted. That technique is arama's own and is now documented in
snora's `docs/src/guides/testing.md`, credited to this team.

## 8. Acceptance criteria

- No literal text size remains in `app/src` or `crates/ui`. Every text size
  comes from a role.
- Every wrapping-prose site sets a line-height from its role.
- §2's high-contrast note is at `body_small` or `body`, and the before/after
  capture shows it.
- Both locales captured. English-only is incomplete.
- Layout shifts caused by size changes are shown and stated, not silently
  absorbed.
- §5.1's restart limitation is recorded in the code.

## 9. Risks

- **A large diff with a small visible effect.** §5 is the mitigation; if the
  diff approaches 108 sites without §11 having chosen that, something has gone
  wrong.
- **Layout regression.** `title` at 18 and `heading` at 24 are meaningfully
  larger than 16 and will reflow containers with fixed widths. The gallery,
  the footer, and the settings dialog's fixed columns are where to look. **The
  captures are the guard, which is why every surface is listed.**
- **`label` is tighter than the default** — 1.2 against 1.3. Annotating a
  button or chip label makes it *tighter*, not looser. That is correct per the
  scale, but it is a visible change in the direction opposite to "more
  readable", and it will look like a mistake to anyone who was not told.
- **Judgement, not mechanism, decides which strings are prose.** The ≥60 and
  ≥30 character bands are a starting point, not a rule. A string that is short
  in English and wraps in Japanese is the case the bands miss.
- **Snapshot brittleness.** If any test asserts on rendered text metrics, size
  changes break it. Not known to exist; check rather than assume.

## 10. What this gives snora, at no cost

snora's release notes say applying more of the scale to their own chrome is
"deferred deliberately… we would rather have evidence from the consumer who
actually renders those widgets than guess."

**arama is that consumer**, and §7's captures are that evidence — produced for
arama's own reasons and costing nothing extra to share. The same shape as the
dialog-card report, which produced two upstream releases.

The three items owed from RFC 040 §6 are **already discharged**: sent
2026-08-15 and credited in snora's `docs/src/guides/testing.md:136`. snora's
0.33.1 note re-offers them because the two messages crossed. **Nothing is owed
today** — this section is an opportunity, not a debt, and it does not gate the
work.

Outward communication remains the owner's, as every prior exchange has been.

## 11. The one decision that is the owner's — settled

> **Settled 2026-08-17.** The owner accepted this RFC without amending §11, so
> **(a) departures only** is the scope, as recommended. Carried into the
> handoff §3.

**How far does annotation go?**

- **(a) Departures only** *(recommended)* — §5. Small diff proportional to the
  actual hierarchy. Ordinary body text keeps iced's default, made token-driven
  by §5.1's one line.
- **(b) Every site names its role** — ~108 sites. No implicit coupling to a
  default anywhere, and the only version where typography is fully
  theme-driven. Larger review surface, and a regression is easier to hide in
  it.

I recommend (a) because the diff stays proportional to the visible change, and
because §5.1 removes the coupling that is (b)'s main argument. (b) is
defensible and I would not argue against it if the owner prefers completeness.

## 12. Open questions

- **Does arama eventually want its own `Typography` values** rather than
  snora's defaults? `Typography` is a plain non-`#[non_exhaustive]` struct, so
  it is available. **Not now** — that decision needs the evidence this RFC
  produces, and making it first would be choosing before looking.
- **Should a preset carry its own type scale?** High-contrast presets changing
  colour but not size is a defensible position and the current one. §5.1's
  restart limitation is the constraint if that ever changes.
