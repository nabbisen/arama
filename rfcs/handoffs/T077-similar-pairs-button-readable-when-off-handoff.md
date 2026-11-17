> **AUTHORIZED 2026-11-17 — owner:** *"The three are approved."* (T077, T078,
> T079). Written 2026-11-17 by the architect.
>
> **Implements [RFC 052](../proposed/052-text-size-and-reach.md) §4**, which was
> held until T061 (RFC 047) landed. T061 is accepted (review 170). Independent
> of T078 and T079.

# T077 Handoff — the Similar Pairs button says what it is, and why it is off

## 1. Purpose

RFC 052 §4: *"A control whose state is the message should be readable when it is
off."* Since T061, the Similar Pairs button's disabled state **is** the message:
the folder has not finished indexing (RFC 047 §3.4, §7.1). Today a user cannot
read that message:

- **The button has no name.** `crates/ui/layout/src/header/view.rs:14` builds
  `button(icon_group())`: an icon, no label, no tooltip. T064 fixed exactly this
  for the navigation. This one control was left out.
- **Its off state is only a fade.** iced's default style draws a disabled button
  at reduced alpha. snora's design path also dims (`snora-style` 0.57.0
  `src/button.rs:57`, ×0.45). RFC 052 §4 measured that at 2.92:1 for text.
  Nothing says *why* it is off.

## 2. Background — the decision

Decided by the architect (the owner's principles: the final design over a patch,
and UI that cannot be misread):

1. **Name it.** The button shows its name beside the icon, always: new key
   `header.similarity_pairs` — *"Similarity pairs"* / *"類似ペア"*, the same
   wording as the existing `toast.similarity_pairs.title`. Styled like the
   header's other text button (`arama_theme::primary`, as the Folder button in
   `crates/ui/layout/src/header/dir_nav/view.rs:14`).
2. **Say why it is off, in words.** While it is disabled, a short line beside
   it, at body text colour, reads *"Available when indexing finishes"* /
   *"インデックス作成が終わると使えます"* (new key
   `header.similarity_pairs.unavailable`). The grey is then never the only
   message. When the button is enabled, the line is not shown.
3. **The label stays readable while disabled.** The disabled button's label
   must measure **at least 4.5:1** against what it is drawn on, on all four
   presets. If `arama_theme::primary`'s disabled state does not reach that,
   give the button an arama style whose `Disabled` arm keeps the text at full
   contrast and marks "off" by shape instead — for example, no fill and an
   outline (WCAG 1.4.1: not by colour alone). Measure; do not estimate.

Not chosen: **a tooltip** (hover is the gate T064 removed), and **hiding the
button until it is ready** (the user then cannot learn the feature exists, or
that indexing is what unlocks it).

## 3. Applicable RFC and requirements

RFC 052 §4; RFC 047 §3.4 and §7.1 (what "disabled" means); WCAG 1.4.3 (inactive
components are exempt — arama holds itself to it anyway, which is §4's point),
1.4.1, and 1.4.11.

## 4. Change scope

- `crates/ui/layout/src/header/view.rs`
- `crates/theme/src/lib.rs`, only if §2.3 needs an arama style
- `crates/i18n/src/en.rs`, `crates/i18n/src/ja.rs`, and the shared plain-key test
  in `crates/i18n/src/tests.rs`
- tests beside them; the T074 matrix (`app/src/core/tests/layout_matrix.rs`), only
  to add a state if one is missing (§6)
- `docs/src/users/using-arama.md` (§8)

## 5. Explicit non-change scope

- **When** the button is enabled — `embedding_cached`, T061's logic. Unchanged.
- The dialog itself.
- `Gallery::embedding_cached()`'s name (a separate follow-up).
- snora. If its disabled style should change for everyone, the architect writes
  to snora; do not patch around it beyond §2.3.

## 6. Required implementation and tests

- The name and the unavailable line, as §2.
- **A test through `App::update`:** with `embedding_cached` false, the header
  renders the name and the unavailable line; with it true, the name and no line.
  Use the existing `iced_test` selector helpers (`find_all_text_with`) on
  `app.view()`, not snapshots.
- **A contrast test** in `crates/theme/src/lib.rs`, the same shape as T064's
  `side_bar_active_fill_meets_wcag_1_4_11_non_text_contrast_every_preset`: for
  each of the four presets, the disabled button's label colour against its
  background ≥ 4.5:1, and the unavailable line's colour against the page ≥ 4.5:1.
  Print the measured ratios.
- **The T074 matrix, all 72 combinations, green.** The header at
  `ja`/Larger/760 × 520 is the worst case for the new label and line; if they do
  not fit, wrap them (the `Row::wrap()` pattern from T063/T064), never clip. The
  gallery states the matrix already has render the header, so confirm the new
  texts are among the targets it checks — name one combination's output.

## 7. Prohibited shortcuts

- A tooltip as the name or the reason.
- Lowering only the fade (say, 0.45 → 0.6) and calling it readable without a
  measurement.
- Colour as the only difference between enabled and disabled.
- A plant in the real working tree (scratch worktree, its own target directory).
- Starting an arama binary other than through `cargo test`, `cargo run` with the
  task's isolation, or `scripts/gui-session.sh`.

## 8. Required documentation updates

`docs/src/users/using-arama.md` contradicts itself today. Lines 28-31 say the
pairs button *"enables once the gallery has cached thumbnails … it does not wait
for the full indexing pass to finish"*. Lines 52-58 (T061) say it stays disabled
until embedding has **completed**. The second is true. Correct the first, and
describe the new name and the unavailable line.

## 9. Acceptance criteria

- The button shows its name in both locales, always.
- While it is disabled, the reason is shown in words, and the label is ≥ 4.5:1,
  measured on all four presets.
- Enabled and disabled differ by more than colour.
- The T074 matrix is green at 72 combinations; the `App::update` test and the
  contrast test pass.
- The docs contradiction is fixed.
- Gates: `fmt --check`, `clippy --all-targets -D warnings`, `test --workspace`,
  `scripts/check-untranslated-strings.sh`, all `--locked`.

## 10. Module boundaries

`arama-ui-layout` (header), `arama-theme` (style, if needed), `arama-i18n`. No
change to `arama-ai`, `arama-cache`, or the dialog.

## 11. Compatibility constraints

No settings, cache or data change. Two new i18n keys, in both locales.

## 12. Security constraints

None — presentation only.

## 13. Known risks

- **Header width at Larger, in Japanese, at 760 px** — the new label and line add
  width beside the path field. The matrix decides; wrap, never clip.
- **iced's `button` cannot be keyboard-focused** (unlike snora's controls).
  Unchanged by this task; Tab routing remains out of scope.

## 14. Required evidence

### 14.1 Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Are the name and reason shown in the right states? | logic and state | the `App::update` test |
| Is the disabled label readable, on every preset? | logic | the contrast test, with ratios printed |
| Does it fit everywhere? | headless | the T074 matrix, 72 combinations |
| Does it read well? | real UI | **one capture**: `ja` / Larger / 760 × 520, disabled (no folder indexed), through `scripts/gui-session.sh` with pre-seeded settings |

### 14.2 Deliverables

The rows above, observed, and the commit(s).

## 15. Review-request format

A package under `.git-exclude/review-request/` with a `## Entry point` section:
the commit(s), the `git show` commands, and full repo-relative paths, one per
line. Report observed output; a check not run is recorded as not run. You cannot
push: report the commits, and the architect pushes.

## 16. Handoff checklist

| Required item | Covered in |
|---|---|
| Task title | Heading |
| Purpose | §1 |
| Background | §2 |
| Applicable RFC and requirements | §3 |
| Change scope | §4 |
| Explicit non-change scope | §5 |
| Required implementation | §2, §6 |
| Required tests | §6 |
| Required documentation updates | §8 |
| Acceptance criteria | §9 |
| Prohibited shortcuts | §7 |
| Relevant code or module boundaries | §10 |
| Compatibility constraints | §11 |
| Security constraints | §12 |
| Known risks | §13 |
| Required evidence | §14 |
| Required review-request format | §15 |
