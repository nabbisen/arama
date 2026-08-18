# RFC 043 Handoff — Typography roles and prose readability

Companion to
[RFC 043](../proposed/043-typography-roles-and-prose-readability.md), accepted
by the project owner 2026-08-17. It stays in `rfcs/proposed/` until it ships,
per [RFC 000](../done/000-rfc-lifecycle-policy.md).

**Read the RFC first.** This handoff does not restate it; it settles what the
RFC left open and names the traps.

> **Sequencing amendment, 2026-08-18 — snora 0.34.0 through 0.37.x.** Two
> rendered changes reach arama across that range: `border` contrast in the
> `light` and `dark` presets (0.34.0), and the modal dim strengthened in **all
> four** presets (0.37.0, `DIM_ALPHA` 0.40 → 0.44). Their release notes name
> visual-regression baselines containing card or dialog borders as the thing to
> re-check — and this RFC's deliverable is exactly such a baseline.
>
> **Do Task 028 (the 0.33 → 0.37.x upgrade) first, alone, with its own captures.**
> If the upgrade lands during this work, every before/after pair carries a
> typography change *and* a border change *and* a dim change, none of them
> attributable. That is the failure RFC 040 §3.1 exists to prevent.
>
> **The dim reaches §7's dialog-over-gallery capture in every preset**, so this
> is not a light/dark-only concern for you.
>
> **§2 below is unchanged and still binding:** once Task 028 has landed, this
> work makes no dependency change of its own. The lockfile it starts from is
> the one Task 028 produced.

## 1. Design authority

1. [RFC 043](../proposed/043-typography-roles-and-prose-readability.md);
2. `.git-exclude/tmp/release-0.33.1/docs/src/design/typography.md` and
   `.../guides/readability.md` — snora's own vocabulary and role-selection
   guidance. **Read both.** The role table is not the interesting part; the
   reasoning about which role fits which kind of text is;
3. [RFC 040](../done/040-snora-0.29-upgrade-and-dialog-surface.md) — the
   rendered-evidence method this work reuses;
4. [RFC 011](../done/011-theme-setting.md) and
   [RFC 010](../done/010-snora-design-system.md) — why `arama_theme::tokens()`
   has the shape it does.

## 2. There is no dependency change. Do not make one.

`snora` with `features = ["design"]` is already in `Cargo.toml`. **The
typography API is present and unchanged across this range** — verified at
`snora-style-0.33.0/src/text.rs:27-57` (`body_size` … `display_size`) and
`snora-design-0.33.0/src/typography.rs:32` (`Typography`). snora records no API
break across 0.34 through 0.37.0, and RFC-036's additive-only covenant freezes
`Typography` and `TextRole` by name.

**Re-verify against whatever version Task 028 leaves resolved**, rather than
against the 0.33.0 figures above. If the API is not reachable as described,
stop and report it — that would mean my verification was wrong, and it is a
better outcome than working around it.

**Do not bundle a dependency change into a visual-change diff** — it costs the
one property that makes this work easy to review, which is that every rendered
difference has a line of arama's own code behind it. That is the whole reason
the snora upgrade was split out into Task 028 rather than folded in here.

## 3. The settled scope decision — departures only

The owner accepted the RFC, whose §11 recommends **(a) departures only**. That
is the scope. **Do not annotate all 108 `text(` call sites.**

**3.1 Make the global default token-driven.** `app/src/core/settings.rs` — set
`default_text_size` from `arama_theme::tokens().typography.body.size`.

This is one line and it is the reason (a) is safe: without it, 102 unannotated
sites depend silently on iced and snora both choosing 16.0. They do today. This
makes it true by construction instead of by coincidence.

**Record the limitation in the code, not in a commit message:** `Settings` is
read once at application start, so this does not track a runtime theme switch.
All four built-in presets share `Typography::default_roles()` today, so there is
nothing to observe — which is exactly why the next person will not think of it.

**3.2 Every literal size goes.** All six, listed in RFC 043 §1. Each becomes a
role or is justified in writing for why it cannot be. "No literal text size
remains" is an acceptance criterion and it is checkable by grep.

**3.3 Wrapping prose gets a line-height.** There is no global lever —
`LineHeight::default()` is a fixed `Relative(1.3)` in iced 0.14 with no settings
hook. Per-site or nothing.

**3.4 Real hierarchy where hierarchy exists.** Dialog titles, page headings, the
fatal-startup title. Do not invent hierarchy that is not there — a screen with
one heading needs one heading, not a `display`.

## 4. Which strings are prose is judgement, and the bands will mislead you

RFC 043 gives 13 English strings ≥60 characters and 51 ≥30. **Those are a
starting point, not a rule**, and they have a specific failure mode:

**A string that is short in English can wrap in Japanese.** The bands were
measured on `en.rs` only, and a byte-count over `ja.rs` would have been wrong
(CJK is three bytes per character in UTF-8), so no equivalent number exists.

**Decide from the rendered result in both locales, not from the string length.**
If a line wraps, it needs a line-height. That is the whole test, and it is
observable rather than inferable.

## 5. Traps

- **`label` is *tighter* than the default.** snora's `label` line-height is
  1.2; iced's default is 1.3. Annotating a button or chip label makes it
  visibly tighter — correct per the scale, and in the opposite direction from
  "more readable". **Anyone reviewing a capture without being told this will
  read it as a regression.** Say it in the review package.

- **`similarity_badge/view.rs:17` has a fixed 60-pixel container.**
  `container(text(s).size(12)).width(60)` renders `"97.3 %"` over a thumbnail.
  Moving 12 → 14 may not fit. This is the one site where a small size might be
  deliberate rather than accidental. **Check what it does before changing it,
  and if `body_small` breaks the layout, say so and propose the alternative** —
  do not widen the container to make a role fit.

- **`title` (18) and `heading` (24) reflow fixed-width containers.**
  `cache_page/view.rs` uses `FillPortion` columns; the footer and settings
  dialog have fixed widths. A size increase inside a fixed column truncates or
  overflows rather than growing.

- **`cache_page/view.rs:176` renders a filesystem path at 13 px.** A path is
  the one string where a misread character changes the meaning. It is also
  long and in a proportional column — check the truncation behaviour at the new
  size rather than only the legibility.

- **The scratch-profile method changed under RFC 041.** RFC 036's and RFC 040's
  CWD-relative isolation no longer exists. Use `ARAMA_DATA_HOME`
  (`env/src/dir.rs:24`), which is the replacement RFC 041 established.

- **`text::secondary` is a contrast reduction, not a size one.** §2 of the RFC
  is about a site carrying *both*. Fixing the size while leaving reduced
  contrast on 104 characters of prose is a partial fix — decide about the style
  deliberately and say what you decided.

## 6. Non-change scope

- **i18n strings, in either locale.** Not one character.
- **Layout, spacing, padding, colour** — except where a size change forces a
  container to be adjusted, which is a consequence to report, not an invitation
  to retune.
- **Custom `Typography` values.** arama takes snora's presets. Supplying our
  own is RFC 043 §12 and needs the evidence this work produces.
- **The five icon `.size()` calls.** Lucide glyphs, not text, not in the scale.
- **snora's prefab widgets.** Dropped in Task 022. Not coming back here.
- **Any dependency change.** §2.
- **RFC 042.** Unrelated.

## 7. Verification — the gates prove nothing here

`cargo test`, `clippy` and `fmt` are necessary and nowhere near sufficient.
This is a visual change with a readability motive, exactly as RFC 040 was.

**Rendered captures are the deliverable.** Before and after, **in both locales**,
for the five surfaces in RFC 043 §7. Four presets are not required per surface —
typography is preset-independent today — **except** the Settings → General
surface, where the defect and the high-contrast theme are the same subject.

**md5 every pair.** A capture that did not change must be *proven* not to have
changed. That technique is this team's own and is now documented in snora's
`docs/src/guides/testing.md:136`, credited to arama.

**If a role makes something worse, that is a result.** The `label` tightening
and the badge container are the two places I expect it. Report it plainly rather
than choosing a role that flatters the capture — RFC 043's whole premise is that
the scale is well-designed, and evidence against that is worth more to this
project and to snora than a clean set of before/afters.

## 8. Acceptance criteria

- No literal text size remains in `app/src` or `crates/ui`. Grep-checkable.
- Every wrapping-prose site sets a line-height from its role.
- `default_text_size` comes from the tokens, with the restart limitation
  recorded in the code.
- The Settings → General high-contrast note is at `body_small` or `body`, shown
  by before/after captures in **both** locales.
- Every capture pair is md5'd, including the ones that did not change.
- Layout shifts are shown and stated, not silently absorbed.
- Gates clean.

## 9. What this gives snora — gather it, do not send it

snora deferred applying more of the scale to their own widgets, saying they
"would rather have evidence from the consumer who actually renders those
widgets than guess." arama is that consumer, and §7's captures are that
evidence at no extra cost.

Gather into the review package: the before/after set, and an honest paragraph on
whether the six roles were enough, whether any role was wrong for arama's text,
and whether anything needed a size the scale does not offer.

**Do not send anything to snora.** Outward communication is the owner's, as
every prior exchange has been.

*The three items owed from RFC 040 §6 are already discharged — sent 2026-08-15
and credited upstream. Nothing is owed today.*

## 10. Required deliverables

A review request under `.git-exclude/review-request/NNN-<slug>/` opening with an
`## Entry point` section giving the pinned commit, the exact `git show` command,
and plain paths to every file. Include the before/after captures with their
md5s, the both-locale coverage, every layout consequence, and §9's paragraph.

Report observed output; a check not run is recorded as not run.
