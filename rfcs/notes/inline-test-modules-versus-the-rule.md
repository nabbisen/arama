# The inline-test-module rule was applied by a third of the codebase

**Measured:** 2026-08-22, after 0.41.0 shipped, while checking whether RFC 044's
growth had tripped the roadmap's ELOC watch. It had not; this is what the
measurement found instead.
**Status:** recorded. A task follows; the rule itself needs a decision the
measurement cannot make.

## The rule

`.git-exclude/rules/project-instructions-rust-gui.md:29-31`, under Testing
Guidelines:

> ⭕ **Good:** `src/some_mod.rs` and `src/some_mod/tests.rs`.
> ❌ **Bad:** `#[test]` modules placed inside `src/some_mod.rs`.

Unqualified. Any inline test module is Bad.

## What the codebase does

> **Corrected 2026-08-22.** The figures first published here were wrong, found
> by the dev team while acting on them (package 128 §1). **The measurement
> counted `mod tests;` — the *external* declaration, which sits near the top of
> a file — as though it were an inline block running to end of file**, so
> hundreds of lines of ordinary production code were counted as test lines in
> three files. The corrected numbers and what they change are in §"Corrected
> counts" below; the original table is left as written, because the error is the
> point.

**26 files carry an inline `mod tests {`. Fourteen already use `mod tests;` and
were compliant all along.** Four exceed 150 lines.

*Original text, retained:* **40 files carry an inline `mod tests`.** Thirteen
have test modules over 100 lines:

| test lines | raw lines | file |
|---|---|---|
| 742 | 1309 | `app/src/core.rs` |
| 389 | 412 | `crates/cache/src/core/image.rs` |
| 340 | 364 | `crates/cache/src/core/video.rs` |
| 310 | 443 | `crates/i18n/src/lib.rs` |
| 256 | 624 | `crates/ui/widgets/src/dialog/media_focus_dialog/similar_media.rs` |
| 164 | 169 | `crates/ai/src/config/video_similarity_config.rs` |
| 160 | 456 | `app/src/core/data_locations.rs` |
| 143 | 213 | `crates/ui/main/src/core/views/setup.rs` |
| 135 | 293 | `env/src/dir.rs` |
| 129 | 183 | `app/src/core/update/keyboard.rs` |
| 122 | 293 | `crates/ai/src/pipeline/encode/image/clip_encoder.rs` |
| 108 | 259 | `.../video/video_similarity_calculator.rs` |
| 105 | 178 | `crates/ai/src/pipeline/score/similarity/image.rs` |

**`crates/cache/src/core/image.rs` is 412 lines of which 389 are tests** — 14
effective lines of implementation. `video_similarity_config.rs` is 169 lines
with 164 of tests.

## Corrected counts

| | first published | actual |
|---|---|---|
| inline `mod tests {` | "40 files break it" | **26** |
| already external, `mod tests;` | not mentioned | **14** |
| over 150 lines | 7 | **4** |

**Three rows of the table above are artifacts, not violations** —
`crates/cache/src/core/image.rs` (real test file: 60 lines),
`crates/cache/src/core/video.rs` (50), and
`crates/ai/src/config/video_similarity_config.rs` (92) were split by earlier
work and were never in breach.

**This changes the characterisation, not the decision.** A rule 26 files broke
while 14 followed it is *inconsistently applied*, not dead — and 14 files doing
it correctly is evidence the rule was workable rather than evidence it was
fiction. The 150-line threshold still does what it was chosen to do; the
argument for it was overstated.

**Three wrong measurements of one question**, each plausible enough to publish:

1. cut at the first `#[cfg(test)]` **attribute** rather than `mod tests` —
   caught before publishing;
2. cut at `^mod tests`, matching both the inline and external forms, and assumed
   the declaration sits at end of file — **published, and caught by the dev team
   when they went to act on it**;
3. the rule change and Task 035's file table were both justified from (2).

**What would have caught it:** checking one file by hand against the arithmetic.
`image.rs` reads 389 "test lines" in a 412-line file that visibly contains an
`impl` block — thirty seconds of looking. The guard this project adopted after
the capture failures — *derive what the value should be from known constants
first* — does not apply to a line count, because there is no constant. For
counting, the equivalent is: **check one instance by hand before trusting the
sweep.**

## Why this is worth a note rather than a quiet fix

**A rule violated by 40 files is not the rule.** It is a stated intention that
nothing enforces, and the codebase has been telling us the real convention for
some time.

This is the same shape as two other findings from the last week — `cargo clippy`
required by every acceptance criterion while nothing in CI ran it, and
`iced_test`'s `matches_image` auto-creating a missing reference and returning
`Ok(true)`. **A check that cannot fail, and a rule nobody applies, fail the same
way: silently, while every report looks clean.**

## What the measurement cannot decide

**Whether the rule is right.** A 105-line test module in a 178-line file is not
a navigability problem; a 742-line one in a 1309-line file is. The rule as
written does not distinguish them, so applying it literally means moving 40
files' tests for no benefit in most of them.

**The likely answer is that the rule needs a threshold**, not that the codebase
needs 40 edits. But that is a change to a project rule, which is the owner's,
and the honest options are:

1. **Add a threshold** — split when the test module passes some size — and fix
   the files above it.
2. **Apply it literally** — 40 files, mechanical, low risk, large diff.
3. **Retire it** — say inline test modules are fine and delete the rule.

**Option 3 has a real cost that is easy to miss:** `app/src/core.rs` is 1309
lines and 57% tests. Whatever the rule says, that file is hard to read, and the
threshold in the same ruleset — 300 ELOC to consider splitting, 500 to strongly
recommend — is about implementation only. Its **436 implementation ELOC is
comfortably under both**, so the ELOC rule does not catch this file at all.

## A correction to how this was first measured

The first pass cut each file at its first `#[cfg(test)]` **attribute** rather
than at `mod tests`. `app/src/core.rs` has a `#[cfg(test)] static` at line 564,
four lines before the real module, so everything after it counted as test code.
That produced a table naming `app/src/core/view.rs` as a 270-line offender; it
has **54** test lines and 176 of implementation, and is not an offender at all.

The corrected method finds the `mod tests` line. **Both numbers looked
plausible**, which is the point — it is the same failure the last three capture
measurements had, and the same guard applies: derive what the value should be
before reading it.
