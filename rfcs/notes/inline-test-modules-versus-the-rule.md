# The inline-test-module rule is violated by 40 files

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

**40 files carry an inline `mod tests`.** Thirteen have test modules over 100
lines:

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
