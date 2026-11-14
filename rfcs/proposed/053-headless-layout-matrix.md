# RFC 053: A headless layout matrix — every text size, language, theme and window size, on every CI run

**Status:** **Accepted by the project owner, 2026-11-14** (*"Accepted."*). Owner
decisions (§7): minimum window **760 × 520**; fonts **(a)**, installed on CI only.
No task is authorized yet.
**Raised by:** the architect, 2026-11-11, applying the owner's updated GUI test
rules (*Test Optimization Rules*, 2026-11-11). The owner asked for this RFC on
2026-11-14 (*"Yes."*).
**Relates to:** [RFC 044](../done/044-keyboard-operability.md) (the first
`iced_test::Simulator` use), [RFC 052](./052-text-size-and-reach.md) (text size —
the first consumer), [RFC 043](../done/043-typography-roles-and-prose-readability.md)

## Summary

The owner's test rules now say: check each property at the cheapest level that
can prove it, and **check layout properties — nothing overflows, every label is
fully visible — headlessly, across every text size, language, theme and window
size, on every CI run.** Real-window captures are kept for what needs eyes, and
are the worst case plus one baseline.

arama has the harness and no such check. `iced_test::Simulator` is used today by
a handful of keyboard tests (RFC 044) and by nothing that asks about layout. So
every layout question so far — the Japanese footer (T055), the Store screenshots
(T053, T057) — has been answered by capture, one combination at a time, and the
next one (RFC 052's *"the layout survives Larger, in both locales, at a narrow
window"*) is a matrix of 3 × 2 × 4 × *n* combinations that no capture plan can
cover.

This RFC builds the matrix once, so RFC 052 and every task after it prove layout
by test.

## 1. What is true today

**The harness can do it — in principle.** Read from `iced_test` 0.14.0 and
`iced_selector` 0.14.0:

- `Simulator::with_size(settings, size, element)` takes an `iced::Settings` —
  which carries `default_text_size` — and a window size.
- `Simulator::find(selector)` returns a `Target`. `Target::Text` carries
  `content`, its layout `bounds`, and `visible_bounds`: the part of it inside the
  viewport and every clipping ancestor, or `None` if none of it is.
- The theme and the locale are process globals arama already sets at runtime
  (`arama_theme::set_theme`, `arama_i18n::set_locale`).

**Whether it detects arama's actual failures is not known.** `visible_bounds`
sees text clipped by a scrollable or a clipping container, or pushed outside
the window. It may **not** see text that overflows a button or a fixed-width
cell that does not clip — which is the shape of the Japanese footer defect.
§3 makes this the first thing established, not assumed.

**Two things the matrix needs do not exist:**

1. **A minimum window size.** `iced::application(...)` in `app/src/core.rs` sets
   no `window::Settings::min_size`, so arama's window can be made as small as
   the system allows. RFC 052 and T063 ask for *"a narrow window"* and define no
   width. A matrix needs a floor, and so do users: below some width the layout
   is not designed to work, and today nothing stops a user from reaching it.
2. **Font parity.** The real app loads one bundled font — Lucide, for icons
   (`app/src/core/settings.rs`) — and draws text with whatever the system
   provides. Headless, `iced_test` measures with the fonts it can find on the
   machine running the test. **On a CI runner with no Japanese font, Japanese
   text is measured with fallback glyphs of the wrong width**, and a layout check
   in `ja` would pass or fail for the wrong reason.

## 2. Design

### 2.1 One matrix, one place

A test module in the `arama` crate (beside `app/src/core/tests.rs`, which
already drives `App` through `iced_test`) that, for each **screen state** and
each **combination**, renders `App::view()` in a `Simulator` and asserts the
**layout properties** (§2.3).

**Combinations** — every one of:

| Dimension | Values |
|---|---|
| Text size | Default, Large, Larger (RFC 052 §2.1). Until T063 lands: Default only, with the dimension wired |
| Locale | `en`, `ja` |
| Theme | the four presets in `crates/theme/src/lib.rs` |
| Window size | the minimum (§2.2), 1280 × 720 (the rules' default), and one large size |

3 × 2 × 4 × 3 = **72 combinations** per screen state.

**Screen states** — every state a user can reach that has its own layout:
setup wizard; gallery (empty, and populated from a fixture); Settings; Cache
page; each dialog (Media Focus, Similar Pairs with results and with its absence
states, Confirm); a toast. Phase 0 (§3) reports which can be built headless and
which need state constructed directly on `App`.

### 2.2 A minimum window size — decided here, enforced in the app

**arama sets `min_size` at launch, and the matrix tests that size.** **Owner's
decision, 2026-11-14: 760 × 520 logical px of content** (§7, option A).

**The height** comes from the smallest common laptop screen,
1366 × 768 at 125% scaling, is 1093 × 614 logical px; a taskbar (≈ 48) and a
title bar (≈ 31) leave a maximised window about **1093 × 535** of content. A
minimum must fit inside that — a minimum larger than the screen is a window the
user cannot fit — so the height has to sit below 535, and 520 leaves a margin.

**The width comes from use, not from today's layout.** A media-similarity tool
is used beside a file manager, each on half the screen. The commonest laptop
setting, 1920 × 1080 at 125 %, gives a half of **768** logical px; 760 leaves
room for the window frame. (960, the architect's first candidate, was a guess at
what the current layout fits; it would rule out side-by-side use on that
setting.) **If the layout does not fit 760 today, the layout changes** — for
example, the side pane collapses below a width — **not the minimum.** Phase 0
(§3) reports what does not fit; the layout work goes to T063, whose *"narrow
window"* this value now defines.

**The launch size goes with it.** arama sets no window size today, so it opens
at the toolkit's default, 1024 × 768 — taller than the 535 px of content the
smallest laptop has. The launch size must fit the same screen as the minimum;
the implementing task sets one (1024 × 520 or smaller).

Enforcing it matters as much as testing it: a minimum the matrix tests and the
window does not enforce is a promise the app does not keep.

### 2.3 The properties checked

1. **Every text is fully visible:** for each `Target::Text`, `visible_bounds ==
   Some(bounds)`. Not merely present — fully.
2. **Nothing overflows its container:** whatever §3 establishes catches the
   non-clipping overflow case. If nothing does, say so and record the gap; do not
   claim the property.
3. **Interactive targets meet a minimum size** — 24 × 24 logical px (WCAG 2.5.8),
   for every control the selector can find.

**Contrast is not checked here.** arama uses snora's presets unchanged, and
snora measures contrast from drawn pixels across its presets (snora RFC-108).
arama's own colours, if it ever adds any, would join this matrix then.

### 2.4 Fonts — the same glyphs headless as on screen

The matrix must measure Japanese with a real Japanese font, the same on CI as on
a developer's machine. Two ways were put to the owner (§7); **(a) was chosen**:

- **(a) Install a pinned font on the CI runner** (for example the distribution's
  Noto CJK package) and require the same locally. Cheapest. Leaves the real app
  as it is.
- **(b) Bundle a Japanese font with arama** (an OFL-licensed one, loaded through
  `Settings::fonts` like Lucide), and have the matrix load the same bytes. The
  headless and real measurements then agree **by construction**. **It also
  changes the product:** today a Linux user without CJK fonts sees boxes for
  every Japanese label, and a Windows or macOS user sees whichever system font
  happens to be the fallback. Costs package size (a subset font is a few MB).

**Owner's decision, 2026-11-14: (a)** — *"Does option (a) make the release asset
slim? If so, (a). A user is not always use Japanese. However, possibly, plans
like (b) should be considered in the future."* (a) adds nothing to any release
asset: the font is installed on the Linux runner only.

**How (a) is done:**

- **Linux runner:** install the distribution's full Noto CJK package, its
  version printed into the log (an exact pin breaks when the runner image
  updates), so every Han variant a Linux user may have is present (see the
  finding below — the engine may pick the Simplified Chinese one).
- **macOS and Windows runners:** install nothing. They carry the system fonts
  users of those platforms have, so the matrix there measures what those users
  see.
- **Locally:** the same Linux package; `docs/src/dev/testing.md` names it.

**Finding, from source, for the future (b) — not yet observed.** When arama's
default font lacks a glyph, `cosmic-text` 0.15.0 falls back by script. For Han
(kanji) it chooses the Japanese font only when the locale is exactly `ja`
(`src/font/fallback/{unix,windows,macos}.rs`, `han_unification`), but the
locale it is given is the **system's**, as `sys_locale` reports it — `ja-JP`,
not `ja` — and never arama's own language setting. Otherwise it chooses
Simplified Chinese: Noto Sans CJK SC, Microsoft YaHei UI, PingFang SC. Kana
always get the Japanese font. So arama may show kanji in Chinese letterforms,
and one label in two fonts, on every platform. Phase 0 records which font the
engine picks (§3, item 4). If it confirms this, it is the case for (b) — a
bundled Japanese font set as the default font — or for a fix upstream, and is
raised with the owner as its own RFC.

### 2.5 Where it runs

In the `arama` crate's default (non-`#[ignore]`) test suite, so it runs on
every push in two places that already exist: `msrv.yaml`'s `cargo test
--workspace --locked` (Linux), and `native-smoke.yaml`'s default
`cargo test -p arama --lib --locked` run on macOS and on Windows. No new
workflow. **Budget: the whole
matrix under 60 s on the Linux runner.** If Phase 0 measures more, it reports
the number and the matrix is split, not trimmed.

### 2.6 A failing check must name the combination

A failure prints the screen state, the combination, the text's `content` and
both rectangles — enough to reproduce it with one test filter and no capture.

## 3. Phase 0 — what the harness can actually see

Before the matrix is built, one probe establishes three facts, each by a
**planted defect** (the check is shown to fail before it is trusted to pass):

1. **Clipping:** a label inside a too-small clipping container. Does
   `visible_bounds` report it?
2. **Non-clipping overflow:** a label wider than a fixed-width button that does
   not clip — the Japanese-footer shape. What, if anything, reports it? Candidates:
   comparing a text's `bounds` with its nearest container's; a layout-tree walk;
   or `Simulator::snapshot` with a pixel check at the container's edge.
3. **Fonts:** the same Japanese label measured with and without the font from
   §2.4. Does the width differ? (It should; if it does not, the font is not being
   used.)
4. **Which font draws kanji:** on each runner, the font face the engine selects
   for a kanji in a Japanese label, and for a kana in the same label. Recorded,
   not asserted — it is evidence for §2.4's finding, not a property of the
   layout.

Plus: the screen states that can be built headless (§2.1), and the matrix's run
time.

**The answers set what §2.3 claims.** A property the probe cannot make fail is
not listed as checked.

## 4. What this changes for tasks

- **Evidence plans** (the handoffs' rule since 2026-11-11) put every layout
  question at this level. A capture is for what needs eyes.
- **RFC 052 / T063:** *"the layout survives Larger, in both locales, at a narrow
  window"* becomes *"the matrix passes with Larger enabled"* — every theme and
  window size included, not only the combination someone captured.
- **A capture that finds a layout problem the matrix missed** gets a new
  property or state in the matrix, so it cannot return unseen (the rules' own
  instruction).

## 5. Non-goals

- **Pixel-snapshot regression suites.** RFC 044 §0.2b found that `iced_test`'s
  `matches_image`/`matches_hash` create *and pass* on a missing reference — a
  footgun. A snapshot may be used inside §3's probe; it is not a reference store.
- **Contrast measurement** (§2.3).
- **Real-window automation.** That stays under the GUI rules' real-window
  section, for what needs eyes.

## 6. Acceptance

- Phase 0's three planted defects, each shown failing, and the run time.
- `min_size` set at launch to the chosen minimum.
- The matrix runs every combination in §2.1 for every screen state Phase 0 can
  build, and asserts every property Phase 0 showed it can detect.
- A failure names its combination (§2.6).
- It runs on every CI push, on Linux, macOS and Windows, within budget.
- Japanese is measured with a real Japanese font on every runner (§2.4): the
  pinned package on Linux, the system fonts on macOS and Windows.
- A launch size that fits the smallest laptop screen (§2.2).

## 7. For the owner

Both decided 2026-11-14:

1. **The minimum window size — A: 760 × 520** logical px of content (§2.2).
2. **Fonts — (a): install on CI only**; nothing added to the release assets
   (§2.4). (b) stays open for the future, with §2.4's finding as its evidence.
